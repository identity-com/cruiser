use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, ValidateArgument,
};
use crate::{AccountInfo, CruiserResult};
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;

verify_account_arg_impl! {
    mod phantom_checks<AI> {
        <AI, T> PhantomData<T> where AI: AccountInfo {
            from: [()];
            validate: [()];
            multi: [];
            single: [];
        }
    }
}

impl<AI, T> AccountArgument<AI> for PhantomData<T> {
    fn write_back(self, _program_id: &Pubkey) -> CruiserResult<()> {
        Ok(())
    }

    fn add_keys(&self, _add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        Ok(())
    }
}
impl<AI, T> FromAccounts<AI, ()> for PhantomData<T> {
    fn from_accounts(
        _program_id: &Pubkey,
        _infos: &mut impl AccountInfoIterator<AI>,
        _arg: (),
    ) -> CruiserResult<Self> {
        Ok(PhantomData)
    }

    fn accounts_usage_hint(_arg: &()) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<AI, T> ValidateArgument<AI, ()> for PhantomData<T> {
    fn validate(&mut self, _program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}
