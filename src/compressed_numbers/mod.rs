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
pub trait CompressedNumber<T = Self>: Copy + BorshSerialize + BorshDeserialize + Eq {
    /// This function is required to have a const version. Will be added when const functions are in traits.
    fn from_number(number: T) -> Self;
    /// This function is required to have a const version. Will be added when const functions are in traits.
    fn into_number(self) -> T;
    /// The number of bytes the compressed version will take
    fn num_bytes(self) -> usize;
    /// The max number of bytes the compressed version will take
    fn max_bytes() -> usize;
}

macro_rules! impl_compressed_for_prim {
    ($ty:ty) => {
        impl const CompressedNumber<$ty> for $ty {
            fn from_number(number: $ty) -> Self {
                number
            }

            fn into_number(self) -> $ty {
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

impl_compressed_for_prim!(u8);
impl_compressed_for_prim!(u16);
impl_compressed_for_prim!(u32);
impl_compressed_for_prim!(u64);
impl_compressed_for_prim!(u128);
impl_compressed_for_prim!(i8);
impl_compressed_for_prim!(i16);
impl_compressed_for_prim!(i32);
impl_compressed_for_prim!(i64);
impl_compressed_for_prim!(i128);
impl_compressed_for_prim!(());
impl_compressed_for_prim!(NonZeroU8);
impl_compressed_for_prim!(NonZeroU16);
impl_compressed_for_prim!(NonZeroU32);
impl_compressed_for_prim!(NonZeroU64);
impl_compressed_for_prim!(NonZeroU128);
impl_compressed_for_prim!(NonZeroI8);
impl_compressed_for_prim!(NonZeroI16);
impl_compressed_for_prim!(NonZeroI32);
impl_compressed_for_prim!(NonZeroI64);
impl_compressed_for_prim!(NonZeroI128);

impl const CompressedNumber<NonZeroU64> for NonZeroU8 {
    fn from_number(number: NonZeroU64) -> Self {
        match number.try_into() {
            Ok(value) => value,
            Err(_) => panic!("NonZeroU8 cannot be created from NonZeroU64 value"),
        }
    }

    fn into_number(self) -> NonZeroU64 {
        self.into()
    }

    fn num_bytes(self) -> usize {
        1
    }

    fn max_bytes() -> usize {
        1
    }
}
impl const CompressedNumber<NonZeroU64> for NonZeroU16 {
    fn from_number(number: NonZeroU64) -> Self {
        match number.try_into() {
            Ok(value) => value,
            Err(_) => panic!("NonZeroU16 cannot be created from NonZeroU64 value"),
        }
    }

    fn into_number(self) -> NonZeroU64 {
        self.into()
    }

    fn num_bytes(self) -> usize {
        2
    }

    fn max_bytes() -> usize {
        2
    }
}
impl const CompressedNumber<NonZeroU64> for NonZeroU32 {
    fn from_number(number: NonZeroU64) -> Self {
        match number.try_into() {
            Ok(value) => value,
            Err(_) => panic!("NonZeroU32 cannot be created from NonZeroU64 value"),
        }
    }

    fn into_number(self) -> NonZeroU64 {
        self.into()
    }

    fn num_bytes(self) -> usize {
        4
    }

    fn max_bytes() -> usize {
        4
    }
}
impl const CompressedNumber<u64> for u8 {
    fn from_number(number: u64) -> Self {
        match number.try_into() {
            Ok(value) => value,
            Err(_) => panic!("u8 cannot be created from u64 value"),
        }
    }

    fn into_number(self) -> u64 {
        self.into()
    }

    fn num_bytes(self) -> usize {
        1
    }

    fn max_bytes() -> usize {
        1
    }
}
impl const CompressedNumber<u64> for u16 {
    fn from_number(number: u64) -> Self {
        match number.try_into() {
            Ok(value) => value,
            Err(_) => panic!("u16 cannot be created from u64 value"),
        }
    }

    fn into_number(self) -> u64 {
        self.into()
    }

    fn num_bytes(self) -> usize {
        2
    }

    fn max_bytes() -> usize {
        2
    }
}
impl const CompressedNumber<u64> for u32 {
    fn from_number(number: u64) -> Self {
        match number.try_into() {
            Ok(value) => value,
            Err(_) => panic!("u32 cannot be created from u64 value"),
        }
    }

    fn into_number(self) -> u64 {
        self.into()
    }

    fn num_bytes(self) -> usize {
        4
    }

    fn max_bytes() -> usize {
        4
    }
}
