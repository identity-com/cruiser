use crate::account_argument::AccountArgument;
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;
use std::iter::FusedIterator;

/// Allows an account argument to be made from the account iterator and data `Arg`.
/// `AI` is the [`AccountInfo`](crate::AccountInfo) type.
/// This is the first step in the instruction lifecycle.
pub trait FromAccounts<AI, Arg>: Sized + AccountArgument<AI> {
    /// Creates this argument from an `AI` iterator and data `Arg`.
    /// - `program_id` is the current program's id.
    /// - `infos` is the iterator of `AI`s
    /// - `arg` is the data argument
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: Arg,
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
    fn accounts_usage_hint(arg: &Arg) -> (usize, Option<usize>);
}

/// A globing trait for an account info iterator
pub trait AccountInfoIterator<AI>:
    Iterator<Item = AI> + DoubleEndedIterator + FusedIterator
{
}
impl<AI, T> AccountInfoIterator<AI> for T where
    T: Iterator<Item = AI> + DoubleEndedIterator + FusedIterator
{
}
