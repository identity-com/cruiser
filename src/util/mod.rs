//! Helper utility functions

use borsh::BorshDeserialize;
use cruiser::account_argument::AccountInfoIterator;
use cruiser::instruction::Instruction;
use num_traits::One;
use solana_program::program::{set_return_data, MAX_RETURN_DATA};
use solana_program::pubkey::Pubkey;
use std::borrow::Cow;
use std::cmp::{max, min};
use std::num::NonZeroU64;
use std::ops::{Add, Bound, Deref, RangeBounds};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};

use crate::account_argument::{AccountArgument, FromAccounts, ValidateArgument};
use crate::instruction::{InstructionProcessor, ReturnValue};
use crate::{CruiserResult, GenericError};

pub use with_data::*;

pub mod assert;
pub(crate) mod bytes_ext;
pub mod short_iter;
pub mod short_vec;
mod with_data;

/// A version of [`Cow`](std::borrow::Cow) that only operates as a ref.
#[derive(Debug, Copy, Clone)]
pub enum MaybeOwned<'a, T> {
    /// Borrowed value
    Borrowed(&'a T),
    /// Owned value
    Owned(T),
}
impl<'a, T> From<T> for MaybeOwned<'a, T> {
    fn from(from: T) -> Self {
        Self::Owned(from)
    }
}
impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    fn from(from: &'a T) -> Self {
        Self::Borrowed(from)
    }
}
impl<'a, T> Deref for MaybeOwned<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            MaybeOwned::Borrowed(borrowed) => *borrowed,
            MaybeOwned::Owned(owned) => owned,
        }
    }
}
impl<'a, T> AsRef<T> for MaybeOwned<'a, T> {
    fn as_ref(&self) -> &T {
        &**self
    }
}
impl<'a, T> MaybeOwned<'a, T> {
    /// Turns this into an owned value if is owned
    pub fn into_owned(self) -> Option<T> {
        match self {
            MaybeOwned::Borrowed(_) => None,
            MaybeOwned::Owned(owned) => Some(owned),
        }
    }

    /// Turns this into an owned value if is owned or clones
    pub fn into_owned_clone(self) -> T
    where
        T: Clone,
    {
        match self {
            MaybeOwned::Borrowed(borrowed) => borrowed.clone(),
            MaybeOwned::Owned(owned) => owned,
        }
    }
}

/// The processing function used for [`InstructionProcessor`]
pub fn process_instruction<AI, I: Instruction<AI>, P: InstructionProcessor<AI, I>, Iter>(
    program_id: &Pubkey,
    accounts: &mut Iter,
    mut data: &[u8],
) -> CruiserResult
where
    I::Data: BorshDeserialize,
    I::Accounts: AccountArgument<AccountInfo = AI>
        + FromAccounts<P::FromAccountsData>
        + ValidateArgument<P::ValidateData>,
    Iter: AccountInfoIterator<Item = AI>,
{
    let data = <I::Data as BorshDeserialize>::deserialize(&mut data)?;
    let (from_data, validate_data, instruction_data) = P::data_to_instruction_arg(data)?;
    let mut accounts =
        <I::Accounts as FromAccounts<_>>::from_accounts(program_id, accounts, from_data)?;
    ValidateArgument::validate(&mut accounts, program_id, validate_data)?;
    let ret = P::process(program_id, instruction_data, &mut accounts)?;
    ret.return_self(set_return_data)?;
    <I::Accounts as AccountArgument>::write_back(accounts, program_id)?;
    Ok(())
}

extern "C" {
    fn sol_get_return_data(data: *mut u8, length: u64, program_id: *mut Pubkey) -> u64;
}
/// Gets return data from a cpi call. Returns the size of data returned, 0 means no return was found.
/// Copied from [`get_return_data`](solana_program::program::get_return_data).
pub fn get_return_data_buffered(
    buffer: &mut [u8; MAX_RETURN_DATA],
    program_id: &mut Pubkey,
) -> CruiserResult<usize> {
    // Copied from solana src
    let size = unsafe { sol_get_return_data(buffer.as_mut_ptr(), buffer.len() as u64, program_id) };

    Ok(min(size as usize, MAX_RETURN_DATA))
}

/// (start, end), inclusive
pub fn convert_range(
    range: &impl RangeBounds<usize>,
    length: usize,
) -> CruiserResult<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Included(val) => *val,
        Bound::Excluded(val) => val + 1,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(val) => *val,
        Bound::Excluded(val) => val - 1,
        Bound::Unbounded => length - 1,
    };
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    if end >= length {
        Err(GenericError::IndexOutOfRange {
            index: format!(
                "{},{}",
                match range.start_bound() {
                    Bound::Included(val) => Cow::Owned(format!("[{}", val)),
                    Bound::Excluded(val) => Cow::Owned(format!("({}", val)),
                    Bound::Unbounded => Cow::Borrowed("["),
                },
                match range.end_bound() {
                    Bound::Included(val) => Cow::Owned(format!("{}]", val)),
                    Bound::Excluded(val) => Cow::Owned(format!("{})", val)),
                    Bound::Unbounded => Cow::Borrowed("]"),
                }
            ),
            possible_range: format!("[0, {})", length),
        }
        .into())
    } else {
        Ok((start, end))
    }
}

/// Helper function to combine multiple size hints with a branch strategy, where the minimum lower bound and maximum upper bound are returned
pub fn combine_hints_branch(
    hints: impl IntoIterator<Item = (usize, Option<usize>)>,
) -> (usize, Option<usize>) {
    let mut hints = hints.into_iter();
    let (mut lower, mut upper) = match hints.next() {
        None => return (0, None),
        Some(hint) => hint,
    };
    for (hint_lower, hint_upper) in hints {
        lower = min(lower, hint_lower);
        upper = match (upper, hint_upper) {
            (Some(upper), Some(hint_upper)) => Some(max(upper, hint_upper)),
            _ => None,
        }
    }
    (lower, upper)
}

/// Helper function to combine multiple size hints with a chain strategy, where the sum of lower and upper bounds are returned
pub fn sum_size_hints(
    mut hints: impl Iterator<Item = (usize, Option<usize>)>,
) -> (usize, Option<usize>) {
    let mut sum = match hints.next() {
        None => return (0, None),
        Some(hint) => hint,
    };
    for hint in hints {
        sum = add_size_hint(sum, hint);
    }
    sum
}

/// Adds two size hints together. If either upper is [`None`] then the returned upper is [`None`].
#[must_use]
pub const fn add_size_hint(
    hint1: (usize, Option<usize>),
    hint2: (usize, Option<usize>),
) -> (usize, Option<usize>) {
    (
        hint1.0 + hint2.0,
        match (hint1.1, hint2.1) {
            (Some(upper1), Some(upper2)) => upper1.checked_add(upper2),
            _ => None,
        },
    )
}

/// Helper function to multiply a size hint by a number
#[must_use]
pub const fn mul_size_hint(hint: (usize, Option<usize>), mul: usize) -> (usize, Option<usize>) {
    (
        hint.0 * mul,
        match hint.1 {
            Some(upper) => upper.checked_mul(mul),
            None => None,
        },
    )
}

/// Length grabbing functions
pub trait Length {
    /// Gets the length
    fn len(&self) -> usize;
    /// Tells whether the length is 0
    #[default_method_body_is_const]
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T> const Length for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<'a, T> const Length for &'a [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}
impl<'a, T> const Length for &'a mut [T] {
    fn len(&self) -> usize {
        <[T]>::len(self)
    }
}
impl<T, const N: usize> const Length for [T; N] {
    fn len(&self) -> usize {
        N
    }
}
impl<'a, T, const N: usize> const Length for &'a [T; N] {
    fn len(&self) -> usize {
        N
    }
}
impl<'a, T, const N: usize> const Length for &'a mut [T; N] {
    fn len(&self) -> usize {
        N
    }
}
// TODO: impl this const when bpf toolchain updated
/// Advances a given slice while maintaining lifetimes
pub trait Advance<'a>: Length {
    /// The output of advancing
    type AdvanceOut;

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Panics if not enough data.
    // #[default_method_body_is_const]
    // #[allow(clippy::trait_duplication_in_bounds)]
    fn advance(&'a mut self, amount: usize) -> Self::AdvanceOut
// where
    //     Self: ~const Length,
    {
        assert!(amount <= self.len());
        // Safety: amount is not greater than the length of self
        unsafe { self.advance_unchecked(amount) }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Errors if not enough data.
    // #[default_method_body_is_const]
    // #[allow(clippy::trait_duplication_in_bounds)]
    fn try_advance(&'a mut self, amount: usize) -> CruiserResult<Self::AdvanceOut>
// where
    //     Self: ~const Length,
    {
        if self.len() < amount {
            Err(GenericError::NotEnoughData {
                needed: amount,
                remaining: self.len(),
            }
            .into())
        } else {
            // Safety: amount is not greater than the length of self
            Ok(unsafe { self.advance_unchecked(amount) })
        }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `amount` is not greater than the length of self.
    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut;
}
// TODO: impl this const when bpf toolchain updated
/// Advances a given slice giving back an array
pub trait AdvanceArray<'a, const N: usize>: Length {
    /// The output of advancing
    type AdvanceOut;

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Panics if not enough data.
    // #[default_method_body_is_const]
    // #[allow(clippy::trait_duplication_in_bounds)]
    fn advance_array(&'a mut self) -> Self::AdvanceOut
// where
    //     Self: ~const Length,
    {
        assert!(N <= self.len());
        // Safety: N is not greater than the length of self
        unsafe { self.advance_array_unchecked() }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Errors if not enough data.
    // #[default_method_body_is_const]
    // #[allow(clippy::trait_duplication_in_bounds)]
    fn try_advance_array(&'a mut self) -> CruiserResult<Self::AdvanceOut>
// where
    //     Self: ~const Length,
    {
        if self.len() < N {
            Err(GenericError::NotEnoughData {
                needed: N,
                remaining: self.len(),
            }
            .into())
        } else {
            // Safety: N is not greater than the length of self
            Ok(unsafe { self.advance_array_unchecked() })
        }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `N` is not greater than the length of self.
    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut;
}
impl<'a, 'b, T> Advance<'a> for &'b mut [T] {
    type AdvanceOut = &'b mut [T];

    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut {
        // Safety neither slice overlaps and points to valid r/w data
        let len = self.len();
        let ptr = self.as_mut_ptr();
        *self = &mut *slice_from_raw_parts_mut(ptr.add(amount), len - amount);
        &mut *slice_from_raw_parts_mut(ptr, amount)
    }
}
impl<'a, 'b, T, const N: usize> AdvanceArray<'a, N> for &'b mut [T] {
    type AdvanceOut = &'b mut [T; N];

    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut {
        // Safe conversion because returned array will always be same size as value passed in (`N`)
        &mut *(
            // Safety: Same requirements as this function
            self.advance_unchecked(N).as_mut_ptr().cast::<[T; N]>()
        )
    }
}
impl<'a, 'b, T> Advance<'a> for &'b [T] {
    type AdvanceOut = &'b [T];

    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut {
        // Safety neither slice overlaps and points to valid r/w data
        let len = self.len();
        let ptr = self.as_ptr();
        *self = &*slice_from_raw_parts(ptr.add(amount), len - amount);
        &*slice_from_raw_parts(ptr, amount)
    }
}
impl<'a, 'b, T, const N: usize> AdvanceArray<'a, N> for &'b [T] {
    type AdvanceOut = &'b [T; N];

    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut {
        // Safe conversion because returned array will always be same size as value passed in (`N`)
        &*(
            // Safety: Same requirements as this function
            self.advance_unchecked(N).as_ptr().cast::<[T; N]>()
        )
    }
}

/// Number can become non-zero, panicking if can't
pub trait ToNonZero {
    /// The non-zero type
    type NonZero;

    /// Converts to non-zero
    fn to_non_zero(self) -> Self::NonZero;
}
impl ToNonZero for u64 {
    type NonZero = NonZeroU64;

    fn to_non_zero(self) -> Self::NonZero {
        NonZeroU64::new(self).unwrap()
    }
}
impl ToNonZero for NonZeroU64 {
    type NonZero = NonZeroU64;

    fn to_non_zero(self) -> Self::NonZero {
        self
    }
}

/// Converts range bounds to a range of `[start, end)`
pub fn range_bounds_to_range<R, T>(range_bounds: R, minimum_lower: T, maximum_upper: T) -> (T, T)
where
    R: RangeBounds<T>,
    T: One + Add<Output = T> + Ord + Copy,
{
    (
        match range_bounds.start_bound() {
            Bound::Included(val) => *val,
            Bound::Excluded(val) => *val + T::one(),
            Bound::Unbounded => minimum_lower,
        }
        .max(minimum_lower),
        match range_bounds.end_bound() {
            Bound::Included(val) => *val + T::one(),
            Bound::Excluded(val) => *val,
            Bound::Unbounded => maximum_upper,
        }
        .min(maximum_upper),
    )
}

/// Can collect into this to void all values
#[derive(Debug, Clone, Copy)]
pub struct VoidCollect;
impl<T> FromIterator<T> for VoidCollect {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        iter.into_iter().for_each(|_| {});
        VoidCollect
    }
}
