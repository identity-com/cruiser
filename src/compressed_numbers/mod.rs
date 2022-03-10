//! Numbers that can be decompressed/compressed on read/write

use std::mem::size_of;
use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroU128, NonZeroU16,
    NonZeroU32, NonZeroU64, NonZeroU8,
};

use borsh::{BorshDeserialize, BorshSerialize};

pub use byte_count::*;
pub use zero_count::*;

mod byte_count;
mod zero_count;

/// Represents a u64 that is compressed and decompressed on reading/writing from/to bytes
///
/// # Safety
/// This trait must ensure that all byte sizes are correct
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

macro_rules! impl_compressed_for_prim {
    (all: $($ty:ty),+ $(,)?) => {
        $(impl_compressed_for_prim!($ty);)+
    };
    ($ty:ty) => {
        unsafe impl CompressedNumber for $ty {
            type Num = $ty;

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
    };
}

impl_compressed_for_prim!(
    all: u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    (),
    NonZeroU8,
    NonZeroU16,
    NonZeroU32,
    NonZeroU64,
    NonZeroU128,
    NonZeroI8,
    NonZeroI16,
    NonZeroI32,
    NonZeroI64,
    NonZeroI128,
);
