//! Sysvar support

use crate::account_argument::{AccountArgument, SingleIndexable};
use crate::account_types::PhantomAccount;
use crate::{AccountInfo, CruiserResult, ToSolanaAccountInfo};
use cruiser::account_argument::MultiIndexable;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use std::ops::Deref;

// verify_account_arg_impl! {
//     mod sys_var_check<AI>{
//         <AI, S> SysVar<AI, S> where AI: AccountInfo, S: Sysvar{
//             from: [()];
//             validate: [()];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// A sysvar, checks the address is the same.
#[derive(AccountArgument, Debug)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo, S: Sysvar])]
pub struct SysVar<AI, S>(#[validate(key = &S::id())] pub AI, PhantomAccount<AI, S>);
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
impl<AI, S, T> MultiIndexable<T> for SysVar<AI, S>
where
    AI: AccountInfo + MultiIndexable<T>,
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
impl<AI, S, T> SingleIndexable<T> for SysVar<AI, S>
where
    AI: AccountInfo + SingleIndexable<T>,
    S: Sysvar,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.0.index_info(indexer)
    }
}
