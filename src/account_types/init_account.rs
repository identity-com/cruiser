//! Initializes an account

use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::account_list::AccountListItem;
use crate::account_types::discriminant_account::{DiscriminantAccount, WriteDiscriminant};
use crate::account_types::system_program::SystemProgram;
use crate::pda_seeds::PDASeedSet;
use crate::AllAny;
use crate::{AccountInfo, CruiserResult};
use cruiser_derive::verify_account_arg_impl;

verify_account_arg_impl! {
    mod init_account_check{
        <AL, A> InitAccount<AL, A>
        where
            AL: AccountListItem<A>,
            A: BorshSerialize + BorshDeserialize{
            from: [
                /// The initial value for the account's data
                A;
            ];
            validate: [<'a> InitArgs<'a>];
            multi: [(); AllAny];
            single: [()];
        }
    }
}

/// The arguments for initializing an account
#[derive(Debug)]
pub struct InitArgs<'a> {
    /// The system program to initalize the account
    pub system_program: &'a SystemProgram,
    /// The space for the account being created
    pub space: usize,
    /// The funder for the newly created account, must be owned by the system program
    pub funder: &'a AccountInfo,
    /// The seeds for the funder if PDA
    pub funder_seeds: Option<&'a PDASeedSet<'a>>,
    /// The rent to use, if [`None`] will use [`Rent::get`].
    pub rent: Option<Rent>,
}

/// Initializes a given account to be rent exempt and owned by the current program.
///
/// - `AL`: The [`AccountList`](crate::account_list::AccountList) that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<A>`](AccountListItem)
#[derive(AccountArgument)]
#[account_argument(no_from, no_validate)]
pub struct InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    account: DiscriminantAccount<AL, A>,
}
impl<AL, A> Debug for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InitAccount")
            .field("account", &self.account)
            .finish()
    }
}
impl<AL, A> Deref for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    type Target = DiscriminantAccount<AL, A>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
impl<AL, A> DerefMut for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}
impl<'a, AL, A> FromAccounts<A> for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: A,
    ) -> CruiserResult<Self> {
        Ok(Self {
            account: DiscriminantAccount::<AL, A>::from_accounts(program_id, infos, (arg,))?,
        })
    }

    fn accounts_usage_hint(_arg: &A) -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint(&())
    }
}
impl<'a, AL, A> ValidateArgument<InitArgs<'a>> for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: InitArgs<'a>) -> CruiserResult<()> {
        let rent = match arg.rent {
            None => Rent::get()?,
            Some(rent) => rent,
        }
        .minimum_balance(arg.space);

        match arg.funder_seeds {
            Some(seeds) => arg.system_program.invoke_signed_create_account(
                &[seeds],
                arg.funder,
                &self.account.info,
                rent,
                arg.space as u64,
                program_id,
            )?,
            None => arg.system_program.invoke_create_account(
                arg.funder,
                &self.account.info,
                rent,
                arg.space as u64,
                program_id,
            )?,
        }
        self.account.validate(program_id, WriteDiscriminant)
    }
}
impl<'a, AL, A, T> MultiIndexable<T> for InitAccount<AL, A>
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
impl<'a, AL, A, T> SingleIndexable<T> for InitAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.account.info(indexer)
    }
}

// /// The size the account will be initialized to.
// #[derive(Clone, Debug)]
// pub enum InitSize {
//     /// Exact size needed for data
//     DataSize,
//     /// Size needed plus extra
//     DataSizePlus(NonZeroU64),
//     /// A set size, will error if not enough
//     SetSize(u64),
// }
// impl Default for InitSize {
//     fn default() -> Self {
//         Self::DataSize
//     }
// }
// /// An account that will be initialized by this instruction, must sign.
// /// State given is owned by system program and not allocated.
// /// Will be allocated and transferred to this program.
// /// Requires system program is passed on write back.
// #[derive(Debug)]
// pub struct InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
// {
//     /// The [`AccountInfo`] for this, data field will be overwritten on write back.
//     pub info: AccountInfo,
//     data: A,
//     /// The size the account will be given on write back, set it directly
//     pub init_size: InitSize,
//     /// The account that will pay the rent for the new account
//     pub funder: Option<AccountInfo>,
//     /// The optional seeds for the account if pda
//     pub account_seeds: Option<PDASeedSet<'static>>,
//     /// The option seeds for the funder if pda
//     pub funder_seeds: Option<PDASeedSet<'static>>,
//     phantom_list: PhantomData<fn() -> AL>,
// }
// impl<AL, A> AccountArgument for InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: BorshSerialize,
// {
//     fn write_back(
//         self,
//         program_id: &Pubkey,
//         system_program: Option<&SystemProgram>,
//     ) -> CruiserResult<()> {
//         let system_program = match system_program {
//             None => return Err(CruiserError::MissingSystemProgram.into()),
//             Some(system_program) => {
//                 debug_assert_eq!(system_program.info.key, &SYSTEM_PROGRAM_ID,);
//                 system_program
//             }
//         };
//
//         let data = self.data.try_to_vec()?;
//         let required_length = (data.len() + AL::compressed_discriminant().num_bytes()) as u64;
//         let size = match self.init_size {
//             InitSize::DataSize => required_length,
//             InitSize::DataSizePlus(plus) => required_length + plus.get(),
//             InitSize::SetSize(size) => {
//                 if size < required_length {
//                     return Err(CruiserError::NotEnoughSpaceInit {
//                         account: self.info.key,
//                         space_given: size,
//                         space_needed: required_length,
//                     }
//                     .into());
//                 }
//                 size
//             }
//         };
//
//         let self_key = self.info.key;
//         let funder = self
//             .funder
//             .ok_or(CruiserError::NoPayerForInit { account: self_key })?;
//         match (
//             self.funder_seeds.is_some() || funder.is_signer,
//             funder.is_writable,
//         ) {
//             (false, _) => {
//                 return Err(CruiserError::AccountIsNotSigner {
//                     account: funder.key,
//                 }
//                 .into())
//             }
//             (_, false) => {
//                 return Err(CruiserError::CannotWrite {
//                     account: funder.key,
//                 }
//                 .into())
//             }
//             (true, true) => {}
//         }
//         let rent = Rent::get()?.minimum_balance(size as usize);
//         if **funder.lamports.borrow() < rent {
//             return Err(CruiserError::NotEnoughLamports {
//                 account: funder.key,
//                 lamports: **funder.lamports.borrow(),
//                 needed_lamports: rent,
//             }
//             .into());
//         }
//         match (self.account_seeds, self.funder_seeds) {
//             (None, None) => invoke(
//                 &create_account(funder.key, self.info.key, rent, size, program_id),
//                 &[&self.info, &funder, &system_program.info],
//             )?,
//             (account_seeds, funder_seeds) => {
//                 let mut seeds = ShortVec::<_, 2>::new();
//
//                 if let Some(account_seeds) = account_seeds {
//                     seeds.push(account_seeds).unwrap();
//                 } else if !self.info.is_signer {
//                     return Err(CruiserError::AccountIsNotSigner {
//                         account: self.info.key,
//                     }
//                     .into());
//                 }
//                 if let Some(funder_seeds) = funder_seeds {
//                     seeds.push(funder_seeds).unwrap();
//                 }
//
//                 PDASeedSet::invoke_signed_multiple(
//                     &create_account(funder.key, self.info.key, rent, size, program_id),
//                     &[&self.info, &funder, &system_program.info],
//                     seeds.as_slice(),
//                 )?;
//             }
//         }
//
//         let mut account_data_ref = self.info.data.borrow_mut();
//         let mut account_data = &mut **account_data_ref.deref_mut();
//         AL::compressed_discriminant().serialize(&mut account_data)?;
//         account_data.write_all(&data)?;
//         Ok(())
//     }
//
//     fn add_keys(
//         &self,
//         add: impl FnMut(&'static Pubkey) -> CruiserResult<()>,
//     ) -> CruiserResult<()> {
//         self.info.add_keys(add)
//     }
// }
// impl<AL, A, Arg> FromAccounts<Arg> for InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: BorshSerialize + BorshDeserialize + Default,
//     AccountInfo: FromAccounts<Arg>,
// {
//     fn from_accounts(
//         program_id: &'static Pubkey,
//         infos: &mut impl AccountInfoIterator,
//         arg: Arg,
//     ) -> CruiserResult<Self> {
//         let info = AccountInfo::from_accounts(program_id, infos, arg)?;
//
//         if *info.owner.borrow() != &SYSTEM_PROGRAM_ID {
//             return Err(CruiserError::AccountOwnerNotEqual {
//                 account: info.key,
//                 owner: **info.owner.borrow(),
//                 expected_owner: vec![SYSTEM_PROGRAM_ID],
//             }
//             .into());
//         }
//
//         if !info.is_writable {
//             return Err(CruiserError::CannotWrite { account: info.key }.into());
//         }
//
//         Ok(Self {
//             info,
//             data: A::default(),
//             init_size: InitSize::default(),
//             funder: None,
//             account_seeds: None,
//             funder_seeds: None,
//             phantom_list: PhantomData,
//         })
//     }
//
//     fn accounts_usage_hint(arg: &Arg) -> (usize, Option<usize>) {
//         AccountInfo::accounts_usage_hint(arg)
//     }
// }
// impl<AL, A> MultiIndexable<()> for InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: BorshSerialize + BorshDeserialize + Default,
// {
//     fn is_signer(&self, indexer: ()) -> CruiserResult<bool> {
//         self.info.is_signer(indexer)
//     }
//
//     fn is_writable(&self, indexer: ()) -> CruiserResult<bool> {
//         self.info.is_writable(indexer)
//     }
//
//     fn is_owner(&self, owner: &Pubkey, indexer: ()) -> CruiserResult<bool> {
//         self.info.is_owner(owner, indexer)
//     }
// }
// impl<AL, A> MultiIndexable<AllAny> for InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: BorshSerialize + Default,
// {
//     fn is_signer(&self, indexer: AllAny) -> CruiserResult<bool> {
//         self.info.is_signer(indexer)
//     }
//
//     fn is_writable(&self, indexer: AllAny) -> CruiserResult<bool> {
//         self.info.is_writable(indexer)
//     }
//
//     fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> CruiserResult<bool> {
//         self.info.is_owner(owner, indexer)
//     }
// }
// impl<AL, A> SingleIndexable<()> for InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
//     A: BorshSerialize + BorshDeserialize + Default,
// {
//     fn info(&self, indexer: ()) -> CruiserResult<&AccountInfo> {
//         self.info.info(indexer)
//     }
// }
// impl<AL, A> Deref for InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
// {
//     type Target = A;
//
//     fn deref(&self) -> &Self::Target {
//         &self.data
//     }
// }
// impl<AL, A> DerefMut for InitAccount<AL, A>
// where
//     AL: AccountListItem<A>,
// {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.data
//     }
// }
