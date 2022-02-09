mod array;
mod option;
mod vec;

pub use array::*;
pub use option::*;
pub use vec::*;

use crate::{Advance, Error, GeneratorError, GeneratorResult};
use array_init::try_array_init;
use std::convert::{Infallible, TryInto};
use std::mem::size_of;

pub trait InPlaceBuilder {
    type InPlaceData<'a>: InPlaceData;
    type SizeError;
    type CreateArg;

    /// This size is cached and should never change since creation.
    /// Incoming length has no guarantees.
    fn data_size(data: &[u8]) -> Result<usize, Self::SizeError>;
    fn create_size(create_arg: &Self::CreateArg) -> usize;
    /// Incoming length has no guarantees.
    fn create(
        data: &mut [u8],
        create_arg: Self::CreateArg,
    ) -> GeneratorResult<Self::InPlaceData<'_>>;
    /// Incoming length has no guarantees.
    fn read(data: &mut [u8]) -> GeneratorResult<Self::InPlaceData<'_>>;
}
/// An in place structure
pub trait StaticSized: InPlaceBuilder {
    /// The size of the data on-chain
    const DATA_SIZE: usize;

    // // TODO: Add back in when https://github.com/rust-lang/rust/issues/92961 resolved
    // /// An optimized version of [`InPlaceBuilder::create`] by avoiding a size check.
    // /// [`InPlaceBuilder::create`] should usually call this function by converting with [`TryInto`].
    // fn create_static(
    //     data: &mut [u8; Self::DATA_SIZE],
    //     create_arg: Self::CreateArg,
    // ) -> GeneratorResult<Self::InPlaceData<'_>>;
    // /// An optimized version of [`InPlaceBuilder::read`] by avoiding a size check.
    // /// [`InPlaceBuilder::read`] should usually call this function by converting with [`TryInto`].
    // fn read_static(data: &mut [u8; Self::DATA_SIZE]) -> GeneratorResult<Self::InPlaceData<'_>>;
}
pub trait InPlaceData {
    fn self_data_size(&self) -> usize;
}
pub trait InPlaceGet<'a, V>: InPlaceData {
    /// Incoming length has no guarantees.
    fn get_value(&'a self) -> V;
    fn read_and_get(data: &[u8]) -> GeneratorResult<V>;
}
pub trait InPlaceSet<'a, V>: InPlaceData {
    fn set_value(&'a mut self, value: V);
}

impl InPlaceBuilder for () {
    type InPlaceData<'a> = ();
    type SizeError = Infallible;
    type CreateArg = ();

    #[inline]
    fn data_size(_data: &[u8]) -> Result<usize, Self::SizeError> {
        Ok(0)
    }

    #[inline]
    fn create_size(_create_arg: &Self::CreateArg) -> usize {
        0
    }

    #[inline]
    fn create(
        _data: &mut [u8],
        _create_arg: Self::CreateArg,
    ) -> GeneratorResult<Self::InPlaceData<'_>> {
        Ok(())
    }

    #[inline]
    fn read(_data: &mut [u8]) -> GeneratorResult<Self::InPlaceData<'_>> {
        Ok(())
    }
}
impl StaticSized for () {
    const DATA_SIZE: usize = 0;

    // fn create_static(
    //     _data: &mut [u8; Self::DATA_SIZE],
    //     _create_arg: Self::CreateArg,
    // ) -> GeneratorResult<Self::InPlaceData<'_>> {
    //     Ok(())
    // }
    //
    // fn read_static(_data: &mut [u8; Self::DATA_SIZE]) -> GeneratorResult<Self::InPlaceData<'_>> {
    //     Ok(())
    // }
}
impl InPlaceData for () {
    fn self_data_size(&self) -> usize {
        0
    }
}

impl<T1, T2> InPlaceBuilder for (T1, T2)
where
    T1: InPlaceBuilder,
    T2: InPlaceBuilder,
    Box<dyn Error>: From<T1::SizeError> + From<T2::SizeError>,
{
    type InPlaceData<'a> = (T1::InPlaceData<'a>, T2::InPlaceData<'a>);
    type SizeError = Box<dyn Error>;
    type CreateArg = (T1::CreateArg, T2::CreateArg);

    fn data_size(data: &[u8]) -> Result<usize, Self::SizeError> {
        let mut size = 0;
        size += T1::data_size(data)?;
        size += T2::data_size(if data.len() < size {
            &[]
        } else {
            &data[size..]
        })?;
        Ok(size)
    }

    fn create_size(create_arg: &Self::CreateArg) -> usize {
        T1::create_size(&create_arg.0) + T2::create_size(&create_arg.1)
    }

    fn create(
        data: &mut [u8],
        create_arg: Self::CreateArg,
    ) -> GeneratorResult<Self::InPlaceData<'_>> {
        let size1 = T1::create_size(&create_arg.0);
        let (a, b) = if size1 > data.len() {
            (data, &mut [] as &mut [u8])
        } else {
            data.split_at_mut(size1)
        };
        Ok((T1::create(a, create_arg.0)?, T2::create(b, create_arg.1)?))
    }

    fn read(data: &mut [u8]) -> GeneratorResult<Self::InPlaceData<'_>> {
        let size1 = T1::data_size(data)?;
        let (a, b) = if size1 > data.len() {
            (data, &mut [] as &mut [u8])
        } else {
            data.split_at_mut(size1)
        };
        Ok((T1::read(a)?, T2::read(b)?))
    }
}
impl<T1, T2> InPlaceData for (T1, T2)
where
    T1: InPlaceData,
    T2: InPlaceData,
{
    fn self_data_size(&self) -> usize {
        self.0.self_data_size() + self.1.self_data_size()
    }
}
impl<T1, T2> StaticSized for (T1, T2)
where
    T1: StaticSized,
    T2: StaticSized,
    Box<dyn Error>: From<T1::SizeError> + From<T2::SizeError>,
{
    const DATA_SIZE: usize = T1::DATA_SIZE + T2::DATA_SIZE;
}

impl<T, const N: usize> InPlaceBuilder for [T; N]
where
    T: StaticSized<SizeError = Infallible>,
{
    type InPlaceData<'a> = [T::InPlaceData<'a>; N];
    type SizeError = Infallible;
    type CreateArg = [T::CreateArg; N];

    fn data_size(_data: &[u8]) -> Result<usize, Self::SizeError> {
        Ok(Self::DATA_SIZE)
    }

    fn create_size(_create_arg: &Self::CreateArg) -> usize {
        Self::DATA_SIZE
    }

    fn create(
        mut data: &mut [u8],
        create_arg: Self::CreateArg,
    ) -> GeneratorResult<Self::InPlaceData<'_>> {
        let mut iter = IntoIterator::into_iter(create_arg);
        try_array_init(|_| T::create(data.try_advance(T::DATA_SIZE)?, iter.next().unwrap()))
    }

    fn read(mut data: &mut [u8]) -> GeneratorResult<Self::InPlaceData<'_>> {
        try_array_init(|_| T::read(data.try_advance(T::DATA_SIZE)?))
    }
}
impl<T, const N: usize> StaticSized for [T; N]
where
    T: StaticSized<SizeError = Infallible>,
{
    const DATA_SIZE: usize = T::DATA_SIZE * N;
}
impl<T, const N: usize> InPlaceData for [T; N]
where
    T: InPlaceData,
{
    fn self_data_size(&self) -> usize {
        self.iter().map(|item| item.self_data_size()).sum()
    }
}

#[derive(Debug)]
pub struct InPlaceNumber<'a, T>(pub(crate) &'a mut [u8; size_of::<T>()])
where
    [(); size_of::<T>()]:;

macro_rules! impl_in_place_for_prim_num {
    (all $(($ty:ty, $type_ident:ident)),+ $(,)?) => {
        $(impl_in_place_for_prim_num!($ty, $type_ident);)+
    };
    ($ty:ty, $type_ident:ident) => {
        pub type $type_ident<'a> = InPlaceNumber<'a, $ty>;
        impl InPlaceBuilder for $ty {
            type InPlaceData<'a> = InPlaceNumber<'a, $ty>;
            type SizeError = Infallible;
            type CreateArg = $ty;

            #[inline]
            fn data_size(_data: &[u8]) -> Result<usize, Self::SizeError> {
                Ok(size_of::<$ty>())
            }

            #[inline]
            fn create_size(_create_arg: &Self::CreateArg) -> usize {
                size_of::<$ty>()
            }

            fn create<'a>(data: &'a mut [u8], create_arg: Self::CreateArg) -> GeneratorResult<Self::InPlaceData<'a>> {
                if data.len() < size_of::<$ty>(){
                    Err(GeneratorError::NotEnoughData {
                        needed: size_of::<$ty>(),
                        remaining: data.len(),
                    }
                    .into())
                } else {
                    data.copy_from_slice(&create_arg.to_le_bytes());
                    Ok(InPlaceNumber((&mut data[..size_of::<$ty>()]).try_into().unwrap()))
                }
            }

            fn read<'a>(data: &'a mut [u8]) -> GeneratorResult<Self::InPlaceData<'a>> {
                if data.len() < size_of::<$ty>() {
                    Err(GeneratorError::NotEnoughData {
                        needed: size_of::<$ty>(),
                        remaining: data.len(),
                    }
                    .into())
                } else {
                    Ok(InPlaceNumber((&mut data[..size_of::<$ty>()]).try_into().unwrap()))
                }
            }
        }
        impl<'a> InPlaceData for InPlaceNumber<'a, $ty>{
            fn self_data_size(&self) -> usize {
                size_of::<$ty>()
            }
        }
        impl StaticSized for $ty {
            const DATA_SIZE: usize = size_of::<$ty>();

            // fn create_static(
            //     data: &mut [u8; Self::DATA_SIZE],
            //     create_arg: Self::CreateArg,
            // ) -> GeneratorResult<Self::InPlaceData<'_>> {
            //     *data = create_arg.to_le_bytes();
            //     Ok(InPlaceNumber(data))
            // }
            //
            // fn read_static(data: &mut [u8; Self::DATA_SIZE]) -> GeneratorResult<Self::InPlaceData<'_>> {
            //     Ok(InPlaceNumber(data))
            // }
        }
        impl<'a> InPlaceNumber<'a, $ty>{
            pub fn set(&mut self, value: $ty){
                self.0.copy_from_slice(&value.to_le_bytes())
            }

            pub fn get(&self) -> $ty{
                <$ty>::from_le_bytes(*self.0)
            }
        }
        impl<'a, 'b> InPlaceGet<'b, $ty> for InPlaceNumber<'a, $ty> {
            fn get_value(&'b self) -> $ty{
                self.get()
            }
            fn read_and_get(data: &[u8]) -> GeneratorResult<$ty>{
                if data.len() < size_of::<$ty>() {
                    Err(GeneratorError::NotEnoughData {
                        needed: size_of::<$ty>(),
                        remaining: data.len(),
                    }
                    .into())
                } else {
                    Ok(<$ty>::from_le_bytes(data[..size_of::<$ty>()].try_into().unwrap()))
                }
            }
        }
        impl<'a, 'b> InPlaceSet<'b, $ty> for InPlaceNumber<'a, $ty> {
            fn set_value(&'b mut self, value: $ty){
                self.0.copy_from_slice(&value.to_le_bytes())
            }
        }
    }
}
impl_in_place_for_prim_num!(
    all(u8, InPlaceU8),
    (u16, InPlaceU16),
    (u32, InPlaceU32),
    (u64, InPlaceU64),
    (u128, InPlaceU128),
    (i8, InPlaceI8),
    (i16, InPlaceI16),
    (i32, InPlaceI32),
    (i64, InPlaceI64),
    (i128, InPlaceI128),
);
