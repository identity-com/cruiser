//! Numbers that can be decompressed/compressed on read/write

mod byte_count;
mod zero_count;

pub use byte_count::*;
use std::mem::size_of;
pub use zero_count::*;

use borsh::{BorshDeserialize, BorshSerialize};

/// Represents a u64 that is compressed and decompressed on reading/writing from/to bytes
///
/// # Safety
/// This trait must ensure that
pub unsafe trait CompressedNumber: Copy + BorshSerialize + BorshDeserialize + Eq {
    /// The number that is represented
    type Num;
    /// This function is required to have a const version. Will be added when const functions are in traits.
    fn from_number(number: Self::Num) -> Self;
    /// This function is required to have a const version. Will be added when const functions are in traits.
    fn into_number(self) -> Self::Num;
    /// The number of bytes the compressed version will take
    fn num_bytes(self) -> usize;
    /// The max number of bytes the compressed version will take
    fn max_bytes() -> usize;
}
unsafe impl CompressedNumber for u64 {
    type Num = u64;

    fn from_number(number: Self::Num) -> Self {
        number
    }

    fn into_number(self) -> Self::Num {
        self
    }

    fn num_bytes(self) -> usize {
        size_of::<Self>()
    }

    fn max_bytes() -> usize {
        size_of::<Self>()
    }
}
