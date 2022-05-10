//! Initializes an account

use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use crate::cpi::CPIMethod;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable, ValidateArgument};
use crate::account_list::AccountListItem;
use crate::account_types::discriminant_account::{DiscriminantAccount, WriteDiscriminant};
use crate::account_types::system_program::{CreateAccount, SystemProgram};
use crate::compressed_numbers::CompressedNumber;
use crate::on_chain_size::OnChainSizeWithArg;
use crate::pda_seeds::PDASeedSet;
use crate::prelude::OnChainSize;
use crate::CruiserResult;
use crate::{AccountInfo, ToSolanaAccountInfo};

// verify_account_arg_impl! {
//     mod init_account_check <AI> {
//         <AI, AL, D> InitAccount<AI, AL, D>
//         where
//             AI: AccountInfo,
//             AL: AccountListItem<D>,
//             D: BorshSerialize + BorshDeserialize{
//             from: [
//                 /// The initial value for the account's data
//                 D;
//             ];
//             validate: [<'a, 'b, C> InitArgs<'a, AI, C> where AI: 'a + ToSolanaAccountInfo<'b>, C: CPI];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// The arguments for initializing an account
#[derive(Debug, Clone)]
pub struct InitArgs<'a, C, F, S, SP> {
    /// The system program to initalize the account
    pub system_program: SP,
    /// The space for the account being created
    pub space: S,
    /// The funder for the newly created account, must be owned by the system program
    pub funder: F,
    /// The seeds for the funder if PDA
    pub funder_seeds: Option<&'a PDASeedSet<'a>>,
    /// The seeds for the account if PDA
    pub account_seeds: Option<&'a PDASeedSet<'a>>,
    /// The rent to use, if [`None`] will use [`Rent::get`].
    pub rent: Option<Rent>,
    /// The CPI method to use
    pub cpi: C,
}

/// Initializes a given account to be rent exempt and owned by the current program.
///
/// - `AL`: The [`AccountList`](crate::account_list::AccountList) that is valid for `A`
/// - `A` The account data, `AL` must implement [`AccountListItem<A>`](AccountListItem)
#[derive(AccountArgument)]
#[account_argument(account_info = AI, no_validate, generics = [where AI: AccountInfo])]
#[from(data = (val: D), generics = [where AI: AccountInfo, AL: AccountListItem < D >, D: BorshSerialize])]
pub struct InitAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    #[from(data = (val,))]
    account: DiscriminantAccount<AI, AL, D>,
}

impl<AI, AL, D> Debug for InitAccount<AI, AL, D>
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

impl<AI, AL, D> Deref for InitAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    type Target = DiscriminantAccount<AI, AL, D>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}

impl<AI, AL, D> DerefMut for InitAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}

/// Bases the struct size on [`OnChainSize`] implementation.
#[derive(Debug, Clone, Copy)]
pub struct InitStaticSized;

impl<'a, 'b, AI, AL, D, C>
    ValidateArgument<InitArgs<'a, C, &'a AI, InitStaticSized, &'a SystemProgram<AI>>>
    for InitAccount<AI, AL, D>
where
    AI: ToSolanaAccountInfo<'b>,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize + OnChainSize,
    C: CPIMethod,
{
    fn validate(
        &mut self,
        program_id: &Pubkey,
        arg: InitArgs<'a, C, &'a AI, InitStaticSized, &'a SystemProgram<AI>>,
    ) -> CruiserResult<()> {
        self.validate(
            program_id,
            InitArgs {
                system_program: arg.system_program,
                space: AL::compressed_discriminant().num_bytes() + D::ON_CHAIN_SIZE,
                funder: arg.funder,
                funder_seeds: arg.funder_seeds,
                account_seeds: arg.account_seeds,
                rent: arg.rent,
                cpi: arg.cpi,
            },
        )
    }
}

/// Bases the struct size on [`OnChainSizeWithArg`] implementation.
#[derive(Debug, Clone, Copy)]
pub struct InitSizeWithArg<A>(pub A);

impl<'a, 'b, AI, AL, D, C, A>
    ValidateArgument<InitArgs<'a, C, &'a AI, InitSizeWithArg<A>, &'a SystemProgram<AI>>>
    for InitAccount<AI, AL, D>
where
    AI: ToSolanaAccountInfo<'b>,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize + OnChainSizeWithArg<A>,
    C: CPIMethod,
{
    fn validate(
        &mut self,
        program_id: &Pubkey,
        arg: InitArgs<'a, C, &'a AI, InitSizeWithArg<A>, &'a SystemProgram<AI>>,
    ) -> CruiserResult<()> {
        self.validate(
            program_id,
            InitArgs {
                system_program: arg.system_program,
                space: AL::compressed_discriminant().num_bytes()
                    + D::on_chain_size_with_arg(arg.space.0),
                funder: arg.funder,
                funder_seeds: arg.funder_seeds,
                account_seeds: arg.account_seeds,
                rent: arg.rent,
                cpi: arg.cpi,
            },
        )
    }
}

impl<'a, 'b, AI, AL, D, C> ValidateArgument<InitArgs<'a, C, &'a AI, usize, &'a SystemProgram<AI>>>
    for InitAccount<AI, AL, D>
where
    AI: ToSolanaAccountInfo<'b>,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    C: CPIMethod,
{
    fn validate(
        &mut self,
        program_id: &Pubkey,
        arg: InitArgs<'a, C, &'a AI, usize, &'a SystemProgram<AI>>,
    ) -> CruiserResult<()> {
        let rent = match arg.rent {
            None => Rent::get()?,
            Some(rent) => rent,
        }
        .minimum_balance(AL::compressed_discriminant().num_bytes() as usize + arg.space);

        let seeds = arg.funder_seeds.into_iter().chain(arg.account_seeds);

        arg.system_program.create_account(
            arg.cpi,
            &CreateAccount {
                funder: arg.funder,
                account: &self.info,
                lamports: rent,
                space: arg.space as u64,
                owner: program_id,
            },
            seeds,
        )?;
        self.account.validate(program_id, WriteDiscriminant)
    }
}

impl<'a, AI, AL, D, T> MultiIndexable<T> for InitAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AI, AL, D>: MultiIndexable<T>,
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

impl<'a, AI, AL, D, T> SingleIndexable<T> for InitAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AI, AL, D>: SingleIndexable<T, AccountInfo = AI>,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.account.index_info(indexer)
    }
}
