//! An account owned by the current program

use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable};
use crate::account_list::AccountListItem;
use crate::account_types::discriminant_account::DiscriminantAccount;
use crate::{AccountInfo, CruiserResult};

// verify_account_arg_impl! {
//     mod data_account_check<AI>{
//         <AI, AL, D> DataAccount<AI, AL, D>
//         where
//             AI: AccountInfo,
//             AL: AccountListItem<D>,
//             D: BorshSerialize + BorshDeserialize,
//         {
//             from: [()];
//             validate: [()];
//             multi: [<T> T where DiscriminantAccount<AI, AL, D>: MultiIndexable<AI, T>];
//             single: [<T> T where DiscriminantAccount<AI, AL, D>: SingleIndexable<AI, T>];
//         }
//     }
// }

/// An account owned by the current program.
/// If not writable should use [`ReadOnlyDataAccount`] instead.
///
/// - `AL`: The [`AccountList`](crate::account_list::AccountList) that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<A>`](AccountListItem)
#[derive(AccountArgument)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo, D: BorshSerialize + BorshDeserialize])]
pub struct DataAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
{
    #[validate(owner = program_id)]
    account: DiscriminantAccount<AI, AL, D>,
}
impl<AI, AL, D> Debug for DataAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    DiscriminantAccount<AI, AL, D>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgramAccount")
            .field("account", &self.account)
            .finish()
    }
}
impl<AI, AL, D> Deref for DataAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
{
    type Target = DiscriminantAccount<AI, AL, D>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
impl<AI, AL, D> DerefMut for DataAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}
impl<AI, AL, D, T> MultiIndexable<T> for DataAccount<AI, AL, D>
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
impl<AI, AL, D, T> SingleIndexable<T> for DataAccount<AI, AL, D>
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
