//! An account owned by the current program

use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable};
use crate::account_list::AccountListItem;
use crate::account_types::discriminant_account::DiscriminantAccount;
use crate::{AccountInfo, CruiserResult};
use cruiser_derive::verify_account_arg_impl;

verify_account_arg_impl! {
    mod program_account_check{
        <AL, A> ProgramAccount<AL, A>
        where
            AL: AccountListItem<A>,
            A: BorshSerialize + BorshDeserialize,
        {
            from: [()];
            validate: [()];
            multi: [<T> T where DiscriminantAccount<AL, A>: MultiIndexable<T>];
            single: [<T> T where DiscriminantAccount<AL, A>: SingleIndexable<T>];
        }
    }
}

/// An account owned by the current program.
///
/// - `AL`: The [`AccountList`](crate::account_list::AccountList) that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<A>`](AccountListItem)
#[derive(AccountArgument)]
pub struct ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    #[validate(owner = program_id)]
    account: DiscriminantAccount<AL, A>,
}
impl<AL, A> Debug for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ProgramAccount")
            .field("account", &self.account)
            .finish()
    }
}
impl<AL, A> Deref for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    type Target = DiscriminantAccount<AL, A>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
impl<AL, A> DerefMut for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}
impl<AL, A, T> MultiIndexable<T> for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<AL, A, T> SingleIndexable<T> for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.account.info(indexer)
    }
}
