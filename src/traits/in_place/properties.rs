use crate::in_place::InPlace;
pub use cruiser_derive::{get_properties, get_properties_mut};

/// In-place data that has a properties accessor
pub trait InPlaceProperties: InPlace {
    /// The properties of this data
    type Properties: InPlacePropertiesList;
}

/// Allows raw data access for properties
pub trait InPlaceRawDataAccess {
    /// Get the raw data
    fn get_raw_data(&self) -> &[u8];
}

/// Allows raw data access for properties mutably
pub trait InPlaceRawDataAccessMut: InPlaceRawDataAccess {
    /// Get the raw data mutably
    fn get_raw_data_mut(&mut self) -> &mut [u8];
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
pub trait InPlaceProperty<const PROP: usize> {
    /// The type of this property
    type Property: InPlace;
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
                out[index] = (offset - last_offset_val, size);
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

#[cfg(test)]
mod test {
    use crate::in_place::InPlacePropertiesList;
    use cruiser::in_place::calc_property_offsets;

    #[derive(Copy, Clone)]
    enum PropertiesList {
        A,
        B,
        C,
    }
    impl const InPlacePropertiesList for PropertiesList {
        fn index(self) -> usize {
            match self {
                PropertiesList::A => 0,
                PropertiesList::B => 1,
                PropertiesList::C => 2,
            }
        }

        fn offset(self) -> usize {
            match self {
                PropertiesList::A => 0,
                PropertiesList::B => 10,
                PropertiesList::C => 12,
            }
        }

        fn size(self) -> Option<usize> {
            match self {
                PropertiesList::A => Some(10),
                PropertiesList::B => Some(2),
                PropertiesList::C => Some(5),
            }
        }
    }

    #[test]
    fn calc_property_offsets_test() {
        const OFFSETS: [(usize, Option<usize>); 2] =
            calc_property_offsets([PropertiesList::A, PropertiesList::C]);
        const OFFSETS2: [(usize, Option<usize>); 3] =
            calc_property_offsets([PropertiesList::A, PropertiesList::B, PropertiesList::C]);
        const OFFSETS3: [(usize, Option<usize>); 1] = calc_property_offsets([PropertiesList::C]);

        assert_eq!(OFFSETS[0], (0, Some(10)));
        assert_eq!(OFFSETS[1], (2, Some(5)));

        assert_eq!(OFFSETS2[0], (0, Some(10)));
        assert_eq!(OFFSETS2[1], (0, Some(2)));
        assert_eq!(OFFSETS2[2], (0, Some(5)));

        assert_eq!(OFFSETS3[0], (12, Some(5)));
    }
}
