//! A zeroed account that will be initialized

use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::account_list::AccountListItem;
use crate::account_types::discriminant_account::{DiscriminantAccount, WriteDiscriminant};
use crate::compressed_numbers::CompressedNumber;
use crate::util::assert::assert_is_owner;
use crate::{AccountInfo, AllAny, CruiserResult, GenericError};

verify_account_arg_impl! {
    mod init_account_check<AI>{
        <AI, AL, D> ZeroedAccount<AI, AL, D>
        where
            AI: AccountInfo,
            AL: AccountListItem<D>,
            D: BorshSerialize + BorshDeserialize,
        {
            from: [
                /// The initial value for the account
                D
            ];
            validate: [
                /// Checks the [`AL::DiscriminantCompressed::max_bytes()`](crate::CompressedNumber::max_bytes) bytes for any non-zero bytes.
                ();
                /// Checks all bytes in the account for non-zero.
                CheckAll;
            ];
            multi: [(); AllAny];
            single: [()];
        }
    }
}

/// Initializes an account that is zeroed out and owned by the current program.
///
/// - `AL`: The [`AccountList`](crate::account_list::AccountList) that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<A>`](AccountListItem)
///
/// Does not guarantee rent exempt, wrap with [`RentExempt`](crate::account_types::rent_exempt::RentExempt) for that.
#[derive(AccountArgument)]
#[account_argument(no_from, no_validate, account_info = AI, generics = [where AI: AccountInfo])]
pub struct ZeroedAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    account: DiscriminantAccount<AI, AL, D>,
}
impl<AI, AL, D> Deref for ZeroedAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    type Target = DiscriminantAccount<AI, AL, D>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
impl<AI, AL, D> DerefMut for ZeroedAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}
impl<AI, AL, D> Debug for ZeroedAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AI, AL, D>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InitAccount")
            .field("account", &self.account)
            .finish()
    }
}
impl<AI, AL, D> FromAccounts<AI, D> for ZeroedAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: D,
    ) -> CruiserResult<Self> {
        Ok(Self {
            account: DiscriminantAccount::from_accounts(program_id, infos, (arg,))?,
        })
    }

    fn accounts_usage_hint(_arg: &D) -> (usize, Option<usize>) {
        DiscriminantAccount::<AI, AL, D>::accounts_usage_hint(&())
    }
}
impl<AI, AL, D> ValidateArgument<AI, ()> for ZeroedAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn validate(&mut self, program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        assert_is_owner(&self.account, program_id, ())?;
        if self.account.info.data()[..AL::DiscriminantCompressed::max_bytes()]
            .iter()
            .any(|val| *val != 0)
        {
            Err(GenericError::NonZeroedData {
                account: *self.account.info.key(),
            }
            .into())
        } else {
            self.account.validate(program_id, WriteDiscriminant)
        }
    }
}
/// Checks all the bytes of a [`ZeroedAccount`]
#[derive(Debug, Copy, Clone)]
pub struct CheckAll;
impl<AI, AL, D> ValidateArgument<AI, CheckAll> for ZeroedAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn validate(&mut self, program_id: &Pubkey, _arg: CheckAll) -> CruiserResult<()> {
        assert_is_owner(&self.account, program_id, ())?;
        if self.account.info.data().iter().any(|val| *val != 0) {
            Err(GenericError::NonZeroedData {
                account: *self.account.info.key(),
            }
            .into())
        } else {
            self.account.validate(program_id, WriteDiscriminant)
        }
    }
}
impl<'a, AI, AL, D, T> MultiIndexable<AI, T> for ZeroedAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AI, AL, D>: MultiIndexable<AI, T>,
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
impl<'a, AI, AL, D, T> SingleIndexable<AI, T> for ZeroedAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AI, AL, D>: SingleIndexable<AI, T>,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.account.index_info(indexer)
    }
}
