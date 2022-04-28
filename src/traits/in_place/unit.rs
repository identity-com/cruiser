use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::util::{MappableRefMut, TryMappableRefMut};
use crate::CruiserResult;
use cruiser::util::{MappableRef, TryMappableRef};
use std::ops::{Deref, DerefMut};

impl InPlace for () {
    type Access<'a, A>
    where
        A: 'a + MappableRef + TryMappableRef,
    = ();
}

impl<'a> InPlaceCreate for () {
    #[inline]
    fn create_with_arg<A>(_data: A, _arg: ()) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        Ok(())
    }
}

impl InPlaceRead for () {
    fn read_with_arg<'a, A>(_data: A, _arg: ()) -> CruiserResult<Self::Access<'a, A>>
    where
        A: 'a + Deref<Target = [u8]> + MappableRef + TryMappableRef,
        Self: 'a,
    {
        Ok(())
    }
}

impl InPlaceWrite for () {
    fn write_with_arg<'a, A>(_data: A, _arg: ()) -> CruiserResult<Self::AccessMut<'a, A>>
    where
        A: 'a
            + DerefMut<Target = [u8]>
            + TryMappableRef
            + MappableRef
            + MappableRefMut
            + TryMappableRefMut,
        Self: 'a,
    {
        Ok(())
    }
}

/// In-place account data create access with no arg, auto derived
pub trait InPlaceUnitCreate: InPlaceCreate {
    /// Create a new instance of `Self::Access` with no argument
    fn create<A>(data: A) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        Self::create_with_arg(data, ())
    }
}

impl<T> InPlaceUnitCreate for T where T: InPlaceCreate {}

/// In-place account data read access with no arg, auto derived
pub trait InPlaceUnitRead: InPlaceRead {
    /// Reads the access type from data
    fn read<'a, A>(data: A) -> CruiserResult<Self::Access<'a, A>>
    where
        A: Deref<Target = [u8]> + MappableRef + TryMappableRef,
    {
        Self::read_with_arg(data, ())
    }
}

impl<T> InPlaceUnitRead for T where T: InPlaceRead {}

/// In-place account data write access with no arg, auto derived
pub trait InPlaceUnitWrite: InPlaceWrite {
    /// Writes the access type to data
    fn write<'a, A>(data: A) -> CruiserResult<Self::AccessMut<'a, A>>
    where
        A: DerefMut<Target = [u8]>
            + MappableRef
            + TryMappableRef
            + MappableRefMut
            + TryMappableRefMut,
    {
        Self::write_with_arg(data, ())
    }
}

impl<T> InPlaceUnitWrite for T where T: InPlaceWrite {}

/// In-place full access with no arg, auto derived
pub trait InPlaceUnit: InPlaceUnitCreate + InPlaceUnitRead + InPlaceUnitWrite {}

impl<T> InPlaceUnit for T where T: InPlaceUnitCreate + InPlaceUnitRead + InPlaceUnitWrite {}
