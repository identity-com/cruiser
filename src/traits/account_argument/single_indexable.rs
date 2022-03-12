use crate::account_argument::MultiIndexable;
use crate::{AccountInfo, CruiserResult};
use solana_program::instruction::AccountMeta as SolanaAccountMeta;

/// An account set that can be indexed to a single account at a time with index `I`.
/// All functions should be infallible if `I` is [`()`].
pub trait SingleIndexable<I>: MultiIndexable<I> {
    /// Gets the account info at index `indexer`
    fn info(&self, indexer: I) -> CruiserResult<&AccountInfo>;
    /// Turns the account at index `indexer` to a [`SolanaAccountMeta`]
    fn to_solana_account_meta(&self, indexer: I) -> CruiserResult<SolanaAccountMeta> {
        let info = self.info(indexer)?;
        Ok(SolanaAccountMeta {
            pubkey: *info.key,
            is_signer: info.is_signer,
            is_writable: info.is_writable,
        })
    }
}

/// Infallible single access functions.
/// Relies on the infallibility of [`()`] for [`SingleIndexable`] and [`MultiIndexable`].
pub trait Single: SingleIndexable<()> {
    /// Gets the account info for this argument.
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
