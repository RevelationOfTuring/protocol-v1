use crate::context::InitializeUserOptionalAccounts;
use crate::error::ErrorCode;
use crate::optional_accounts::get_whitelist_token;
use crate::state::state::State;
use crate::state::user::{User, UserPositions};
use anchor_lang::prelude::*;

// 初始化pda<User>和account<UserPositions>
pub fn initialize(
    state: &Account<State>,
    user: &mut Box<Account<User>>,
    user_positions: &AccountLoader<UserPositions>,
    authority: &Signer,
    // 即ctx.remaining_accounts
    remaining_accounts: &[AccountInfo],
    optional_accounts: InitializeUserOptionalAccounts,
) -> Result<()> {
    if !state.whitelist_mint.eq(&Pubkey::default()) {
        // 如果state.whitelist_mint中不是默认值
        // 从ctx.remaining_accounts中的唯一account地址，解析出对应的TokenAccount
        let whitelist_token =
            get_whitelist_token(optional_accounts, remaining_accounts, &state.whitelist_mint)?;

        // 要求whitelist_token不能是None，即最外层instruction的传参InitializeUserOptionalAccounts.whitelist_token为false时，报错
        if whitelist_token.is_none() {
            return Err(ErrorCode::WhitelistTokenNotFound.into());
        }

        let whitelist_token = whitelist_token.unwrap();
        // 检查whitelist_token的owner为signer，否则报错
        if !whitelist_token.owner.eq(authority.key) {
            return Err(ErrorCode::InvalidWhitelistToken.into());
        }

        // 要求whitelist_token的余额大于0
        if whitelist_token.amount == 0 {
            return Err(ErrorCode::WhitelistTokenNotFound.into());
        }
    }

    // 1. 初始化pda<User>
    // user.authority为signer
    user.authority = *authority.key;
    user.collateral = 0;
    user.cumulative_deposits = 0;
    // user.positions为account<UserPositions>的key
    user.positions = *user_positions.to_account_info().key;

    user.collateral_claimed = 0;
    user.last_collateral_available_to_claim = 0;
    user.forgo_position_settlement = 0;
    user.has_settled_position = 0;

    user.padding1 = 0;
    user.padding2 = [0; 14];

    // 2. 初始化account<UserPositions>
    let user_positions = &mut user_positions.load_init()?;
    // user_positions.user设置为pda<User>的key
    user_positions.user = *user.to_account_info().key;

    Ok(())
}
