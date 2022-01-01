use crate::compressed_numbers::CompressedU64;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_generator::bytes_ext::{ReadExt, WriteExt};
use std::io::Write;
use std::mem::size_of;

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ZeroCount<T>(T);
impl ZeroCount<u64> {
    const BYTES_NEEDED_LOOKUP_TABLE: &'static [u8] = &[
        9, 9, 9, 9, 9, 9, 9, 9, 8, 8, 8, 8, 8, 8, 8, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 5,
        5, 5, 5, 5, 5, 5, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1,
        1, 1, 1, 1, 1,
    ];
    const SHIFT_LOOKUP_TABLE: &'static [u8] = &[
        0, 0, 0, 0, 0, 0, 0, 0, 7, 7, 7, 7, 7, 7, 7, 7, 6, 6, 6, 6, 6, 6, 6, 6, 5, 5, 5, 5, 5, 5,
        5, 5, 4, 4, 4, 4, 4, 4, 4, 4, 3, 3, 3, 3, 3, 3, 3, 3, 2, 2, 2, 2, 2, 2, 2, 2, 1, 1, 1, 1,
        1, 1, 1, 1, 1,
    ];

    const fn from_u64(number: u64) -> Self {
        Self(number)
    }

    const fn into_u64(self) -> u64 {
        self.0
    }
}
unsafe impl CompressedU64 for ZeroCount<u64> {
    fn from_u64(number: u64) -> Self {
        Self::from_u64(number)
    }

    fn into_u64(self) -> u64 {
        self.into_u64()
    }
}
impl BorshSerialize for ZeroCount<u64> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        let zeros = self.0.leading_zeros() as usize;
        let bytes_needed = Self::BYTES_NEEDED_LOOKUP_TABLE[zeros];
        let shift = Self::SHIFT_LOOKUP_TABLE[zeros];
        let mut number = self.0 << shift;
        if shift != 0 {
            number |= 1 << (shift - 1);
        } else {
            writer.write_u8(0)?;
        }
        writer.write_all(&number.to_le_bytes()[..bytes_needed as usize])
    }
}
impl BorshDeserialize for ZeroCount<u64> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let mut byte = buf.read_u8()?;
        let mut bytes_to_read = 0;
        let mut leading_zeros = 0;
        loop {
            leading_zeros = byte.leading_zeros();
            bytes_to_read += leading_zeros;
            if leading_zeros != 8 {
                byte = buf.read_u8()?;
            } else {
                break;
            }
        }
        let mut bytes = [0; 8];
        let bytes_write = &mut bytes;
        let shift = (bytes_to_read + 1) % 8;
        if leading_zeros != 7 {
            bytes_write.
        }
    }
}
