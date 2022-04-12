//! Types for manipulating account dat in-place (aka zero-copy)

use crate::on_chain_size::OnChainStaticSize;
use crate::util::{Advance, AdvanceArray, WithData, WithDataIter};
use crate::{CruiserResult, GenericError};
use solana_program::pubkey::Pubkey;
use std::array::IntoIter;
use std::collections::Bound;
use std::iter::Take;
use std::marker::PhantomData;
use std::ops::{Range, RangeBounds};

/// In-place account data access
pub trait InPlace<'a> {
    /// The type accessed
    type Access;
    /// The type of the argument used in [`InPlace::create_with_arg`]
    type CreateArg;
    /// The type of the argument used in [`InPlace::read_with_arg`]
    type ReadArg;

    /// Create a new instance of `Self::Access` with the given argument
    fn create_with_arg(
        data: &mut &'a mut [u8],
        arg: Self::CreateArg,
    ) -> CruiserResult<Self::Access>;
    /// Reads the access type from data and an arg
    fn read_with_arg(data: &mut &'a mut [u8], arg: Self::ReadArg) -> CruiserResult<Self::Access>;
}
/// In-place account data create access with no arg, auto derived
pub trait InPlaceUnitCreate<'a>: InPlace<'a, CreateArg = ()> {
    /// Create a new instance of `Self::Access` with no argument
    fn create(data: &mut &'a mut [u8]) -> CruiserResult<Self::Access> {
        Self::create_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitCreate<'a> for T where T: InPlace<'a, CreateArg = ()> {}
/// In-place account data read access with no arg, auto derived
pub trait InPlaceUnitRead<'a>: InPlace<'a, ReadArg = ()> {
    /// Reads the access type from data
    fn read(data: &mut &'a mut [u8]) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitRead<'a> for T where T: InPlace<'a, ReadArg = ()> {}

/// Gets a value from the access
pub trait InPlaceGet<'a>: Sized + InPlace<'a> {
    /// Gets the access value
    fn get_with_access(access: &Self::Access) -> CruiserResult<Self>;
}
impl<'a, T> InPlaceGet<'a> for T
where
    T: InPlace<'a>,
    T::Access: InPlaceAccessGet<Self>,
{
    fn get_with_access(access: &Self::Access) -> CruiserResult<Self> {
        access.get()
    }
}
/// Sets a value to an access
pub trait InPlaceSet<'a>: Sized + InPlace<'a> {
    /// Sets a value to an access
    fn set_with_access(access: &mut Self::Access, val: Self) -> CruiserResult;
}
impl<'a, T> InPlaceSet<'a> for T
where
    T: InPlace<'a>,
    T::Access: InPlaceAccessSet<Self>,
{
    fn set_with_access(access: &mut Self::Access, val: Self) -> CruiserResult {
        access.set(val)
    }
}

/// An access that can get a value out
pub trait InPlaceAccessGet<V> {
    /// Gets a value out
    fn get(&self) -> CruiserResult<V>;
}
/// An access that can be set to a value
pub trait InPlaceAccessSet<V> {
    /// Sets this to a value
    fn set(&mut self, val: V) -> CruiserResult;
}

impl<'a> InPlace<'a> for u8 {
    type Access = &'a mut u8;
    type CreateArg = ();
    type ReadArg = ();

    fn create_with_arg(
        data: &mut &'a mut [u8],
        arg: Self::CreateArg,
    ) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, arg)
    }

    fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let out: &mut [_; 1] = data.try_advance_array()?;
        Ok(&mut out[0])
    }
}
impl<'a> InPlaceAccessGet<u8> for &'a mut u8 {
    fn get(&self) -> CruiserResult<u8> {
        Ok(**self)
    }
}
impl<'a> InPlaceAccessSet<u8> for &'a mut u8 {
    fn set(&mut self, val: u8) -> CruiserResult {
        **self = val;
        Ok(())
    }
}
impl<'a> InPlace<'a> for i8 {
    type Access = &'a mut i8;
    type CreateArg = ();
    type ReadArg = ();

    fn create_with_arg(
        data: &mut &'a mut [u8],
        arg: Self::CreateArg,
    ) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, arg)
    }

    fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let out: &mut [_; 1] = data.try_advance_array()?;
        Ok(unsafe { &mut *(out.as_mut_ptr().cast::<i8>()) })
    }
}
impl<'a> InPlaceAccessGet<i8> for &'a mut i8 {
    fn get(&self) -> CruiserResult<i8> {
        Ok(**self)
    }
}
impl<'a> InPlaceAccessSet<i8> for &'a mut i8 {
    fn set(&mut self, val: i8) -> CruiserResult {
        **self = val;
        Ok(())
    }
}

/// An inplace version of primitive numbers to adhere to alignment
#[derive(Debug)]
pub struct PrimNumInPlace<'a, T, const N: usize>(&'a mut [u8; N], PhantomData<T>);
impl<'a, T, const N: usize> InPlaceAccessGet<T> for PrimNumInPlace<'a, T, N>
where
    T: FromNE<N>,
{
    fn get(&self) -> CruiserResult<T> {
        Ok(T::from_ne_bytes(*self.0))
    }
}
impl<'a, T, const N: usize> InPlaceAccessSet<T> for PrimNumInPlace<'a, T, N>
where
    T: FromNE<N>,
{
    fn set(&mut self, val: T) -> CruiserResult {
        *self.0 = val.into_ne_bytes();
        Ok(())
    }
}

/// A number that can be derived from native-endian bytes
pub trait FromNE<const N: usize>: Sized {
    /// Creates this from native endian-bytes
    #[must_use]
    fn from_ne_bytes(bytes: [u8; N]) -> Self;
    /// Turns this into native-endian bytes
    #[must_use]
    fn into_ne_bytes(self) -> [u8; N];
}
/// This can be turned into a `usize` on solana (64-bit `usize`)
pub trait ToSolanaUsize {
    /// Turns this into a solana `usize` (64-bit)
    fn to_solana_usize(self) -> usize;

    /// Turns a usize into a `Self`
    fn from_solana_usize(val: usize) -> Self;
}
/// Value is initialized to 0
pub trait InitToZero {}
macro_rules! impl_from_ne {
    ($ty:ty, $size:expr) => {
        impl FromNE<$size> for $ty {
            fn from_ne_bytes(bytes: [u8; $size]) -> Self {
                Self::from_ne_bytes(bytes)
            }

            fn into_ne_bytes(self) -> [u8; $size] {
                self.to_ne_bytes()
            }
        }
        impl<'a> InPlace<'a> for $ty {
            type Access = PrimNumInPlace<'a, $ty, $size>;
            type CreateArg = ();
            type ReadArg = ();

            fn create_with_arg(
                data: &mut &'a mut [u8],
                _arg: Self::CreateArg,
            ) -> CruiserResult<Self::Access> {
                Self::read_with_arg(data, ())
            }

            fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
                Ok(PrimNumInPlace(data.try_advance_array()?, PhantomData))
            }
        }
        impl ToSolanaUsize for $ty {
            fn to_solana_usize(self) -> usize {
                self as usize
            }

            fn from_solana_usize(val: usize) -> Self {
                val as $ty
            }
        }
        impl InitToZero for $ty {}
    };
}
impl_from_ne!(u16, 2);
impl_from_ne!(u32, 4);
impl_from_ne!(u64, 8);
impl_from_ne!(u128, 16);
impl_from_ne!(i16, 2);
impl_from_ne!(i32, 4);
impl_from_ne!(i64, 8);
impl_from_ne!(i128, 16);

impl<'a> InPlace<'a> for Pubkey {
    type Access = &'a mut Pubkey;
    type CreateArg = ();
    type ReadArg = ();

    fn create_with_arg(
        data: &mut &'a mut [u8],
        arg: Self::CreateArg,
    ) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, arg)
    }

    fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let data: &mut [u8; 32] = data.try_advance_array()?;
        Ok(unsafe { &mut *data.as_mut_ptr().cast::<Pubkey>() })
    }
}
impl<'a> InPlaceAccessGet<Pubkey> for &'a mut Pubkey {
    fn get(&self) -> CruiserResult<Pubkey> {
        Ok(**self)
    }
}
impl<'a> InPlaceAccessSet<Pubkey> for &'a mut Pubkey {
    fn set(&mut self, val: Pubkey) -> CruiserResult {
        **self = val;
        Ok(())
    }
}

/// In-place access to arrays
#[derive(Debug)]
pub struct InPlaceArray<'a, T, const N: usize> {
    element_length: usize,
    data: &'a mut [u8],
    phantom_t: PhantomData<fn() -> T>,
}
impl<'a, T, const N: usize> InPlaceArray<'a, T, N> {
    /// Gets an item in the array with a read arg
    pub fn get_with_arg<'b, A>(
        &'b mut self,
        index: usize,
        arg: A,
    ) -> CruiserResult<Option<T::Access>>
    where
        T: InPlace<'b, ReadArg = A>,
    {
        if index < N {
            Ok(Some(T::read_with_arg(
                &mut &mut self.data[self.element_length * index..],
                arg,
            )?))
        } else {
            Ok(None)
        }
    }

    /// Gets an item in the array
    pub fn get<'b>(&'b mut self, index: usize) -> CruiserResult<Option<T::Access>>
    where
        T: InPlaceUnitRead<'b>,
    {
        self.get_with_arg(index, ())
    }

    /// Gets an iterator over the array in the range cloning the arg
    #[allow(clippy::type_complexity)]
    pub fn range_with_clone_arg<'b, A>(
        &'b mut self,
        range: impl RangeBounds<usize>,
        arg: A,
    ) -> CruiserResult<
        WithDataIter<
            Range<usize>,
            (&'b mut [u8], A),
            fn(usize, &mut (&'b mut [u8], A)) -> CruiserResult<T::Access>,
        >,
    >
    where
        T: InPlace<'b, ReadArg = A>,
        A: Clone,
    {
        let start = match range.start_bound() {
            Bound::Included(bound) => *bound,
            Bound::Excluded(bound) => *bound + 1,
            Bound::Unbounded => 0,
        }
        .min(N);
        let end = match range.end_bound() {
            Bound::Included(bound) => *bound + 1,
            Bound::Excluded(bound) => *bound,
            Bound::Unbounded => N,
        }
        .max(start)
        .min(N);
        Ok((start..end).map_with_data(
            (self.data.split_at_mut(start * self.element_length).1, arg),
            |_, (data, arg)| T::read_with_arg(data, arg.clone()),
        ))
    }
    /// Gets an iterator over the array in the range
    #[allow(clippy::type_complexity)]
    pub fn range<'b>(
        &'b mut self,
        range: impl RangeBounds<usize>,
    ) -> CruiserResult<
        WithDataIter<
            Range<usize>,
            (&'b mut [u8], ()),
            fn(usize, &mut (&'b mut [u8], ())) -> CruiserResult<T::Access>,
        >,
    >
    where
        T: InPlace<'b, ReadArg = ()>,
    {
        self.range_with_clone_arg(range, ())
    }

    /// Gets an iterator over the array cloning the arg
    #[allow(clippy::type_complexity)]
    pub fn all_with_clone_arg<'b, A>(
        &'b mut self,
        arg: A,
    ) -> CruiserResult<
        WithDataIter<
            Range<usize>,
            (&'b mut [u8], A),
            fn(usize, &mut (&'b mut [u8], A)) -> CruiserResult<T::Access>,
        >,
    >
    where
        T: InPlace<'b, ReadArg = A>,
        A: Clone,
    {
        self.range_with_clone_arg(.., arg)
    }

    /// Gets an iterator over all the elements with an argument array
    #[allow(clippy::type_complexity)]
    pub fn all_with_args<'b, A>(
        &'b mut self,
        args: [A; N],
    ) -> CruiserResult<
        WithDataIter<
            IntoIter<A, N>,
            &'b mut [u8],
            fn(A, &mut &'b mut [u8]) -> CruiserResult<T::Access>,
        >,
    >
    where
        T: InPlace<'b, ReadArg = A>,
    {
        Ok(args
            .into_iter()
            .map_with_data(&mut *self.data, |arg, data| T::read_with_arg(data, arg)))
    }

    /// Gets an iterator over all the elements
    #[allow(clippy::type_complexity)]
    pub fn all<'b>(
        &'b mut self,
    ) -> CruiserResult<
        WithDataIter<
            Range<usize>,
            (&'b mut [u8], ()),
            fn(usize, &mut (&'b mut [u8], ())) -> CruiserResult<T::Access>,
        >,
    >
    where
        T: InPlace<'b, ReadArg = ()>,
    {
        self.all_with_clone_arg(())
    }
}
impl<'a, T, const N: usize> InPlace<'a> for [T; N]
where
    T: OnChainStaticSize,
{
    type Access = InPlaceArray<'a, T, N>;
    type CreateArg = ();
    type ReadArg = ();

    fn create_with_arg(
        data: &mut &'a mut [u8],
        arg: Self::CreateArg,
    ) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, arg)
    }

    fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let element_length = T::on_chain_static_size();
        Ok(InPlaceArray {
            element_length,
            data: data.try_advance(element_length * N)?,
            phantom_t: PhantomData,
        })
    }
}

/// A vector with a static max size
#[derive(Debug)]
pub struct StaticSizeVec<T, L, const N: usize>(Vec<T>, PhantomData<fn() -> (T, L)>);
/// The access for [`StaticSizeVec`]
#[derive(Debug)]
pub struct StaticSizeVecAccess<'a, T, L, const N: usize>
where
    L: InPlace<'a, ReadArg = ()>,
{
    length: L::Access,
    data: &'a mut [u8],
    element_length: usize,
    phantom_t: PhantomData<fn() -> T>,
}
impl<'a, T, L, const N: usize> StaticSizeVecAccess<'a, T, L, N>
where
    L: InPlace<'a, ReadArg = ()>,
{
    /// Adds an element to the end of the vector
    pub fn push<'b, A>(&'b mut self, arg: A) -> CruiserResult<T::Access>
    where
        T: InPlace<'b, CreateArg = A>,
        L: ToSolanaUsize,
        L::Access: InPlaceAccessGet<L> + InPlaceAccessSet<L>,
        A: Clone,
    {
        let length = self.length.get()?.to_solana_usize();
        if length >= N {
            return Err(GenericError::Custom {
                error: "Cannot push to vec".to_string(),
            }
            .into());
        }
        let mut data = self.data.split_at_mut(self.element_length * length).1;
        let access = T::create_with_arg(&mut data, arg)?;
        self.length.set(L::from_solana_usize(length + 1))?;
        Ok(access)
    }

    /// Gets a specific element with a given argument
    pub fn get_with_arg<'b, A>(
        &'b mut self,
        index: usize,
        read_arg: A,
    ) -> CruiserResult<Option<T::Access>>
    where
        L: InPlaceGet<'a, ReadArg = ()> + ToSolanaUsize,
        T: InPlace<'b, ReadArg = A>,
    {
        if index < L::get_with_access(&self.length)?.to_solana_usize() {
            Ok(Some(T::read_with_arg(
                &mut &mut self.data[index * self.element_length..],
                read_arg,
            )?))
        } else {
            Ok(None)
        }
    }

    /// Gets a specific element
    pub fn get<'b>(&'b mut self, index: usize) -> CruiserResult<Option<T::Access>>
    where
        L: InPlaceGet<'a, ReadArg = ()> + ToSolanaUsize,
        T: InPlace<'b, ReadArg = ()>,
    {
        self.get_with_arg(index, ())
    }

    /// Gets all elements with a cloned argument
    #[allow(clippy::type_complexity)]
    pub fn all_with_clone_arg<'b, A>(
        &'b mut self,
        arg: A,
    ) -> CruiserResult<
        WithDataIter<
            Range<usize>,
            (&'b mut [u8], A),
            fn(usize, &mut (&'b mut [u8], A)) -> CruiserResult<T::Access>,
        >,
    >
    where
        L: InPlaceGet<'a, ReadArg = ()> + ToSolanaUsize,
        T: InPlace<'b, ReadArg = A>,
        A: Clone,
    {
        Ok((0..L::get_with_access(&self.length)?.to_solana_usize())
            .map_with_data((&mut *self.data, arg), |_, (data, arg)| {
                T::read_with_arg(data, arg.clone())
            }))
    }

    /// Gets all elements with an argument iterator, will shorten to length of args iter if too short.
    #[allow(clippy::type_complexity)]
    pub fn all_with_args<'b, A, I>(
        &'b mut self,
        args: I,
    ) -> CruiserResult<
        WithDataIter<
            Take<I::IntoIter>,
            &'b mut [u8],
            fn(A, &mut &'b mut [u8]) -> CruiserResult<T::Access>,
        >,
    >
    where
        L: InPlaceGet<'a, ReadArg = ()> + ToSolanaUsize,
        T: InPlace<'b, ReadArg = A>,
        I: IntoIterator<Item = A>,
        A: Clone,
    {
        Ok(args
            .into_iter()
            .take(L::get_with_access(&self.length)?.to_solana_usize())
            .map_with_data(&mut *self.data, |arg, data| T::read_with_arg(data, arg)))
    }

    /// Gets an iterator over all the elements
    #[allow(clippy::type_complexity)]
    pub fn all<'b>(
        &'b mut self,
    ) -> CruiserResult<
        WithDataIter<
            Range<usize>,
            (&'b mut [u8], ()),
            fn(usize, &mut (&'b mut [u8], ())) -> CruiserResult<T::Access>,
        >,
    >
    where
        L: InPlaceGet<'a, ReadArg = ()> + ToSolanaUsize,
        T: InPlace<'b, ReadArg = ()>,
    {
        self.all_with_clone_arg(())
    }
}
impl<'a, T, L, const N: usize> InPlace<'a> for StaticSizeVec<T, L, N>
where
    T: OnChainStaticSize,
    L: InPlace<'a, ReadArg = (), CreateArg = ()> + InitToZero,
{
    type Access = StaticSizeVecAccess<'a, T, L, N>;
    type CreateArg = ();
    type ReadArg = ();

    fn create_with_arg(
        data: &mut &'a mut [u8],
        _arg: Self::CreateArg,
    ) -> CruiserResult<Self::Access> {
        let length = L::create_with_arg(data, ())?;
        let element_length = T::on_chain_static_size();
        let data = data.try_advance(element_length * N)?;
        Ok(StaticSizeVecAccess {
            length,
            data,
            element_length,
            phantom_t: PhantomData,
        })
    }

    fn read_with_arg(data: &mut &'a mut [u8], arg: ()) -> CruiserResult<Self::Access> {
        let length = L::read(data)?;
        let element_length = T::on_chain_max_size(arg);
        let data = data.try_advance(element_length * N)?;
        Ok(StaticSizeVecAccess {
            length,
            data,
            element_length,
            phantom_t: PhantomData,
        })
    }
}
