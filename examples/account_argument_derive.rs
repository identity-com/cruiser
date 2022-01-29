use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_generator::{
    AccountArgument, AccountList, AccountListItem, InitAccount, ProgramAccount, ZeroedAccount,
};
use solana_program::pubkey::Pubkey;
use std::num::NonZeroU64;

#[derive(AccountArgument)]
#[from(data = (cool: u8, hi: usize), log_level = trace)]
pub struct EmptyStruct {}

#[derive(AccountArgument)]
pub struct EmptyTupple();

#[derive(AccountArgument)]
pub struct Empty;

#[derive(BorshSerialize, BorshDeserialize)]
pub enum TestAccountList {
    CoolAccount(CoolAccount),
    I8(i8),
}
impl AccountList for TestAccountList {
    type DiscriminantCompressed = u64;
}
unsafe impl AccountListItem<CoolAccount> for TestAccountList {
    fn discriminant() -> NonZeroU64 {
        NonZeroU64::new(1).unwrap()
    }

    fn from_account(account: CoolAccount) -> Self {
        Self::CoolAccount(account)
    }

    fn into_account(self) -> Result<CoolAccount, Self> {
        if let Self::CoolAccount(account) = self {
            Ok(account)
        } else {
            Err(self)
        }
    }
}
unsafe impl AccountListItem<i8> for TestAccountList {
    fn discriminant() -> NonZeroU64 {
        NonZeroU64::new(2).unwrap()
    }

    fn from_account(account: i8) -> Self {
        Self::I8(account)
    }

    fn into_account(self) -> Result<i8, Self> {
        if let Self::I8(account) = self {
            Ok(account)
        } else {
            Err(self)
        }
    }
}

#[derive(AccountArgument)]
#[from(data = (init_size: u64))]
pub struct FullStruct {
    data_account: ProgramAccount<TestAccountList, CoolAccount>,
    #[account_argument(signer, writable, owner(0) = &get_pubkey(), from_data = vec![(); init_size as usize])]
    init_accounts: Vec<InitAccount<TestAccountList, CoolAccount>>,
    #[account_argument(signer, writable(3), owner(0..4) = &get_pubkey())]
    other_accounts: [ZeroedAccount<TestAccountList, i8>; 8],
}

#[derive(AccountArgument)]
#[from(data = (init_size: u64))]
pub struct FullStruct2 {
    data_account: ProgramAccount<TestAccountList, CoolAccount>,
    #[account_argument(from_data = vec![(); init_size as usize])]
    init_accounts: Vec<InitAccount<TestAccountList, CoolAccount>>,
    #[account_argument(signer, writable(3), owner(0..4) = &get_pubkey())]
    other_accounts: [ZeroedAccount<TestAccountList, i8>; 8],
}

#[derive(Default, BorshSerialize, BorshDeserialize, BorshSchema)]
pub struct CoolAccount {
    data_1: u64,
    cool_data: [u8; 32],
}

fn get_pubkey() -> Pubkey {
    Pubkey::new_unique()
}
