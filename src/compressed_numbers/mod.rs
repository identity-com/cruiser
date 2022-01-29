mod byte_count;
mod zero_count;

pub use byte_count::*;
use std::mem::size_of;
pub use zero_count::*;

use borsh::{BorshDeserialize, BorshSerialize};

pub unsafe trait CompressedU64: Copy + BorshSerialize + BorshDeserialize + Eq {
    /// This function is required to have a const version. Will be added when const functions are in traits.
    fn from_u64(number: u64) -> Self;
    /// This function is required to have a const version. Will be added when const functions are in traits.
    fn into_u64(self) -> u64;
    fn num_bytes(self) -> usize;
}
unsafe impl CompressedU64 for u64 {
    fn from_u64(number: u64) -> Self {
        number
    }

    fn into_u64(self) -> u64 {
        self
    }

    fn num_bytes(self) -> usize {
        size_of::<Self>()
    }
}
