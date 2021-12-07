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
