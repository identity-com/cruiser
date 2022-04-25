use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::on_chain_size::OnChainStaticSize;
use crate::CruiserResult;
use std::ops::{Deref, DerefMut};

impl InPlace for () {
    type Access<A> = ();
}
impl<'a> InPlaceCreate for () {
    fn create_with_arg<A>(_data: A, _arg: ()) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        Ok(())
    }
}
impl InPlaceRead for () {
    fn read_with_arg<A>(_data: A, _arg: ()) -> CruiserResult<Self::Access<A>>
    where
        A: Deref<Target = [u8]>,
    {
        Ok(())
    }
}
impl<'a> InPlaceWrite for () {
    fn write_with_arg<A>(_data: A, _arg: ()) -> CruiserResult<Self::Access<A>>
    where
        A: DerefMut<Target = [u8]>,
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
    fn read<A>(data: A) -> CruiserResult<Self::Access<A>>
    where
        A: Deref<Target = [u8]>,
    {
        Self::read_with_arg(data, ())
    }
}
impl<T> InPlaceUnitRead for T where T: InPlaceRead {}

/// In-place account data write access with no arg, auto derived
pub trait InPlaceUnitWrite: InPlaceWrite {
    /// Writes the access type to data
    fn write<A>(data: A) -> CruiserResult<Self::Access<A>>
    where
        A: DerefMut<Target = [u8]>,
    {
        Self::write_with_arg(data, ())
    }
}
impl<T> InPlaceUnitWrite for T where T: InPlaceWrite {}

/// In-place full access with no arg, auto derived
pub trait InPlaceUnit: InPlaceUnitCreate + InPlaceUnitRead {}
impl<T> InPlaceUnit for T where T: InPlaceUnitCreate + InPlaceUnitRead {}

impl<T1, T2> InPlace for (T1, T2)
where
    T1: InPlace,
    T2: InPlace,
{
    type Access<A> = (T1::Access<A>, T2::Access<A>);
}
impl<T1, T2> InPlaceCreate for (T1, T2)
where
    T1: InPlaceCreate + OnChainStaticSize,
    T2: InPlaceCreate,
{
    fn create_with_arg<A>(data: A, arg: ()) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        Self::create_with_arg(data, (arg, arg))
    }
}
impl<T1, T2, C1, C2> InPlaceCreate<(C1, C2)> for (T1, T2)
where
    T1: InPlaceCreate<C1> + OnChainStaticSize,
    T2: InPlaceCreate<C2>,
{
    fn create_with_arg<A: DerefMut<Target = [u8]>>(data: A, arg: (C1, C2)) -> CruiserResult {
        let (arg1, arg2) = arg;
        let (data1, data2) = data.split_at_mut(T1::on_chain_static_size());
        T1::create_with_arg(data1, arg1)?;
        T2::create_with_arg(data2, arg2)?;
        Ok(())
    }
}
impl<T1, T2> InPlaceRead for (T1, T2)
where
    T1: InPlaceRead + OnChainStaticSize,
    T2: InPlaceRead,
{
    fn read_with_arg<A: Deref<Target = [u8]>>(data: A, arg: ()) -> CruiserResult<Self::Access<A>> {
        Self::read_with_arg(data, (arg, arg))
    }
}
impl<T1, T2, R1, R2> InPlaceRead<(R1, R2)> for (T1, T2)
where
    T1: InPlaceRead<R1> + OnChainStaticSize,
    T2: InPlaceRead<R2>,
{
    fn read_with_arg<A: Deref<Target = [u8]>>(
        data: A,
        arg: (R1, R2),
    ) -> CruiserResult<Self::Access<A>> {
        let (arg1, arg2) = arg;
        let (data1, data2) = data.split_at(T1::on_chain_static_size());
        Ok((
            T1::read_with_arg(data1, arg1)?,
            T2::read_with_arg(data2, arg2)?,
        ))
    }
}
impl<T1, T2> InPlaceWrite for (T1, T2)
where
    T1: InPlaceWrite + OnChainStaticSize,
    T2: InPlaceWrite,
{
    fn write_with_arg<A: DerefMut<Target = [u8]>>(
        data: A,
        arg: (),
    ) -> CruiserResult<Self::Access<A>> {
        Self::write_with_arg(data, (arg, arg))
    }
}
impl<T1, T2, W1, W2> InPlaceWrite<(W1, W2)> for (T1, T2)
where
    T1: InPlaceWrite<W1> + OnChainStaticSize,
    T2: InPlaceWrite<W2>,
{
    fn write_with_arg<A: DerefMut<Target = [u8]>>(
        data: A,
        arg: (W1, W2),
    ) -> CruiserResult<Self::Access<A>> {
        let (arg1, arg2) = arg;
        let (data1, data2) = data.split_at_mut(T1::on_chain_static_size());
        Ok((
            T1::write_with_arg(data1, arg1)?,
            T2::write_with_arg(data2, arg2)?,
        ))
    }
}

impl<T1, T2, T3> InPlace for (T1, T2, T3)
where
    T1: InPlace,
    T2: InPlace,
    T3: InPlace,
{
    type Access<A> = (T1::Access<A>, T2::Access<A>, T3::Access<A>);
}
impl<T1, T2, T3> InPlaceCreate for (T1, T2, T3)
where
    T1: InPlaceCreate + OnChainStaticSize,
    T2: InPlaceCreate + OnChainStaticSize,
    T3: InPlaceCreate,
{
    fn create_with_arg<A>(data: A, arg: ()) -> CruiserResult {
        Self::create_with_arg(data, (arg, arg, arg))
    }
}
impl<T1, T2, T3, C1, C2, C3> InPlaceCreate<(C1, C2, C3)> for (T1, T2, T3)
where
    T1: InPlaceCreate<C1> + OnChainStaticSize,
    T2: InPlaceCreate<C2> + OnChainStaticSize,
    T3: InPlaceCreate<C3>,
{
    fn create_with_arg<A>(mut data: A, arg: (C1, C2, C3)) -> CruiserResult {
        let (arg1, arg2, arg3) = arg;
        T1::create_with_arg(data.try_advance(T1::on_chain_static_size())?, arg1)?;
        T2::create_with_arg(data.try_advance(T2::on_chain_static_size())?, arg2)?;
        T3::create_with_arg(data, arg3)?;
        Ok(())
    }
}
impl<T1, T2, T3> InPlaceRead for (T1, T2, T3)
where
    T1: InPlaceRead + OnChainStaticSize,
    T2: InPlaceRead + OnChainStaticSize,
    T3: InPlaceRead,
{
    fn read_with_arg<A: Deref<Target = [u8]>>(data: A, arg: ()) -> CruiserResult<Self::Access<A>> {
        Self::read_with_arg(data, (arg, arg, arg))
    }
}
impl<T1, T2, T3, R1, R2, R3> InPlaceRead<(R1, R2, R3)> for (T1, T2, T3)
where
    T1: InPlaceRead<R1> + OnChainStaticSize,
    T2: InPlaceRead<R2> + OnChainStaticSize,
    T3: InPlaceRead<R3>,
{
    fn read_with_arg<A: Deref<Target = [u8]>>(
        mut data: A,
        arg: (R1, R2, R3),
    ) -> CruiserResult<Self::Access<A>> {
        let (arg1, arg2, arg3) = arg;
        Ok((
            T1::read_with_arg(data.try_advance(T1::on_chain_static_size())?, arg1)?,
            T2::read_with_arg(data.try_advance(T2::on_chain_static_size())?, arg2)?,
            T3::read_with_arg(data, arg3)?,
        ))
    }
}
impl<T1, T2, T3> InPlaceWrite for (T1, T2, T3)
where
    T1: InPlaceWrite + OnChainStaticSize,
    T2: InPlaceWrite + OnChainStaticSize,
    T3: InPlaceWrite,
{
    fn write_with_arg<A: DerefMut<Target = [u8]>>(
        data: A,
        arg: (),
    ) -> CruiserResult<Self::Access<A>> {
        Self::write_with_arg(data, (arg, arg, arg))
    }
}
impl<T1, T2, T3, W1, W2, W3> InPlaceWrite<(W1, W2, W3)> for (T1, T2, T3)
where
    T1: InPlaceWrite<W1> + OnChainStaticSize,
    T2: InPlaceWrite<W2> + OnChainStaticSize,
    T3: InPlaceWrite<W3>,
{
    fn write_with_arg<A: DerefMut<Target = [u8]>>(
        mut data: A,
        arg: (W1, W2, W3),
    ) -> CruiserResult<Self::Access<A>> {
        let (arg1, arg2, arg3) = arg;
        Ok((
            T1::write_with_arg(data.try_advance(T1::on_chain_static_size())?, arg1)?,
            T2::write_with_arg(data.try_advance(T2::on_chain_static_size())?, arg2)?,
            T3::write_with_arg(data, arg3)?,
        ))
    }
}
