//! Closes a given single account

use std::ops::{Deref, DerefMut};

use solana_program::pubkey::Pubkey;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, Single, SingleIndexable,
    ValidateArgument,
};
use crate::util::assert::assert_is_owner;
use crate::{AccountInfo, CruiserResult, GenericError};
// use cruiser_derive::verify_account_arg_impl;

// verify_account_arg_impl! {
//     mod init_account_check <AI>{
//         <AI, Arg> CloseAccount<AI, Arg>
//         where
//             AI: AccountInfo,
//             Arg: SingleIndexable<AI, ()>{
//             from: [<T> T where AI: AccountInfo, Arg: FromAccounts<AI, T>];
//             validate: [<T> T where AI: AccountInfo, Arg: ValidateArgument<AI, T>];
//             multi: [<T> T where AI: AccountInfo, Arg: MultiIndexable<AI, T>];
//             single: [<T> T where AI: AccountInfo, Arg: SingleIndexable<AI, T>];
//         }
//     }
// }

/// Wraps a single argument and closes the account to `fundee` on cleanup.
/// Account must be owned by current program
/// [`CloseAccount::set_fundee`] needs to be called during the instruction.
#[derive(Debug)]
pub struct CloseAccount<AI, Arg>(Arg, Option<AI>);
impl<AI, Arg> CloseAccount<AI, Arg> {
    /// Sets the account that receives the funds on close.
    pub fn set_fundee(&mut self, fundee: AI) {
        self.1 = Some(fundee);
    }
}
impl<AI, Arg> Deref for CloseAccount<AI, Arg> {
    type Target = Arg;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<AI, Arg> DerefMut for CloseAccount<AI, Arg> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<AI, Arg> AccountArgument for CloseAccount<AI, Arg>
where
    AI: AccountInfo,
    Arg: SingleIndexable<(), AccountInfo = AI>,
{
    type AccountInfo = AI;

    fn write_back(self, _program_id: &Pubkey) -> CruiserResult<()> {
        let self_info = self.0.info();
        let fundee = self.1.ok_or_else(|| GenericError::Custom {
            error: format!("Close `{}` is missing fundee", self_info.key()),
        })?;
        let mut self_lamports = self_info.lamports_mut();
        *fundee.lamports_mut() += *self_lamports;
        *self_lamports = 0;
        Ok(())
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.0.add_keys(add)
    }
}
impl<AI, Arg, T> FromAccounts<T> for CloseAccount<AI, Arg>
where
    AI: AccountInfo,
    Arg: SingleIndexable<(), AccountInfo = AI> + FromAccounts<T, AccountInfo = AI>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = AI>,
        arg: T,
    ) -> CruiserResult<Self> {
        Ok(Self(Arg::from_accounts(program_id, infos, arg)?, None))
    }

    fn accounts_usage_hint(arg: &T) -> (usize, Option<usize>) {
        Arg::accounts_usage_hint(arg)
    }
}
impl<AI, Arg, T> ValidateArgument<T> for CloseAccount<AI, Arg>
where
    AI: AccountInfo,
    Arg: AccountArgument<AccountInfo = AI> + SingleIndexable + ValidateArgument<T>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: T) -> CruiserResult<()> {
        self.0.validate(program_id, arg)?;
        assert_is_owner(self.0.info(), program_id, ())
    }
}
impl<AI, Arg, T> MultiIndexable<T> for CloseAccount<AI, Arg>
where
    AI: AccountInfo,
    Arg: AccountArgument<AccountInfo = AI> + SingleIndexable + MultiIndexable<T>,
{
    fn index_is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.0.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.0.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.0.index_is_owner(owner, indexer)
    }
}
impl<AI, Arg, T> SingleIndexable<T> for CloseAccount<AI, Arg>
where
    AI: AccountInfo,
    Arg: AccountArgument<AccountInfo = AI> + SingleIndexable + SingleIndexable<T>,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.0.index_info(indexer)
    }
}
