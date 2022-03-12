//! Assertions used in generated code and standard types.

use crate::account_argument::{MultiIndexable, SingleIndexable};
use crate::{CruiserError, CruiserResult};
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

/// Asserts that the account at index `indexer` is a signer.
pub fn assert_is_signer<I>(argument: &impl MultiIndexable<I>, indexer: I) -> CruiserResult<()>
where
    I: Debug + Clone,
{
    if argument.is_signer(indexer.clone())? {
        Ok(())
    } else {
        Err(CruiserError::AccountsSignerError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
        }
        .into())
    }
}

/// Asserts that the account at index `indexer` is writable.
pub fn assert_is_writable<I>(argument: &impl MultiIndexable<I>, indexer: I) -> CruiserResult<()>
where
    I: Debug + Clone,
{
    if argument.is_writable(indexer.clone())? {
        Ok(())
    } else {
        Err(CruiserError::AccountsWritableError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
        }
        .into())
    }
}

/// Asserts that the account at index `indexer` is a certain key.
pub fn assert_is_key<I>(
    argument: &impl SingleIndexable<I>,
    key: &Pubkey,
    indexer: I,
) -> CruiserResult<()>
where
    I: Debug + Clone,
{
    let account = argument.info(indexer)?.key;
    if account == key {
        Ok(())
    } else {
        Err(CruiserError::InvalidAccount {
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
) -> CruiserResult<()>
where
    I: Debug + Clone,
{
    if argument.is_owner(owner, indexer.clone())? {
        Ok(())
    } else {
        Err(CruiserError::AccountsOwnerError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
            owner: *owner,
        }
        .into())
    }
}
