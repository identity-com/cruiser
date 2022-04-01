use crate::account_argument::MultiIndexable;
use crate::{AccountInfo, AccountInfoAccess, CruiserResult};
use solana_program::instruction::AccountMeta as SolanaAccountMeta;

/// An account set that can be indexed to a single account at a time with index `I`.
/// All functions should be infallible if `I` is [`()`].
pub trait SingleIndexable<I>: MultiIndexable<I> {
    /// Gets the account info at index `indexer`
    fn index_info(&self, indexer: I) -> CruiserResult<&Self::AccountInfo>;
    /// Turns the account at index `indexer` to a [`SolanaAccountMeta`]
    fn index_to_solana_account_meta(&self, indexer: I) -> CruiserResult<SolanaAccountMeta>
    where
        Self::AccountInfo: AccountInfo,
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
pub trait Single: SingleIndexable<()> {
    /// Gets the account info for this argument.
    fn info(&self) -> &Self::AccountInfo;
}
impl<T> Single for T
where
    T: SingleIndexable<()>,
{
    fn info(&self) -> &Self::AccountInfo {
        self.index_info(()).expect("`()` info is not infallible!")
    }
}

/// Can be turned into a [`SolanaAccountMeta`]
pub trait ToSolanaAccountMeta {
    /// Turns the account to a [`SolanaAccountMeta`]
    fn to_solana_account_meta(&self) -> SolanaAccountMeta;
}
impl<T> ToSolanaAccountMeta for T
where
    T: SingleIndexable<()>,
    T::AccountInfo: AccountInfo,
{
    fn to_solana_account_meta(&self) -> SolanaAccountMeta {
        self.index_to_solana_account_meta(())
            .expect("`()` info is not infallible!")
    }
}
impl ToSolanaAccountMeta for SolanaAccountMeta {
    fn to_solana_account_meta(&self) -> SolanaAccountMeta {
        self.clone()
    }
}
