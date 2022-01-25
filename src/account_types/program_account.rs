use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::account_types::system_program::SystemProgram;
use crate::compressed_numbers::CompressedU64;
use crate::traits::AccountArgument;
use crate::{
    AccountInfo, AccountInfoIterator, AccountListItem, AllAny, FromAccounts, GeneratorError,
    GeneratorResult, MultiIndexableAccountArgument, SingleIndexableAccountArgument,
};
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

/// A data account owned by this program. Checks that is owned by this program.
#[derive(Debug)]
pub struct ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    /// The [`AccountInfo`] for this, data field will be overwritten on write back.
    pub info: AccountInfo,
    data: A,
    phantom_list: PhantomData<fn() -> AL>,
}
impl<AL, A> AccountArgument for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn write_back(
        self,
        _program_id: &'static Pubkey,
        _system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        let mut account_data_ref = self.info.data.borrow_mut();
        let mut account_data = &mut **account_data_ref.deref_mut();

        AL::compressed_discriminant().serialize(&mut account_data)?;
        self.data.serialize(&mut account_data)?;
        Ok(())
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.info.add_keys(add)
    }
}
impl<AL, A, Arg> FromAccounts<Arg> for ProgramAccount<AL, A>
where
    AccountInfo: FromAccounts<Arg>,
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: Arg,
    ) -> GeneratorResult<Self> {
        let info = AccountInfo::from_accounts(program_id, infos, arg)?;

        if *info.owner.borrow() != program_id {
            return Err(GeneratorError::AccountOwnerNotEqual {
                account: info.key,
                owner: **info.owner.borrow(),
                expected_owner: vec![*program_id],
            }
            .into());
        }
        let account_data_ref = info.data.borrow();
        let mut account_data = &**account_data_ref.deref();

        let in_discriminant =
            AL::DiscriminantCompressed::deserialize(&mut account_data)?.into_u64();
        if in_discriminant != AL::discriminant().get() {
            return Err(GeneratorError::MismatchedDiscriminant {
                account: info.key,
                received: in_discriminant,
                expected: AL::discriminant().get(),
            }
            .into());
        }

        let data = match A::deserialize(&mut account_data) {
            Ok(data) => data,
            Err(_) => {
                return Err(GeneratorError::CouldNotDeserialize {
                    what: format!("account: `{}`", info.key),
                }
                .into())
            }
        };
        drop(account_data_ref);
        Ok(Self {
            info,
            data,
            phantom_list: PhantomData,
        })
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint()
    }
}
impl<AL, A> MultiIndexableAccountArgument<()> for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn is_signer(&self, indexer: ()) -> GeneratorResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: ()) -> GeneratorResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: ()) -> GeneratorResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<AL, A> MultiIndexableAccountArgument<AllAny> for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn is_signer(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> GeneratorResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<AL, A> SingleIndexableAccountArgument<()> for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn owner(&self, indexer: ()) -> GeneratorResult<&Rc<RefCell<&'static mut Pubkey>>> {
        self.info.owner(indexer)
    }

    fn key(&self, indexer: ()) -> GeneratorResult<&'static Pubkey> {
        self.info.key(indexer)
    }
}
impl<AL, A> Deref for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<AL, A> DerefMut for ProgramAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
