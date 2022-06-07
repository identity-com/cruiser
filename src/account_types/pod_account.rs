//! An account that allows the usage of any [`Pod`] type.

use crate::account_argument::AccountInfoIterator;
use crate::compressed_numbers::CompressedNumber;
use crate::prelude::{AccountArgument, AccountListItem};
use crate::util::{Advance, TryMappableRef, TryMappableRefMut};
use crate::{AccountInfo, CruiserResult, GenericError};
use borsh::BorshDeserialize;
use bytemuck::Pod;
use cruiser::prelude::FromAccounts;
use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;
use std::mem::{align_of, size_of};
use std::ops::{Deref, DerefMut};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

/// The type of data a [`PodAccount`] allows access to.
#[derive(Debug)]
#[repr(C)]
pub struct PodData<D> {
    data: D,
    remaining: [u8],
}

/// An account that allows the usage of any [`Pod`] type.
#[derive(Debug)]
pub struct PodAccount<AI, AL, D> {
    info: AI,
    phantom_d: PhantomData<fn() -> D>,
    phantom_al: PhantomData<fn() -> AL>,
}
impl<AI, AL, D> PodAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: Pod,
{
    /// Gets the offset to the start of the data.
    #[must_use]
    pub fn data_offset() -> usize {
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
impl<AI, AL, D> AccountArgument for PodAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
{
    type AccountInfo = AI;

    #[inline]
    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        self.info.write_back(program_id)
    }

    #[inline]
    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.info.add_keys(add)
    }
}
impl<AI, AL, D> FromAccounts<()> for PodAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: Pod,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: (),
    ) -> CruiserResult<Self> {
        let info = AI::from_accounts(program_id, infos, arg)?;
        let data = info.data();
        let mut buffer: &[u8] = &*data;
        let discriminant = AL::DiscriminantCompressed::deserialize(&mut buffer)?;
        if discriminant != AL::compressed_discriminant() {
            return Err(GenericError::MismatchedDiscriminant {
                account: *info.key(),
                received: discriminant.into_number().get(),
                expected: AL::compressed_discriminant().into_number(),
            }
            .into());
        }
        drop(data);

        Ok(Self {
            info,
            phantom_d: PhantomData,
            phantom_al: PhantomData,
        })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        AI::accounts_usage_hint(arg)
    }
}
