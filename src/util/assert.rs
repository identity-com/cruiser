//! Assertions used in generated code and standard types.

use crate::account_argument::{MultiIndexable, SingleIndexable};
use crate::{AccountInfo, CruiserResult, GenericError};
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;

/// Asserts that the account at index `indexer` is a signer.
pub fn assert_is_signer<AI, I>(
    argument: &impl MultiIndexable<I, AccountInfo = AI>,
    indexer: I,
) -> CruiserResult<()>
where
    AI: AccountInfo,
    I: Debug + Clone,
{
    if argument.index_is_signer(indexer.clone())? {
        Ok(())
    } else {
        Err(GenericError::AccountsSignerError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
        }
        .into())
    }
}

/// Asserts that the account at index `indexer` is writable.
pub fn assert_is_writable<AI, I>(
    argument: &impl MultiIndexable<I, AccountInfo = AI>,
    indexer: I,
) -> CruiserResult<()>
where
    AI: AccountInfo,
    I: Debug + Clone,
{
    if argument.index_is_writable(indexer.clone())? {
        Ok(())
    } else {
        Err(GenericError::AccountsWritableError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
        }
        .into())
    }
}

/// Asserts that the account at index `indexer` is a certain key.
pub fn assert_is_key<AI, I>(
    argument: &impl SingleIndexable<I, AccountInfo = AI>,
    key: &Pubkey,
    indexer: I,
) -> CruiserResult<()>
where
    AI: AccountInfo,
    I: Debug + Clone,
{
    let account = argument.index_info(indexer)?.key();
    if account == key {
        Ok(())
    } else {
        Err(GenericError::InvalidAccount {
            account: *account,
            expected: *key,
        }
        .into())
    }
}

/// Asserts that the account at index `indexer`'s owner is `owner`.
pub fn assert_is_owner<AI, I>(
    argument: &impl MultiIndexable<I, AccountInfo = AI>,
    owner: &Pubkey,
    indexer: I,
) -> CruiserResult<()>
where
    AI: AccountInfo,
    I: Debug + Clone,
{
    if argument.index_is_owner(owner, indexer.clone())? {
        Ok(())
    } else {
        Err(GenericError::AccountsOwnerError {
            accounts: argument.keys()?,
            indexer: format!("{:?}", indexer),
            owner: *owner,
        }
        .into())
    }
}
