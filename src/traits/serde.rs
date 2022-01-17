use crate::{GeneratorError, GeneratorResult};
use borsh::{BorshDeserialize, BorshSerialize};
use std::mem::{size_of, take};

pub trait SerializeBitOffset: BorshSerialize {
    fn serialize_bit_offset(
        &self,
        bytes: &mut &mut [u8],
        bit_offset: &mut u8,
    ) -> GeneratorResult<()>;
}

pub trait DeserializeBitOffset: BorshDeserialize {
    fn deserialize_bit_offset(bytes: &mut &[u8], bit_offset: &mut u8) -> GeneratorResult<Self>;
}

impl SerializeBitOffset for () {
    fn serialize_bit_offset(
        &self,
        _bytes: &mut &mut [u8],
        _bit_offset: &mut u8,
    ) -> GeneratorResult<()> {
        Ok(())
    }
}
impl DeserializeBitOffset for () {
    fn deserialize_bit_offset(_bytes: &mut &[u8], _bit_offset: &mut u8) -> GeneratorResult<Self> {
        Ok(())
    }
}

impl<T1, T2> SerializeBitOffset for (T1, T2)
where
    T1: SerializeBitOffset,
    T2: SerializeBitOffset,
{
    fn serialize_bit_offset(
        &self,
        bytes: &mut &mut [u8],
        bit_offset: &mut u8,
    ) -> GeneratorResult<()> {
        self.0.serialize_bit_offset(bytes, bit_offset)?;
        self.1.serialize_bit_offset(bytes, bit_offset)
    }
}
impl<T1, T2> DeserializeBitOffset for (T1, T2)
where
    T1: DeserializeBitOffset,
    T2: DeserializeBitOffset,
{
    fn deserialize_bit_offset(bytes: &mut &[u8], bit_offset: &mut u8) -> GeneratorResult<Self> {
        Ok((
            T1::deserialize_bit_offset(bytes, bit_offset)?,
            T2::deserialize_bit_offset(bytes, bit_offset)?,
        ))
    }
}

macro_rules! impl_serde_for_prim_num {
    (all $($ty:ty),+) => {
        $(impl_serde_for_prim_num!($ty);)+
    };
    ($ty:ty) => {
        impl SerializeBitOffset for $ty {
            fn serialize_bit_offset(
                &self,
                bytes: &mut &mut [u8],
                bit_offset: &mut u8,
            ) -> GeneratorResult<()>{
                let number_bytes = self.to_le_bytes();
                let bytes_needed = number_bytes.len() + 1.min(*bit_offset as usize);
                if bytes.len() < bytes_needed {
                    return Err(GeneratorError::NotEnoughData {
                        needed: bytes_needed,
                        remaining: bytes.len(),
                    }
                    .into());
                }
                for (index, number_byte) in number_bytes.iter().enumerate() {
                    bytes[index] |= number_byte >> *bit_offset;
                    if bytes.len() > index + 1 {
                        bytes[index + 1] = number_byte << (8 - *bit_offset);
                    }
                }
                let (_, b) = take(bytes).split_at_mut(number_bytes.len());
                *bytes = b;
                Ok(())
            }
        }
        impl DeserializeBitOffset for $ty {
            fn deserialize_bit_offset(
                bytes: &mut &[u8],
                bit_offset: &mut u8,
            ) -> GeneratorResult<Self> {
                let bytes_needed = size_of::<Self>() + 1.min(*bit_offset as usize);
                if bytes.len() < bytes_needed {
                    return Err(GeneratorError::NotEnoughData {
                        needed: bytes_needed,
                        remaining: bytes.len(),
                    }
                    .into());
                }
                let mut number_bytes = [0; size_of::<Self>()];
                for index in 0..bytes_needed {
                    number_bytes[index] |= bytes[index] << *bit_offset;
                    if *bit_offset != 0 {
                        number_bytes[index] |= bytes[index + 1] >> (8 - *bit_offset);
                    }
                }
                Ok(Self::from_le_bytes(number_bytes))
            }
        }
    };
}
impl_serde_for_prim_num!(all u8, u16, u32, u64, u128, i8, i16, i32, i64, i128);
