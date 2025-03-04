use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

// 转移spl token
pub fn send<'info>(
    token_program: &Program<'info, Token>,
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    // 通常为from账户的owner
    authority: &AccountInfo<'info>,
    nonce: u8,
    amount: u64,
) -> Result<()> {
    // from的token账户key
    let from_key = from.key();
    // signature seeds为：from ata key + nonce
    let signature_seeds = [from_key.as_ref(), bytemuck::bytes_of(&nonce)];
    let signers = &[&signature_seeds[..]];
    
    // 构建cpi的accounts
    let cpi_accounts = Transfer {
        from: from.to_account_info().clone(),
        to: to.to_account_info().clone(),
        authority: authority.to_account_info().clone(),
    };

    // cpi调用的program为：token_program
    let cpi_program = token_program.to_account_info();
    // 构建cpi context
    let cpi_context = CpiContext::new_with_signer(cpi_program, cpi_accounts, signers);
    // 执行cpi调用进行转账
    token::transfer(cpi_context, amount)
}

pub fn receive<'info>(
    token_program: &Program<'info, Token>,
    from: &Account<'info, TokenAccount>,
    to: &Account<'info, TokenAccount>,
    authority: &AccountInfo<'info>,
    amount: u64,
) -> Result<()> {
    let cpi_accounts = Transfer {
        from: from.to_account_info().clone(),
        to: to.to_account_info().clone(),
        authority: authority.to_account_info().clone(),
    };
    let cpi_program = token_program.to_account_info();
    let cpi_context = CpiContext::new(cpi_program, cpi_accounts);
    token::transfer(cpi_context, amount)
}
