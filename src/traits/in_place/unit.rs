use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::on_chain_size::OnChainStaticSize;
use crate::util::Advance;
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

impl<'a, T1, T2> InPlace<'a> for (T1, T2)
where
    T1: InPlace<'a>,
    T2: InPlace<'a>,
{
    type Access = (T1::Access, T2::Access);
    type AccessMut = (T1::AccessMut, T2::AccessMut);
}
impl<'a, T1, T2> InPlaceCreate<'a, ()> for (T1, T2)
where
    T1: InPlaceCreate<'a, ()> + OnChainStaticSize,
    T2: InPlaceCreate<'a, ()>,
{
    fn create_with_arg(data: &mut [u8], arg: ()) -> CruiserResult {
        Self::create_with_arg(data, (arg, arg))
    }
}
impl<'a, T1, T2, C1, C2> InPlaceCreate<'a, (C1, C2)> for (T1, T2)
where
    T1: InPlaceCreate<'a, C1> + OnChainStaticSize,
    T2: InPlaceCreate<'a, C2>,
{
    fn create_with_arg(data: &mut [u8], arg: (C1, C2)) -> CruiserResult {
        let (arg1, arg2) = arg;
        let (data1, data2) = data.split_at_mut(T1::on_chain_static_size());
        T1::create_with_arg(data1, arg1)?;
        T2::create_with_arg(data2, arg2)?;
        Ok(())
    }
}
impl<'a, T1, T2> InPlaceRead<'a, ()> for (T1, T2)
where
    T1: InPlaceRead<'a, ()> + OnChainStaticSize,
    T2: InPlaceRead<'a, ()>,
{
    fn read_with_arg(data: &'a [u8], arg: ()) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, (arg, arg))
    }
}
impl<'a, T1, T2, R1, R2> InPlaceRead<'a, (R1, R2)> for (T1, T2)
where
    T1: InPlaceRead<'a, R1> + OnChainStaticSize,
    T2: InPlaceRead<'a, R2>,
{
    fn read_with_arg(data: &'a [u8], arg: (R1, R2)) -> CruiserResult<Self::Access> {
        let (arg1, arg2) = arg;
        let (data1, data2) = data.split_at(T1::on_chain_static_size());
        Ok((
            T1::read_with_arg(data1, arg1)?,
            T2::read_with_arg(data2, arg2)?,
        ))
    }
}
impl<'a, T1, T2> InPlaceWrite<'a, ()> for (T1, T2)
where
    T1: InPlaceWrite<'a, ()> + OnChainStaticSize,
    T2: InPlaceWrite<'a, ()>,
{
    fn write_with_arg(data: &'a mut [u8], arg: ()) -> CruiserResult<Self::AccessMut> {
        Self::write_with_arg(data, (arg, arg))
    }
}
impl<'a, T1, T2, W1, W2> InPlaceWrite<'a, (W1, W2)> for (T1, T2)
where
    T1: InPlaceWrite<'a, W1> + OnChainStaticSize,
    T2: InPlaceWrite<'a, W2>,
{
    fn write_with_arg(data: &'a mut [u8], arg: (W1, W2)) -> CruiserResult<Self::AccessMut> {
        let (arg1, arg2) = arg;
        let (data1, data2) = data.split_at_mut(T1::on_chain_static_size());
        Ok((
            T1::write_with_arg(data1, arg1)?,
            T2::write_with_arg(data2, arg2)?,
        ))
    }
}

impl<'a, T1, T2, T3> InPlace<'a> for (T1, T2, T3)
where
    T1: InPlace<'a>,
    T2: InPlace<'a>,
    T3: InPlace<'a>,
{
    type Access = (T1::Access, T2::Access, T3::Access);
    type AccessMut = (T1::AccessMut, T2::AccessMut, T3::AccessMut);
}
impl<'a, T1, T2, T3> InPlaceCreate<'a, ()> for (T1, T2, T3)
where
    T1: InPlaceCreate<'a, ()> + OnChainStaticSize,
    T2: InPlaceCreate<'a, ()> + OnChainStaticSize,
    T3: InPlaceCreate<'a, ()>,
{
    fn create_with_arg(data: &mut [u8], arg: ()) -> CruiserResult {
        Self::create_with_arg(data, (arg, arg, arg))
    }
}
impl<'a, T1, T2, T3, C1, C2, C3> InPlaceCreate<'a, (C1, C2, C3)> for (T1, T2, T3)
where
    T1: InPlaceCreate<'a, C1> + OnChainStaticSize,
    T2: InPlaceCreate<'a, C2> + OnChainStaticSize,
    T3: InPlaceCreate<'a, C3>,
{
    fn create_with_arg(mut data: &mut [u8], arg: (C1, C2, C3)) -> CruiserResult {
        let (arg1, arg2, arg3) = arg;
        T1::create_with_arg(data.try_advance(T1::on_chain_static_size())?, arg1)?;
        T2::create_with_arg(data.try_advance(T2::on_chain_static_size())?, arg2)?;
        T3::create_with_arg(data, arg3)?;
        Ok(())
    }
}
impl<'a, T1, T2, T3> InPlaceRead<'a, ()> for (T1, T2, T3)
where
    T1: InPlaceRead<'a, ()> + OnChainStaticSize,
    T2: InPlaceRead<'a, ()> + OnChainStaticSize,
    T3: InPlaceRead<'a, ()>,
{
    fn read_with_arg(data: &'a [u8], arg: ()) -> CruiserResult<Self::Access> {
        Self::read_with_arg(data, (arg, arg, arg))
    }
}
impl<'a, T1, T2, T3, R1, R2, R3> InPlaceRead<'a, (R1, R2, R3)> for (T1, T2, T3)
where
    T1: InPlaceRead<'a, R1> + OnChainStaticSize,
    T2: InPlaceRead<'a, R2> + OnChainStaticSize,
    T3: InPlaceRead<'a, R3>,
{
    fn read_with_arg(mut data: &'a [u8], arg: (R1, R2, R3)) -> CruiserResult<Self::Access> {
        let (arg1, arg2, arg3) = arg;
        Ok((
            T1::read_with_arg(data.try_advance(T1::on_chain_static_size())?, arg1)?,
            T2::read_with_arg(data.try_advance(T2::on_chain_static_size())?, arg2)?,
            T3::read_with_arg(data, arg3)?,
        ))
    }
}
impl<'a, T1, T2, T3> InPlaceWrite<'a, ()> for (T1, T2, T3)
where
    T1: InPlaceWrite<'a, ()> + OnChainStaticSize,
    T2: InPlaceWrite<'a, ()> + OnChainStaticSize,
    T3: InPlaceWrite<'a, ()>,
{
    fn write_with_arg(data: &'a mut [u8], arg: ()) -> CruiserResult<Self::AccessMut> {
        Self::write_with_arg(data, (arg, arg, arg))
    }
}
impl<'a, T1, T2, T3, W1, W2, W3> InPlaceWrite<'a, (W1, W2, W3)> for (T1, T2, T3)
where
    T1: InPlaceWrite<'a, W1> + OnChainStaticSize,
    T2: InPlaceWrite<'a, W2> + OnChainStaticSize,
    T3: InPlaceWrite<'a, W3>,
{
    fn write_with_arg(mut data: &'a mut [u8], arg: (W1, W2, W3)) -> CruiserResult<Self::AccessMut> {
        let (arg1, arg2, arg3) = arg;
        Ok((
            T1::write_with_arg(data.try_advance(T1::on_chain_static_size())?, arg1)?,
            T2::write_with_arg(data.try_advance(T2::on_chain_static_size())?, arg2)?,
            T3::write_with_arg(data, arg3)?,
        ))
    }
}
