#![cfg_attr(
    feature = "in_place",
    feature(generic_const_exprs, const_fn_trait_bound, exclusive_range_pattern)
)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![cfg_attr(feature = "in_place", allow(incomplete_features))]
#![feature(generic_associated_types)]
#![warn(
    unused_import_braces,
    unused_imports,
    missing_docs,
    missing_debug_implementations,
    clippy::pedantic
)]
#![allow(
    clippy::cast_possible_truncation,
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::too_many_lines,
    clippy::mut_mut
)]
//! A generator program that will be able to generate solana program code from a much easier starting place.
//!
//! # How it works
//! The standard lifecycle of an instruction (standard derive of [`InstructionListProcessor`]):
//! 1. [`Instruction::Data`] is deserialized with [`BorshDeserialize::deserialize`] from incoming data
//! 1. [`Instruction::Data`] is split into [`Instruction::FromAccountsData`] and [`Instruction::InstructionData`] with [`Instruction::data_to_instruction_arg`]
//! 1. [`Instruction::Accounts`] is created from [`Instruction::FromAccountsData`] by [`FromAccounts::from_accounts`]
//! 1. [`InstructionProcessor::process`] is called with [`Instruction::InstructionData`] and [`Instruction::Accounts`]
//! 1. [`Instruction::Accounts`] is cleaned up by with [`AccountArgument::write_back`]
//!
//! [`InstructionListProcessor`]: crate::instruction_list::InstructionListProcessor
//! [`BorshDeserialize::deserialize`]: crate::borsh::BorshDeserialize::deserialize
//! [`Instruction::Data`]: crate::instruction::Instruction::Data
//! [`Instruction::FromAccountsData`]: crate::instruction::Instruction::FromAccountsData
//! [`Instruction::InstructionData`]: crate::instruction::Instruction::InstructionData
//! [`Instruction::data_to_instruction_arg`]: crate::instruction::Instruction::data_to_instruction_arg
//! [`Instruction::Accounts`]: crate::instruction::Instruction::Accounts
//! [`FromAccounts::from_accounts`]: crate::account_argument::FromAccounts::from_accounts
//! [`InstructionProcessor::process`]: crate::instruction::InstructionProcessor::process
//! [`AccountArgument::write_back`]: crate::account_argument::AccountArgument::write_back

extern crate self as cruiser;

#[macro_use]
mod macros;

pub mod account_types;
pub mod compressed_numbers;
pub mod entrypoint;
pub mod indexer;
pub mod pda_seeds;
#[cfg(any(feature = "spl-token"))]
pub mod spl;
pub mod types;
pub mod util;

mod account_info;
mod impls;
mod traits;

pub use account_info::*;
pub use borsh;
pub use cruiser_derive::verify_account_arg_impl;
pub use generic_error::*;
pub use indexer::AllAny;
pub use solana_program;
pub use solana_program::account_info::AccountInfo as SolanaAccountInfo;
pub use solana_program::msg;
pub use solana_program::{
    clock::UnixTimestamp,
    instruction::{AccountMeta as SolanaAccountMeta, Instruction as SolanaInstruction},
    pubkey::Pubkey,
};
pub use static_assertions;
pub use traits::error::CruiserResult;
pub use traits::*;

use solana_program::entrypoint::ProgramResult;
use solana_program::program::{
    invoke_signed as solana_invoke_signed,
    invoke_signed_unchecked as solana_invoke_signed_unchecked,
};

mod generic_error;

use array_init::array_init;

/// A way of executing CPI calls
pub trait CPI: Sized {
    /// The raw execution function.
    /// Usually ends up at either [`solana_program::program::invoke_signed`] or [`solana_program::program::invoke_signed_unchecked`]
    fn raw_invoke_signed(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[SolanaAccountInfo],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult;

    /// Invokes another solana program.
    fn invoke<'a, AI, const N: usize>(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[&AI; N],
    ) -> ProgramResult
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        self.invoke_signed(instruction, account_infos, &[])
    }

    /// Invokes another solana program, signing with seeds.
    fn invoke_signed<'a, AI, const N: usize>(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[&AI; N],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        self.raw_invoke_signed(
            instruction,
            &array_init::<_, _, N>(|x| unsafe { account_infos[x].to_solana_account_info() }),
            signer_seeds,
        )
    }

    /// Invokes another solana program with a variable number of accounts.
    /// Less efficient than [`CPI::invoke`].
    fn invoke_variable_size<'a, 'b, AI, I>(
        self,
        instruction: &SolanaInstruction,
        account_infos: I,
    ) -> ProgramResult
    where
        AI: 'a + ToSolanaAccountInfo<'b>,
        I: IntoIterator<Item = &'a AI>,
    {
        self.invoke_signed_variable_size(instruction, account_infos, &[])
    }

    /// Invokes another solana program with a variable number of accounts, signing with seeds.
    /// Less efficient than [`invoke_signed`].
    /// Equivalent to [`solana_program::program::invoke_signed`] but with custom [`AccountInfo`].
    fn invoke_signed_variable_size<'a, 'b, AI, I>(
        self,
        instruction: &SolanaInstruction,
        account_infos: I,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult
    where
        AI: 'a + ToSolanaAccountInfo<'b>,
        I: IntoIterator<Item = &'a AI>,
    {
        self.raw_invoke_signed(
            instruction,
            &account_infos
                .into_iter()
                .map(|info| unsafe { info.to_solana_account_info() })
                .collect::<Vec<_>>(),
            signer_seeds,
        )
    }
}

/// CPI functions that check each account for outstanding usages.
/// Less efficient than [`CPIUnchecked`] but will avoid unsafe situations.
/// Suggested to use this for validation and then swap to [`CPIUnchecked`].
/// Uses [`solana_program::program::invoke_signed`]
#[derive(Copy, Clone, Debug)]
pub struct CPIChecked;
impl CPI for CPIChecked {
    #[inline]
    fn raw_invoke_signed(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[SolanaAccountInfo],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        solana_invoke_signed(instruction, account_infos, signer_seeds)
    }
}

/// CPI functions that doesn't check each account for outstanding usages.
/// Can result in unsafe situations but is more efficient than [`CPIChecked`].
/// Uses [`solana_program::program::invoke_signed_unchecked`]
#[derive(Copy, Clone, Debug)]
pub struct CPIUnchecked;
impl CPI for CPIUnchecked {
    #[inline]
    fn raw_invoke_signed(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[SolanaAccountInfo],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        solana_invoke_signed_unchecked(instruction, account_infos, signer_seeds)
    }
}
