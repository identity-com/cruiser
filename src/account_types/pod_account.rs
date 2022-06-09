//! An account that allows the usage of any [`Pod`] type.

use crate::prelude::*;
use crate::util::validate_discriminant;
use cruiser::util::assert_is_zeroed;
use std::mem::{align_of, size_of};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

/// The type of data a [`PodAccount`] allows access to.
#[derive(Debug)]
#[repr(C)]
pub struct PodData<D> {
    /// The data
    pub data: D,
    /// The remaining bytes
    pub remaining: [u8],
}

/// An account that allows the usage of any [`Pod`] type.
#[derive(AccountArgument, Debug, Clone)]
#[account_argument(
    account_info = AI,
    no_validate,
    generics = [where AI: AccountInfo, AL: AccountListItem<D>, D: Pod],
)]
#[from(data = (from_arg: A), generics = [<A> where AI: FromAccounts<A>], no_single_tupple)]
pub struct PodAccount<AI, AL, D> {
    #[from(data = from_arg)]
    info: AI,
    phantom_d: PhantomAccount<AI, D>,
    phantom_al: PhantomAccount<AI, AL>,
}
impl<AI, AL, D> PodAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: Pod,
{
    /// Gets the offset to the start of the data.
    ///
    /// [`u128`]'s alignment on bpf is [`u64`]'s align rather than the normal double that.
    /// This means that if you use a [`u128`] you need to pack it to the alignment of a [`u64`] to maintain parity.
    #[must_use]
    pub fn data_offset() -> usize {
        assert!(
            align_of::<D>() <= align_of::<u64>(),
            "Data has too large of alignment"
        );
        let x = 1u64;
        unsafe {
            std::ptr::addr_of!(x)
                .cast::<u8>()
                .add(AL::compressed_discriminant().num_bytes())
                .align_offset(align_of::<D>())
        }
    }

    /// Gets the data withing the account.
    pub fn data(&self) -> CruiserResult<impl Deref<Target = PodData<D>> + '_> {
        self.info.data().try_map_ref(|mut data: &[u8]| {
            data.try_advance(Self::data_offset())?;
            let pod_data = data.try_advance(size_of::<D>())?;
            unsafe {
                Ok(&*(slice_from_raw_parts(pod_data.as_ptr(), data.len()) as *const PodData<D>))
            }
        })
    }

    /// Gets the data withing the account mutably.
    pub fn data_mut(&mut self) -> CruiserResult<impl DerefMut<Target = PodData<D>> + '_> {
        self.info.data_mut().try_map_ref_mut(|mut data: &mut [u8]| {
            data.try_advance(Self::data_offset())?;
            let pod_data = data.try_advance(size_of::<D>())?;
            unsafe {
                Ok(
                    &mut *(slice_from_raw_parts_mut(pod_data.as_mut_ptr(), data.len())
                        as *mut PodData<D>),
                )
            }
        })
    }
}
impl<AI, AL, D> ValidateArgument<()> for PodAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: Pod,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult<()> {
        self.info.validate(program_id, arg)?;
        assert_is_owner(&self.info, program_id, ())?;
        validate_discriminant::<AL, D>(&mut &*self.info.data())?;
        Ok(())
    }
}
/// Checks that the account is owned by the given program id.
#[derive(Debug, Clone)]
pub struct PodOwner<'a>(pub &'a Pubkey);
impl<'a, AI, AL, D> ValidateArgument<PodOwner<'a>> for PodAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: Pod,
{
    fn validate(&mut self, program_id: &Pubkey, arg: PodOwner<'a>) -> CruiserResult<()> {
        self.info.validate(program_id, ())?;
        assert_is_owner(&self.info, arg.0, ())?;
        validate_discriminant::<AL, D>(&mut &*self.info.data())?;
        Ok(())
    }
}
/// Checks the account was zeroed and sets the discriminant
#[derive(Debug, Clone)]
pub struct PodFromZeroed;
impl<AI, AL, D> ValidateArgument<PodFromZeroed> for PodAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: Pod,
{
    fn validate(&mut self, program_id: &Pubkey, _arg: PodFromZeroed) -> CruiserResult<()> {
        assert_is_owner(&self.info, program_id, ())?;
        let mut data = self.info.data_mut();
        assert_is_zeroed::<AL>(&*data, self.info.key(), false)?;

        AL::compressed_discriminant().serialize(&mut &mut *data)?;
        Ok(())
    }
}
/// Initializes a [`PodAccount`] with a CPI to the system program.
#[derive(Debug, Clone)]
pub struct PodInit<'a, AI, C> {
    /// The system program
    pub system_program: &'a SystemProgram<AI>,
    /// The funder for the new account
    pub funder: &'a AI,
    /// The seeds for the account if PDA
    pub account_seeds: Option<&'a PDASeedSet<'a>>,
    /// The seeds for the funder if PDA
    pub funder_seeds: Option<&'a PDASeedSet<'a>>,
    /// Additional space on the end in addition to the space needed for the discriminant and data
    pub extra_space: usize,
    /// The rent object to use for rent calculation. If [`None`] then [`Rent::get`] is used.
    pub rent: Option<Rent>,
    /// The [`CPIMethod`] to use for the initialization.
    pub cpi: C,
}
impl<'a, AI, C> PodInit<'a, AI, C> {
    /// Crates a new [`PodInit`] with minimally required arguments
    #[must_use]
    pub fn new(system_program: &'a SystemProgram<AI>, funder: &'a AI, cpi: C) -> Self {
        Self {
            system_program,
            funder,
            account_seeds: None,
            funder_seeds: None,
            extra_space: 0,
            rent: None,
            cpi,
        }
    }

    /// Sets the [`PodInit::account_seeds`] field.
    #[must_use]
    pub fn account_seeds(mut self, account_seeds: &'a PDASeedSet<'a>) -> Self {
        self.account_seeds = Some(account_seeds);
        self
    }

    /// Sets the [`PodInit::funder_seeds`] field.
    #[must_use]
    pub fn funder_seeds(mut self, funder_seeds: &'a PDASeedSet<'a>) -> Self {
        self.funder_seeds = Some(funder_seeds);
        self
    }

    /// Sets the [`PodInit::extra_space`] field.
    #[must_use]
    pub fn extra_space(mut self, extra_space: usize) -> Self {
        self.extra_space = extra_space;
        self
    }

    /// Sets the [`PodInit::rent`] field.
    #[must_use]
    pub fn rent(mut self, rent: Rent) -> Self {
        self.rent = Some(rent);
        self
    }
}
impl<'a, AI, AL, D, C> ValidateArgument<PodInit<'a, AI, C>> for PodAccount<AI, AL, D>
where
    AI: ToSolanaAccountInfo<'a>,
    AL: AccountListItem<D>,
    D: Pod,
    C: CPIMethod,
{
    fn validate(&mut self, program_id: &Pubkey, arg: PodInit<'a, AI, C>) -> CruiserResult<()> {
        let rent = match arg.rent {
            None => Rent::get()?,
            Some(rent) => rent,
        };
        let space = Self::data_offset() + arg.extra_space;
        arg.system_program.create_account(
            arg.cpi,
            &CreateAccount {
                funder: arg.funder,
                account: &self.info,
                lamports: rent.minimum_balance(space),
                space: space as u64,
                owner: program_id,
            },
            arg.funder_seeds.into_iter().chain(arg.account_seeds),
        )?;

        Ok(())
    }
}
impl<AI, AL, D, A> MultiIndexable<A> for PodAccount<AI, AL, D>
where
    AI: AccountInfo + MultiIndexable<A>,
    AL: AccountListItem<D>,
    D: Pod,
{
    fn index_is_signer(&self, indexer: A) -> CruiserResult<bool> {
        self.info.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: A) -> CruiserResult<bool> {
        self.info.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: A) -> CruiserResult<bool> {
        self.info.index_is_owner(owner, indexer)
    }
}
impl<AI, AL, D, A> SingleIndexable<A> for PodAccount<AI, AL, D>
where
    AI: AccountInfo + SingleIndexable<A>,
    AL: AccountListItem<D>,
    D: Pod,
{
    fn index_info(&self, indexer: A) -> CruiserResult<&Self::AccountInfo> {
        self.info.index_info(indexer)
    }
}
