//! [`Discriminant`] type and relevant extras

use std::borrow::Cow;
use std::io::{ErrorKind, Read, Write};
use std::ops::Deref;

use crate::{GeneratorError, GeneratorResult};
use borsh::{BorshDeserialize, BorshSerialize};

/// A compressed discriminant that uses 1 bytes for 1 sized arrays with values 0-127 or one extra byte for arrays of size 1-127 (1 sized arrays only with values 128-255).
/// This is accomplished by having the most significant bit of the first byte being a flag whether the first byte is the value or number of remaining values.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Discriminant {
    value: u64,
    length: u8,
}
impl Discriminant {
    /// The bit that determines if this is sized
    pub const SIZED_BIT: u8 = 1 << 7;

    /// Creates a discriminant from an array
    pub const fn from_u64(value: u64) -> Self {
        Self {
            value,
            length: Self::discriminant_serialized_length(value),
        }
    }

    pub const fn value(&self) -> u64 {
        self.value
    }

    pub const fn length(&self) -> u8 {
        self.length
    }

    /// The length of this discriminant when it is serialized
    const fn discriminant_serialized_length(value: u64) -> u8 {
        let mut leading_zero_bytes = 0u8;
        let bytes = value.to_be_bytes();
        while (leading_zero_bytes as usize) < bytes.len() && bytes[leading_zero_bytes as usize] == 0
        {
            leading_zero_bytes += 1;
        }
        if leading_zero_bytes >= 7 && value.to_be_bytes()[0] & Self::SIZED_BIT == 0 {
            1
        } else {
            1 + 8 - leading_zero_bytes
        }
    }
}
impl BorshSerialize for Discriminant {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if self.length == 1 {
            writer.write_all(&self.value.to_be_bytes()[0..1])?;
        } else {
            writer.write_all(&[self.length & Self::SIZED_BIT])?;
            writer.write_all(&self.value.to_be_bytes()[0..self.length as usize])?;
        }
        Ok(())
    }
}
impl BorshDeserialize for Discriminant {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let mut first = [0];
        buf.read_exact(&mut first)?;
        if first[0] & Self::SIZED_BIT == 0 {
            Ok(Self::from_u64(first[0] as u64))
        } else {
            let length = (first[0] & !Self::SIZED_BIT) as usize;
            let mut out = [0; 8];
            buf.read_exact(&mut out[0..length])?;
            Ok(Self::from_u64(u64::from_be_bytes(out)))
        }
    }
}

// #[cfg(test)]
// mod test {
//     use borsh::{BorshDeserialize, BorshSerialize};
//     use rand::{thread_rng, Rng};
//
//     use crate::discriminant::Discriminant;
//
//     #[test]
//     fn discriminant_borsh_single_test() {
//         let mut rng = thread_rng();
//         for _ in 0..128 {
//             let data = [rng.gen()];
//             let discriminant = Discriminant::from_array(&data);
//             let bytes = BorshSerialize::try_to_vec(&discriminant)
//                 .unwrap_or_else(|_| panic!("Could not serialize: {:?}", data));
//             let de_discriminant = BorshDeserialize::try_from_slice(&bytes)
//                 .unwrap_or_else(|_| panic!("Could not deserialize: {:?}", bytes));
//             assert_eq!(discriminant, de_discriminant);
//         }
//     }
//
//     #[test]
//     fn discriminant_borsh_test() {
//         let mut rng = thread_rng();
//         for _ in 0..128 {
//             let length = rng.gen_range(2..=16);
//             let mut data = vec![0; length];
//             for data_item in &mut data {
//                 *data_item = rng.gen();
//             }
//             let discriminant = Discriminant::from_array(&data);
//             let bytes = BorshSerialize::try_to_vec(&discriminant)
//                 .unwrap_or_else(|_| panic!("Could not serialize: {:?}", data));
//             let de_discriminant = BorshDeserialize::try_from_slice(&bytes)
//                 .unwrap_or_else(|_| panic!("Could not deserialize: {:?}", bytes));
//             assert_eq!(discriminant, de_discriminant);
//         }
//     }
// }
