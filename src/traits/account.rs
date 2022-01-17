use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
pub use solana_generator_derive::Account;

/// Data that can be stored within an account
pub trait Account: BorshSerialize + BorshDeserialize + BorshSchema {}
