use crate::compressed_numbers::CompressedNumber;
use borsh::{BorshDeserialize, BorshSerialize};
use cruiser::bytes_ext::{ReadExt, WriteExt};
use std::io::Write;
use std::mem::size_of;

/// A compressed number whose first
#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub struct ByteCount<T>(T);
impl<T> ByteCount<T> {
    const COUNT_BIT: u8 = 1 << 7;
}
impl ByteCount<u64> {
    const fn from_u64(number: u64) -> Self {
        Self(number)
    }

    const fn into_u64(self) -> u64 {
        self.0
    }

    const fn num_bytes(self) -> usize {
        if self.0 >= Self::COUNT_BIT as u64 {
            size_of::<u64>() - self.0.leading_zeros() as usize / 8 + 1
        } else {
            1
        }
    }
}
unsafe impl CompressedNumber for ByteCount<u64> {
    type Num = u64;

    #[inline]
    fn from_number(number: Self::Num) -> Self {
        Self::from_u64(number)
    }

    #[inline]
    fn into_number(self) -> Self::Num {
        self.into_u64()
    }

    #[inline]
    fn num_bytes(self) -> usize {
        self.num_bytes()
    }

    #[inline]
    fn max_bytes() -> usize {
        9
    }
}
impl BorshSerialize for ByteCount<u64> {
    fn serialize<W: Write>(&self, writer: &mut W) -> std::io::Result<()> {
        if self.0 > u64::from(!Self::COUNT_BIT) {
            let count = size_of::<u64>() - self.0.leading_zeros() as usize / 8;
            let bytes = self.0.to_le_bytes();
            writer.write_u8(count as u8 | Self::COUNT_BIT)?;
            writer.write_all(&bytes[..count])
        } else {
            writer.write_u8(self.0 as u8)
        }
    }
}
impl BorshDeserialize for ByteCount<u64> {
    fn deserialize(buf: &mut &[u8]) -> std::io::Result<Self> {
        let first = buf.read_u8()?;
        Ok(Self(if first & Self::COUNT_BIT > 0 {
            let count = first & !Self::COUNT_BIT;
            let mut bytes = [0; size_of::<u64>()];
            solana_program::program_memory::sol_memcpy(
                &mut bytes,
                *buf,
                (count as usize).min(size_of::<u64>()),
            );
            *buf = &buf[count as usize..];
            u64::from_le_bytes(bytes)
        } else {
            u64::from(first)
        }))
    }
}

#[cfg(test)]
mod test {
    use crate::compressed_numbers::ByteCount;
    use borsh::{BorshDeserialize, BorshSerialize};
    use rand::{thread_rng, Rng};

    #[test]
    fn random_test() {
        let mut rng = thread_rng();
        for _ in 0..1 << 18 {
            let num_bytes = rng.gen_range(1..=8u64);
            let val = rng.gen_range(0..=1 << (num_bytes * 8 - 1));
            let before = ByteCount::from_u64(val);
            let bytes = before.try_to_vec().unwrap();
            let after = ByteCount::try_from_slice(&bytes).unwrap_or_else(|error| {
                panic!(
                    "Error encountered: {}\n number: {:?}, bytes: {:?}",
                    error, before, bytes
                )
            });
            assert_eq!(before, after, "Bytes: {:?}", bytes);
            assert_eq!(val, after.into_u64());
        }
    }
}
