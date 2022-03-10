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
    clippy::mut_mut,
    missing_docs
)]
//! A generator program that will be able to generate solana program code from a much easier starting place.

extern crate self as cruiser;

use array_init::array_init;
pub use borsh;
pub use solana_program;
pub use solana_program::{
    clock::UnixTimestamp,
    instruction::{AccountMeta as SolanaAccountMeta, Instruction as SolanaInstruction},
    pubkey::Pubkey,
};
use solana_program::entrypoint::ProgramResult;
pub use solana_program::msg;
use solana_program::program::{invoke as solana_invoke, invoke_signed as solana_invoke_signed};
pub use static_assertions;

pub use account_info::*;
pub use account_types::*;
pub use cruiser_derive::verify_account_arg_impl;
pub use error::*;
pub use pda_seeds::*;
pub use traits::*;
pub use util::*;

#[macro_use]
mod macros;

mod account_info;
mod account_types;
pub mod compressed_numbers;
pub mod entrypoint;
mod error;
mod impls;
mod pda_seeds;
pub mod spl;
mod traits;
pub mod types;
pub mod util;

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
