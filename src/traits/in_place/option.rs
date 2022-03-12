use crate::util::AdvanceArray;
use core::convert::Infallible;
use cruiser::traits::error::CruiserResult;
use cruiser::traits::in_place::{InPlaceBuilder, InPlaceData, StaticSized};

/// The option version of in-place data
#[derive(Debug)]
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

    /// Gets the optional value
    pub fn get(&mut self) -> CruiserResult<Option<T::InPlaceData<'_>>> {
        match *self.discriminant {
            0 => Ok(None),
            1 => Ok(Some(T::read(self.value)?)),
            _ => unreachable!(),
        }
    }
}
impl<T> InPlaceBuilder for Option<T>
where
    T: StaticSized,
    [(); T::DATA_SIZE]:,
{
    type InPlaceData<'a> = InPlaceOption<'a, T>;
    type SizeError = Infallible;
    type CreateArg = ();

    fn data_size(_data: &mut [u8]) -> Result<usize, Self::SizeError> {
        Ok(InPlaceOption::<T>::data_size())
    }

    fn create_size(_create_arg: &Self::CreateArg) -> usize {
        InPlaceOption::<T>::data_size()
    }

    fn create(
        mut data: &mut [u8],
        _create_arg: Self::CreateArg,
    ) -> CruiserResult<Self::InPlaceData<'_>> {
        let discriminant: &mut [u8; 1] = data.try_advance_array()?;
        discriminant[0] = 0;
        Ok(InPlaceOption {
            discriminant: &mut discriminant[0],
            value: data.try_advance_array()?,
        })
    }

    fn read(mut data: &mut [u8]) -> CruiserResult<Self::InPlaceData<'_>> {
        let discriminant: &mut [u8; 1] = data.try_advance_array()?;
        Ok(InPlaceOption {
            discriminant: &mut discriminant[0],
            value: data.try_advance_array()?,
        })
    }
}
impl<T> StaticSized for Option<T>
where
    T: StaticSized,
    [(); T::DATA_SIZE]:,
{
    const DATA_SIZE: usize = T::DATA_SIZE + 1;

    // fn create_static(
    //     data: &mut [u8; Self::DATA_SIZE],
    //     _create_arg: Self::CreateArg,
    // ) -> CruiserResult<Self::InPlaceData<'_>> {
    //     let [discriminant, value @ ..] = data;
    //     Ok(InPlaceOption {
    //         discriminant,
    //         value,
    //     })
    // }
    //
    // fn read_static(data: &mut [u8; Self::DATA_SIZE]) -> CruiserResult<Self::InPlaceData<'_>> {
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
