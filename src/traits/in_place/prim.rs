use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::util::{MappableRef, MappableRefMut, TryMappableRef, TryMappableRefMut};
use crate::{CruiserResult, GenericError};
use cruiser::on_chain_size::OnChainSize;
use num_traits::Num;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// An inplace version of primitive numbers to adhere to alignment
#[derive(Debug)]
pub struct PrimNumInPlace<T, A, const N: usize>(A, PhantomData<T>);
fn new_prim<'a, T, A, const N: usize>(
    data: A,
) -> CruiserResult<PrimNumInPlace<T, A::Output<'a, [u8; N]>, N>>
where
    T: NativeEndian<N>,
    A: Deref<Target = [u8]> + TryMappableRef,
{
    Ok(PrimNumInPlace(
        data.try_map_ref(|r| {
            if r.len() < N {
                Err(GenericError::NotEnoughData {
                    needed: N,
                    remaining: r.len(),
                })
            } else {
                Ok((&r[..N]).try_into().unwrap())
            }
        })?,
        PhantomData,
    ))
}
fn new_prim_mut<'a, T, A, const N: usize>(
    data: A,
) -> CruiserResult<PrimNumInPlace<T, A::Output<'a, [u8; N]>, N>>
where
    T: NativeEndian<N>,
    A: DerefMut<Target = [u8]> + TryMappableRefMut,
{
    Ok(PrimNumInPlace(
        data.try_map_ref_mut(|r| {
            if r.len() < N {
                Err(GenericError::NotEnoughData {
                    needed: N,
                    remaining: r.len(),
                })
            } else {
                Ok((&mut r[..N]).try_into().unwrap())
            }
        })?,
        PhantomData,
    ))
}

/// Gets a number from an in-place accessor
pub trait GetNum {
    /// The type of the number
    type Num;
    /// Gets the number
    fn get_num(&self) -> Self::Num;
}
/// Sets a number in an in-place accessor
pub trait SetNum: GetNum {
    /// Sets the number
    fn set_num(&mut self, value: Self::Num);
}
impl<T, A, const N: usize> GetNum for PrimNumInPlace<T, A, N>
where
    T: NativeEndian<N>,
    A: Deref<Target = [u8; N]>,
{
    type Num = T;
    fn get_num(&self) -> Self::Num {
        T::from_ne_bytes(*self.0)
    }
}
impl<T, A, const N: usize> SetNum for PrimNumInPlace<T, A, N>
where
    T: NativeEndian<N>,
    A: DerefMut<Target = [u8; N]>,
{
    fn set_num(&mut self, value: Self::Num) {
        *self.0 = value.into_ne_bytes();
    }
}

/// A number that can be derived from native-endian bytes
pub trait NativeEndian<const N: usize>: OnChainSize + Sized + Num {
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
        impl NativeEndian<$size> for $ty {
            fn from_ne_bytes(bytes: [u8; $size]) -> Self {
                Self::from_ne_bytes(bytes)
            }

            fn into_ne_bytes(self) -> [u8; $size] {
                self.to_ne_bytes()
            }
        }
        impl InPlace for $ty {
            type Access<'a, A>
            where
                Self: 'a,
                A: 'a + MappableRef + TryMappableRef,
            = PrimNumInPlace<$ty, <A as TryMappableRef>::Output<'a, [u8; $size]>, $size>;

            type AccessMut<'a, A>
            where
                Self: 'a,
                A: 'a + MappableRef + TryMappableRef + MappableRefMut + TryMappableRefMut,
            = PrimNumInPlace<$ty, <A as TryMappableRefMut>::Output<'a, [u8; $size]>, $size>;
        }
        impl InPlaceCreate for $ty {
            fn create_with_arg<A: DerefMut<Target = [u8]>>(_data: A, _arg: ()) -> CruiserResult {
                Ok(())
            }
        }
        impl InPlaceCreate<$ty> for $ty {
            fn create_with_arg<A: DerefMut<Target = [u8]>>(mut data: A, arg: $ty) -> CruiserResult {
                data[..$size].copy_from_slice(&arg.into_ne_bytes());
                Ok(())
            }
        }
        impl InPlaceRead for $ty {
            fn read_with_arg<'a, A>(data: A, _arg: ()) -> CruiserResult<Self::Access<'a, A>>
            where
                Self: 'a,
                A: 'a + Deref<Target = [u8]> + MappableRef + TryMappableRef,
            {
                new_prim(data)
            }
        }
        impl InPlaceWrite for $ty {
            fn write_with_arg<'a, A>(data: A, _arg: ()) -> CruiserResult<Self::AccessMut<'a, A>>
            where
                Self: 'a,
                A: 'a
                    + DerefMut<Target = [u8]>
                    + MappableRef
                    + TryMappableRef
                    + MappableRefMut
                    + TryMappableRefMut,
            {
                new_prim_mut(data)
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
impl_from_ne!(u8, 1);
impl_from_ne!(u16, 2);
impl_from_ne!(u32, 4);
impl_from_ne!(u64, 8);
impl_from_ne!(u128, 16);
impl_from_ne!(i8, 1);
impl_from_ne!(i16, 2);
impl_from_ne!(i32, 4);
impl_from_ne!(i64, 8);
impl_from_ne!(i128, 16);

#[cfg(test)]
mod test {
    use crate::in_place::{GetNum, InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite, SetNum};
    use cruiser::in_place::NativeEndian;
    use num_traits::Zero;
    use rand::distributions::{Distribution, Standard};
    use rand::{thread_rng, Rng};
    use std::fmt::Debug;

    fn prim_test_func<R, T, const N: usize>(rng: &mut R)
    where
        R: Rng,
        Standard: Distribution<T>,
        T: NativeEndian<N>
            + Copy
            + PartialEq
            + Debug
            + InPlace
            + InPlaceCreate
            + InPlaceRead
            + InPlaceWrite
            + Zero,
        for<'a> T::Access<'a, &'a [u8]>: GetNum<Num = T>,
        for<'a> T::AccessMut<'a, &'a mut [u8]>: SetNum<Num = T>,
    {
        let value: T = rng.gen();
        let bytes = value.into_ne_bytes();
        let mut write_bytes = [0u8; N];
        let value2 = T::from_ne_bytes(bytes);
        assert_eq!(value, value2);

        T::create_with_arg(write_bytes.as_mut_slice(), ()).expect("Could not create");
        let in_place = T::read_with_arg(write_bytes.as_slice(), ()).expect("Could not read");
        assert_eq!(in_place.get_num(), T::zero());
        drop(in_place);
        let mut in_place =
            T::write_with_arg(write_bytes.as_mut_slice(), ()).expect("Could not write");
        in_place.set_num(value);
        assert_eq!(in_place.get_num(), value);
    }

    #[test]
    fn prim_test() {
        let mut rng = thread_rng();
        for _ in 0..1024 {
            prim_test_func::<_, u8, 1>(&mut rng);
            prim_test_func::<_, u16, 2>(&mut rng);
            prim_test_func::<_, u32, 4>(&mut rng);
            prim_test_func::<_, u64, 8>(&mut rng);
            prim_test_func::<_, u128, 16>(&mut rng);
            prim_test_func::<_, i8, 1>(&mut rng);
            prim_test_func::<_, i16, 2>(&mut rng);
            prim_test_func::<_, i32, 4>(&mut rng);
            prim_test_func::<_, i64, 8>(&mut rng);
            prim_test_func::<_, i128, 16>(&mut rng);
        }
    }
}
