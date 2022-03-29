use borsh::{BorshDeserialize, BorshSerialize};
use cruiser::account_argument::{AccountArgument, Single};
use cruiser::account_list::AccountList;
use cruiser::account_types::data_account::DataAccount;
use cruiser::{AccountInfo, AllAny};
use solana_program::pubkey::Pubkey;

// verify_account_arg_impl! {
//     mod full_checks<AI>{
//         <AI> FullStruct<AI> where AI: AccountInfo {
//             from: [u64; (u64,)];
//             validate: [];
//             multi: [];
//             single: [];
//         };
//         <AI> FullStruct2<AI> where AI: AccountInfo {
//             from: [u64; (u64,)];
//             validate: [];
//             multi: [];
//             single: [];
//         };
//     }
// }

#[derive(AccountList, BorshSerialize, BorshDeserialize)]
pub enum TestAccountList {
    CoolAccount(CoolAccount),
    I8(i8),
}

#[derive(AccountArgument)]
#[account_argument(account_info = AI)]
#[from(data = (init_size: u64))]
pub struct FullStruct<AI>
where
    AI: AccountInfo,
{
    data_account: DataAccount<AI, TestAccountList, CoolAccount>,
    #[from(data = init_size as usize)]
    #[validate(signer, writable, owner(0) = &get_pubkey())]
    init_accounts: Vec<DataAccount<AI, TestAccountList, CoolAccount>>,
    #[validate(signer, writable(3), owner((0..4, AllAny::All, ())) = &get_pubkey(), owner(7) = self.data_account.info().key())]
    other_accounts: [DataAccount<AI, TestAccountList, i8>; 8],
}

#[derive(AccountArgument)]
#[account_argument(account_info = AI)]
#[from(data = (init_size: u64))]
pub struct FullStruct2<AI>
where
    AI: AccountInfo,
{
    data_account: DataAccount<AI, TestAccountList, CoolAccount>,
    #[from(data = vec![(); init_size as usize])]
    init_accounts: Vec<DataAccount<AI, TestAccountList, CoolAccount>>,
    #[validate(signer, writable(3), owner((0..4, AllAny::Any, ())) = &get_pubkey())]
    other_accounts: [DataAccount<AI, TestAccountList, i8>; 8],
}

#[derive(Default, BorshSerialize, BorshDeserialize)]
pub struct CoolAccount {
    data_1: u64,
    cool_data: [u8; 32],
}

fn get_pubkey() -> Pubkey {
    Pubkey::new_unique()
}
