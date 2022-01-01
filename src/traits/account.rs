use crate::discriminant::Discriminant;
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
pub use solana_generator_derive::Account;
use solana_program::pubkey::Pubkey;

/// Data that can be stored within an account
pub trait Account: BorshSerialize + BorshDeserialize + BorshSchema {
    /// The discriminant for this account.
    /// A given discriminant should not be duplicated or your program will be open to a confusion attack.
    /// All Discriminants for the range [100, 127] are reserved for system implementations.
    const DISCRIMINANT: Discriminant;
}
macro_rules! impl_account {
    ($ty:ty, $expr:expr) => {
        impl Account for $ty {
            const DISCRIMINANT: Discriminant = Discriminant::from_u64($expr);
        }
    };
}

impl_account!(String, 124);
impl_account!(Pubkey, 125);
impl_account!(Vec<Pubkey>, 126);
impl_account!(Vec<u8>, 127);
