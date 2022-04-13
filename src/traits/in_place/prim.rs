use crate::in_place::{InPlace, InPlaceCreate, InPlaceGet, InPlaceRead, InPlaceSet, InPlaceWrite};
use crate::util::AdvanceArray;
use crate::{CruiserResult, GenericError};
use std::marker::PhantomData;

impl<'a> InPlace<'a> for u8 {
    type Access = &'a u8;
    type AccessMut = &'a mut u8;
}
impl<'a> InPlaceCreate<'a, ()> for u8 {
    fn create_with_arg(_data: &mut [u8], _arg: ()) -> CruiserResult {
        Ok(())
    }
}
impl<'a> InPlaceRead<'a, ()> for u8 {
    fn read_with_arg(mut data: &'a [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let out: &[_; 1] = data.try_advance_array()?;
        Ok(&out[0])
    }
}
impl<'a> InPlaceWrite<'a, ()> for u8 {
    fn write_with_arg(mut data: &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
        let out: &mut [_; 1] = data.try_advance_array()?;
        Ok(&mut out[0])
    }
}
impl<'a> InPlaceGet<u8> for <u8 as InPlace<'a>>::Access {
    fn get(&self) -> CruiserResult<u8> {
        Ok(**self)
    }
}
impl<'a> InPlaceGet<u8> for <u8 as InPlace<'a>>::AccessMut {
    fn get(&self) -> CruiserResult<u8> {
        Ok(**self)
    }
}
impl<'a> InPlaceSet<u8> for <u8 as InPlace<'a>>::AccessMut {
    fn set(&mut self, val: u8) -> CruiserResult {
        **self = val;
        Ok(())
    }
}
impl<'a> InPlaceGet<usize> for <u8 as InPlace<'a>>::Access {
    fn get(&self) -> CruiserResult<usize> {
        Ok(**self as usize)
    }
}
impl<'a> InPlaceGet<usize> for <u8 as InPlace<'a>>::AccessMut {
    fn get(&self) -> CruiserResult<usize> {
        Ok(**self as usize)
    }
}
impl<'a> InPlaceSet<usize> for <u8 as InPlace<'a>>::AccessMut {
    fn set(&mut self, val: usize) -> CruiserResult {
        **self = val.try_into().map_err(|_| GenericError::SizeInvalid {
            min: 0,
            max: u8::MAX as usize,
            value: val,
        })?;
        Ok(())
    }
}

impl<'a> InPlace<'a> for i8 {
    type Access = &'a i8;
    type AccessMut = &'a mut i8;
}
impl<'a> InPlaceCreate<'a, ()> for i8 {
    fn create_with_arg(_data: &mut [u8], _arg: ()) -> CruiserResult {
        Ok(())
    }
}
impl<'a> InPlaceRead<'a, ()> for i8 {
    fn read_with_arg(mut data: &'a [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let out: &[_; 1] = data.try_advance_array()?;
        Ok(unsafe { &*(out.as_ptr().cast::<i8>()) })
    }
}
impl<'a> InPlaceWrite<'a, ()> for i8 {
    fn write_with_arg(mut data: &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
        let out: &mut [_; 1] = data.try_advance_array()?;
        Ok(unsafe { &mut *(out.as_mut_ptr().cast::<i8>()) })
    }
}
impl<'a> InPlaceGet<i8> for <i8 as InPlace<'a>>::Access {
    fn get(&self) -> CruiserResult<i8> {
        Ok(**self)
    }
}
impl<'a> InPlaceSet<i8> for <i8 as InPlace<'a>>::AccessMut {
    fn set(&mut self, val: i8) -> CruiserResult {
        **self = val;
        Ok(())
    }
}

/// An inplace version of primitive numbers to adhere to alignment
#[derive(Debug)]
pub struct PrimNumInPlace<T, D, const N: usize>(D, PhantomData<T>);
impl<T, D, const N: usize> InPlaceGet<T> for PrimNumInPlace<T, D, N>
where
    T: FromNE<N>,
    D: AsRef<[u8; N]>,
{
    fn get(&self) -> CruiserResult<T> {
        Ok(T::from_ne_bytes(*self.0.as_ref()))
    }
}
impl<T, D, const N: usize> InPlaceSet<T> for PrimNumInPlace<T, D, N>
where
    T: FromNE<N>,
    D: AsMut<[u8; N]>,
{
    fn set(&mut self, val: T) -> CruiserResult {
        *self.0.as_mut() = val.into_ne_bytes();
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
    ($ty:ty, $size:expr$(, $($larger:ty),*)?) => {
        impl FromNE<$size> for $ty {
            fn from_ne_bytes(bytes: [u8; $size]) -> Self {
                Self::from_ne_bytes(bytes)
            }

            fn into_ne_bytes(self) -> [u8; $size] {
                self.to_ne_bytes()
            }
        }
        impl<'a> InPlace<'a> for $ty {
            type Access = PrimNumInPlace<$ty, &'a [u8; $size], $size>;
            type AccessMut = PrimNumInPlace<$ty, &'a mut [u8; $size], $size>;
        }
        impl<'a> InPlaceCreate<'a, ()> for $ty {
            fn create_with_arg(_data: &mut [u8], _arg: ()) -> CruiserResult {
                Ok(())
            }
        }
        impl<'a> InPlaceRead<'a, ()> for $ty {
            fn read_with_arg(mut data: &'a [u8], _arg: ()) -> CruiserResult<Self::Access> {
                Ok(PrimNumInPlace(data.try_advance_array()?, PhantomData))
            }
        }
        impl<'a> InPlaceWrite<'a, ()> for $ty {
            fn write_with_arg(mut data: &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
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
        $($(
        impl<D> InPlaceGet<$larger> for PrimNumInPlace<$ty, D, $size> where PrimNumInPlace<$ty, D, $size>: InPlaceGet<$ty> {
            fn get(&self) -> CruiserResult<$larger> {
                Ok(<Self as InPlaceGet<$ty>>::get(self)? as $larger)
            }
        }
        impl<D> InPlaceSet<$larger> for PrimNumInPlace<$ty, D, $size> where PrimNumInPlace<$ty, D, $size>: InPlaceSet<$ty> {
            fn set(&mut self, val: $larger) -> CruiserResult {
                <Self as InPlaceSet<$ty>>::set(self, val.try_into().map_err(|_| GenericError::SizeInvalid {
                    min: 0,
                    max: <$ty>::MAX as usize,
                    value: val as usize,
                })?)
            }
        }
        )*)?
    };
}
impl_from_ne!(u16, 2, u32, u64, usize);
impl_from_ne!(u32, 4, u64, usize);
impl_from_ne!(u64, 8);
impl_from_ne!(u128, 16);
impl_from_ne!(i16, 2);
impl_from_ne!(i32, 4);
impl_from_ne!(i64, 8);
impl_from_ne!(i128, 16);
