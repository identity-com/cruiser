//! Checks and writes discriminants of account data

use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::account_list::AccountListItem;
use crate::account_types::PhantomAccount;
use crate::compressed_numbers::CompressedNumber;
use crate::AccountInfo;
use crate::{CruiserAccountInfo, CruiserResult, GenericError};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

// verify_account_arg_impl! {
//     mod discriminant_account_check <AI>{
//         <AI, AL, D> DiscriminantAccount<AI, AL, D> where AI: AccountInfo, AL: AccountListItem<D>, D: BorshSerialize{
//             from: [
//                 /// Reads from the account for the value.
//                 () where D: BorshDeserialize;
//                 /// Uses this value rather than reading from the account.
//                 (D,);
//             ];
//             validate: [
//                 /// Verifies the discriminant on the account.
//                 ();
//                 /// Writes the discriminant to the account.
//                 WriteDiscriminant;
//             ];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// An account whose data is discriminated based on an account list.
///
/// - `AL`: The [`AccountList`](crate::account_list::AccountList) that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<D>`](AccountListItem)
pub struct DiscriminantAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
{
    /// The [`AccountInfo`] of this account.
    pub info: AI,
    #[allow(dead_code)]
    phantom_al: PhantomAccount<AI, AL>,
    data: D,
}
impl<AI, AL, D> Deref for DiscriminantAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
{
    type Target = D;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<AI, AL, D> DerefMut for DiscriminantAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
impl<AI, AL, D> Debug for DiscriminantAccount<AI, AL, D>
where
    AI: Debug,
    AL: AccountListItem<D>,
    D: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiscriminantAccount")
            .field("info", &self.info)
            .field("data", &self.data)
            .finish()
    }
}
impl<AI, AL, D> AccountArgument for DiscriminantAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize,
{
    type AccountInfo = AI;

    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        let mut data_ref = self.info.data_mut();
        let mut data = &mut data_ref[AL::compressed_discriminant().num_bytes()..];
        self.data.serialize(&mut data)?;
        drop(data_ref);
        self.info.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.info.add_keys(add)
    }
}
impl<AI, AL, D> FromAccounts for DiscriminantAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = AI>,
        arg: (),
    ) -> CruiserResult<Self> {
        let info = AI::from_accounts(program_id, infos, arg)?;
        let num_discriminant_bytes = AL::compressed_discriminant().num_bytes();
        if num_discriminant_bytes > info.data().len() {
            return Err(GenericError::NotEnoughDataInAccount {
                account: *info.key(),
                needed: num_discriminant_bytes,
                size: info.data().len(),
            }
            .into());
        }
        let data = D::deserialize(&mut &info.data()[num_discriminant_bytes..])?;
        Ok(Self {
            info,
            phantom_al: PhantomAccount::default(),
            data,
        })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        CruiserAccountInfo::accounts_usage_hint(arg)
    }
}
impl<AI, AL, D> FromAccounts<(D,)> for DiscriminantAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = AI>,
        arg: (D,),
    ) -> CruiserResult<Self> {
        let info = AI::from_accounts(program_id, infos, ())?;
        let data = arg.0;
        Ok(Self {
            info,
            phantom_al: PhantomAccount::default(),
            data,
        })
    }

    fn accounts_usage_hint(_arg: &(D,)) -> (usize, Option<usize>) {
        CruiserAccountInfo::accounts_usage_hint(&())
    }
}
impl<AI, AL, D> ValidateArgument for DiscriminantAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult<()> {
        self.info.validate(program_id, arg)?;
        let discriminant = AL::DiscriminantCompressed::deserialize(&mut &*self.info.data())?;
        if discriminant == AL::compressed_discriminant() {
            Ok(())
        } else {
            Err(GenericError::MismatchedDiscriminant {
                account: *self.info.key(),
                received: discriminant.into_number().get(),
                expected: AL::discriminant(),
            }
            .into())
        }
    }
}
/// Writes the discriminant to the account rather than verifying it
#[derive(Debug, Copy, Clone)]
pub struct WriteDiscriminant;
impl<AI, AL, D> ValidateArgument<WriteDiscriminant> for DiscriminantAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize,
{
    fn validate(&mut self, program_id: &Pubkey, _arg: WriteDiscriminant) -> CruiserResult<()> {
        self.info.validate(program_id, ())?;
        AL::compressed_discriminant().serialize(&mut &mut *self.info.data_mut())?;
        Ok(())
    }
}
impl<AI, AL, D, T> MultiIndexable<T> for DiscriminantAccount<AI, AL, D>
where
    AI: AccountInfo + MultiIndexable<T>,
    AL: AccountListItem<D>,
    D: BorshSerialize,
{
    fn index_is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_owner(owner, indexer)
    }
}
impl<AI, AL, D, T> SingleIndexable<T> for DiscriminantAccount<AI, AL, D>
where
    AI: AccountInfo + SingleIndexable<T>,
    AL: AccountListItem<D>,
    D: BorshSerialize,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.info.index_info(indexer)
    }
}
