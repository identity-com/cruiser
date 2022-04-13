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
    type AccessMut;
}
pub trait InPlaceCreate<'a, C>: InPlace<'a> {
    /// Create a new instance of `Self::Access` with the given argument
    fn create_with_arg(data: &mut &'a mut [u8], arg: C) -> CruiserResult<Self::AccessMut>;
}
pub trait InPlaceRead<'a, R>: InPlace<'a> {
    /// Reads the access type from data and an arg
    fn read_with_arg(data: &mut &'a [u8], arg: R) -> CruiserResult<Self::Access>;
}
pub trait InPlaceWrite<'a, W>: InPlace<'a> {
    /// Writes the access type to data and an arg
    fn write_with_arg(data: &mut &'a mut [u8], arg: W) -> CruiserResult<Self::AccessMut>;
}

/// In-place account data create access with no arg, auto derived
pub trait InPlaceUnitCreate<'a>: InPlaceCreate<'a, ()> {
    /// Create a new instance of `Self::Access` with no argument
    fn create(data: &mut &'a mut [u8]) -> CruiserResult<Self::Access> {
        Self::create_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitCreate<'a> for T where T: InPlaceCreate<'a, ()> {}

/// In-place account data read access with no arg, auto derived
pub trait InPlaceUnitRead<'a>: InPlaceRead<'a, ()> {
    /// Reads the access type from data
    fn read(data: &mut &'a [u8]) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitRead<'a> for T where T: InPlaceRead<'a, ()> {}

pub trait InPlaceUnitWrite<'a>: InPlaceWrite<'a, ()> {
    /// Writes the access type to data
    fn write(data: &mut &'a mut [u8]) -> CruiserResult<Self::AccessMut> {
        Self::write_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitWrite<'a> for T where T: InPlaceWrite<'a, ()> {}

pub trait InPlaceUnit<'a>: InPlaceUnitCreate<'a> + InPlaceUnitRead<'a> {}
impl<'a, T> InPlaceUnit<'a> for T where T: InPlaceUnitCreate<'a> + InPlaceUnitRead<'a> {}

/// An access that can get a value out
pub trait InPlaceGet<V> {
    /// Gets a value out
    fn get(&self) -> CruiserResult<V>;
}
/// An access that can be set to a value
pub trait InPlaceSet<V> {
    /// Sets this to a value
    fn set(&mut self, val: V) -> CruiserResult;
}

impl<'a> InPlace<'a> for u8 {
    type Access = &'a u8;
    type AccessMut = &'a mut u8;
}
impl<'a> InPlaceCreate<'a, ()> for u8 {
    fn create_with_arg(data: &mut &'a mut [u8], arg: ()) -> CruiserResult<Self::AccessMut> {
        Self::write_with_arg(data, arg)
    }
}
impl<'a> InPlaceRead<'a, ()> for u8 {
    fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let out: &[_; 1] = data.try_advance_array()?;
        Ok(&out[0])
    }
}
impl<'a> InPlaceWrite<'a, ()> for u8 {
    fn write_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
        let out: &mut [_; 1] = data.try_advance_array()?;
        Ok(&mut out[0])
    }
}
impl<'a> InPlaceGet<u8> for u8::Access {
    fn get(&self) -> CruiserResult<u8> {
        Ok(**self)
    }
}
impl<'a> InPlaceGet<u8> for u8::AccessMut {
    fn get(&self) -> CruiserResult<u8> {
        Ok(**self)
    }
}
impl<'a> InPlaceSet<u8> for u8::AccessMut {
    fn set(&mut self, val: u8) -> CruiserResult {
        **self = val;
        Ok(())
    }
}
impl InPlaceGet<usize> for u8::Access {
    fn get(&self) -> CruiserResult<usize> {
        Ok(**self as usize)
    }
}
impl InPlaceGet<usize> for u8::AccessMut {
    fn get(&self) -> CruiserResult<usize> {
        Ok(**self as usize)
    }
}
impl InPlaceSet<usize> for u8::AccessMut {
    fn set(&mut self, val: usize) -> CruiserResult {
        **self = val.try_into()?;
        Ok(())
    }
}

impl<'a> InPlace<'a> for i8 {
    type Access = &'a i8;
    type AccessMut = &'a mut i8;
}
impl<'a> InPlaceCreate<'a, ()> for i8 {
    fn create_with_arg(data: &mut &'a mut [u8], arg: ()) -> CruiserResult<Self::AccessMut> {
        Self::write_with_arg(data, arg)
    }
}
impl<'a> InPlaceRead<'a, ()> for i8 {
    fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let out: &[_; 1] = data.try_advance_array()?;
        Ok(unsafe { &*(out.as_ptr().cast::<i8>()) })
    }
}
impl<'a> InPlaceWrite<'a, ()> for i8 {
    fn write_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
        let out: &mut [_; 1] = data.try_advance_array()?;
        Ok(unsafe { &mut *(out.as_mut_ptr().cast::<i8>()) })
    }
}
impl<'a> InPlaceGet<i8> for i8::Access {
    fn get(&self) -> CruiserResult<i8> {
        Ok(**self)
    }
}
impl<'a> InPlaceSet<i8> for i8::AccessMut {
    fn set(&mut self, val: i8) -> CruiserResult {
        **self = val;
        Ok(())
    }
}

/// An inplace version of primitive numbers to adhere to alignment
#[derive(Debug)]
pub struct PrimNumInPlace<'a, T, const N: usize>(&'a mut [u8; N], PhantomData<T>);
impl<'a, T, const N: usize> InPlaceGet<T> for PrimNumInPlace<'a, T, N>
where
    T: FromNE<N>,
{
    fn get(&self) -> CruiserResult<T> {
        Ok(T::from_ne_bytes(*self.0))
    }
}
impl<'a, T, const N: usize> InPlaceSet<T> for PrimNumInPlace<'a, T, N>
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
        }
        impl<'a> InPlaceCreate<'a, ()> for $ty {
            fn create_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
                Self::read_with_arg(data, ())
            }
        }
        impl<'a> InPlaceRead<'a, ()> for $ty {
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
    type Access = &'a Pubkey;
    type AccessMut = &'a mut Pubkey;
}
impl<'a> InPlaceCreate<'a, ()> for Pubkey {
    fn create_with_arg(data: &mut &'a mut [u8], arg: ()) -> CruiserResult<Self::AccessMut> {
        Self::write_with_arg(data, arg)
    }
}
impl<'a> InPlaceRead<'a, ()> for Pubkey {
    fn read_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let data: &mut [u8; 32] = data.try_advance_array()?;
        // Safe because Pubkey is transparent to [u8; 32]
        Ok(unsafe { &*data.as_ptr().cast::<Pubkey>() })
    }
}
impl<'a> InPlaceWrite<'a, ()> for Pubkey {
    fn write_with_arg(data: &mut &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
        let data: &mut [u8; 32] = data.try_advance_array()?;
        // Safe because Pubkey is transparent to [u8; 32]
        Ok(unsafe { &mut *data.as_mut_ptr().cast::<Pubkey>() })
    }
}
impl<'a> InPlaceGet<Pubkey> for Pubkey::Access {
    fn get(&self) -> CruiserResult<Pubkey> {
        Ok(**self)
    }
}
impl<'a> InPlaceGet<Pubkey> for Pubkey::AccessMut {
    fn get(&self) -> CruiserResult<Pubkey> {
        Ok(**self)
    }
}
impl<'a> InPlaceSet<Pubkey> for Pubkey::AccessMut {
    fn set(&mut self, val: Pubkey) -> CruiserResult {
        **self = val;
        Ok(())
    }
}

/// In-place access to arrays
#[derive(Debug)]
pub struct InPlaceArray<T, D, const N: usize> {
    element_length: usize,
    data: D,
    phantom_t: PhantomData<fn() -> T>,
}
impl<T, D, const N: usize> InPlaceArray<T, D, N>
where
    D: AsRef<[u8]>,
{
    /// Gets an item in the array with a read arg
    pub fn get_with_arg<'b, A>(&'b self, index: usize, arg: A) -> CruiserResult<Option<T::Access>>
    where
        T: InPlaceRead<'b, A>,
    {
        if index < N {
            Ok(Some(T::read_with_arg(
                &mut &self.data.as_ref()[self.element_length * index..],
                arg,
            )?))
        } else {
            Ok(None)
        }
    }

    pub fn get_with_arg_mut<'b, A>(
        &'b mut self,
        index: usize,
        arg: A,
    ) -> CruiserResult<Option<T::AccessMut>>
    where
        T: InPlaceWrite<'b, A>,
        D: AsMut<[u8]>,
    {
        if index < N {
            Ok(Some(T::write_with_arg(
                &mut &mut self.data.as_mut()[self.element_length * index..],
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

    pub fn get_mut<'b>(&'b mut self, index: usize) -> CruiserResult<Option<T::AccessMut>>
    where
        T: InPlaceUnitWrite<'b>,
        D: AsMut<[u8]>,
    {
        self.get_with_arg_mut(index, ())
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
    type AccessMut = ();
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
        L::Access: InPlaceGet<L> + InPlaceSet<L>,
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
