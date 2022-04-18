//! The system program

use std::fmt::Debug;

use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::{create_account, transfer};

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable};
use crate::cpi::CPIMethod;
use crate::pda_seeds::PDASeedSet;
use crate::program::{Program, ProgramKey};
use crate::{AccountInfo, CruiserResult, ToSolanaAccountInfo};

// verify_account_arg_impl! {
//     mod init_account_check<AI>{
//         <AI> SystemProgram<AI> where AI: AccountInfo{
//             from: [()];
//             validate: [()];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// The system program, will be checked that it actually is.
#[derive(AccountArgument, Debug, Clone)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
pub struct SystemProgram<AI> {
    /// The system program's [`account info`].
    ///
    /// If `is_signer` or `is_writable` is ever [`true`] you probably just got a big bug bounty from Solana!
    #[validate(key = &Self::KEY)]
    pub info: AI,
}
impl<AI> ProgramKey for SystemProgram<AI> {
    const KEY: Pubkey = Pubkey::new_from_array([0; 32]);
}
impl<AI> Program for SystemProgram<AI> where AI: AccountInfo {}

/// Argument for [`SystemProgram::create_account`]
#[derive(Copy, Clone, Debug)]
pub struct CreateAccount<'a, AI> {
    /// The funder of the new account
    pub funder: &'a AI,
    /// The account to create
    pub account: &'a AI,
    /// The amount of lamports to give the new account
    pub lamports: u64,
    /// The amount of space to allocate to the new account
    pub space: u64,
    /// The owning program of the new account
    pub owner: &'a Pubkey,
}
impl<'a, AI> SystemProgram<AI>
where
    AI: ToSolanaAccountInfo<'a>,
{
    /// Calls the system program's [`create_account`] instruction with given PDA seeds.
    pub fn create_account<'b, 'c: 'b>(
        &self,
        cpi: impl CPIMethod,
        create: &CreateAccount<AI>,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> ProgramResult {
        PDASeedSet::invoke_signed_multiple(
            cpi,
            &create_account(
                create.funder.key(),
                create.account.key(),
                create.lamports,
                create.space,
                create.owner,
            ),
            &[&self.info, create.funder, create.account],
            seeds,
        )
    }

    /// Calls the system program's [`transfer`] instruction with given PDA seeds.
    pub fn transfer<'b, 'c: 'b>(
        &self,
        cpi: impl CPIMethod,
        from: &AI,
        to: &AI,
        lamports: u64,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> ProgramResult {
        PDASeedSet::invoke_signed_multiple(
            cpi,
            &transfer(from.key(), to.key(), lamports),
            &[&self.info, from, to],
            seeds,
        )
    }
}
impl<AI, T> MultiIndexable<T> for SystemProgram<AI>
where
    AI: AccountInfo + MultiIndexable<T>,
{
    fn index_is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_owner(owner, indexer)
    }
}
impl<AI, T> SingleIndexable<T> for SystemProgram<AI>
where
    AI: AccountInfo + SingleIndexable<T>,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.info.index_info(indexer)
    }
}
