use crate::account_argument::MultiIndexable;
use crate::{AccountInfo, CruiserResult};
use solana_program::instruction::AccountMeta as SolanaAccountMeta;

/// An account set that can be indexed to a single account at a time with index `I`.
/// All functions should be infallible if `I` is [`()`].
pub trait SingleIndexable<AI, I>: MultiIndexable<AI, I> {
    /// Gets the account info at index `indexer`
    fn index_info(&self, indexer: I) -> CruiserResult<&AI>;
    /// Turns the account at index `indexer` to a [`SolanaAccountMeta`]
    fn to_solana_account_meta(&self, indexer: I) -> CruiserResult<SolanaAccountMeta>
    where
        AI: AccountInfo,
    {
        let info = self.index_info(indexer)?;
        Ok(SolanaAccountMeta {
            pubkey: *info.key(),
            is_signer: info.is_signer(),
            is_writable: info.is_writable(),
        })
    }
}

/// Infallible single access functions.
/// Relies on the infallibility of [`()`] for [`SingleIndexable`] and [`MultiIndexable`].
pub trait Single<AI>: SingleIndexable<AI, ()> {
    /// Gets the account info for this argument.
    fn info(&self) -> &AI;
}
impl<AI, T> Single<AI> for T
where
    T: SingleIndexable<AI, ()>,
{
    fn info(&self) -> &AI {
        self.index_info(()).expect("`()` info is not infallible!")
    }
}
