use crate::{
    AccountArgument, AccountInfoIterator, FromAccounts, GeneratorResult, SystemProgram,
    ValidateArgument,
};
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;

verify_account_arg_impl! {
    mod unit_checks {
        () {
            from: [()];
            validate: [()];
            multi: [];
            single: [];
        }
    }
}

impl AccountArgument for () {
    fn write_back(
        self,
        _program_id: &'static Pubkey,
        _system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        Ok(())
    }

    fn add_keys(
        &self,
        _add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        Ok(())
    }
}
impl FromAccounts<()> for () {
    fn from_accounts(
        _program_id: &'static Pubkey,
        _infos: &mut impl AccountInfoIterator,
        _arg: (),
    ) -> GeneratorResult<Self> {
        Ok(())
    }

    fn accounts_usage_hint(_arg: &()) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl ValidateArgument<()> for () {
    fn validate(&mut self, _program_id: &'static Pubkey, _arg: ()) -> GeneratorResult<()> {
        Ok(())
    }
}
