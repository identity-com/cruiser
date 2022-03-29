use crate::account_argument::AccountArgument;
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;

/// An account set that can be indexed by 0+ accounts at time with index `I`.
/// All functions should be infallible if `I` is [`()`].
pub trait MultiIndexable<I>: AccountArgument {
    /// Returns whether the account at index `indexer` is a signer.
    fn index_is_signer(&self, indexer: I) -> CruiserResult<bool>;
    /// Returns whether the account at index `indexer` is writable.
    fn index_is_writable(&self, indexer: I) -> CruiserResult<bool>;
    /// Returns whether the account at index `indexer`'s owner is `owner`.
    fn index_is_owner(&self, owner: &Pubkey, indexer: I) -> CruiserResult<bool>;
}
