use anchor_lang::prelude::*;
use anchor_spl::token::{self, Mint, Token, TokenAccount};

#[cfg(feature = "mainnet-beta")]
declare_id!("AmNeSW4UMPFBodCjEJD22G3kA8EraUGkhxr3GmdyEF4f");
#[cfg(not(feature = "mainnet-beta"))]
declare_id!("GvMhkYZmtnCL7jfVsTKz5zi1Jmd5dqRTHaXJL2ps1Gb");

// mock usdc是自己发的一个spl token，其mint authority不是一个wallet地址，而是一个pda地址。
// 该pda地址的seeds为mock usdc mint地址，owner为本program
#[program]
pub mod mock_usdc_faucet {
    use super::*;
    use anchor_spl::token::MintTo;

    // 初始化
    pub fn initialize(
        ctx: Context<InitializeMockUSDCFaucet>,
        _mock_usdc_faucet_nonce: u8,
    ) -> Result<()> {
        // 获取mock usdc mint账户地址
        let mint_account_key = ctx.accounts.mint_account.to_account_info().key;
        // 计算pda地址作为mint_authority，该地址的owner为本program，seeds为mock usdc mint
        // mint_authority_nonce为该pda地址对应的bump
        let (mint_authority, mint_authority_nonce) =
            Pubkey::find_program_address(&[mint_account_key.as_ref()], ctx.program_id);

        // 检查mock usdc mint地址就是刚刚生成的pda：mint_authority
        if ctx.accounts.mint_account.mint_authority.unwrap() != mint_authority {
            return Err(ErrorCode::InvalidMintAccountAuthority.into());
        }

        // 初始化mock_usdc_faucet_state账户内容
        **ctx.accounts.mock_usdc_faucet_state = MockUSDCFaucetState {
            // admin为本次调用的signer
            admin: *ctx.accounts.admin.key,
            // mock usdc的mint地址
            mint: *mint_account_key,
            // mock usdc的mint authority (本program的一个pda)
            mint_authority,
            // mint_authority pda对应的bump
            mint_authority_nonce,
        };

        Ok(())
    }

    // 给user铸造amount数量的mock usdc
    pub fn mint_to_user(ctx: Context<MintToUser>, amount: u64) -> Result<()> {
        // 由于cpi调用token program的mint_to方法的signer是一个pda地址，所以需要seeds
        // 该seeds为对应生成pda地址的seeds和bump
        let mint_signature_seeds = [
            ctx.accounts.mock_usdc_faucet_state.mint.as_ref(),
            bytemuck::bytes_of(&ctx.accounts.mock_usdc_faucet_state.mint_authority_nonce),
        ];
        let signers = &[&mint_signature_seeds[..]];
        // 构建调用token program的mint_to方法时的accounts
        let cpi_accounts = MintTo {
            // mock usdc的mint的AccountInfo
            mint: ctx.accounts.mint_account.to_account_info(),
            // to地址的AccountInfo
            to: ctx.accounts.user_token_account.to_account_info(),
            // mock usdc的mint的authority的AccountInfo
            authority: ctx.accounts.mint_authority.clone(),
        };
        // cpi调用的token program对应的AccountInfo
        let cpi_program = ctx.accounts.token_program.to_account_info();
        // 构建cpi调用的ctx（应用于目标program要求调用者提供pda签名，即pda作为Signer）
        let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
        // 调用token program的mint_to方法
        token::mint_to(cpi_context, amount).unwrap();
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction(mock_usdc_faucet_nonce: u8)]
pub struct InitializeMockUSDCFaucet<'info> {
    // 全局唯一account，存储MockUSDCFaucetState结构
    #[account(
        init,
        seeds = [b"mock_usdc_faucet".as_ref()],
        space = std::mem::size_of::<MockUSDCFaucetState>() + 8,
        bump,
        payer = admin
    )]
    pub mock_usdc_faucet_state: Box<Account<'info, MockUSDCFaucetState>>,
    #[account(mut)]
    pub admin: Signer<'info>,
    // USDC的mint账户
    pub mint_account: Box<Account<'info, Mint>>,
    pub rent: Sysvar<'info, Rent>,
    pub system_program: Program<'info, System>,
}

#[account]
#[derive(Default)]
pub struct MockUSDCFaucetState {
    pub admin: Pubkey,
    pub mint: Pubkey,
    pub mint_authority: Pubkey,
    pub mint_authority_nonce: u8,
}

#[derive(Accounts)]
pub struct MintToUser<'info> {
    // 全局唯一account，存储MockUSDCFaucetState结构
    pub mock_usdc_faucet_state: Box<Account<'info, MockUSDCFaucetState>>,
    // mock usdc的mint账户
    #[account(mut)]
    pub mint_account: Box<Account<'info, Mint>>,
    // 用户用于接收铸币的token account
    #[account(mut)]
    pub user_token_account: Box<Account<'info, TokenAccount>>,
    /// CHECK: Checked by spl_token
    pub mint_authority: AccountInfo<'info>,
    // mock usdc对应的token program（不是token2022）
    pub token_program: Program<'info, Token>,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Program not mint authority")]
    InvalidMintAccountAuthority,
    #[msg("Signer must be MockUSDCFaucet admin")]
    Unauthorized,
}
