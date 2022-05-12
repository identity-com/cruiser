//! An account that can only be read from.

use crate::prelude::AccountListItem;
use cruiser::prelude::*;
use std::fmt::{Debug, Formatter};

/// An account owned by the current program ([`ValidateArgumet<()>`](ValidateArgumet))
/// or another program ([`ValidateArgumet<&Pubkey>`](ValidateArgumet)).
/// Can only be read from.
///
/// - `AL`: The [`AccountList`] that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<A>`](AccountListItem)
#[derive(AccountArgument)]
#[account_argument(
    account_info = AI,
    generics = [where AI: AccountInfo, D: BorshSerialize + BorshDeserialize],
    write_back = |account, program_id| account.account.info.write_back(program_id),
)]
#[validate(data = ())]
#[validate(id = cpi, data = (other_program_id: &Pubkey))]
pub struct ReadOnlyDataAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
{
    #[validate(owner = program_id)]
    #[validate(id = cpi, owner = other_program_id)]
    account: DiscriminantAccount<AI, AL, D>,
}

impl<AI, AL, D> Debug for ReadOnlyDataAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    DiscriminantAccount<AI, AL, D>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ReadOnlyDataAccount")
            .field("account", &self.account)
            .finish()
    }
}
impl<AI, AL, D> Deref for ReadOnlyDataAccount<AI, AL, D>
    where
        AL: AccountListItem<D>,
{
    type Target = DiscriminantAccount<AI, AL, D>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
impl<AI, AL, D, T> MultiIndexable<T> for ReadOnlyDataAccount<AI, AL, D>
    where
        AI: AccountInfo,
        AL: AccountListItem<D>,
        D: BorshSerialize + BorshDeserialize,
        DiscriminantAccount<AI, AL, D>: MultiIndexable<T>,
{
    fn index_is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.account.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.account.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.account.index_is_owner(owner, indexer)
    }
}
impl<AI, AL, D, T> SingleIndexable<T> for ReadOnlyDataAccount<AI, AL, D>
    where
        AI: AccountInfo,
        AL: AccountListItem<D>,
        D: BorshSerialize + BorshDeserialize,
        DiscriminantAccount<AI, AL, D>: SingleIndexable<T, AccountInfo = AI>,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.account.index_info(indexer)
    }
}
