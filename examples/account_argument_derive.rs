use borsh::{BorshDeserialize, BorshSerialize};
use cruiser::account_argument::{AccountArgument, Single};
use cruiser::account_list::AccountList;
use cruiser::account_types::program_account::ProgramAccount;
use cruiser::{verify_account_arg_impl, AllAny};
use solana_program::pubkey::Pubkey;

verify_account_arg_impl! {
    mod empty_checks{
        EmptyStruct {
            from: [(u8, usize)];
            validate: [];
            multi: [];
            single: [];
        };
        EmptyTupple {
            from: [();];
            validate: [];
            multi: [];
            single: [];
        };
        FullStruct {
            from: [u64; (u64,)];
            validate: [];
            multi: [];
            single: [];
        };
        FullStruct2 {
            from: [u64; (u64,)];
            validate: [];
            multi: [];
            single: [];
        };
    }
}

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
    #[from(data = init_size as usize)]
    #[validate(signer, writable, owner(0) = &get_pubkey())]
    init_accounts: Vec<ProgramAccount<TestAccountList, CoolAccount>>,
    #[validate(signer, writable(3), owner((0..4, AllAny::All, ())) = &get_pubkey(), owner(7) = self.data_account.get_info().key)]
    other_accounts: [ProgramAccount<TestAccountList, i8>; 8],
}

#[derive(AccountArgument)]
#[from(data = (init_size: u64))]
pub struct FullStruct2 {
    data_account: ProgramAccount<TestAccountList, CoolAccount>,
    #[from(data = vec![(); init_size as usize])]
    init_accounts: Vec<ProgramAccount<TestAccountList, CoolAccount>>,
    #[validate(signer, writable(3), owner((0..4, AllAny::Any, ())) = &get_pubkey())]
    other_accounts: [ProgramAccount<TestAccountList, i8>; 8],
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct CoolAccount {
    data_1: u64,
    cool_data: [u8; 32],
}

fn get_pubkey() -> Pubkey {
    Pubkey::new_unique()
}
