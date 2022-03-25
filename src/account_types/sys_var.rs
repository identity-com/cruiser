//! Sysvar support

use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;
use std::ops::Deref;

use cruiser::account_argument::MultiIndexable;
use cruiser_derive::verify_account_arg_impl;
use solana_program::sysvar::Sysvar;

use crate::account_argument::{AccountArgument, SingleIndexable};
use crate::{AccountInfo, AllAny, CruiserResult, ToSolanaAccountInfo};

verify_account_arg_impl! {
    mod sys_var_check<AI>{
        <AI, S> SysVar<AI, S> where AI: AccountInfo, S: Sysvar{
            from: [()];
            validate: [()];
            multi: [(); AllAny];
            single: [()];
        }
    }
}

/// A sysvar, checks the address is the same.
#[derive(AccountArgument, Debug)]
#[account_argument(account_info = AI)]
pub struct SysVar<AI, S>(#[validate(key = &S::id())] pub AI, PhantomData<fn() -> S>)
where
    AI: AccountInfo,
    S: Sysvar;
impl<'a, AI, S> SysVar<AI, S>
where
    AI: ToSolanaAccountInfo<'a>,
    S: Sysvar,
{
    /// Gets the sysvar, may be unsupported for large sys vars
    pub fn get(&self) -> CruiserResult<S> {
        unsafe { Ok(S::from_account_info(&self.0.to_solana_account_info())?) }
    }
}
impl<AI, S> Deref for SysVar<AI, S>
where
    AI: AccountInfo,
    S: Sysvar,
{
    type Target = AI;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<AI, S, T> MultiIndexable<AI, T> for SysVar<AI, S>
where
    AI: AccountInfo + MultiIndexable<AI, T>,
    S: Sysvar,
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
impl<AI, S, T> SingleIndexable<AI, T> for SysVar<AI, S>
where
    AI: AccountInfo + SingleIndexable<AI, T>,
    S: Sysvar,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.0.index_info(indexer)
    }
}
