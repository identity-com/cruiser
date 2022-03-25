use solana_program::pubkey::Pubkey;

use cruiser_derive::verify_account_arg_impl;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, ValidateArgument,
};
use crate::{AccountInfo, CruiserResult};

verify_account_arg_impl! {
    mod unit_checks<AI> {
        <AI> () where AI: AccountInfo {
            from: [()];
            validate: [()];
            multi: [];
            single: [];
        }
    }
}

impl<AI> AccountArgument<AI> for () {
    fn write_back(self, _program_id: &Pubkey) -> CruiserResult<()> {
        Ok(())
    }

    fn add_keys(&self, _add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        Ok(())
    }
}
impl<AI> FromAccounts<AI, ()> for () {
    fn from_accounts(
        _program_id: &Pubkey,
        _infos: &mut impl AccountInfoIterator<AI>,
        _arg: (),
    ) -> CruiserResult<Self> {
        Ok(())
    }

    fn accounts_usage_hint(_arg: &()) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<AI> ValidateArgument<AI, ()> for () {
    fn validate(&mut self, _program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}
