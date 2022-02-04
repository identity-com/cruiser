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
    /// The number of bytes the compressed version will take
    fn num_bytes(self) -> usize;
    /// The max number of bytes the compressed version will take
    fn max_bytes() -> usize;
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

    fn max_bytes() -> usize {
        size_of::<Self>()
    }
}
