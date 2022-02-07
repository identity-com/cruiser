use crate::compressed_numbers::CompressedNumber;
pub use solana_generator_derive::AccountList;
use std::num::NonZeroU64;

/// A list of all accounts used by a program.
pub trait AccountList {
    /// The compression algorithm
    type DiscriminantCompressed: CompressedNumber<Num = u64>;
}
/// Allows an account list to support an account type
///
/// # Safety
/// Implementor must guarantee that no two discriminates match
pub unsafe trait AccountListItem<T>: Sized + AccountList {
    /// The discriminant of the account type
    fn discriminant() -> NonZeroU64;
    /// The discriminant of the account type compressed
    #[inline]
    fn compressed_discriminant() -> Self::DiscriminantCompressed {
        Self::DiscriminantCompressed::from_number(Self::discriminant().get())
    }
    /// Creates a list item from this type
    fn from_account(account: T) -> Self;
    /// Turns the list into a type, returning self if it's not the proper type
    fn into_account(self) -> Result<T, Self>;
}
