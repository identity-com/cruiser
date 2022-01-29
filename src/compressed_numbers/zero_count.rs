// use crate::compressed_numbers::CompressedU64;
// use borsh::{BorshDeserialize, BorshSerialize};
// use solana_generator::bytes_ext::ReadExt;
// use solana_program::program_memory::sol_memcpy;
// use std::io::{Read, Write};
// use std::mem::size_of;
//
// #[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
// pub struct ZeroCount<T>(T);
// impl ZeroCount<u64> {
//     const BYTES_NEEDED_LOOKUP_TABLE: &'static [u8] = &[
//         9, 9, 9, 9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 5,
//         5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1,
//         1, 1, 1, 1, 1,
//     ];
//     const MAX_BYTES_NEEDED: u8 = {
//         let mut max = u8::MIN;
//         let mut index = 0;
//         loop {
//             if index >= Self::BYTES_NEEDED_LOOKUP_TABLE.len() {
//                 break;
//             }
//             if Self::BYTES_NEEDED_LOOKUP_TABLE[index] > max {
//                 max = Self::BYTES_NEEDED_LOOKUP_TABLE[index];
//             }
//             index += 1;
//         }
//         max
//     };
//
//     const fn from_u64(number: u64) -> Self {
//         Self(number)
//     }
//
//     const fn into_u64(self) -> u64 {
//         self.0
//     }
// }
// unsafe impl CompressedU64 for ZeroCount<u64> {
//     fn from_u64(number: u64) -> Self {
//         Self::from_u64(number)
//     }
//
//     fn into_u64(self) -> u64 {
//         self.into_u64()
//     }
//
//     fn num_bytes(self) -> usize {
//         todo!()
//     }
// }
// impl BorshSerialize for ZeroCount<u64> {
//     fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
//         let zeros = self.0.leading_zeros() as usize;
//         let bytes_needed = Self::BYTES_NEEDED_LOOKUP_TABLE[zeros] as usize;
//         let mut bytes = [0; Self::MAX_BYTES_NEEDED as usize];
//         sol_memcpy(&mut bytes[1..], &self.0.to_be_bytes(), size_of::<u64>());
//         let start_byte = bytes.len() - bytes_needed;
//         bytes[start_byte] |= (1 << 7) >> (bytes_needed - 1);
//         writer.write_all(&bytes[start_byte..])
//     }
// }
// impl BorshDeserialize for ZeroCount<u64> {
//     fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
//         let first_byte = buf.read_u8()?;
//         let leading_zeros = first_byte.leading_zeros() as usize;
//         let mut bytes = [0; size_of::<u64>()];
//         let write_offset = if leading_zeros != 8 {
//             let write_offset = 7 - leading_zeros;
//             bytes[write_offset] = first_byte & !(1 << write_offset);
//             write_offset + 1
//         } else {
//             0
//         };
//         buf.read_exact(&mut bytes[write_offset..])?;
//         Ok(Self(u64::from_le_bytes(bytes)))
//     }
// }
//
// #[cfg(test)]
// mod test {
//     use super::*;
//     use rand::{thread_rng, Rng};
//     #[test]
//     fn serde_test() {
//         let mut rng = thread_rng();
//         for index in 0..u64::BITS as usize {
//             let val =
//                 ((rng.gen::<u64>() << index) >> index) & (1 << (size_of::<u64>() - index - 1));
//             let before = ZeroCount::from_u64(val);
//             let data = before.try_to_vec().unwrap();
//             let after = ZeroCount::try_from_slice(&data).unwrap();
//             assert_eq!(before, after);
//         }
//     }
// }
