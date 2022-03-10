use std::fmt::Debug;

use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::create_account;

use cruiser_derive::verify_account_arg_impl;

use crate::{
    AccountInfo, AllAny, GeneratorResult, invoke, MultiIndexable, PDASeedSet, SingleIndexable,
};
use crate::traits::AccountArgument;

verify_account_arg_impl! {
    mod init_account_check{
        SystemProgram{
            from: [()];
            validate: [()];
            multi: [(); AllAny];
            single: [()];
        }
    }
}

/// The system program, will be checked that it actually is.
#[derive(AccountArgument, Debug, Clone)]
pub struct SystemProgram {
    /// The system program's [`account info`].
    ///
    /// If `is_signer` or `is_writable` is ever [`true`] you probably just got a big bug bounty from Solana!
    #[validate(key = &Self::KEY)]
    pub info: AccountInfo,
}
impl SystemProgram {
    /// The key of the sytem program
    pub const KEY: Pubkey = Pubkey::new_from_array([
        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
        0, 0,
    ]);

    pub fn invoke_create_account(
        &self,
        funder: &AccountInfo,
        account: &AccountInfo,
        lamports: u64,
        space: u64,
        owner: &Pubkey,
    ) -> ProgramResult {
        invoke(
            &create_account(funder.key, account.key, lamports, space, owner),
            &[&self.info, funder, account],
        )
    }

    pub fn invoke_signed_create_account(
        &self,
        seeds: &PDASeedSet,
        funder: &AccountInfo,
        account: &AccountInfo,
        lamports: u64,
        space: u64,
        owner: &Pubkey,
    ) -> ProgramResult {
        seeds.invoke_signed(
            &create_account(funder.key, account.key, lamports, space, owner),
            &[&self.info, funder, account],
        )
    }
}
impl<T> MultiIndexable<T> for SystemProgram
where
    AccountInfo: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> GeneratorResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> GeneratorResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> GeneratorResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<T> SingleIndexable<T> for SystemProgram
where
    AccountInfo: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> GeneratorResult<&AccountInfo> {
        self.info.info(indexer)
    }
}
