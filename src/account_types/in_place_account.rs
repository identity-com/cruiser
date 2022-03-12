//! In place access account. Experimental.

// TODO: Update this

use std::cell::RefMut;
use std::marker::PhantomData;
use std::ops::DerefMut;

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::account_argument::{AccountArgument, AccountInfoIterator, FromAccounts};
use crate::account_list::AccountListItem;
use crate::compressed_numbers::CompressedNumber;
use crate::in_place::InPlaceBuilder;
use crate::{AccountInfo, CruiserError, CruiserResult};

/// Access a given account in-place. Experimental.
#[derive(Debug)]
pub struct InPlaceProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    account: AccountInfo,
    phantom_al_a: PhantomData<fn() -> (AL, A)>,
}
impl<AL, A> InPlaceProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    /// Borrows the account mutably
    pub fn borrow_mut(&mut self) -> InPlaceMutHolder<'_, AL, A> {
        InPlaceMutHolder {
            value: self.account.data.borrow_mut(),
            phantom_al_a: PhantomData,
        }
    }
}
impl<AL, A> AccountArgument for InPlaceProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn write_back(self, program_id: &'static Pubkey) -> CruiserResult<()> {
        self.account.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(&'static Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.account.add_keys(add)
    }
}
impl<T, AL, A> FromAccounts<T> for InPlaceProgramAccount<AL, A>
where
    AccountInfo: FromAccounts<T>,
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: T,
    ) -> CruiserResult<Self> {
        let account = AccountInfo::from_accounts(program_id, infos, arg)?;
        if *account.owner.borrow() != program_id {
            return Err(CruiserError::AccountOwnerNotEqual {
                account: account.key,
                owner: **account.owner.borrow(),
                expected_owner: vec![*program_id],
            }
            .into());
        }
        let discriminant =
            AL::DiscriminantCompressed::deserialize(&mut &**account.data.borrow())?.into_number();
        if discriminant != AL::discriminant() {
            return Err(CruiserError::MismatchedDiscriminant {
                account: account.key,
                received: discriminant.get(),
                expected: AL::discriminant(),
            }
            .into());
        }
        Ok(Self {
            account,
            phantom_al_a: PhantomData,
        })
    }

    fn accounts_usage_hint(_arg: &T) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}
// impl<AL, A> MultiIndexable<AllAny> for InPlaceProgramAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: InPlaceBuilder,
// {
//     fn is_signer(&self, indexer: AllAny) -> CruiserResult<bool> {
//         self.account.is_signer(indexer)
//     }
//
//     fn is_writable(&self, indexer: AllAny) -> CruiserResult<bool> {
//         self.account.is_writable(indexer)
//     }
//
//     fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> CruiserResult<bool> {
//         self.account.is_owner(owner, indexer)
//     }
// }
// impl<AL, A> MultiIndexable<()> for InPlaceProgramAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: InPlaceBuilder,
// {
//     fn is_signer(&self, indexer: ()) -> CruiserResult<bool> {
//         self.account.is_signer(indexer)
//     }
//
//     fn is_writable(&self, indexer: ()) -> CruiserResult<bool> {
//         self.account.is_writable(indexer)
//     }
//
//     fn is_owner(&self, owner: &Pubkey, indexer: ()) -> CruiserResult<bool> {
//         self.account.is_owner(owner, indexer)
//     }
// }
// impl<AL, A> SingleIndexable<()> for InPlaceProgramAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: InPlaceBuilder,
// {
//     fn owner(&self, indexer: ()) -> CruiserResult<&Rc<RefCell<&'static mut Pubkey>>> {
//         self.account.owner(indexer)
//     }
//
//     fn key(&self, indexer: ()) -> CruiserResult<&'static Pubkey> {
//         self.account.key(indexer)
//     }
// }

/// Holds an in place mutable reference
#[derive(Debug)]
pub struct InPlaceMutHolder<'a, AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    value: RefMut<'a, &'static mut [u8]>,
    phantom_al_a: PhantomData<fn() -> (AL, A)>,
}
impl<'a, AL, A> InPlaceMutHolder<'a, AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    /// Gets the in-place data
    pub fn get_data(&mut self) -> CruiserResult<A::InPlaceData<'_>> {
        A::read(&mut self.value.deref_mut()[AL::discriminant().get() as usize..])
    }
}

/// A zeroed account accessed in-place
#[derive(Debug)]
pub struct InPlaceZeroed<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    account: AccountInfo,
    phantom_al_a: PhantomData<fn() -> (AL, A)>,
}
impl<AL, A> InPlaceZeroed<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    /// Gets the data mutably
    pub fn borrow_mut(&mut self) -> InPlaceMutHolder<'_, AL, A> {
        InPlaceMutHolder {
            value: self.account.data.borrow_mut(),
            phantom_al_a: PhantomData,
        }
    }
}
impl<AL, A> AccountArgument for InPlaceZeroed<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn write_back(self, program_id: &'static Pubkey) -> CruiserResult<()> {
        self.account.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(&'static Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.account.add_keys(add)
    }
}
impl<C, AL, A> FromAccounts<C> for InPlaceZeroed<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder<CreateArg = C>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: C,
    ) -> CruiserResult<Self> {
        let account = AccountInfo::from_accounts(program_id, infos, ())?;
        if *account.owner.borrow() != program_id {
            return Err(CruiserError::AccountOwnerNotEqual {
                account: account.key,
                owner: **account.owner.borrow(),
                expected_owner: vec![*program_id],
            }
            .into());
        }
        let mut data = account.data.borrow_mut();
        for x in 0..AL::DiscriminantCompressed::max_bytes() {
            if data[x] != 0 {
                return Err(CruiserError::NonZeroedData {
                    account: account.key,
                }
                .into());
            }
        }
        let mut data_mut = &mut **data;
        AL::compressed_discriminant().serialize(&mut data_mut)?;
        A::create(data_mut, arg)?;
        drop(data);
        Ok(Self {
            account,
            phantom_al_a: PhantomData,
        })
    }

    fn accounts_usage_hint(_arg: &C) -> (usize, Option<usize>) {
        (1, Some(1))
    }
}
// impl<AL, A> MultiIndexable<AllAny> for InPlaceZeroed<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: InPlaceBuilder,
// {
//     fn is_signer(&self, indexer: AllAny) -> CruiserResult<bool> {
//         self.account.is_signer(indexer)
//     }
//
//     fn is_writable(&self, indexer: AllAny) -> CruiserResult<bool> {
//         self.account.is_writable(indexer)
//     }
//
//     fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> CruiserResult<bool> {
//         self.account.is_owner(owner, indexer)
//     }
// }
// impl<AL, A> MultiIndexable<()> for InPlaceZeroed<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: InPlaceBuilder,
// {
//     fn is_signer(&self, indexer: ()) -> CruiserResult<bool> {
//         self.account.is_signer(indexer)
//     }
//
//     fn is_writable(&self, indexer: ()) -> CruiserResult<bool> {
//         self.account.is_writable(indexer)
//     }
//
//     fn is_owner(&self, owner: &Pubkey, indexer: ()) -> CruiserResult<bool> {
//         self.account.is_owner(owner, indexer)
//     }
// }
// impl<AL, A> SingleIndexable<()> for InPlaceZeroed<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: InPlaceBuilder,
// {
//     fn owner(&self, indexer: ()) -> CruiserResult<&Rc<RefCell<&'static mut Pubkey>>> {
//         self.account.owner(indexer)
//     }
//
//     fn key(&self, indexer: ()) -> CruiserResult<&'static Pubkey> {
//         self.account.key(indexer)
//     }
// }
