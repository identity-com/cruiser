//! Implementations for spl tokens

mod mint_account;
mod program;
mod token_account;

pub use mint_account::*;
pub use program::*;
pub use token_account::*;

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable};
use crate::{AccountInfo, CruiserResult};
use solana_program::pubkey::Pubkey;
use std::ops::{Deref, DerefMut};

// verify_account_arg_impl! {
//     mod token_program_account_check <AI>{
//         <AI> TokenProgramAccount<AI> where AI: AccountInfo {
//             from: [()];
//             validate: [()];
//             multi: [<I> I where AI: MultiIndexable<AI, I>];
//             single: [<I> I where AI: SingleIndexable<AI, I>];
//         };
//     }
// }

/// Account owned by the token program
#[derive(AccountArgument, Debug, Clone)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
pub struct TokenProgramAccount<AI>(#[validate(owner = &spl_token::ID)] pub AI);
impl<AI> Deref for TokenProgramAccount<AI> {
    type Target = AI;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<AI> DerefMut for TokenProgramAccount<AI> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<AI, I> MultiIndexable<I> for TokenProgramAccount<AI>
where
    AI: AccountInfo + MultiIndexable<I>,
{
    fn index_is_signer(&self, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_owner(owner, indexer)
    }
}
impl<AI, I> SingleIndexable<I> for TokenProgramAccount<AI>
where
    AI: AccountInfo + SingleIndexable<I>,
{
    fn index_info(&self, indexer: I) -> CruiserResult<&AI> {
        self.0.index_info(indexer)
    }
}
