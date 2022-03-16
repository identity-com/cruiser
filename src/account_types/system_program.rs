//! The system program

use std::fmt::Debug;

use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::create_account;

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable};
use crate::pda_seeds::PDASeedSet;
use crate::program::Program;
use crate::{AccountInfo, AllAny, CruiserResult};
use cruiser_derive::verify_account_arg_impl;

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
impl Program for SystemProgram {
    const KEY: Pubkey = Pubkey::new_from_array([0; 32]);
}
impl SystemProgram {
    /// Calls the system program's [`create_account`] instruction with given PDA seeds.
    pub fn create_account<'a>(
        &self,
        funder: &AccountInfo,
        account: &AccountInfo,
        lamports: u64,
        space: u64,
        owner: &Pubkey,
        seeds: impl IntoIterator<Item = &'a PDASeedSet<'a>>,
    ) -> ProgramResult {
        PDASeedSet::invoke_signed_multiple(
            &create_account(funder.key, account.key, lamports, space, owner),
            &[&self.info, funder, account],
            seeds,
        )
    }
}
impl<T> MultiIndexable<T> for SystemProgram
where
    AccountInfo: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<T> SingleIndexable<T> for SystemProgram
where
    AccountInfo: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.info.info(indexer)
    }
}
