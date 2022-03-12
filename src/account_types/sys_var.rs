//! Sysvar support

use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;
use std::ops::Deref;

use cruiser::account_argument::MultiIndexable;
use cruiser_derive::verify_account_arg_impl;
use solana_program::sysvar::Sysvar;

use crate::account_argument::{AccountArgument, SingleIndexable};
use crate::{AccountInfo, AllAny, CruiserResult};

verify_account_arg_impl! {
    mod sys_var_check{
        <S> SysVar<S> where S: Sysvar{
            from: [()];
            validate: [()];
            multi: [(); AllAny];
            single: [()];
        }
    }
}

/// A sysvar, checks the address is the same.
#[derive(AccountArgument, Debug)]
pub struct SysVar<S>(
    #[validate(key = &S::id())] pub AccountInfo,
    PhantomData<fn() -> S>,
)
where
    S: Sysvar;
impl<S> SysVar<S>
where
    S: Sysvar,
{
    /// Gets the sysvar, may be unsupported for large sys vars
    pub fn get(&self) -> CruiserResult<S> {
        unsafe { Ok(S::from_account_info(&self.0.to_solana_account_info())?) }
    }
}
impl<S> Deref for SysVar<S>
where
    S: Sysvar,
{
    type Target = AccountInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<S, T> MultiIndexable<T> for SysVar<S>
where
    AccountInfo: MultiIndexable<T>,
    S: Sysvar,
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
impl<S, T> SingleIndexable<T> for SysVar<S>
where
    AccountInfo: SingleIndexable<T>,
    S: Sysvar,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.0.info(indexer)
    }
}
