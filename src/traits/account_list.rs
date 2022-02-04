use crate::compressed_numbers::CompressedU64;
use borsh::{BorshDeserialize, BorshSerialize};
pub use solana_generator_derive::AccountList;
use std::num::NonZeroU64;

pub trait AccountList: BorshSerialize + BorshDeserialize {
    type DiscriminantCompressed: CompressedU64;
}
/// # Safety
/// Implementor must guarantee that no two discriminates match
pub unsafe trait AccountListItem<T>: AccountList {
    fn discriminant() -> NonZeroU64;
    fn compressed_discriminant() -> Self::DiscriminantCompressed {
        Self::DiscriminantCompressed::from_u64(Self::discriminant().get())
    }
    fn from_account(account: T) -> Self;
    fn into_account(self) -> Result<T, Self>;
}
