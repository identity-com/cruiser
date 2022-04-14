use crate::in_place::{InPlace, InPlaceCreate, InPlaceGet, InPlaceRead, InPlaceSet, InPlaceWrite};
use crate::util::AdvanceArray;
use crate::CruiserResult;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

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
    fn get_in_place(&self) -> u8 {
        **self
    }
}
impl<'a> InPlaceGet<u8> for <u8 as InPlace<'a>>::AccessMut {
    fn get_in_place(&self) -> u8 {
        **self
    }
}
impl<'a> InPlaceSet<u8> for <u8 as InPlace<'a>>::AccessMut {
    fn set_in_place(&mut self, val: u8) {
        **self = val;
    }
}
impl<'a> InPlaceGet<usize> for <u8 as InPlace<'a>>::Access {
    fn get_in_place(&self) -> usize {
        **self as usize
    }
}
impl<'a> InPlaceGet<usize> for <u8 as InPlace<'a>>::AccessMut {
    fn get_in_place(&self) -> usize {
        **self as usize
    }
}
impl<'a> InPlaceSet<usize> for <u8 as InPlace<'a>>::AccessMut {
    fn set_in_place(&mut self, val: usize) {
        **self = val.try_into().expect("usize is too large to fit in u8");
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
    fn get_in_place(&self) -> i8 {
        **self
    }
}
impl<'a> InPlaceGet<i8> for <i8 as InPlace<'a>>::AccessMut {
    fn get_in_place(&self) -> i8 {
        **self
    }
}
impl<'a> InPlaceSet<i8> for <i8 as InPlace<'a>>::AccessMut {
    fn set_in_place(&mut self, val: i8) {
        **self = val;
    }
}

/// An inplace version of primitive numbers to adhere to alignment
#[derive(Debug)]
pub struct PrimNumInPlace<T, D, const N: usize>(D, PhantomData<T>);
impl<T, D, const N: usize> InPlaceGet<T> for PrimNumInPlace<T, D, N>
where
    T: FromNE<N>,
    D: Deref<Target = [u8; N]>,
{
    fn get_in_place(&self) -> T {
        T::from_ne_bytes(*self.0.deref())
    }
}
impl<T, D, const N: usize> InPlaceSet<T> for PrimNumInPlace<T, D, N>
where
    T: FromNE<N>,
    D: DerefMut<Target = [u8; N]>,
{
    fn set_in_place(&mut self, val: T) {
        *self.0.deref_mut() = val.into_ne_bytes();
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
            fn get_in_place(&self) -> $larger {
                <Self as InPlaceGet<$ty>>::get_in_place(self) as $larger
            }
        }
        impl<D> InPlaceSet<$larger> for PrimNumInPlace<$ty, D, $size> where PrimNumInPlace<$ty, D, $size>: InPlaceSet<$ty> {
            fn set_in_place(&mut self, val: $larger) {
                <Self as InPlaceSet<$ty>>::set_in_place(self, val.try_into().unwrap())
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

#[cfg(test)]
mod test {
    use crate::in_place::{
        InPlace, InPlaceCreate, InPlaceGet, InPlaceRead, InPlaceSet, InPlaceUnitCreate,
        InPlaceUnitRead, InPlaceUnitWrite, InPlaceWrite,
    };
    use cruiser::in_place::FromNE;
    use num_traits::Zero;
    use rand::distributions::{Distribution, Standard};
    use rand::{thread_rng, Rng};
    use std::fmt::Debug;

    fn prim_test_func<R, T, const N: usize>(rng: &mut R)
    where
        R: Rng,
        Standard: Distribution<T>,
        for<'a> T: FromNE<N>
            + Copy
            + PartialEq
            + Debug
            + InPlace<'a>
            + InPlaceCreate<'a, ()>
            + InPlaceRead<'a, ()>
            + InPlaceWrite<'a, ()>
            + Zero,
        for<'a> <T as InPlace<'a>>::Access: InPlaceGet<T>,
        for<'a> <T as InPlace<'a>>::AccessMut: InPlaceGet<T> + InPlaceSet<T>,
    {
        let value: T = rng.gen();
        let bytes = value.into_ne_bytes();
        let mut write_bytes = [0u8; N];
        let value2 = T::from_ne_bytes(bytes);
        assert_eq!(value, value2);

        T::create(&mut write_bytes).expect("Could not create");
        let in_place = T::read(&write_bytes).expect("Could not read");
        assert_eq!(in_place.get_in_place(), T::zero());
        drop(in_place);
        let mut in_place = T::write(&mut write_bytes).expect("Could not write");
        in_place.set_in_place(value);
        assert_eq!(in_place.get_in_place(), value);
    }

    #[test]
    fn prim_test() {
        let mut rng = thread_rng();
        for _ in 0..1024 {
            prim_test_func::<_, u16, 2>(&mut rng);
            prim_test_func::<_, u32, 4>(&mut rng);
            prim_test_func::<_, u64, 8>(&mut rng);
            prim_test_func::<_, u128, 16>(&mut rng);
            prim_test_func::<_, i16, 2>(&mut rng);
            prim_test_func::<_, i32, 4>(&mut rng);
            prim_test_func::<_, i64, 8>(&mut rng);
            prim_test_func::<_, i128, 16>(&mut rng);
        }
    }

    #[test]
    fn short_prim_test() {
        let mut rng = thread_rng();
        for _ in 0..1024 {
            let val = rng.gen::<u8>();
            let mut val_data = [0];
            u8::create(&mut val_data).expect("Could not create");
            let in_place = u8::read(&val_data).expect("Could not read");
            let got: u8 = in_place.get_in_place();
            assert_eq!(got, 0u8);
            let mut in_place = u8::write(&mut val_data).expect("Could not write");
            in_place.set_in_place(val);
            let got: u8 = in_place.get_in_place();
            assert_eq!(got, val);

            let val = rng.gen::<i8>();
            let mut val_data = [0];
            i8::create(&mut val_data).expect("Could not create");
            let in_place = i8::read(&val_data).expect("Could not read");
            assert_eq!(in_place.get_in_place(), 0);
            let mut in_place = i8::write(&mut val_data).expect("Could not write");
            in_place.set_in_place(val);
            assert_eq!(in_place.get_in_place(), val);
        }
    }
}
