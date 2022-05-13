//! Helper utility functions

use borsh::BorshDeserialize;
use cruiser::account_argument::AccountInfoIterator;
use cruiser::instruction::Instruction;
use num_traits::One;
use solana_program::program::{set_return_data, MAX_RETURN_DATA};
use solana_program::pubkey::Pubkey;
use std::any::Any;
use std::borrow::Cow;
use std::cell::{Ref, RefMut};
use std::cmp::{max, min};
use std::convert::Infallible;
use std::fmt::{Debug, Formatter};
use std::marker::PhantomPinned;
use std::mem::{size_of, transmute, ManuallyDrop, MaybeUninit};
use std::num::NonZeroU64;
use std::ops::{Add, Bound, Deref, DerefMut, RangeBounds};
use std::pin::Pin;
use std::ptr::{addr_of, addr_of_mut, slice_from_raw_parts, slice_from_raw_parts_mut};

use crate::account_argument::{AccountArgument, FromAccounts, ValidateArgument};
use crate::instruction::{InstructionProcessor, ReturnValue};
use crate::{CruiserResult, GenericError};

use crate::util::inner_ptr::InnerPtr;
pub use chain_exact_size::*;
pub use with_data::*;

pub mod assert;
pub(crate) mod bytes_ext;
mod chain_exact_size;
pub mod short_vec;
mod with_data;

/// Saturating assignment functions.
pub trait SaturatingAssign {
    /// Adds `rhs` to self saturating at bounds.
    fn saturating_add_assign(&mut self, rhs: Self);
    /// Subs `rhs` from self saturating at bounds.
    fn saturating_sub_assign(&mut self, rhs: Self);
    /// Multiplies self by `rhs` saturating at bounds.
    fn saturating_mul_assign(&mut self, rhs: Self);
    /// Divides self by `rhs` saturating at bounds.
    fn saturating_div_assign(&mut self, rhs: Self);
}
macro_rules! impl_assign {
    ($ty:ty) => {
        impl SaturatingAssign for $ty {
            fn saturating_add_assign(&mut self, rhs: Self) {
                *self = self.saturating_add(rhs);
            }

            fn saturating_sub_assign(&mut self, rhs: Self) {
                *self = self.saturating_sub(rhs);
            }

            fn saturating_mul_assign(&mut self, rhs: Self) {
                *self = self.saturating_mul(rhs);
            }

            fn saturating_div_assign(&mut self, rhs: Self) {
                *self = self.saturating_div(rhs);
            }
        }
    };
}
impl_assign!(u8);
impl_assign!(u16);
impl_assign!(u32);
impl_assign!(u64);
impl_assign!(u128);
impl_assign!(i8);
impl_assign!(i16);
impl_assign!(i32);
impl_assign!(i64);
impl_assign!(i128);

/// Transparent wrapper that only implements [`Deref`].
/// Intended to block [`DerefMut`] for a specific field.
#[repr(transparent)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ReadOnly<T>(T);
impl<T> ReadOnly<T> {
    /// Creates a new read-only wrapper.
    pub const fn new(val: T) -> Self {
        Self(val)
    }

    /// Gets the value out of the read-only wrapper.
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Gets a reference to the value in the read-only wrapper.
    pub const fn to_inner_ref(&self) -> &T {
        &self.0
    }
}
impl<T> const From<T> for ReadOnly<T> {
    fn from(val: T) -> Self {
        Self::new(val)
    }
}
impl<T> const Deref for ReadOnly<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> const AsRef<T> for ReadOnly<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

/// Gets the maximum value from an array of [`usize`]s
#[must_use]
pub const fn usize_array_max<const N: usize>(vals: [usize; N]) -> usize {
    let mut max = usize::MIN;
    let mut index = 0;
    while index < N {
        if vals[index] > max {
            max = vals[index];
        }
        index += 1;
    }
    max
}

/// Mapping function for a [`Deref`].
/// Default implemented for all `T` that implement [`Deref`].
pub trait MappableRef: Deref {
    /// The output of the mapping.
    type Output<'a, R: ?Sized>: Deref<Target = R>
    where
        R: 'a,
        Self: 'a;

    /// Maps the reference.
    fn map_ref<'a, R: ?Sized>(self, f: impl FnOnce(&Self::Target) -> &R) -> Self::Output<'a, R>
    where
        Self: 'a;
}

/// Mapping function for a [`Deref`].
/// Default implemented for all `T` that implement [`Deref`].
/// Mapping function can error.
pub trait TryMappableRef: Deref {
    /// The output of the mapping.
    type Output<'a, R: ?Sized>: Deref<Target = R>
    where
        R: 'a,
        Self: 'a;

    /// Tries to map the reference.
    fn try_map_ref<'a, R, E>(
        self,
        f: impl FnOnce(&Self::Target) -> Result<&R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a;
}

/// Mapping function for a [`DerefMut`].
/// Default implemented for all `T` that implement [`DerefMut`].
pub trait MappableRefMut: DerefMut {
    /// The output of the mapping.
    type Output<'a, R: ?Sized>: DerefMut<Target = R>
    where
        R: 'a,
        Self: 'a;

    /// Maps the mutable reference.
    fn map_ref_mut<'a, R>(self, f: impl FnOnce(&mut Self::Target) -> &mut R) -> Self::Output<'a, R>
    where
        Self: 'a;
}

/// Mapping function for a [`DerefMut`].
/// Default implemented for all `T` that implement [`DerefMut`].
/// Mapping function can error.
pub trait TryMappableRefMut: DerefMut {
    /// The output of the mapping.
    type Output<'a, R: ?Sized>: DerefMut<Target = R>
    where
        R: 'a,
        Self: 'a;

    /// Tries to map the mutable reference.
    fn try_map_ref_mut<'a, R, E>(
        self,
        f: impl FnOnce(&mut Self::Target) -> Result<&mut R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a;
}

impl<T: ?Sized> MappableRef for &'_ T {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = &'a R;

    fn map_ref<'a, R: ?Sized>(self, f: impl FnOnce(&Self::Target) -> &R) -> Self::Output<'a, R>
    where
        Self: 'a,
    {
        f(self)
    }
}

impl<T: ?Sized> TryMappableRef for &'_ T {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = &'a R;

    fn try_map_ref<'a, R, E>(
        self,
        f: impl FnOnce(&Self::Target) -> Result<&R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a,
    {
        f(self)
    }
}

impl<T: ?Sized> MappableRef for &'_ mut T {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = &'a R;

    fn map_ref<'a, R: ?Sized>(self, f: impl FnOnce(&Self::Target) -> &R) -> Self::Output<'a, R>
    where
        Self: 'a,
    {
        f(&*self)
    }
}

impl<T: ?Sized> TryMappableRef for &'_ mut T {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = &'a R;

    fn try_map_ref<'a, R, E>(
        self,
        f: impl FnOnce(&Self::Target) -> Result<&R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a,
    {
        f(self)
    }
}

impl<T: ?Sized> MappableRefMut for &'_ mut T {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = &'a mut R;

    fn map_ref_mut<'a, R>(self, f: impl FnOnce(&mut Self::Target) -> &mut R) -> Self::Output<'a, R>
    where
        Self: 'a,
    {
        f(self)
    }
}

impl<T: ?Sized> TryMappableRefMut for &'_ mut T {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = &'a mut R;

    fn try_map_ref_mut<'a, R, E>(
        self,
        f: impl FnOnce(&mut Self::Target) -> Result<&mut R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a,
    {
        f(self)
    }
}

impl<T: ?Sized> MappableRef for Ref<'_, T> {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = Ref<'a, R>;

    fn map_ref<'a, R: ?Sized>(self, f: impl FnOnce(&Self::Target) -> &R) -> Self::Output<'a, R>
    where
        Self: 'a,
    {
        Ref::map(self, f)
    }
}

impl<T: ?Sized> TryMappableRef for Ref<'_, T> {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = RefMap<Self, &'a R>;

    fn try_map_ref<'a, R, E>(
        self,
        f: impl FnOnce(&Self::Target) -> Result<&R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a,
    {
        RefMap::try_new(self, f)
    }
}

impl<T: ?Sized> MappableRef for RefMut<'_, T> {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = RefMap<Self, &'a R>;

    fn map_ref<'a, R: ?Sized>(
        self,
        f: impl for<'b> FnOnce(&Self::Target) -> &R,
    ) -> Self::Output<'a, R>
    where
        Self: 'a,
    {
        RefMap::new(self, f)
    }
}

impl<T: ?Sized> TryMappableRef for RefMut<'_, T> {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = RefMap<Self, &'a R>;

    fn try_map_ref<'a, R, E>(
        self,
        f: impl FnOnce(&Self::Target) -> Result<&R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a,
    {
        RefMap::try_new(self, f)
    }
}

impl<T: ?Sized> MappableRefMut for RefMut<'_, T> {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = RefMut<'a, R>;

    fn map_ref_mut<'a, R>(self, f: impl FnOnce(&mut Self::Target) -> &mut R) -> Self::Output<'a, R>
    where
        Self: 'a,
    {
        RefMut::map(self, f)
    }
}

impl<T: ?Sized> TryMappableRefMut for RefMut<'_, T> {
    type Output<'a, R: ?Sized>
    where
        R: 'a,
        Self: 'a,
    = RefMap<Self, &'a mut R>;

    fn try_map_ref_mut<'a, R, E>(
        self,
        f: impl FnOnce(&mut Self::Target) -> Result<&mut R, E>,
    ) -> Result<Self::Output<'a, R>, E>
    where
        Self: 'a,
    {
        RefMap::try_new_mut(self, f)
    }
}

#[derive(Debug)]
struct RefMapInner<A, R> {
    data_in: A,
    data_out: InnerPtr<R>,
    _pinned: PhantomPinned,
}
mod inner_ptr {
    use super::*;

    pub union InnerPtr<R> {
        data: ManuallyDrop<R>,
        _fat_ptr: &'static dyn Any,
    }

    impl<R> InnerPtr<R> {
        pub fn new(data: R) -> Self {
            Self {
                data: ManuallyDrop::new(data),
            }
        }
    }

    impl<R> Debug for InnerPtr<R>
    where
        R: Debug,
    {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            unsafe {
                f.debug_struct("InnerPtr")
                    .field("data", &self.data)
                    .finish()
            }
        }
    }

    impl<R> Drop for InnerPtr<R> {
        fn drop(&mut self) {
            unsafe {
                ManuallyDrop::drop(&mut self.data);
            }
        }
    }

    impl<R> Deref for InnerPtr<R> {
        type Target = R;

        fn deref(&self) -> &Self::Target {
            unsafe { &self.data }
        }
    }

    impl<R> DerefMut for InnerPtr<R> {
        fn deref_mut(&mut self) -> &mut Self::Target {
            unsafe { &mut self.data }
        }
    }
}

impl<'a, A, R: ?Sized> RefMap<A, &'a R>
where
    A: Deref + 'a,
    A::Target: 'a,
{
    /// Create a new `RefMap` from data and a mapping function.
    pub fn new(data_in: A, func: impl FnOnce(&'a A::Target) -> &'a R) -> Self {
        #[allow(unused_qualifications)]
        Self::try_new(data_in, |input| Result::<_, Infallible>::Ok(func(input))).unwrap()
    }

    /// Tries to create a new `RefMap` from data and a mapping function.
    pub fn try_new<E>(
        data_in: A,
        func: impl FnOnce(&'a A::Target) -> Result<&'a R, E>,
    ) -> Result<Self, E> {
        let mut out = Box::pin(RefMapInner {
            data_in,
            data_out: InnerPtr::new(MaybeUninit::uninit()),
            _pinned: PhantomPinned,
        });
        unsafe {
            let out = out.as_mut().get_unchecked_mut();
            *out.data_out = MaybeUninit::new(func(&*addr_of!(*out.data_in))?);
        }
        unsafe {
            Ok(RefMap(transmute::<
                Pin<Box<RefMapInner<A, MaybeUninit<&'a R>>>>,
                Pin<Box<RefMapInner<A, &'a R>>>,
            >(out)))
        }
    }
}

impl<'a, A, R: ?Sized> RefMap<A, &'a mut R>
where
    A: DerefMut + 'a,
    A::Target: 'a,
{
    /// Create a new `RefMap` from mutable data and a mapping function.
    pub fn new_mut(data_in: A, func: impl FnOnce(&'a mut A::Target) -> &'a mut R) -> Self {
        #[allow(unused_qualifications)]
        Self::try_new_mut(data_in, |input| Result::<_, Infallible>::Ok(func(input))).unwrap()
    }

    /// Tries to create a new `RefMap` from mutable data and a mapping function.
    pub fn try_new_mut<E>(
        data_in: A,
        func: impl FnOnce(&'a mut A::Target) -> Result<&'a mut R, E>,
    ) -> Result<Self, E> {
        let mut out = Box::pin(RefMapInner {
            data_in,
            data_out: inner_ptr::InnerPtr::new(MaybeUninit::uninit()),
            _pinned: PhantomPinned,
        });
        unsafe {
            let out = out.as_mut().get_unchecked_mut();
            *out.data_out = MaybeUninit::new(func(&mut *addr_of_mut!(*out.data_in))?);
        }
        unsafe {
            Ok(RefMap(transmute::<
                Pin<Box<RefMapInner<A, MaybeUninit<&'a mut R>>>>,
                Pin<Box<RefMapInner<A, &'a mut R>>>,
            >(out)))
        }
    }
}

/// Maps a given deref using a function
#[derive(Debug)]
pub struct RefMap<A, R>(Pin<Box<RefMapInner<A, R>>>);

impl<A, R> Deref for RefMap<A, R>
where
    R: Deref,
{
    type Target = R::Target;
    fn deref(&self) -> &Self::Target {
        &*self.0.data_out
    }
}

impl<A, R> DerefMut for RefMap<A, R>
where
    R: DerefMut,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        unsafe { &mut *(self.0.as_mut().get_unchecked_mut()).data_out }
    }
}

impl<A, R1: ?Sized> MappableRef for RefMap<A, &'_ R1> {
    type Output<'a, R2: ?Sized>
    where
        R2: 'a,
        Self: 'a,
    = RefMap<A, &'a R2>;

    fn map_ref<'a, R2: ?Sized>(
        mut self,
        f: impl FnOnce(&Self::Target) -> &R2,
    ) -> Self::Output<'a, R2>
    where
        Self: 'a,
    {
        unsafe {
            let old = self.0.as_mut().get_unchecked_mut();
            assert!(size_of::<InnerPtr<&R2>>() <= size_of::<InnerPtr<&R1>>());
            let new = f(&old.data_out);
            let old = &mut *((old as *mut RefMapInner<A, &R1>).cast::<RefMapInner<A, &R2>>());
            *old.data_out = new;
            transmute::<RefMap<A, &'a R1>, RefMap<A, &'a R2>>(self)
        }
    }
}

/// Chunks an array into equal chunks.
#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
pub fn chunk_array<T, const N: usize, const LEN: usize>(array: &[T; N]) -> [&[T; LEN]; N / LEN]
where
    [(); N / LEN]:,
    [(); 0 - (N % LEN)]:,
{
    let ptr = array.as_ptr();
    let mut out = MaybeUninit::uninit_array();
    unsafe {
        for (index, item) in out.iter_mut().enumerate() {
            *item = MaybeUninit::new(&*ptr.add(index * LEN).cast::<[T; LEN]>());
        }
        MaybeUninit::array_assume_init(out)
    }
}

/// Chunks an array into equal mutable chunks.
#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
pub fn chunck_array_mut<T, const N: usize, const LEN: usize>(
    array: &mut [T; N],
) -> [&mut [T; LEN]; N / LEN]
where
    [(); N / LEN]:,
    [(); 0 - (N % LEN)]:,
{
    let ptr = array.as_mut_ptr();
    let mut out = MaybeUninit::uninit_array();
    unsafe {
        for (index, item) in out.iter_mut().enumerate() {
            *item = MaybeUninit::new(&mut *ptr.add(index * LEN).cast::<[T; LEN]>());
        }
        MaybeUninit::array_assume_init(out)
    }
}

/// Asserts that data is at least need in length
pub fn assert_data_len(data_len: usize, need: usize) -> CruiserResult {
    if data_len < need {
        Err(GenericError::NotEnoughData {
            needed: need,
            remaining: data_len,
        }
        .into())
    } else {
        Ok(())
    }
}

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
    if I::ReturnType::max_size() > 0 {
        ret.return_self(set_return_data)?;
    }
    <I::Accounts as AccountArgument>::write_back(accounts, program_id)?;
    Ok(())
}

extern "C" {
    fn sol_get_return_data(data: *mut u8, length: u64, program_id: *mut Pubkey) -> u64;
}

/// Gets return data from a cpi call. Returns the size of data returned, 0 means no return was found.
/// Copied from [`get_return_data`](solana_program::program::get_return_data).
pub fn get_return_data_buffered(
    buffer: &mut [u8],
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
