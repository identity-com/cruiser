use std::fmt::Debug;
use std::num::NonZeroU64;

use solana_program::pubkey::Pubkey;
use strum::EnumDiscriminants;

use crate::error::Error;

/// General errors issued by the generator.
#[derive(Clone, Debug, Error, EnumDiscriminants)]
#[error(start = 0)]
pub enum GenericError {
    /// Custom error message for infrequent one-off errors
    #[error_msg("{}", error)]
    Custom {
        /// The error message to print
        error: String,
    },
    /// Error for invalid sysvar
    #[error_msg("`{:?}` is an invalid sysvar", actual)]
    InvalidSysVar {
        /// The invalid sysvar
        actual: Pubkey,
    },
    /// Discriminant mismatch for accounts. Usually caused by passing the wrong account for a slot
    #[error_msg(
        "Mismatched Discriminant for account `{}`. Received: `{:?}`, Expected: `{:?}`",
        account,
        received,
        expected
    )]
    MismatchedDiscriminant {
        /// The account that has the discriminant mismatch
        account: Pubkey,
        /// The discriminant of the account
        received: u64,
        /// The discriminant that was expected
        expected: NonZeroU64,
    },
    /// Accounts are either writable when should not be or not writable when should be depending on the indexer
    #[error_msg(
        "Accounts writable error for accounts `{:?}` with indexer `{}`",
        accounts,
        indexer
    )]
    AccountsWritableError {
        /// The accounts that are indexed
        accounts: Vec<Pubkey>,
        /// The index of the accounts
        indexer: String,
    },
    /// Account is not writable when should be
    #[error_msg("Cannot write to account `{}` when should be able to", account)]
    CannotWrite {
        /// The account that is not writable
        account: Pubkey,
    },
    /// Accounts are either singing when should not be or not signing when should be depending on the indexer
    #[error_msg(
        "Accounts signer error for accounts `{:?}` with indexer `{}`",
        accounts,
        indexer
    )]
    AccountsSignerError {
        /// The accounts that are indexed
        accounts: Vec<Pubkey>,
        /// The index of the accounts
        indexer: String,
    },
    /// Account is not a signer when should be
    #[error_msg("Account `{}` is not signer when should be", account)]
    AccountIsNotSigner {
        /// Account that is not a signer
        account: Pubkey,
    },
    /// System program is missing when required.
    #[error_msg("Missing SystemProgram")]
    MissingSystemProgram,
    /// Account init size is not large enough
    #[error_msg("Not enough space for initialization of account `{}`. Space Given: `{}`, Space Needed: `{}`", account, space_given, space_needed)]
    NotEnoughSpaceInit {
        /// The account that would have been initialized
        account: Pubkey,
        /// The space the account was given
        space_given: u64,
        /// The space the account needed
        space_needed: u64,
    },
    /// Account data was not zeroed when supposed to be
    #[error_msg("Account data was not zeroed for account `{}`", account)]
    NonZeroedData {
        /// The account with non-zero data
        account: Pubkey,
    },
    /// Account has wrong owner based on index. May be caused by owner matching or not matching.
    #[error_msg(
        "Accounts owner error for accounts `{:?}` with indexer `{}`. Owner input: `{}`",
        accounts,
        indexer,
        owner
    )]
    AccountsOwnerError {
        /// The accounts indexed
        accounts: Vec<Pubkey>,
        /// The indexer for the accounts
        indexer: String,
        /// The owner the indexer was matching against
        owner: Pubkey,
    },
    /// Account owner was not equal to expected value.
    #[error_msg(
        "Account (`{}`) owner (`{}`) not equal to any of `{:?}` when should be",
        account,
        owner,
        expected_owner
    )]
    AccountOwnerNotEqual {
        /// Account whose owner is wrong
        account: Pubkey,
        /// The owner of the account
        owner: Pubkey,
        /// The expected possible owners that were not matched
        expected_owner: Vec<Pubkey>,
    },
    /// Expected a different account than given
    #[error_msg("Invalid account `{}`, expected `{}`", account, expected)]
    InvalidAccount {
        /// Account given
        account: Pubkey,
        /// Account expected
        expected: Pubkey,
    },
    /// Indexer went out of possible range
    #[error_msg(
        "Index out of range. Index: `{}`, Possible Range: `{}`",
        index,
        possible_range
    )]
    IndexOutOfRange {
        /// The index given
        index: String,
        /// The possible range for the index
        possible_range: String,
    },
    /// An unknown instruction was given
    #[error_msg("Unknown instruction: `{}`", instruction)]
    UnknownInstruction {
        /// The unknown instruction
        instruction: String,
    },
    /// No payer on initialization
    #[error_msg("No payer to init account: `{}`", account)]
    NoPayerForInit {
        /// The account needing a payer
        account: Pubkey,
    },
    /// Not enough lamports in an account
    #[error_msg(
        "Not enough lamports in account `{}`. Need `{}`, have `{}`",
        account,
        needed_lamports,
        lamports
    )]
    NotEnoughLamports {
        /// Account with not enough lamports
        account: Pubkey,
        /// Lamports in `account`
        lamports: u64,
        /// Lamports needed
        needed_lamports: u64,
    },
    /// No Account could be created from seeds
    #[error_msg("No account could be created from seeds: `{:?}`", seeds)]
    NoAccountFromSeeds {
        /// The seeds that could not create an account
        seeds: Vec<String>,
    },
    /// Account not generated from expected seeds.
    #[error_msg(
        "Account `{}` not from seeds `{:?}` and program `{}`",
        account,
        seeds,
        program_id
    )]
    AccountNotFromSeeds {
        /// Account that is not from `seeds`
        account: Pubkey,
        /// Seeds that should have generated `account`
        seeds: Vec<String>,
        /// The program id for seeding
        program_id: Pubkey,
    },
    /// Interface is not yet supported.
    #[error_msg("Interfaces are not yet supported")]
    UnsupportedInterface,
    /// Discriminant is empty
    #[error_msg("Discriminant is empty, must contain at least one byte")]
    EmptyDiscriminant,
    /// Could not deserialize something
    #[error_msg("Could not deserialize: {}", what)]
    CouldNotDeserialize {
        /// What could not be deserialized
        what: String,
    },
    /// Size was invalid
    #[error_msg("Size mismatch for range [`{}`, `{}`]. Got: `{}`", min, max, value)]
    SizeInvalid {
        /// Min valid (inclusive)
        min: usize,
        /// Max valid (inclusive)
        max: usize,
        /// The value that is invalid
        value: usize,
    },
    /// Not enough data left for deserialization
    #[error_msg(
        "Not enough data left for deserialization, needed: `{}`, remaining: `{}`",
        needed,
        remaining
    )]
    NotEnoughData {
        /// Amount of data needed
        needed: usize,
        /// Amount of data remaining
        remaining: usize,
    },
    /// Not enough data in an account
    #[error_msg(
        "Not enough data in account (`{}`), needed: `{}`, size: `{}`",
        account,
        needed,
        size
    )]
    NotEnoughDataInAccount {
        /// The account with not enough data
        account: Pubkey,
        /// Amount of data needed
        needed: usize,
        /// Amount of data remaining
        size: usize,
    },
    /// Data was reallocated too large
    #[error_msg(
        "Data was reallocated too large, original_len: `{}`, new_len: `{}`, max_new_len: `{}`",
        original_len,
        new_len,
        max_new_len
    )]
    TooLargeDataIncrease {
        /// The original data size
        original_len: usize,
        /// The new length requested
        new_len: usize,
        /// The maximum new data length
        max_new_len: usize,
    },
}
