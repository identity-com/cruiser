use solana_program::pubkey::Pubkey;

pub use cruiser_derive::AccountArgument;

use crate::{AccountInfo, GeneratorError, GeneratorResult, SolanaAccountMeta, SystemProgram};
use std::fmt::Debug;
use std::iter::FusedIterator;

/// A set of accounts that can be derived from an iterator over [`AccountInfo`]s and instruction data
///
/// # Included Derivations
/// ## [`Vec`]:
/// [`AccountArgument`] is implemented for [`Vec`]s over type `T` where `T` implements [`AccountArgument`].
/// Its [`InstructionArg`](AccountArgument::InstructionArg) is `(usize, T::InstructionArg)` and requires that `T::InstructionArg` implement [`Clone`].
/// The indexes of the tuple are:
/// 0. The size of the vector (`0` is acceptable)
/// 1. The instruction argument that will be copied to all indices
///
/// ## [`VecDeque`]:
/// Same as `Vec` implementation
///
/// ## `[T; N]`:
/// [`AccountArgument`] is implemented for all arrays `[T; N]` where `T` implements [`AccountArgument`].
/// It's instruction argument is `[T::InstructionArg; N]`.
/// Each index will be passed its corresponding argument.
pub trait AccountArgument: Sized {
    /// Writes the accounts back to the chain.
    /// - `program_id` is the current program's id.
    /// - `system_program` is an option reference to the system program's account info.
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()>;
    /// Adds all keys from this account to a given function.
    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()>;
    /// Collects all the keys in a [`Vec`].
    fn keys(&self) -> GeneratorResult<Vec<&'static Pubkey>> {
        let mut out = Vec::new();
        self.add_keys(|key| {
            out.push(key);
            Ok(())
        })?;
        Ok(out)
    }
}
/// Automatically derived with [`AccountArgument`]. Allows this set of accounts to be created from an argument `A`.
pub trait FromAccounts<A>: Sized + AccountArgument {
    /// Creates this argument from an [`AccountInfo`] iterator and [`InstructionArg`](AccountArgument::InstructionArg).
    /// - `program_id` is the current program's id.
    /// - `infos` is the iterator of [`AccountInfo`]s
    /// - `arg` is the [`InstructionArg`](AccountArgument::InstructionArg)
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: A,
    ) -> GeneratorResult<Self>;

    /// A hint as to the number of accounts that this will use when [`FromAccounts::from_accounts`] is called.
    /// Returns `(lower_bound, upper_bound)` where `lower_bound` is the minimum and `upper_bound` is the maximum or [`None`] if there is no maximum.
    ///
    /// Should only be used as an optimization hint, not relied on.
    ///
    /// The default return of `(0, None)` is valid for all although may not be as accurate as possible.
    // TODO: Make this const once const trait functions are stabilized
    // TODO: Figure out how to make this derivable
    #[must_use]
    fn accounts_usage_hint() -> (usize, Option<usize>) {
        (0, None)
    }
}
pub trait ValidateArgument<A>: Sized + AccountArgument {
    fn validate(&mut self, program_id: &'static Pubkey, arg: A) -> GeneratorResult<()>;
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

/// An account set that can be indexed by 0+ accounts at time with index `I`.
/// All should be infallible if `I` is [`()`].
pub trait MultiIndexable<I>: AccountArgument {
    /// Returns whether the account at index `indexer` is a signer.
    fn is_signer(&self, indexer: I) -> GeneratorResult<bool>;
    /// Returns whether the account at index `indexer` is writable.
    fn is_writable(&self, indexer: I) -> GeneratorResult<bool>;
    /// Returns whether the account at index `indexer`'s owner is `owner`.
    fn is_owner(&self, owner: &Pubkey, indexer: I) -> GeneratorResult<bool>;
}
/// An account set that can be indexed to a single account at a time with index `I`.
/// All should be infallible if `I` is [`()`].
pub trait SingleIndexable<I>: MultiIndexable<I> {
    /// Gets the account info at index `indexer`
    fn info(&self, indexer: I) -> GeneratorResult<&AccountInfo>;
    /// Turns the account at index `indexer` to a [`SolanaAccountMeta`]
    fn to_solana_account_meta(&self, indexer: I) -> GeneratorResult<SolanaAccountMeta> {
        let info = self.info(indexer)?;
        Ok(SolanaAccountMeta {
            pubkey: *info.key,
            is_signer: info.is_signer,
            is_writable: info.is_writable,
        })
    }
}

/// Infallible single access functions.
/// Relies on the infallibility of [`()`] for [`SingleIndexableAccountArgument`] and [`MultiIndexableAccountArgument`].
pub trait Single: SingleIndexable<()> {
    fn get_info(&self) -> &AccountInfo;
}
impl<T> Single for T
where
    T: SingleIndexable<()>,
{
    fn get_info(&self) -> &AccountInfo {
        self.info(()).expect("`()` info is not infallible!")
    }
}

/// Asserts that the account at index `indexer` is a signer.
pub fn assert_is_signer<I>(argument: &impl MultiIndexable<I>, indexer: I) -> GeneratorResult<()>
where
    I: Debug + Clone,
{
    if argument.is_signer(indexer.clone())? {
        Ok(())
    } else {
        Err(GeneratorError::AccountsSignerError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
        }
        .into())
    }
}

/// Asserts that the account at index `indexer` is writable.
pub fn assert_is_writable<I>(argument: &impl MultiIndexable<I>, indexer: I) -> GeneratorResult<()>
where
    I: Debug + Clone,
{
    if argument.is_writable(indexer.clone())? {
        Ok(())
    } else {
        Err(GeneratorError::AccountsWritableError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
        }
        .into())
    }
}

/// Asserts that the account at index `indexer` is a certain key.
pub fn assert_is_key<I>(
    argument: &impl SingleIndexable<I>,
    indexer: I,
    key: &Pubkey,
) -> GeneratorResult<()>
where
    I: Debug + Clone,
{
    let account = argument.info(indexer)?.key;
    if account == key {
        Ok(())
    } else {
        Err(GeneratorError::InvalidAccount {
            account: *account,
            expected: *key,
        }
        .into())
    }
}

/// Asserts that the account at index `indexer`'s owner is `owner`.
pub fn assert_is_owner<I>(
    argument: &impl MultiIndexable<I>,
    owner: &Pubkey,
    indexer: I,
) -> GeneratorResult<()>
where
    I: Debug + Clone,
{
    if argument.is_owner(owner, indexer.clone())? {
        Ok(())
    } else {
        Err(GeneratorError::AccountsOwnerError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
            owner: *owner,
        }
        .into())
    }
}
