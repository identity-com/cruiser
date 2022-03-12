//! Small size vectors for additional space savings than the. Still experimental.

use std::convert::TryFrom;
use std::io::Write;
use std::ops::{Deref, Index, IndexMut};

use borsh::{BorshDeserialize, BorshSerialize};

use crate::account_argument::AccountArgument;
use crate::util::bytes_ext::{ReadExt, WriteExt};
use crate::{CruiserError, CruiserResult, Pubkey};

macro_rules! small_vec {
    ($ident:ident, $ty:ty, $write:ident, $read:ident, $docs:expr) => {
        #[derive(Debug, Clone, PartialEq, Eq)]
        #[doc=$docs]
        pub struct $ident<T>(Vec<T>);
        impl<T> TryFrom<Vec<T>> for $ident<T> {
            type Error = CruiserError;

            fn try_from(value: Vec<T>) -> Result<Self, Self::Error> {
                if <$ty>::try_from(value.len()).is_ok() {
                    Ok(Self(value))
                } else {
                    Err(CruiserError::SizeInvalid {
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
        impl<T> Index<usize> for $ident<T> {
            type Output = <Vec<T> as Index<usize>>::Output;

            fn index(&self, index: usize) -> &Self::Output {
                self.0.index(index)
            }
        }
        impl<T> IndexMut<usize> for $ident<T> {
            fn index_mut(&mut self, index: usize) -> &mut Self::Output {
                self.0.index_mut(index)
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
        impl<T> AccountArgument for $ident<T>
        where
            T: AccountArgument,
        {
            fn write_back(self, program_id: &'static Pubkey) -> CruiserResult<()> {
                for val in self.0 {
                    val.write_back(program_id)?;
                }
                Ok(())
            }

            fn add_keys(
                &self,
                mut add: impl FnMut(&'static Pubkey) -> CruiserResult<()>,
            ) -> CruiserResult<()> {
                for val in &self.0 {
                    val.add_keys(&mut add)?;
                }
                Ok(())
            }
        }
        impl<T> IntoIterator for $ident<T> {
            type Item = <Vec<T> as IntoIterator>::Item;
            type IntoIter = <Vec<T> as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                self.0.into_iter()
            }
        }
        impl<'a, T> IntoIterator for &'a $ident<T> {
            type Item = <&'a Vec<T> as IntoIterator>::Item;
            type IntoIter = <&'a Vec<T> as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&self.0).into_iter()
            }
        }
        impl<'a, T> IntoIterator for &'a mut $ident<T> {
            type Item = <&'a mut Vec<T> as IntoIterator>::Item;
            type IntoIter = <&'a mut Vec<T> as IntoIterator>::IntoIter;

            fn into_iter(self) -> Self::IntoIter {
                (&mut self.0).into_iter()
            }
        }
        impl<T> Default for $ident<T> {
            fn default() -> Self {
                Self(vec![])
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

#[cfg(test)]
mod test {
    use std::convert::TryInto;

    use rand::{thread_rng, Rng};

    use super::*;

    #[test]
    fn vec8_test() {
        let mut rand = thread_rng();
        for len in u8::MIN..u8::MAX {
            let mut vec = vec![0u8; len as usize];
            for val in &mut vec {
                *val = rand.gen();
            }
            let small_vec: Vec8<_> = vec
                .try_into()
                .unwrap_or_else(|_| panic!("Could not convert vec of length `{}`", len));
            let bytes = BorshSerialize::try_to_vec(&small_vec).expect("Could not serialize");
            assert_eq!(bytes.len(), len as usize + 1);
            let deserialized =
                BorshDeserialize::try_from_slice(&bytes).expect("Could not deserialize");
            assert_eq!(small_vec, deserialized);
        }
    }

    #[test]
    fn vec16_test() {
        let mut rand = thread_rng();
        for len in (u16::MIN..u16::MAX).step_by(u16::MAX as usize / 157) {
            let mut vec = vec![0u8; len as usize];
            for val in &mut vec {
                *val = rand.gen();
            }
            let small_vec: Vec16<_> = vec
                .try_into()
                .unwrap_or_else(|_| panic!("Could not convert vec of length `{}`", len));
            let bytes = BorshSerialize::try_to_vec(&small_vec).expect("Could not serialize");
            assert_eq!(bytes.len(), len as usize + 2);
            let deserialized =
                BorshDeserialize::try_from_slice(&bytes).expect("Could not deserialize");
            assert_eq!(small_vec, deserialized);
        }
    }
}
