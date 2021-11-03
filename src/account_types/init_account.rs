use std::io::Write;
use std::num::NonZeroU64;
use std::ops::{Deref, DerefMut};

use borsh::BorshSerialize;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::create_account;

use crate::traits::AccountArgument;
use crate::{
    invoke, Account, AccountInfo, AccountInfoIterator, AllAny, FromAccounts, GeneratorError,
    GeneratorResult, MultiIndexableAccountArgument, PDASeedSet, ShortVec,
    SingleIndexableAccountArgument, SystemProgram,
};

use super::SYSTEM_PROGRAM_ID;
use crate::solana_program::rent::Rent;
use crate::solana_program::sysvar::Sysvar;
use std::fmt::Debug;

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
pub struct InitAccount<T>
where
    T: Account,
{
    /// The [`AccountInfo`] for this, data field will be overwritten on write back.
    pub info: AccountInfo,
    data: T,
    /// The size the account will be given on write back, set it directly
    pub init_size: InitSize,
    /// The account that will pay the rent for the new account
    pub funder: Option<AccountInfo>,
    /// The optional seeds for the account if pda
    pub account_seeds: Option<PDASeedSet<'static>>,
    /// The option seeds for the funder if pda
    pub funder_seeds: Option<PDASeedSet<'static>>,
}
impl<T> AccountArgument for InitAccount<T>
where
    T: Account,
{
    fn write_back(
        self,
        program_id: Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        let system_program = match system_program {
            None => return Err(GeneratorError::MissingSystemProgram.into()),
            Some(system_program) => {
                assert_eq!(system_program.info.key, SYSTEM_PROGRAM_ID);
                system_program
            }
        };

        let data = self.data.try_to_vec()?;
        let size = match self.init_size {
            InitSize::DataSize => {
                (data.len() + T::DISCRIMINANT.discriminant_serialized_length()?) as u64
            }
            InitSize::DataSizePlus(plus) => {
                (data.len() + T::DISCRIMINANT.discriminant_serialized_length()?) as u64 + plus.get()
            }
            InitSize::SetSize(size) => {
                if size < (data.len() + T::DISCRIMINANT.discriminant_serialized_length()?) as u64 {
                    return Err(GeneratorError::NotEnoughSpaceInit {
                        account: self.info.key,
                        space_given: size,
                        space_needed: data.len() as u64,
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
                &create_account(&funder.key, &self.info.key, rent, size, &program_id),
                &[&self.info, &funder, &system_program.info],
            )?,
            (account_seeds, funder_seeds) => {
                let mut seeds = ShortVec::<_, 2>::new();

                if let Some(account_seeds) = account_seeds {
                    seeds.push(account_seeds).unwrap();
                }
                if let Some(funder_seeds) = funder_seeds {
                    seeds.push(funder_seeds).unwrap();
                }

                PDASeedSet::invoke_signed_multiple(
                    &create_account(&funder.key, &self.info.key, rent, size, &program_id),
                    &[&self.info, &funder, &system_program.info],
                    seeds.as_slice(),
                )?
            }
        }

        let mut account_data_ref = self.info.data.borrow_mut();
        let mut account_data = &mut **account_data_ref.deref_mut();
        T::DISCRIMINANT.serialize(&mut account_data)?;
        account_data.write_all(&data)?;
        Ok(())
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> GeneratorResult<()>) -> GeneratorResult<()> {
        self.info.add_keys(add)
    }
}
impl<T, A> FromAccounts<A> for InitAccount<T>
where
    T: Account + Default,
    AccountInfo: FromAccounts<A>,
{
    fn from_accounts(
        program_id: Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: A,
    ) -> GeneratorResult<Self> {
        let info = AccountInfo::from_accounts(program_id, infos, arg)?;

        if *info.owner.borrow() != &SYSTEM_PROGRAM_ID {
            return Err(GeneratorError::AccountOwnerNotEqual {
                account: info.key,
                owner: **info.owner.borrow(),
                expected_owner: Default::default(),
            }
            .into());
        }

        if !info.is_signer {
            return Err(GeneratorError::AccountIsNotSigner { account: info.key }.into());
        }

        if !info.is_writable {
            return Err(GeneratorError::CannotWrite { account: info.key }.into());
        }

        Ok(Self {
            info,
            data: T::default(),
            init_size: InitSize::default(),
            funder: None,
            account_seeds: None,
            funder_seeds: None,
        })
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint()
    }
}
impl<T> MultiIndexableAccountArgument<()> for InitAccount<T>
where
    T: Account + Default,
{
    fn is_signer(&self, indexer: ()) -> GeneratorResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: ()) -> GeneratorResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: Pubkey, indexer: ()) -> GeneratorResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<T> MultiIndexableAccountArgument<AllAny> for InitAccount<T>
where
    T: Account + Default,
{
    fn is_signer(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: Pubkey, indexer: AllAny) -> GeneratorResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<T> SingleIndexableAccountArgument<()> for InitAccount<T>
where
    T: Account + Default,
{
    fn owner(&self, indexer: ()) -> GeneratorResult<Pubkey> {
        self.info.owner(indexer)
    }

    fn key(&self, indexer: ()) -> GeneratorResult<Pubkey> {
        self.info.key(indexer)
    }
}
impl<T> Deref for InitAccount<T>
where
    T: Account,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<T> DerefMut for InitAccount<T>
where
    T: Account,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}
