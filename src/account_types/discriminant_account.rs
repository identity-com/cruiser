//! Checks and writes discriminants of account data

use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::account_list::AccountListItem;
use crate::compressed_numbers::CompressedNumber;
use crate::AllAny;
use crate::{AccountInfo, CruiserResult, GenericError};
use borsh::{BorshDeserialize, BorshSerialize};
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;

verify_account_arg_impl! {
    mod discriminant_account_check{
        <AL, A> DiscriminantAccount<AL, A> where AL: AccountListItem<A>, A: BorshSerialize{
            from: [
                /// Reads from the account for the value.
                () where A: BorshDeserialize;
                /// Uses this value rather than reading from the account.
                (A,);
            ];
            validate: [
                /// Verifies the discriminant on the account.
                ();
                /// Writes the discriminant to the account.
                WriteDiscriminant;
            ];
            multi: [(); AllAny];
            single: [()];
        }
    }
}

/// An account whose data is discriminated based on an account list.
///
/// - `AL`: The [`AccountList`](crate::account_list::AccountList) that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<A>`](AccountListItem)
pub struct DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    /// The [`AccountInfo`] of this account.
    pub info: AccountInfo,
    /// The discriminant of this account.
    pub discriminant: AL::DiscriminantCompressed,
    data: A,
}
impl<AL, A> Deref for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    AL::DiscriminantCompressed: Debug,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<AL, A> DerefMut for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    AL::DiscriminantCompressed: Debug,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl<AL, A> Debug for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    AL::DiscriminantCompressed: Debug,
    A: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscriminantAccount")
            .field("info", &self.info)
            .field("discriminant", &self.discriminant)
            .field("data", &self.data)
            .finish()
    }
}
impl<AL, A> AccountArgument for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn write_back(self, program_id: &'static Pubkey) -> CruiserResult<()> {
        let mut data_ref = self.info.data.borrow_mut();
        let mut data = &mut data_ref[self.discriminant.num_bytes()..];
        self.data.serialize(&mut data)?;
        drop(data_ref);
        self.info.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(&'static Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.info.add_keys(add)
    }
}
impl<AL, A> FromAccounts<()> for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (),
    ) -> CruiserResult<Self> {
        let info = AccountInfo::from_accounts(program_id, infos, arg)?;
        let data_ref = info.data.borrow();
        let mut data = &**data_ref;
        let discriminant = AL::DiscriminantCompressed::deserialize(&mut data)?;
        let data = A::deserialize(&mut data)?;
        drop(data_ref);
        Ok(Self {
            info,
            discriminant,
            data,
        })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint(arg)
    }
}
impl<AL, A> FromAccounts<(A,)> for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (A,),
    ) -> CruiserResult<Self> {
        let info = AccountInfo::from_accounts(program_id, infos, ())?;
        let discriminant = AL::compressed_discriminant();
        discriminant.serialize(&mut &mut **info.data.borrow_mut())?;
        let data = arg.0;
        Ok(Self {
            info,
            discriminant,
            data,
        })
    }

    fn accounts_usage_hint(_arg: &(A,)) -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint(&())
    }
}
impl<AL, A> ValidateArgument<()> for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: ()) -> CruiserResult<()> {
        self.info.validate(program_id, arg)?;
        if self.discriminant == AL::compressed_discriminant() {
            Ok(())
        } else {
            Err(GenericError::MismatchedDiscriminant {
                account: self.info.key,
                received: self.discriminant.into_number().get(),
                expected: AL::discriminant(),
            }
            .into())
        }
    }
}
/// Writes the discriminant to the account rather than verifying it
#[derive(Debug, Copy, Clone)]
pub struct WriteDiscriminant;
impl<AL, A> ValidateArgument<WriteDiscriminant> for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn validate(
        &mut self,
        program_id: &'static Pubkey,
        _arg: WriteDiscriminant,
    ) -> CruiserResult<()> {
        self.info.validate(program_id, ())?;
        self.discriminant
            .serialize(&mut &mut **self.info.data.borrow_mut())?;
        Ok(())
    }
}
impl<AL, A, T> MultiIndexable<T> for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
    AccountInfo: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<AL, A, T> SingleIndexable<T> for DiscriminantAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
    AccountInfo: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.info.info(indexer)
    }
}
