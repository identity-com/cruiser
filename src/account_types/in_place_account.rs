//! An account that lets you access data in-place.

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable, ValidateArgument};
use crate::account_list::AccountListItem;
use crate::account_types::system_program::{CreateAccount, SystemProgram};
use crate::account_types::PhantomAccount;
use crate::compressed_numbers::CompressedNumber;
use crate::in_place::InPlaceCreate;
use crate::pda_seeds::PDASeedSet;
use crate::program::ProgramKey;
use crate::util::short_iter::ShortIter;
use crate::util::{MappableRef, MappableRefMut, TryMappableRef, TryMappableRefMut};
use crate::{AccountInfo, CPIMethod, CruiserResult, GenericError, ToSolanaAccountInfo};
use borsh::{BorshDeserialize, BorshSerialize};
use cruiser::in_place::{InPlaceRead, InPlaceWrite};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

/// An account that lets you access data in-place. If created will use init or zeroed logic.
#[derive(AccountArgument, Debug)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo], no_validate)]
pub struct InPlaceAccount<AI, AL, D>(AI, PhantomAccount<AI, (AL, D)>);

impl<AI, AL, D> InPlaceAccount<AI, AL, D>
where
    AI: AccountInfo,
{
    /// Reads the in-place data with an arg
    pub fn read_with_arg<'a, A>(&'a self, arg: A) -> CruiserResult<D::Access<'a, AI::Data<'a>>>
    where
        D: InPlaceRead<A>,
        AI::Data<'a>: MappableRef + TryMappableRef,
    {
        D::read_with_arg(self.0.data(), arg)
    }

    /// Reads the in-place data
    pub fn read<'a>(&'a self) -> CruiserResult<D::Access<'a, AI::Data<'a>>>
    where
        D: InPlaceRead,
        AI::Data<'a>: MappableRef + TryMappableRef,
    {
        self.read_with_arg(())
    }

    /// Writes the in-place data with an arg
    pub fn write_with_arg<'a, A>(
        &'a self,
        arg: A,
    ) -> CruiserResult<D::AccessMut<'a, AI::DataMut<'a>>>
    where
        D: InPlaceWrite<A>,
        AI::DataMut<'a>: MappableRef + TryMappableRef + MappableRefMut + TryMappableRefMut,
    {
        D::write_with_arg(self.0.data_mut(), arg)
    }

    /// Writes the in-place data
    pub fn write<'a>(&'a self) -> CruiserResult<D::AccessMut<'a, AI::DataMut<'a>>>
    where
        D: InPlaceWrite,
        AI::DataMut<'a>: MappableRef + TryMappableRef + MappableRefMut + TryMappableRefMut,
    {
        self.write_with_arg(())
    }
}

impl<AI, AL, D> ValidateArgument for InPlaceAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult {
        self.0.validate(program_id, arg)?;
        self.1.validate(program_id, arg)?;

        let discriminant = AL::DiscriminantCompressed::deserialize(&mut &*self.0.data())?;
        if discriminant == AL::compressed_discriminant() {
            Ok(())
        } else {
            Err(GenericError::MismatchedDiscriminant {
                account: *self.0.key(),
                received: discriminant.into_number().get(),
                expected: AL::discriminant(),
            }
            .into())
        }
    }
}

/// Allows [`InPlaceAccount`] to init a zeroed or system program account.
#[derive(Debug)]
pub struct Create<'a, AI, T, CPI> {
    /// The creation data. See [`InPlaceCreate`] for more details.
    pub data: T,
    /// The system program.
    pub system_program: &'a SystemProgram<AI>,
    /// The rent to use, if non will call [`Rent::get`].
    pub rent: Option<Rent>,
    /// The space to allocate for the account.
    pub space: usize,
    /// The funder of the account.
    pub funder: &'a AI,
    /// The seeds for the funder. Optional.
    pub funder_seeds: Option<&'a PDASeedSet<'a>>,
    /// The seeds for the account. Optional.
    pub account_seeds: Option<&'a PDASeedSet<'a>>,
    /// The [`CPIMethod`] to use.
    pub cpi: CPI,
}

impl<'a, 'b, AI, AL, D, C, CPI> ValidateArgument<Create<'a, AI, C, CPI>>
    for InPlaceAccount<AI, AL, D>
where
    AI: ToSolanaAccountInfo<'b>,
    AL: AccountListItem<D>,
    D: InPlaceCreate<C>,
    CPI: CPIMethod,
{
    fn validate(&mut self, program_id: &Pubkey, arg: Create<'a, AI, C, CPI>) -> CruiserResult {
        self.0.validate(program_id, ())?;
        self.1.validate(program_id, ())?;

        let rent = match arg.rent {
            Some(rent) => rent,
            None => Rent::get()?,
        }
        .minimum_balance(AL::compressed_discriminant().num_bytes() + arg.space);

        let mut seeds = ShortIter::<_, 2>::new();
        if let Some(funder_seeds) = arg.funder_seeds {
            seeds.push(funder_seeds);
        }
        if let Some(account_seeds) = arg.account_seeds {
            seeds.push(account_seeds);
        }

        if *self.0.owner() == SystemProgram::<()>::KEY {
            arg.system_program.create_account(
                arg.cpi,
                &CreateAccount {
                    funder: arg.funder,
                    account: &self.0,
                    lamports: rent,
                    space: 0,
                    owner: program_id,
                },
                seeds,
            )?;
        } else if &*self.0.owner() == program_id {
            if (*self.0.data())
                .iter()
                .take(AL::DiscriminantCompressed::max_bytes())
                .any(|b| *b != 0)
            {
                return Err(GenericError::NonZeroedData {
                    account: *self.0.key(),
                }
                .into());
            }
        } else {
            return Err(GenericError::AccountOwnerNotEqual {
                account: *self.0.key(),
                owner: *self.0.owner(),
                expected_owner: vec![*program_id, SystemProgram::<()>::KEY],
            }
            .into());
        }

        let mut data = &mut *self.0.data_mut();
        AL::compressed_discriminant().serialize(&mut data)?;
        D::create_with_arg(data, arg.data)?;

        Ok(())
    }
}

impl<AI, AL, D, I> MultiIndexable<I> for InPlaceAccount<AI, AL, D>
where
    AI: MultiIndexable<I> + AccountInfo,
{
    fn index_is_signer(&self, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_signer(indexer)
    }
    fn index_is_writable(&self, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_writable(indexer)
    }
    fn index_is_owner(&self, owner: &Pubkey, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_owner(owner, indexer)
    }
}

impl<AI, AL, D, I> SingleIndexable<I> for InPlaceAccount<AI, AL, D>
where
    AI: SingleIndexable<I> + AccountInfo,
{
    fn index_info(&self, indexer: I) -> CruiserResult<&Self::AccountInfo> {
        self.0.index_info(indexer)
    }
}
