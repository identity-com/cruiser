#![cfg_attr(
    feature = "nightly",
    feature(
        generic_associated_types,
        generic_const_exprs,
        const_fn_trait_bound,
        exclusive_range_pattern
    )
)]
#![cfg_attr(feature = "nightly", allow(incomplete_features))]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
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
use solana_program::program::{invoke as solana_invoke, invoke_signed as solana_invoke_signed};

mod generic_error;

use array_init::array_init;

/// Invokes another solana program.
/// Equivalent to [`solana_program::program::invoke`] but with custom [`AccountInfo`].
pub fn invoke<const N: usize>(
    instruction: &SolanaInstruction,
    account_infos: &[&AccountInfo; N],
) -> ProgramResult {
    solana_invoke(
        instruction,
        &array_init::<_, _, N>(|x| unsafe { account_infos[x].to_solana_account_info() }),
    )
}

/// Invokes another solana program, signing with seeds.
/// Equivalent to [`solana_program::program::invoke_signed`] but with custom [`AccountInfo`].
pub fn invoke_signed<const N: usize>(
    instruction: &SolanaInstruction,
    account_infos: &[&AccountInfo; N],
    signer_seeds: &[&[&[u8]]],
) -> ProgramResult {
    solana_invoke_signed(
        instruction,
        &array_init::<_, _, N>(|x| unsafe { account_infos[x].to_solana_account_info() }),
        signer_seeds,
    )
}

/// Invokes another solana program with a variable number of accounts.
/// Less efficient than [`invoke`].
/// Equivalent to [`solana_program::program::invoke`] but with custom [`AccountInfo`].
pub fn invoke_variable_size(
    instruction: &SolanaInstruction,
    account_infos: &[&AccountInfo],
) -> ProgramResult {
    solana_invoke(
        instruction,
        &account_infos
            .iter()
            .map(|info| unsafe { info.to_solana_account_info() })
            .collect::<Vec<_>>(),
    )
}

/// Invokes another solana program with a variable number of accounts, signing with seeds.
/// Less efficient than [`invoke_signed`].
/// Equivalent to [`solana_program::program::invoke_signed`] but with custom [`AccountInfo`].
pub fn invoke_signed_variable_size(
    instruction: &SolanaInstruction,
    account_infos: &[&AccountInfo],
    signer_seeds: &[&[&[u8]]],
) -> ProgramResult {
    solana_invoke_signed(
        instruction,
        &account_infos
            .iter()
            .map(|info| unsafe { info.to_solana_account_info() })
            .collect::<Vec<_>>(),
        signer_seeds,
    )
}
