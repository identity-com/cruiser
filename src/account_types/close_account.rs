//! Closes a given single account

use std::ops::{Deref, DerefMut};

use solana_program::pubkey::Pubkey;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, Single, SingleIndexable,
    ValidateArgument,
};
use crate::util::assert::assert_is_owner;
use crate::{AccountInfo, CruiserError, CruiserResult};
use cruiser_derive::verify_account_arg_impl;

verify_account_arg_impl! {
    mod init_account_check{
        <A> CloseAccount<A>
        where
            A: AccountArgument + SingleIndexable<()>{
            from: [<T> T where A: FromAccounts<T>];
            validate: [<T> T where A: ValidateArgument<T>];
            multi: [<T> T where A: MultiIndexable<T>];
            single: [<T> T where A: SingleIndexable<T>];
        }
    }
}

/// Wraps a single argument and closes the account to `fundee` on cleanup.
/// Account must be owned by current program
/// [`CloseAccount::set_fundee`] needs to be called during the instruction.
#[derive(Debug)]
pub struct CloseAccount<A>(A, Option<AccountInfo>);
impl<A> CloseAccount<A> {
    /// Sets the account that receives the funds on close.
    pub fn set_fundee(&mut self, fundee: AccountInfo) {
        self.1 = Some(fundee);
    }
}
impl<A> Deref for CloseAccount<A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A> DerefMut for CloseAccount<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<A> AccountArgument for CloseAccount<A>
where
    A: AccountArgument + SingleIndexable<()>,
{
    fn write_back(self, _program_id: &'static Pubkey) -> CruiserResult<()> {
        let self_info = self.0.get_info();
        let fundee = self.1.ok_or_else(|| CruiserError::Custom {
            error: format!("Close `{}` is missing fundee", self_info.key),
        })?;
        let mut self_lamports = self_info.lamports.borrow_mut();
        **fundee.lamports.borrow_mut() += **self_lamports;
        **self_lamports = 0;
        Ok(())
    }

    fn add_keys(&self, add: impl FnMut(&'static Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.0.add_keys(add)
    }
}
impl<A, T> FromAccounts<T> for CloseAccount<A>
where
    A: AccountArgument + SingleIndexable<()> + FromAccounts<T>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: T,
    ) -> CruiserResult<Self> {
        Ok(Self(A::from_accounts(program_id, infos, arg)?, None))
    }

    fn accounts_usage_hint(arg: &T) -> (usize, Option<usize>) {
        A::accounts_usage_hint(arg)
    }
}
impl<A, T> ValidateArgument<T> for CloseAccount<A>
where
    A: AccountArgument + SingleIndexable<()> + ValidateArgument<T>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: T) -> CruiserResult<()> {
        self.0.validate(program_id, arg)?;
        assert_is_owner(self.0.get_info(), program_id, ())
    }
}
impl<A, T> MultiIndexable<T> for CloseAccount<A>
where
    A: AccountArgument + SingleIndexable<()> + MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.0.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.0.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.0.is_owner(owner, indexer)
    }
}
impl<A, T> SingleIndexable<T> for CloseAccount<A>
where
    A: AccountArgument + SingleIndexable<()> + SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.0.info(indexer)
    }
}
