use crate::account_argument::AccountArgument;
use crate::{AccountInfo, CruiserResult};
use solana_program::pubkey::Pubkey;
use std::iter::FusedIterator;

/// Allows an account argument to be made from the account iterator and data `A`.
/// This is the first step in the instruction lifecycle.
pub trait FromAccounts<A>: Sized + AccountArgument {
    /// Creates this argument from an [`AccountInfo`] iterator and data `A`.
    /// - `program_id` is the current program's id.
    /// - `infos` is the iterator of [`AccountInfo`]s
    /// - `arg` is the data argument
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: A,
    ) -> CruiserResult<Self>;

    /// A hint as to the number of accounts that this will use when [`FromAccounts::from_accounts`] is called.
    /// Returns `(lower_bound, upper_bound)` where `lower_bound` is the minimum and `upper_bound` is the maximum or [`None`] if there is no maximum.
    ///
    /// Should only be used as an optimization hint, not relied on.
    ///
    /// A default return of `(0, None)` is valid for all although may not be as accurate as possible.
    // TODO: Make this const once const trait functions are stabilized
    // TODO: Figure out how to make this derivable
    #[must_use]
    fn accounts_usage_hint(arg: &A) -> (usize, Option<usize>);
}

/// A globing trait for an account info iterator
pub trait AccountInfoIterator:
    Iterator<Item = AccountInfo> + DoubleEndedIterator + FusedIterator
{
}
impl<T> AccountInfoIterator for T where
    T: Iterator<Item = AccountInfo> + DoubleEndedIterator + FusedIterator
{
}
