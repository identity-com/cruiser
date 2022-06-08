//! Account to store lists of [`Pod`] data.

use crate::prelude::*;
use crate::util::assert_is_zeroed;
use std::cmp::Ordering;
use std::fmt::Debug;
use std::mem::{align_of, size_of};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

/// The type of data a [`PodListAccount`] allows access to.
#[derive(Debug)]
pub struct PodListData<H, L> {
    /// The header data
    pub header: H,
    /// The list items
    pub list: [L],
}

/// An account that allows the usage of any [`Pod`] type.
/// It contains a header (`H`) and a list of items (`L`), both requiring [`Pod`].
/// The header's alignment must be >= the alignment of the list's elements.
/// The list item (`L`)'s size must not be zero.
#[derive(AccountArgument, Debug)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo], no_validate)]
pub struct PodListAccount<AI, AL, H, L> {
    info: AI,
    phantom_al: PhantomAccount<AI, AL>,
    phantom_h: PhantomAccount<AI, H>,
    phantom_l: PhantomAccount<AI, L>,
}
impl<AI, AL, H, L> PodListAccount<AI, AL, H, L>
where
    AI: AccountInfo,
    AL: AccountListItem<(H, [L])>,
    H: Pod,
    L: Pod,
{
    /// Gets the offset to the start of the header.
    ///
    /// [`u128`]'s alignment on bpf is [`u64`]'s align rather than the normal double that.
    /// This means that if you use a [`u128`] you need to pack it to the alignment of a [`u64`] to maintain parity.
    #[must_use]
    pub fn header_offset() -> usize {
        assert!(
            align_of::<H>() <= align_of::<u64>(),
            "Header has too large of alignment"
        );
        assert!(
            align_of::<L>() <= align_of::<H>(),
            "List alignment greater than header alignment"
        );
        assert!(size_of::<L>() > 0, "List item has size 0");
        let x = 1u64;
        unsafe {
            std::ptr::addr_of!(x)
                .cast::<u8>()
                .add(AL::compressed_discriminant().num_bytes())
                .align_offset(align_of::<H>())
        }
    }

    /// Gets the data within the account.
    pub fn data(&self) -> CruiserResult<impl Deref<Target = PodListData<H, L>> + '_> {
        self.info.data().try_map_ref(|mut data: &[u8]| {
            data.try_advance(Self::header_offset())?;
            let header = data.try_advance(size_of::<H>())?;
            let list_length = data.len() / size_of::<L>();
            unsafe {
                Ok(&*(slice_from_raw_parts(header.as_ptr(), list_length)
                    as *const PodListData<H, L>))
            }
        })
    }

    /// Gets the data within the account mutably.
    pub fn data_mut(&mut self) -> CruiserResult<impl DerefMut<Target = PodListData<H, L>> + '_> {
        self.info.data_mut().try_map_ref_mut(|mut data: &mut [u8]| {
            data.try_advance(Self::header_offset())?;
            let header = data.try_advance(size_of::<H>())?;
            let list_length = data.len() / size_of::<L>();
            unsafe {
                Ok(
                    &mut *(slice_from_raw_parts_mut(header.as_mut_ptr(), list_length)
                        as *mut PodListData<H, L>),
                )
            }
        })
    }

    /// Gets the length of the list.
    pub fn list_len(&self) -> usize {
        Self::list_len_inner(&*self.info.data())
    }

    fn list_len_inner(data: &[u8]) -> usize {
        data.len()
            .saturating_sub(Self::header_offset() + size_of::<H>())
            / size_of::<L>()
    }

    #[allow(clippy::too_many_arguments)]
    fn set_list_length_inner<'a>(
        &mut self,
        realloc: impl FnOnce(&AI, usize, bool) -> CruiserResult,
        length: usize,
        zero_init: bool,
        funds: &AI,
        system_program: Option<(&SystemProgram<AI>, impl CPIMethod)>,
        funder_seeds: Option<&PDASeedSet>,
        rent: Option<Rent>,
    ) -> CruiserResult
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        let new_space = Self::header_offset() + length * size_of::<L>();
        realloc(&self.info, new_space, zero_init)?;
        let rent = match rent {
            Some(rent) => rent,
            None => Rent::get()?,
        }
        .minimum_balance(new_space);
        let mut self_lamports = self.info.lamports_mut();
        match rent.cmp(&*self_lamports) {
            Ordering::Less => {
                *funds.lamports_mut() += *self_lamports - rent;
                *self_lamports = rent;
            }
            Ordering::Equal => {}
            Ordering::Greater => match system_program {
                None => {
                    *funds.lamports_mut() -= rent - *self_lamports;
                    *self_lamports = rent;
                }
                Some((system_program, cpi)) => {
                    system_program.transfer(
                        cpi,
                        funds,
                        &self.info,
                        rent - *self_lamports,
                        funder_seeds,
                    )?;
                }
            },
        }
        Ok(())
    }

    /// Sets the list length using the [`AccountInfo::realloc_unsafe`] method.
    /// `system_program` should be [`Some`] if funder is owned by the system program or [`None`] if funder is owned by this program.
    /// `funder_seeds` is only used if `system_program` is [`Some`].
    /// `rent` will be defaulted to [`Rent::get`] if [`None`].
    ///
    /// # Safety
    /// This function has the same requirements as [`AccountInfo::realloc_unsafe`].
    pub unsafe fn set_list_length_unsafe<'a>(
        &mut self,
        length: usize,
        zero_init: bool,
        funds: &AI,
        system_program: Option<(&SystemProgram<AI>, impl CPIMethod)>,
        funder_seeds: Option<&PDASeedSet>,
        rent: Option<Rent>,
    ) -> CruiserResult
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        self.set_list_length_inner(
            |ai, length, zero| ai.realloc_unsafe(length, zero),
            length,
            zero_init,
            funds,
            system_program,
            funder_seeds,
            rent,
        )
    }

    /// Sets the list length using the [`SafeRealloc::realloc`] method.
    /// `system_program` should be [`Some`] if funder is owned by the system program or [`None`] if funder is owned by this program.
    /// `funder_seeds` is only used if `system_program` is [`Some`].
    /// `rent` will be defaulted to [`Rent::get`] if [`None`].
    pub fn set_list_length<'a>(
        &mut self,
        length: usize,
        zero_init: bool,
        funds: &AI,
        system_program: Option<(&SystemProgram<AI>, impl CPIMethod)>,
        funder_seeds: Option<&PDASeedSet>,
        rent: Option<Rent>,
    ) -> CruiserResult
    where
        AI: ToSolanaAccountInfo<'a> + SafeRealloc,
    {
        self.set_list_length_inner(
            AI::realloc,
            length,
            zero_init,
            funds,
            system_program,
            funder_seeds,
            rent,
        )
    }
}
impl<AI, AL, H, L> ValidateArgument for PodListAccount<AI, AL, H, L>
where
    AI: AccountInfo,
    AL: AccountListItem<(H, [L])>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult<()> {
        self.info.validate(program_id, arg)?;
        assert_is_owner(&self.info, program_id, ())?;
        validate_discriminant::<AL, (H, [L])>(&mut &*self.info.data())?;
        Ok(())
    }
}
impl<'a, AI, AL, H, L> ValidateArgument<PodOwner<'a>> for PodListAccount<AI, AL, H, L>
where
    AI: AccountInfo,
    AL: AccountListItem<(H, [L])>,
    H: Pod,
    L: Pod,
{
    fn validate(&mut self, program_id: &Pubkey, arg: PodOwner<'a>) -> CruiserResult<()> {
        self.info.validate(program_id, ())?;
        assert_is_owner(&self.info, arg.0, ())?;
        validate_discriminant::<AL, (H, [L])>(&mut &*self.info.data())?;
        Ok(())
    }
}
impl<AI, AL, H, L> ValidateArgument<PodFromZeroed> for PodListAccount<AI, AL, H, L>
where
    AI: AccountInfo,
    AL: AccountListItem<(H, [L])>,
    H: Pod,
    L: Pod,
{
    fn validate(&mut self, program_id: &Pubkey, _arg: PodFromZeroed) -> CruiserResult<()> {
        assert_is_owner(&self.info, program_id, ())?;
        let mut data = self.info.data_mut();
        assert_is_zeroed::<AL>(&*data, self.info.key(), false)?;

        AL::compressed_discriminant().serialize(&mut &mut *data)?;
        Ok(())
    }
}

/// Initializes a [`PodListAccount`] with a CPI to the system program.
#[derive(Debug, Clone)]
pub struct PodListInit<'a, AI, C> {
    /// The system program
    pub system_program: &'a SystemProgram<AI>,
    /// The funder for the new account
    pub funder: &'a AI,
    /// The seeds for the account if PDA
    pub account_seeds: Option<&'a PDASeedSet<'a>>,
    /// The seeds for the funder if PDA
    pub funder_seeds: Option<&'a PDASeedSet<'a>>,
    /// Number of items in the list
    pub item_count: usize,
    /// The rent object to use for rent calculation. If [`None`] then [`Rent::get`] is used.
    pub rent: Option<Rent>,
    /// The [`CPIMethod`] to use for the initialization.
    pub cpi: C,
}
impl<'a, AI, C> PodListInit<'a, AI, C> {
    /// Crates a new [`PodListInit`] with minimally required arguments
    #[must_use]
    pub fn new(
        system_program: &'a SystemProgram<AI>,
        funder: &'a AI,
        cpi: C,
        item_count: usize,
    ) -> Self {
        Self {
            system_program,
            funder,
            account_seeds: None,
            funder_seeds: None,
            item_count,
            rent: None,
            cpi,
        }
    }

    /// Sets the [`PodListInit::account_seeds`] field.
    #[must_use]
    pub fn account_seeds(mut self, account_seeds: &'a PDASeedSet<'a>) -> Self {
        self.account_seeds = Some(account_seeds);
        self
    }

    /// Sets the [`PodListInit::funder_seeds`] field.
    #[must_use]
    pub fn funder_seeds(mut self, funder_seeds: &'a PDASeedSet<'a>) -> Self {
        self.funder_seeds = Some(funder_seeds);
        self
    }

    /// Sets the [`PodListInit::rent`] field.
    #[must_use]
    pub fn rent(mut self, rent: Rent) -> Self {
        self.rent = Some(rent);
        self
    }
}
impl<'a, AI, AL, H, L, C> ValidateArgument<PodListInit<'a, AI, C>> for PodListAccount<AI, AL, H, L>
where
    AI: ToSolanaAccountInfo<'a>,
    AL: AccountListItem<(H, [L])>,
    H: Pod,
    L: Pod,
    C: CPIMethod,
{
    fn validate(&mut self, program_id: &Pubkey, arg: PodListInit<'a, AI, C>) -> CruiserResult<()> {
        let rent = match arg.rent {
            None => Rent::get()?,
            Some(rent) => rent,
        };
        let space = Self::header_offset() + arg.item_count * size_of::<L>();
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
impl<AI, AL, H, L, A> MultiIndexable<A> for PodListAccount<AI, AL, H, L>
where
    AI: AccountInfo + MultiIndexable<A>,
    AL: AccountListItem<(H, [L])>,
    H: Pod,
    L: Pod,
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
impl<AI, AL, H, L, A> SingleIndexable<A> for PodListAccount<AI, AL, H, L>
where
    AI: AccountInfo + SingleIndexable<A>,
    AL: AccountListItem<(H, [L])>,
    H: Pod,
    L: Pod,
{
    fn index_info(&self, indexer: A) -> CruiserResult<&Self::AccountInfo> {
        self.info.index_info(indexer)
    }
}
