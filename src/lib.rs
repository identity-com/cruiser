#![feature(associated_type_defaults)]
#![cfg_attr(all(doc, CHANNEL_NIGHTLY), feature(doc_auto_cfg))]
#![allow(stable_features)]
#![feature(associated_type_defaults)]
#![feature(const_trait_impl)]
#![feature(const_fn_trait_bound)]
#![feature(const_mut_refs)]
#![feature(const_for)]
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
    clippy::too_many_lines
)]
//! A generator program that will be able to generate solana program code from a much easier starting place.
//!
//! # How it works
//! The standard lifecycle of an instruction (standard derive of [`InstructionListProcessor`]):
//! 1. [`Instruction::Data`] is deserialized with [`BorshDeserialize::deserialize`] from incoming data
//! 1. [`Instruction::Data`] is split into [`InstructionProcessor::FromAccountsData`], [`InstructionProcessor::ValidateData`], and [`InstructionProcessor::InstructionData`] with [`InstructionProcessor::data_to_instruction_arg`]
//! 1. [`Instruction::Accounts`] is created from [`InstructionProcessor::FromAccountsData`] by [`FromAccounts::from_accounts`]
//! 1. [`InstructionProcessor::process`] is called with [`InstructionProcessor::InstructionData`] and [`Instruction::Accounts`]
//! 1. [`Instruction::Accounts`] is cleaned up by with [`AccountArgument::write_back`]
//!
//! [`InstructionListProcessor`]: crate::instruction_list::InstructionListProcessor
//! [`BorshDeserialize::deserialize`]: crate::borsh::BorshDeserialize::deserialize
//! [`Instruction::Data`]: crate::instruction::Instruction::Data
//! [`InstructionProcessor::FromAccountsData`]: crate::instruction::InstructionProcessor::FromAccountsData
//! [`InstructionProcessor::ValidateData`]: crate::instruction::InstructionProcessor::ValidateData
//! [`InstructionProcessor::InstructionData`]: crate::instruction::InstructionProcessor::InstructionData
//! [`InstructionProcessor::data_to_instruction_arg`]: crate::instruction::InstructionProcessor::data_to_instruction_arg
//! [`Instruction::Accounts`]: crate::instruction::Instruction::Accounts
//! [`FromAccounts::from_accounts`]: crate::account_argument::FromAccounts::from_accounts
//! [`InstructionProcessor::process`]: crate::instruction::InstructionProcessor::process
//! [`AccountArgument::write_back`]: crate::account_argument::AccountArgument::write_back

extern crate core;
extern crate self as cruiser;

#[macro_use]
mod macros;

pub mod account_types;
#[cfg(feature = "client")]
pub mod client;
pub mod compressed_numbers;
pub mod entrypoint;
pub mod indexer;
pub mod pda_seeds;
#[cfg(feature = "spl-token")]
pub mod spl;
pub mod types;
pub mod util;

mod account_info;
mod cpi;
mod generic_error;
mod impls;
mod traits;

pub use account_info::*;
pub use borsh;
pub use cpi::*;
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
#[cfg(feature = "spl-token")]
pub use spl_token;
pub use static_assertions;
pub use traits::error::CruiserResult;
pub use traits::*;

#[cfg(feature = "rand")]
pub use rand;
#[cfg(feature = "rand_chacha")]
pub use rand_chacha;
#[cfg(feature = "solana-client")]
pub use solana_client;
#[cfg(feature = "solana-program-test")]
pub use solana_program_test;
#[cfg(feature = "solana-sdk")]
pub use solana_sdk;
#[cfg(feature = "solana-transaction-status")]
pub use solana_transaction_status;
