use crate::in_place::{InPlace, InPlaceGetData, InPlaceRead, InPlaceWrite};
use crate::CruiserResult;
pub use cruiser_derive::get_properties;
use std::ops::{Deref, DerefMut};

/// In-place data that has a properties accessor
pub trait InPlaceProperties: InPlace {
    /// The properties of this data
    type Properties: InPlacePropertiesList;
}

/// A list of properties on an in-place item
pub trait InPlacePropertiesList: Copy {
    /// The index of the property, must be unique
    fn index(self) -> usize;
    /// The data offset of the property
    fn offset(self) -> usize;
    /// The size of the property
    fn size(self) -> Option<usize>;
}
/// An individual property, indexed by `PROP`
pub trait InPlaceProperty<const PROP: usize>: InPlaceGetData {
    /// The type of this property
    type Property: InPlace;
}
/// Reads a given property
pub trait InPlaceReadProperty<const PROP: usize>: InPlaceProperty<PROP> {
    /// Reads the property
    fn read_property_with_arg<R>(
        &self,
        arg: R,
    ) -> CruiserResult<<Self::Property as InPlace>::Access<&'_ [u8]>>
    where
        Self::Property: InPlaceRead<R>,
        Self::Accessor: Deref<Target = [u8]>;
}
/// Writes a given property
pub trait InPlaceWriteProperty<const PROP: usize>: InPlaceProperty<PROP> {
    /// Writes the property
    fn write_property_with_arg<W>(
        &mut self,
        arg: W,
    ) -> CruiserResult<<Self::Property as InPlace>::Access<&'_ mut [u8]>>
    where
        Self::Property: InPlaceWrite<W>,
        Self::Accessor: DerefMut<Target = [u8]>;
}

/// Calculates offsets for properties. Will panic if `properties` is not sorted
pub const fn calc_property_offsets<T, const N: usize>(
    properties: [T; N],
) -> [(usize, Option<usize>); N]
where
    T: ~const InPlacePropertiesList,
{
    let mut out = [(0, None); N];
    let mut last_offset = Some(0);
    let mut index = 0;
    while index < N {
        let property = properties[index];
        let offset = property.offset();
        let size = property.size();
        match last_offset {
            Some(last_offset_val) => {
                assert!(offset >= last_offset_val, "Properties are not in order");
                out[index] = (offset, size);
                last_offset = match size {
                    Some(size) => Some(offset + size),
                    None => None,
                };
            }
            None => panic!("Unsized is not at end!"),
        }
        index += 1;
    }
    out
}
