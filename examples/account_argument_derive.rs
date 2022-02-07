use borsh::{BorshDeserialize, BorshSerialize};
use solana_generator::{AccountArgument, AccountList, InitAccount, ProgramAccount, ZeroedAccount};
use solana_program::pubkey::Pubkey;

#[derive(AccountArgument)]
#[from(data = (cool: u8, hi: usize), log_level = trace)]
pub struct EmptyStruct {}

#[derive(AccountArgument)]
pub struct EmptyTupple();

#[derive(AccountArgument)]
pub struct Empty;

#[derive(AccountList, BorshSerialize, BorshDeserialize)]
pub enum TestAccountList {
    CoolAccount(CoolAccount),
    I8(i8),
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

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct CoolAccount {
    data_1: u64,
    cool_data: [u8; 32],
}

fn get_pubkey() -> Pubkey {
    Pubkey::new_unique()
}
