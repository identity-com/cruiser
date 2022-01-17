use crate::compressed_numbers::CompressedU64;
use crate::Account;
use borsh::{BorshDeserialize, BorshSerialize};
use std::num::NonZeroU64;

pub trait AccountList: BorshSerialize + BorshDeserialize {
    type DiscriminantCompressed: CompressedU64;
}
/// # Safety
/// Implementor must guarantee that no two discriminates match
pub unsafe trait AccountListItem<T>: AccountList
where
    T: Account,
{
    fn discriminant() -> NonZeroU64;
    fn compressed_discriminant() -> Self::DiscriminantCompressed {
        Self::DiscriminantCompressed::from_u64(Self::discriminant().get())
    }
    fn from_account(account: T) -> Self;
    fn into_account(self) -> Result<T, Self>;
}
