//! An account that represents a program written with cruiser and therefore easily callable.

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable};
use crate::account_types::PhantomAccount;
use crate::instruction_list::{InstructionListClientDynamic, InstructionListClientStatic};
use crate::pda_seeds::PDASeedSet;
use crate::program::{CruiserProgram, Program, ProgramKey};
use crate::{AccountInfo, CruiserResult, ToSolanaAccountInfo, CPI};
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use std::iter::once;

// verify_account_arg_impl! {
//     mod cruiser_program_account_check<AI>{
//         <AI, P> CruiserProgramAccount<AI, P> where AI: AccountInfo, P: CruiserProgram{
//             from: [()];
//             validate: [()];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// A cruiser program that can be called with its client functions
#[derive(AccountArgument, Debug, Clone)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
pub struct CruiserProgramAccount<AI, P>(#[validate(key = &P::KEY)] pub AI, PhantomAccount<AI, P>)
where
    P: CruiserProgram;
impl<'a, AI, P> CruiserProgramAccount<AI, P>
where
    AI: ToSolanaAccountInfo<'a>,
    P: CruiserProgram,
{
    /// Calls one of this program's functions that has statically sized account length
    pub fn invoke<'b, 'c: 'b, const N: usize>(
        &self,
        cpi: impl CPI,
        instruction: &mut impl InstructionListClientStatic<P::InstructionList, N, AccountInfo = AI>,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> ProgramResult {
        PDASeedSet::invoke_signed_multiple(
            cpi,
            &instruction.instruction(&Self::KEY),
            &instruction.to_accounts_static(&self.0),
            seeds,
        )
    }

    /// Calls one of this program's functions that has dynamically sized account length
    pub fn invoke_variable_sized<'b, 'c: 'b>(
        &self,
        cpi: impl CPI,
        instruction: &mut impl InstructionListClientDynamic<P::InstructionList, AccountInfo = AI>,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> ProgramResult {
        PDASeedSet::invoke_signed_variable_size_multiple(
            cpi,
            &instruction.instruction(&Self::KEY),
            instruction.to_accounts_dynamic().chain(once(&self.0)),
            seeds,
        )
    }
}
impl<AI, P> ProgramKey for CruiserProgramAccount<AI, P>
where
    P: CruiserProgram,
{
    const KEY: Pubkey = P::KEY;
}

impl<AI, P> Program for CruiserProgramAccount<AI, P>
where
    AI: AccountInfo,
    P: CruiserProgram,
{
}
impl<AI, P, I> MultiIndexable<I> for CruiserProgramAccount<AI, P>
where
    AI: AccountInfo + MultiIndexable<I>,
    P: CruiserProgram,
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
impl<AI, P, I> SingleIndexable<I> for CruiserProgramAccount<AI, P>
where
    AI: AccountInfo + SingleIndexable<I>,
    P: CruiserProgram,
{
    fn index_info(&self, indexer: I) -> CruiserResult<&AI> {
        self.0.index_info(indexer)
    }
}
