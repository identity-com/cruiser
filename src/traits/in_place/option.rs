use crate::AdvanceArray;
use core::convert::Infallible;
use solana_generator::traits::error::GeneratorResult;
use solana_generator::traits::in_place::{InPlaceBuilder, InPlaceData, StaticSized};

pub struct InPlaceOption<'a, T>
where
    T: StaticSized,
    [(); T::DATA_SIZE]:,
{
    discriminant: &'a mut u8,
    value: &'a mut [u8; T::DATA_SIZE],
}
impl<'a, T> InPlaceOption<'a, T>
where
    T: StaticSized,
    [(); T::DATA_SIZE]:,
{
    const fn data_size() -> usize {
        T::DATA_SIZE + 1
    }

    fn get(&mut self) -> GeneratorResult<Option<T::InPlaceData<'_>>> {
        match *self.discriminant {
            0 => Ok(None),
            1 => Ok(Some(T::read(self.value)?)),
            _ => unreachable!(),
        }
    }
}
impl<'a, T> InPlaceBuilder for InPlaceOption<'a, T>
where
    T: StaticSized,
    [(); T::DATA_SIZE]:,
{
    type InPlaceData<'b> = InPlaceOption<'b, T>;
    type SizeError = Infallible;
    type CreateArg = ();

    fn data_size(_data: &[u8]) -> Result<usize, Self::SizeError> {
        Ok(Self::data_size())
    }

    fn create_size(_create_arg: &Self::CreateArg) -> usize {
        Self::data_size()
    }

    fn create(
        mut data: &mut [u8],
        _create_arg: Self::CreateArg,
    ) -> GeneratorResult<Self::InPlaceData<'_>> {
        let discriminant: &mut [u8; 1] = data.try_advance_array()?;
        discriminant[0] = 0;
        Ok(InPlaceOption {
            discriminant: &mut discriminant[0],
            value: data.try_advance_array()?,
        })
    }

    fn read(mut data: &mut [u8]) -> GeneratorResult<Self::InPlaceData<'_>> {
        let discriminant: &mut [u8; 1] = data.try_advance_array()?;
        Ok(InPlaceOption {
            discriminant: &mut discriminant[0],
            value: data.try_advance_array()?,
        })
    }
}
impl<'a, T> StaticSized for InPlaceOption<'a, T>
where
    T: StaticSized,
    [(); T::DATA_SIZE]:,
{
    const DATA_SIZE: usize = T::DATA_SIZE + 1;

    // fn create_static(
    //     data: &mut [u8; Self::DATA_SIZE],
    //     _create_arg: Self::CreateArg,
    // ) -> GeneratorResult<Self::InPlaceData<'_>> {
    //     let [discriminant, value @ ..] = data;
    //     Ok(InPlaceOption {
    //         discriminant,
    //         value,
    //     })
    // }
    //
    // fn read_static(data: &mut [u8; Self::DATA_SIZE]) -> GeneratorResult<Self::InPlaceData<'_>> {
    //     let [discriminant, value @ ..] = data;
    //     Ok(InPlaceOption {
    //         discriminant,
    //         value,
    //     })
    // }
}
impl<'a, T> InPlaceData for InPlaceOption<'a, T>
where
    T: StaticSized,
    [(); T::DATA_SIZE]:,
{
    fn self_data_size(&self) -> usize {
        Self::data_size()
    }
}
