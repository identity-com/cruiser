use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, ValidateArgument,
};
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;

// verify_account_arg_impl! {
//     mod phantom_checks<AI> {
//         <T> PhantomData<T> where AI: AccountInfo {
//             from: [()];
//             validate: [()];
//             multi: [];
//             single: [];
//         }
//     }
// }

impl<T> AccountArgument for PhantomData<T>
where
    T: AccountArgument,
{
    type AccountInfo = T::AccountInfo;

    fn write_back(self, _program_id: &Pubkey) -> CruiserResult<()> {
        Ok(())
    }

    fn add_keys(&self, _add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        Ok(())
    }
}
impl<T> FromAccounts for PhantomData<T>
where
    T: AccountArgument,
{
    fn from_accounts(
        _program_id: &Pubkey,
        _infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        _arg: (),
    ) -> CruiserResult<Self> {
        Ok(PhantomData)
    }

    fn accounts_usage_hint(_arg: &()) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<T> ValidateArgument for PhantomData<T>
where
    T: AccountArgument,
{
    fn validate(&mut self, _program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}
