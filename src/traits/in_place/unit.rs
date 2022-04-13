use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::CruiserResult;

impl<'a> InPlace<'a> for () {
    type Access = ();
    type AccessMut = ();
}
impl<'a> InPlaceCreate<'a, ()> for () {
    fn create_with_arg(_data: &mut [u8], _arg: ()) -> CruiserResult {
        Ok(())
    }
}
impl<'a> InPlaceRead<'a, ()> for () {
    fn read_with_arg(_data: &'a [u8], _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}
impl<'a> InPlaceWrite<'a, ()> for () {
    fn write_with_arg(_data: &'a mut [u8], _arg: ()) -> CruiserResult {
        Ok(())
    }
}

/// In-place account data create access with no arg, auto derived
pub trait InPlaceUnitCreate<'a>: InPlaceCreate<'a, ()> {
    /// Create a new instance of `Self::Access` with no argument
    fn create(data: &mut [u8]) -> CruiserResult {
        Self::create_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitCreate<'a> for T where T: InPlaceCreate<'a, ()> {}

/// In-place account data read access with no arg, auto derived
pub trait InPlaceUnitRead<'a>: InPlaceRead<'a, ()> {
    /// Reads the access type from data
    fn read(data: &'a [u8]) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitRead<'a> for T where T: InPlaceRead<'a, ()> {}

/// In-place account data write access with no arg, auto derived
pub trait InPlaceUnitWrite<'a>: InPlaceWrite<'a, ()> {
    /// Writes the access type to data
    fn write(data: &'a mut [u8]) -> CruiserResult<Self::AccessMut> {
        Self::write_with_arg(data, ())
    }
}
impl<'a, T> InPlaceUnitWrite<'a> for T where T: InPlaceWrite<'a, ()> {}

/// In-place full access with no arg, auto derived
pub trait InPlaceUnit<'a>: InPlaceUnitCreate<'a> + InPlaceUnitRead<'a> {}
impl<'a, T> InPlaceUnit<'a> for T where T: InPlaceUnitCreate<'a> + InPlaceUnitRead<'a> {}
