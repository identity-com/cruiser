use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, ValidateArgument,
};
use crate::CruiserResult;
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;

verify_account_arg_impl! {
    mod unit_checks {
        <T> PhantomData<T> {
            from: [()];
            validate: [()];
            multi: [];
            single: [];
        }
    }
}

impl<T> AccountArgument for PhantomData<T> {
    fn write_back(self, _program_id: &'static Pubkey) -> CruiserResult<()> {
        Ok(())
    }

    fn add_keys(
        &self,
        _add: impl FnMut(&'static Pubkey) -> CruiserResult<()>,
    ) -> CruiserResult<()> {
        Ok(())
    }
}
impl<T> FromAccounts<()> for PhantomData<T> {
    fn from_accounts(
        _program_id: &'static Pubkey,
        _infos: &mut impl AccountInfoIterator,
        _arg: (),
    ) -> CruiserResult<Self> {
        Ok(PhantomData)
    }

    fn accounts_usage_hint(_arg: &()) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<T> ValidateArgument<()> for PhantomData<T> {
    fn validate(&mut self, _program_id: &'static Pubkey, _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}
