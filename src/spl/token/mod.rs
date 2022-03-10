//! Implementations for spl tokens

use std::ops::{Deref, DerefMut};

pub use mint_account::*;
pub use program::*;
pub use token_account::*;

use crate::{
    verify_account_arg_impl, AccountArgument, AccountInfo, AllAny, GeneratorResult, MultiIndexable,
    Pubkey, SingleIndexable,
};

mod mint_account;
mod program;
mod token_account;

verify_account_arg_impl! {
    mod token_program_account_check{
        TokenProgram{
            from: [()];
            validate: [()];
            multi: [(); AllAny];
            single: [()];
        };
        TokenProgramAccount{
            from: [()];
            validate: [()];
            multi: [<I> I where AccountInfo: MultiIndexable<I>];
            single: [<I> I where AccountInfo: SingleIndexable<I>];
        };
    }
}

/// Account owned by the token program
#[derive(AccountArgument, Debug)]
pub struct TokenProgramAccount(#[validate(owner = &spl_token::ID)] pub AccountInfo);
impl Deref for TokenProgramAccount {
    type Target = AccountInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for TokenProgramAccount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<I> MultiIndexable<I> for TokenProgramAccount
where
    AccountInfo: MultiIndexable<I>,
{
    fn is_signer(&self, indexer: I) -> GeneratorResult<bool> {
        self.0.is_signer(indexer)
    }

    fn is_writable(&self, indexer: I) -> GeneratorResult<bool> {
        self.0.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: I) -> GeneratorResult<bool> {
        self.0.is_owner(owner, indexer)
    }
}
impl<I> SingleIndexable<I> for TokenProgramAccount
where
    AccountInfo: SingleIndexable<I>,
{
    fn info(&self, indexer: I) -> GeneratorResult<&AccountInfo> {
        self.0.info(indexer)
    }
}
