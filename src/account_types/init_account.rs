use std::io::Write;
use std::num::NonZeroU64;
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::create_account;

use crate::traits::AccountArgument;
use crate::{
    invoke, AccountInfo, AccountInfoIterator, AccountListItem, AllAny, FromAccounts,
    GeneratorError, GeneratorResult, MultiIndexableAccountArgument, PDASeedSet, ShortVec,
    SingleIndexableAccountArgument, SystemProgram,
};

use super::SYSTEM_PROGRAM_ID;
use crate::compressed_numbers::CompressedNumber;
use crate::solana_program::rent::Rent;
use crate::solana_program::sysvar::Sysvar;
use std::fmt::Debug;
use std::marker::PhantomData;

/// The size the account will be initialized to.
#[derive(Clone, Debug)]
pub enum InitSize {
    /// Exact size needed for data
    DataSize,
    /// Size needed plus extra
    DataSizePlus(NonZeroU64),
    /// A set size, will error if not enough
    SetSize(u64),
}
impl Default for InitSize {
    fn default() -> Self {
        Self::DataSize
    }
}
/// An account that will be initialized by this instruction, must sign.
/// State given is owned by system program and not allocated.
/// Will be allocated and transferred to this program.
/// Requires system program is passed on write back.
#[derive(Debug)]
pub struct InitAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    /// The [`AccountInfo`] for this, data field will be overwritten on write back.
    pub info: AccountInfo,
    data: A,
    /// The size the account will be given on write back, set it directly
    pub init_size: InitSize,
    /// The account that will pay the rent for the new account
    pub funder: Option<AccountInfo>,
    /// The optional seeds for the account if pda
    pub account_seeds: Option<PDASeedSet<'static>>,
    /// The option seeds for the funder if pda
    pub funder_seeds: Option<PDASeedSet<'static>>,
    phantom_list: PhantomData<fn() -> AL>,
}
impl<AL, A> AccountArgument for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn write_back(
        self,
        program_id: &Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        let system_program = match system_program {
            None => return Err(GeneratorError::MissingSystemProgram.into()),
            Some(system_program) => {
                debug_assert_eq!(system_program.info.key, &SYSTEM_PROGRAM_ID,);
                system_program
            }
        };

        let data = self.data.try_to_vec()?;
        let required_length = (data.len() + AL::compressed_discriminant().num_bytes()) as u64;
        let size = match self.init_size {
            InitSize::DataSize => required_length,
            InitSize::DataSizePlus(plus) => required_length + plus.get(),
            InitSize::SetSize(size) => {
                if size < required_length {
                    return Err(GeneratorError::NotEnoughSpaceInit {
                        account: self.info.key,
                        space_given: size,
                        space_needed: required_length,
                    }
                    .into());
                }
                size
            }
        };

        let self_key = self.info.key;
        let funder = self
            .funder
            .ok_or(GeneratorError::NoPayerForInit { account: self_key })?;
        match (
            self.funder_seeds.is_some() || funder.is_signer,
            funder.is_writable,
        ) {
            (false, _) => {
                return Err(GeneratorError::AccountIsNotSigner {
                    account: funder.key,
                }
                .into())
            }
            (_, false) => {
                return Err(GeneratorError::CannotWrite {
                    account: funder.key,
                }
                .into())
            }
            (true, true) => {}
        }
        let rent = Rent::get()?.minimum_balance(size as usize);
        if **funder.lamports.borrow() < rent {
            return Err(GeneratorError::NotEnoughLamports {
                account: funder.key,
                lamports: **funder.lamports.borrow(),
                needed_lamports: rent,
            }
            .into());
        }
        match (self.account_seeds, self.funder_seeds) {
            (None, None) => invoke(
                &create_account(funder.key, self.info.key, rent, size, program_id),
                &[&self.info, &funder, &system_program.info],
            )?,
            (account_seeds, funder_seeds) => {
                let mut seeds = ShortVec::<_, 2>::new();

                if let Some(account_seeds) = account_seeds {
                    seeds.push(account_seeds).unwrap();
                } else if !self.info.is_signer {
                    return Err(GeneratorError::AccountIsNotSigner {
                        account: self.info.key,
                    }
                    .into());
                }
                if let Some(funder_seeds) = funder_seeds {
                    seeds.push(funder_seeds).unwrap();
                }

                PDASeedSet::invoke_signed_multiple(
                    &create_account(funder.key, self.info.key, rent, size, program_id),
                    &[&self.info, &funder, &system_program.info],
                    seeds.as_slice(),
                )?
            }
        }

        let mut account_data_ref = self.info.data.borrow_mut();
        let mut account_data = &mut **account_data_ref.deref_mut();
        AL::compressed_discriminant().serialize(&mut account_data)?;
        account_data.write_all(&data)?;
        Ok(())
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.info.add_keys(add)
    }
}
impl<AL, A, Arg> FromAccounts<Arg> for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize + Default,
    AccountInfo: FromAccounts<Arg>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: Arg,
    ) -> GeneratorResult<Self> {
        let info = AccountInfo::from_accounts(program_id, infos, arg)?;

        if *info.owner.borrow() != &SYSTEM_PROGRAM_ID {
            return Err(GeneratorError::AccountOwnerNotEqual {
                account: info.key,
                owner: **info.owner.borrow(),
                expected_owner: vec![SYSTEM_PROGRAM_ID],
            }
            .into());
        }

        if !info.is_writable {
            return Err(GeneratorError::CannotWrite { account: info.key }.into());
        }

        Ok(Self {
            info,
            data: A::default(),
            init_size: InitSize::default(),
            funder: None,
            account_seeds: None,
            funder_seeds: None,
            phantom_list: PhantomData,
        })
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint()
    }
}
impl<AL, A> MultiIndexableAccountArgument<()> for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize + Default,
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
impl<AL, A> MultiIndexableAccountArgument<AllAny> for InitAccount<AL, A>
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
impl<AL, A> SingleIndexableAccountArgument<()> for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize + Default,
{
    fn info(&self, indexer: ()) -> GeneratorResult<&AccountInfo> {
        self.info.info(indexer)
    }
}
impl<AL, A> Deref for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<AL, A> DerefMut for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
