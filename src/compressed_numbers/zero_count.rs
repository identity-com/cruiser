// use crate::compressed_numbers::CompressedU64;
// use borsh::{BorshDeserialize, BorshSerialize};
// use solana_generator::bytes_ext::{ReadExt, WriteExt};
// use std::io::Write;
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
//         loop{
//             if index >= Self::BYTES_NEEDED_LOOKUP_TABLE.len(){
//                 break;
//             }
//             if Self::BYTES_NEEDED_LOOKUP_TABLE[index] > max{
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
// }
// impl BorshSerialize for ZeroCount<u64> {
//     fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
//         let zeros = self.0.leading_zeros() as usize;
//         let bytes_needed = Self::BYTES_NEEDED_LOOKUP_TABLE[zeros];
//         let mut bytes = [0; Self::MAX_BYTES_NEEDED];
//         solana_program::program_memory::sol_memcpy(&mut bytes[1..], &self.0.to_be_bytes(), size_of::<u64>());
//         let start_byte = bytes.len() - bytes_needed;
//         bytes[start_byte] |= (1 << 7) >> (bytes_needed - 1);
//         writer.write_all(&bytes[start_byte..])
//     }
// }
// impl BorshDeserialize for ZeroCount<u64> {
//     fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
//         let mut first_byte = buf.read_u8()?;
//         let mut leading_zeros = first_byte.leading_zeros();
//         let mut bytes = [0; size_of::<u64>()];
//         let mut current_byte = size_of::<u64>() - leading_zeros - 1;
//         if leading_zeros != 7{
//             b
//         }
//         let shift = (leading_zeros + 1) % 8;
//         let mut write_offset = 0;
//         if shift != 0 {
//             bytes[write_offset] = first_byte << shift;
//             write_offset += 1;
//         }
//         for (index, byte) in bytes_write.iter_mut().take(bytes_to_read).enumerate(){
//             if write_offset != 0
//             *byte =
//         }
//     }
// }
// 0 1 2 3 4 5 6 7
// n n n y y y y y
