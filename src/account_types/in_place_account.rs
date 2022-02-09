use crate::compressed_numbers::CompressedNumber;
use crate::{
    AccountInfo, AccountInfoIterator, FromAccounts, GeneratorError, GeneratorResult,
    InPlaceBuilder, MultiIndexableAccountArgument, SystemProgram,
};
use borsh::{BorshDeserialize, BorshSerialize};
use solana_generator::{AccountArgument, AccountListItem, AllAny, SingleIndexableAccountArgument};
use solana_program::pubkey::Pubkey;
use std::cell::{RefCell, RefMut};
use std::marker::PhantomData;
use std::ops::DerefMut;
use std::rc::Rc;

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
    pub fn borrow_mut(&mut self) -> InPlaceHolder<'_, AL, A> {
        InPlaceHolder {
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
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        self.account.write_back(program_id, system_program)
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
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
    ) -> GeneratorResult<Self> {
        let account = AccountInfo::from_accounts(program_id, infos, arg)?;
        if *account.owner.borrow() != program_id {
            return Err(GeneratorError::AccountOwnerNotEqual {
                account: account.key,
                owner: **account.owner.borrow(),
                expected_owner: vec![*program_id],
            }
            .into());
        }
        let discriminant =
            AL::DiscriminantCompressed::deserialize(&mut &**account.data.borrow())?.into_number();
        if discriminant != AL::discriminant().get() {
            return Err(GeneratorError::MismatchedDiscriminant {
                account: account.key,
                received: discriminant,
                expected: AL::discriminant().get(),
            }
            .into());
        }
        Ok(Self {
            account,
            phantom_al_a: PhantomData,
        })
    }
}
impl<AL, A> MultiIndexableAccountArgument<AllAny> for InPlaceProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn is_signer(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> GeneratorResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<AL, A> MultiIndexableAccountArgument<()> for InPlaceProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn is_signer(&self, indexer: ()) -> GeneratorResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: ()) -> GeneratorResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: ()) -> GeneratorResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<AL, A> SingleIndexableAccountArgument<()> for InPlaceProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn owner(&self, indexer: ()) -> GeneratorResult<&Rc<RefCell<&'static mut Pubkey>>> {
        self.account.owner(indexer)
    }

    fn key(&self, indexer: ()) -> GeneratorResult<&'static Pubkey> {
        self.account.key(indexer)
    }
}

#[derive(Debug)]
pub struct InPlaceHolder<'a, AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    value: RefMut<'a, &'static mut [u8]>,
    phantom_al_a: PhantomData<fn() -> (AL, A)>,
}
impl<'a, AL, A> InPlaceHolder<'a, AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    pub fn get_data(&mut self) -> GeneratorResult<A::InPlaceData<'_>> {
        A::read(&mut self.value.deref_mut()[AL::discriminant().get() as usize..])
    }
}

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
    pub fn borrow_mut(&mut self) -> InPlaceHolder<'_, AL, A> {
        InPlaceHolder {
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
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        self.account.write_back(program_id, system_program)
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
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
    ) -> GeneratorResult<Self> {
        let account = AccountInfo::from_accounts(program_id, infos, ())?;
        if *account.owner.borrow() != program_id {
            return Err(GeneratorError::AccountOwnerNotEqual {
                account: account.key,
                owner: **account.owner.borrow(),
                expected_owner: vec![*program_id],
            }
            .into());
        }
        let mut data = account.data.borrow_mut();
        for x in 0..AL::DiscriminantCompressed::max_bytes() {
            if data[x] != 0 {
                return Err(GeneratorError::NonZeroedData {
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
}
impl<AL, A> MultiIndexableAccountArgument<AllAny> for InPlaceZeroed<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn is_signer(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> GeneratorResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<AL, A> MultiIndexableAccountArgument<()> for InPlaceZeroed<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn is_signer(&self, indexer: ()) -> GeneratorResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: ()) -> GeneratorResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: ()) -> GeneratorResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<AL, A> SingleIndexableAccountArgument<()> for InPlaceZeroed<AL, A>
where
    AL: AccountListItem<A>,
    A: InPlaceBuilder,
{
    fn owner(&self, indexer: ()) -> GeneratorResult<&Rc<RefCell<&'static mut Pubkey>>> {
        self.account.owner(indexer)
    }

    fn key(&self, indexer: ()) -> GeneratorResult<&'static Pubkey> {
        self.account.key(indexer)
    }
}
