//! Small size vectors for additional space savings than the

use crate::{
    AccountArgument, GeneratorError, GeneratorResult, Pubkey, SystemProgram, USIZE_DECLARATION,
};
use borsh::schema::{Declaration, Definition, Fields};
use borsh::{BorshDeserialize, BorshSchema, BorshSerialize};
use solana_generator::bytes_ext::{ReadExt, WriteExt};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::io::{ErrorKind, Read, Write};
use std::mem::size_of;
use std::ops::Deref;

macro_rules! small_vec {
    ($ident:ident, $ty:ty, $write:ident, $read:ident, $docs:expr) => {
        #[derive(Debug, Clone)]
        #[doc=$docs]
        pub struct $ident<T>(Vec<T>);
        impl<T> TryFrom<Vec<T>> for $ident<T> {
            type Error = GeneratorError<'static>;

            fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
                if value.len() <= <$ty>::MAX as usize {
                    Ok(Self(value))
                } else {
                    Err(GeneratorError::SizeInvalid {
                        min: 0,
                        max: <$ty>::MAX as usize,
                        value: value.len(),
                    })
                }
            }
        }
        impl<T> From<$ident<T>> for Vec<T> {
            fn from(from: $ident<T>) -> Self {
                from.0
            }
        }
        impl<T> Deref for $ident<T> {
            type Target = Vec<T>;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }
        impl<T> BorshSerialize for $ident<T>
        where
            T: BorshSerialize,
        {
            fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
                writer.$write(self.len() as $ty)?;
                for val in self.iter() {
                    val.serialize(writer)?;
                }
                Ok(())
            }
        }
        impl<T> BorshDeserialize for $ident<T>
        where
            T: BorshDeserialize,
        {
            fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
                let len = buf.$read()?;
                let mut out = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    out.push(T::deserialize(buf)?);
                }
                Ok(Self(out))
            }
        }
        impl<T> BorshSchema for $ident<T>
        where
            T: BorshSchema,
        {
            fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
                Self::add_definition(
                    Self::declaration(),
                    Definition::Sequence {
                        elements: T::declaration(),
                    },
                    definitions,
                );
                T::add_definitions_recursively(definitions);
            }

            fn declaration() -> Declaration {
                stringify!($ident).to_string()
            }
        }
        impl<T> AccountArgument for $ident<T>
        where
            T: AccountArgument,
        {
            fn write_back(
                self,
                program_id: Pubkey,
                system_program: Option<&SystemProgram>,
            ) -> GeneratorResult<()> {
                for val in self.0 {
                    val.write_back(program_id, system_program)?;
                }
                Ok(())
            }

            fn add_keys(
                &self,
                mut add: impl FnMut(Pubkey) -> GeneratorResult<()>,
            ) -> GeneratorResult<()> {
                for val in &self.0 {
                    val.add_keys(&mut add)?;
                }
                Ok(())
            }
        }
    };
}

small_vec!(
    Vec8,
    u8,
    write_u8,
    read_u8,
    "A vector with max size in a u8"
);
small_vec!(
    Vec16,
    u16,
    write_u16_le,
    read_u16_le,
    "A vector with max size in a u16"
);

const fn num_bytes_for_len(len: usize) -> Option<usize> {
    let mut i = 1;
    while i <= size_of::<usize>() {
        // Every byte needs one bit to signify and 1 starts at 1 bit over so 7 * byte count - 1
        if len < 1 << (7 * i - 1) {
            return Some(i);
        }
        i += 1;
    }
    None
}

/// A compressed version of `T`
pub trait Compressed<T>: Sized {
    /// The error returned if compression fails
    type Error;

    /// Compresses the `T`
    fn compress(value: T) -> Result<Self, Self::Error>;
    /// Uncompresses this back into a `T`
    fn uncompressed(self) -> T;
}

/// A [`usize`] compressed by the first `1` bit being at the length of the total number of bytes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct CompressedUsize(usize);
impl CompressedUsize {
    /// The maximum valid value for this
    pub const MAX_SIZE: usize = 1 << (7 * size_of::<usize>() - 1);

    /// Trys to create this from a [`usize`]
    pub fn from_usize(num: usize) -> Result<Self, GeneratorError<'static>> {
        if num <= Self::MAX_SIZE {
            Ok(Self(num))
        } else {
            Err(GeneratorError::SizeInvalid {
                min: 0,
                max: Self::MAX_SIZE,
                value: num,
            })
        }
    }
}
impl Compressed<usize> for CompressedUsize {
    type Error = GeneratorError<'static>;

    fn compress(value: usize) -> Result<Self, Self::Error> {
        Self::from_usize(value)
    }

    fn uncompressed(self) -> usize {
        self.0
    }
}
impl BorshSerialize for CompressedUsize {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let self_val = self.0;
        let num_bytes = num_bytes_for_len(self_val).ok_or_else(|| {
            std::io::Error::new(
                ErrorKind::InvalidData,
                GeneratorError::SizeInvalid {
                    min: 0,
                    max: Self::MAX_SIZE,
                    value: self_val,
                },
            )
        })?;
        let mut length_bytes = self_val.to_be_bytes();
        let final_bytes = &mut length_bytes[size_of::<usize>() - num_bytes..];
        final_bytes[(num_bytes - 1) / 8] |= 1u8 << (7 - num_bytes % 8);
        writer.write_all(final_bytes)?;
        Ok(())
    }
}
impl BorshDeserialize for CompressedUsize {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let mut size = 1usize;
        let mut length_byte = buf.read_u8()?;
        let mut read = 1usize;
        while length_byte & 1 << (7 - size % 8) == 0 {
            size += 1;
            if size % 8 == 1 {
                length_byte = buf.read_u8()?;
                read += 1;
            }
        }

        let mut bytes = [0; size_of::<usize>()];
        bytes[size_of::<usize>() - size] = length_byte & !(1u8 << (7 - size % 8));
        buf.read_exact(&mut bytes[size_of::<usize>() - (size - read)..])?;
        Ok(Self(usize::from_be_bytes(bytes)))
    }
}
impl BorshSchema for CompressedUsize {
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        Self::add_definition(
            Self::declaration(),
            Definition::Struct {
                fields: Fields::UnnamedFields(vec![USIZE_DECLARATION
                    .expect("usize invalid length")
                    .to_string()]),
            },
            definitions,
        );
    }

    fn declaration() -> Declaration {
        "CompressedUsize".to_string()
    }
}

/// A Vector whose serialized length size is based on `L`
#[derive(Debug)]
pub struct AdaptiveVec<L, T>
where
    L: Compressed<usize>,
{
    length: L,
    data: Vec<T>,
}
impl<L, T> AdaptiveVec<L, T>
where
    L: Compressed<usize>,
{
    /// Creates an [`AdaptiveVec`] from a [`Vec`]
    pub fn from_vec(value: Vec<T>) -> Result<Self, L::Error> {
        Ok(Self {
            length: L::compress(value.len())?,
            data: value,
        })
    }
}
impl<L, T> From<AdaptiveVec<L, T>> for Vec<T>
where
    L: Compressed<usize>,
{
    fn from(from: AdaptiveVec<L, T>) -> Self {
        from.data
    }
}
impl<L, T> Deref for AdaptiveVec<L, T>
where
    L: Compressed<usize>,
{
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<L, T> BorshSerialize for AdaptiveVec<L, T>
where
    L: Compressed<usize> + BorshSerialize,
    T: BorshSerialize,
{
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        self.length.serialize(writer)?;
        for val in self.iter() {
            val.serialize(writer)?;
        }
        Ok(())
    }
}
impl<L, T> BorshDeserialize for AdaptiveVec<L, T>
where
    L: Compressed<usize> + BorshDeserialize + Clone,
    T: BorshDeserialize,
{
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let length = L::deserialize(buf)?;
        let length_uncompressed = length.clone().uncompressed();
        let mut data = Vec::with_capacity(length_uncompressed);
        for _ in 0..length_uncompressed {
            data.push(T::deserialize(buf)?);
        }
        Ok(Self { length, data })
    }
}
impl<L, T> BorshSchema for AdaptiveVec<L, T>
where
    L: Compressed<usize>,
    T: BorshSchema,
{
    fn add_definitions_recursively(definitions: &mut HashMap<Declaration, Definition>) {
        Self::add_definition(
            Self::declaration(),
            Definition::Sequence {
                elements: T::declaration(),
            },
            definitions,
        );
        T::add_definitions_recursively(definitions);
    }

    fn declaration() -> Declaration {
        stringify!(AdaptiveVec).to_string()
    }
}
impl<L, T> AccountArgument for AdaptiveVec<L, T>
where
    L: Compressed<usize>,
    T: AccountArgument,
{
    fn write_back(
        self,
        program_id: Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        for val in self.data {
            val.write_back(program_id, system_program)?;
        }
        Ok(())
    }

    fn add_keys(&self, mut add: impl FnMut(Pubkey) -> GeneratorResult<()>) -> GeneratorResult<()> {
        for val in &self.data {
            val.add_keys(&mut add)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::types::small_vec::{Compressed, CompressedUsize};
    use borsh::{BorshDeserialize, BorshSerialize};
    use rand::{thread_rng, Rng};

    #[test]
    fn adaptive_test() {
        for _ in 0..1 << 17 {
            let in_length: usize = thread_rng().gen_range(0..=CompressedUsize::MAX_SIZE);
            let compressed = CompressedUsize::compress(in_length).expect("Could not compress");
            let mut bytes = Vec::new();
            compressed.serialize(&mut bytes).expect("Could not write");
            let mut bytes_read = bytes.as_slice();
            let length = CompressedUsize::deserialize(&mut bytes_read).expect("Could not read");
            assert_eq!(length.uncompressed(), in_length);
        }
    }
}
