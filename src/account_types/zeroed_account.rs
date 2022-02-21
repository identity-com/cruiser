use std::cell::RefCell;
use std::ops::{Deref, DerefMut};

use borsh::BorshSerialize;
use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;

use crate::compressed_numbers::CompressedNumber;
use crate::solana_program::sysvar::Sysvar;
use crate::traits::AccountArgument;
use crate::{
    AccountInfo, AccountInfoIterator, AccountListItem, AllAny, FromAccounts, GeneratorError,
    GeneratorResult, MultiIndexableAccountArgument, SingleIndexableAccountArgument, SystemProgram,
};
use solana_program::rent::Rent;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::rc::Rc;

/// An account that will be initialized by this program, all data is checked to be zeroed and owner is this program.
/// Account must be rent exempt.
#[derive(Debug)]
pub struct ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    /// The [`AccountInfo`] for this, data field will be overwritten on write back.
    pub info: AccountInfo,
    data: A,
    phantom_list: PhantomData<fn() -> AL>,
}
impl<AL, A> AccountArgument for ZeroedAccount<AL, A>
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
impl<AL, A, Arg> FromAccounts<Arg> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + Default,
    AccountInfo: FromAccounts<Arg>,
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

        if !info.is_writable {
            return Err(GeneratorError::CannotWrite { account: info.key }.into());
        }

        if !info
            .data
            .borrow()
            .iter()
            .take(AL::DiscriminantCompressed::max_bytes())
            .all(|&byte| byte == 0)
        {
            return Err(GeneratorError::NonZeroedData { account: info.key }.into());
        }

        let rent = Rent::get()?.minimum_balance(info.data.borrow().len());
        if **info.lamports.borrow() < rent {
            return Err(ProgramError::AccountNotRentExempt.into());
        }

        Ok(Self {
            info,
            data: A::default(),
            phantom_list: PhantomData,
        })
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint()
    }
}
impl<AL, A> MultiIndexableAccountArgument<()> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + Default,
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
impl<AL, A> MultiIndexableAccountArgument<AllAny> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + Default,
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
impl<AL, A> SingleIndexableAccountArgument<()> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + Default,
{
    fn info(&self, indexer: ()) -> GeneratorResult<&AccountInfo> {
        self.info.info(indexer)
    }
}
impl<AL, A> Deref for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<AL, A> DerefMut for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
