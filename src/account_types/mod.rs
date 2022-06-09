//! Standard account types. These are all optional, you can build your own if you don't like something in one of them.

pub mod close_account;
pub mod cruiser_program_account;
pub mod data_account;
pub mod discriminant_account;
pub mod init_account;
pub mod init_or_zeroed_account;
pub mod pod_account;
pub mod pod_list;
pub mod read_only_data_account;
pub mod rent_exempt;
pub mod rest;
pub mod seeds;
pub mod sys_var;
pub mod system_program;
pub mod zeroed_account;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, ValidateArgument,
};
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;
use std::marker::PhantomData;

/// The equivalent of [`PhantomData`] for deriving [`AccountArgument`].
#[derive(Debug, Copy, Clone)]
pub struct PhantomAccount<AI, T> {
    phantom_ai: PhantomData<fn() -> AI>,
    phantom_t: PhantomData<fn() -> T>,
}
impl<AI, T> Default for PhantomAccount<AI, T> {
    fn default() -> Self {
        Self {
            phantom_ai: PhantomData,
            phantom_t: PhantomData,
        }
    }
}
impl<AI, T> AccountArgument for PhantomAccount<AI, T> {
    type AccountInfo = AI;

    #[inline]
    fn write_back(self, _program_id: &Pubkey) -> CruiserResult<()> {
        Ok(())
    }

    #[inline]
    fn add_keys(&self, _add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        Ok(())
    }
}
impl<AI, T> FromAccounts for PhantomAccount<AI, T> {
    #[inline]
    fn from_accounts(
        _program_id: &Pubkey,
        _infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        _arg: (),
    ) -> CruiserResult<Self> {
        Ok(Self::default())
    }

    #[inline]
    fn accounts_usage_hint(_arg: &()) -> (usize, Option<usize>) {
        (0, Some(0))
    }
}
impl<AI, T> ValidateArgument for PhantomAccount<AI, T> {
    #[inline]
    fn validate(&mut self, _program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}
