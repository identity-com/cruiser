use crate::in_place::{
    InPlace, InPlaceCreate, InPlaceGet, InPlaceRead, InPlaceSet, InPlaceWrite, InitToZero,
};
use crate::on_chain_size::{OnChainSize, OnChainStaticSize};
use crate::util::Advance;
use crate::{CruiserResult, GenericError};
use std::marker::PhantomData;

/// A vector with a static max size
#[derive(Debug)]
pub struct StaticSizeVec<T, L, const N: usize>(Vec<T>, PhantomData<fn() -> (T, L)>);
impl<T, L, const N: usize> const OnChainSize<()> for StaticSizeVec<T, L, N>
where
    T: ~const OnChainStaticSize,
    L: ~const OnChainStaticSize,
{
    fn on_chain_max_size(_arg: ()) -> usize {
        L::on_chain_static_size() + T::on_chain_static_size() * N
    }
}
/// The [`InPlace::Access`] and [`InPlace::AccessMut`] for [`StaticSizeVec`]
#[derive(Debug)]
pub struct StaticSizeVecAccess<T, L, D, const N: usize> {
    length: L,
    data: D,
    phantom_t: PhantomData<fn() -> T>,
}
impl<T, L, D, const N: usize> StaticSizeVecAccess<T, L, D, N> {
    /// Gets an item in the vector with a read arg
    pub fn get_with_arg<'b, R>(&'b self, index: usize, arg: R) -> CruiserResult<Option<T::Access>>
    where
        T: OnChainStaticSize + InPlaceRead<'b, R>,
        L: InPlaceGet<usize>,
        D: AsRef<[u8]>,
    {
        let length = self.length.get()?;
        if index < length {
            let mut data = self.data.as_ref();
            data.advance(index * T::on_chain_static_size());
            T::read_with_arg(data.try_advance(T::on_chain_static_size())?, arg).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Gets an item in the vector
    pub fn get<'b>(&'b self, index: usize) -> CruiserResult<Option<T::Access>>
    where
        T: OnChainStaticSize + InPlaceRead<'b, ()>,
        L: InPlaceGet<usize>,
        D: AsRef<[u8]>,
    {
        self.get_with_arg(index, ())
    }

    /// Gets an item in the vector with a write arg
    pub fn get_with_arg_mut<'b, W>(
        &'b mut self,
        index: usize,
        arg: W,
    ) -> CruiserResult<Option<T::AccessMut>>
    where
        T: OnChainStaticSize + InPlaceWrite<'b, W>,
        L: InPlaceGet<usize>,
        D: AsMut<[u8]>,
    {
        let length = self.length.get()?;
        if index < length {
            let mut data = self.data.as_mut();
            data.try_advance(index * T::on_chain_static_size())?;
            T::write_with_arg(data.try_advance(T::on_chain_static_size())?, arg).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Gets an item in the vector mutably
    pub fn get_mut<'b>(&'b mut self, index: usize) -> CruiserResult<Option<T::AccessMut>>
    where
        T: OnChainStaticSize + InPlaceWrite<'b, ()>,
        L: InPlaceGet<usize>,
        D: AsMut<[u8]>,
    {
        self.get_with_arg_mut(index, ())
    }

    /// Pushes an item to the list
    pub fn push_with_arg<'b, C>(&'b mut self, arg: C) -> CruiserResult<()>
    where
        T: OnChainStaticSize + InPlaceCreate<'b, C>,
        L: InPlaceGet<usize> + InPlaceSet<usize>,
        D: AsMut<[u8]>,
    {
        let length = self.length.get()?;
        if length < N {
            let mut data = self.data.as_mut();
            data.try_advance(length * T::on_chain_static_size())?;
            T::create_with_arg(data.try_advance(T::on_chain_static_size())?, arg)?;
            self.length.set(length + 1)?;
            Ok(())
        } else {
            Err(GenericError::Custom {
                error: format!("StaticSizeVec is full, length: {}", N),
            }
            .into())
        }
    }

    /// Pushes an item to the list
    pub fn push<'b>(&'b mut self) -> CruiserResult<()>
    where
        T: OnChainStaticSize + InPlaceCreate<'b, ()>,
        L: InPlaceGet<usize> + InPlaceSet<usize>,
        D: AsMut<[u8]>,
    {
        self.push_with_arg(())
    }
}

impl<'a, T, L, const N: usize> const InPlace<'a> for StaticSizeVec<T, L, N>
where
    L: InPlace<'a>,
{
    type Access = StaticSizeVecAccess<T, L::Access, &'a [u8], N>;
    type AccessMut = StaticSizeVecAccess<T, L::AccessMut, &'a mut [u8], N>;
}
impl<'a, T, A, L, const N: usize> InPlaceCreate<'a, A> for StaticSizeVec<T, L, N>
where
    L: InPlaceCreate<'a, A> + OnChainStaticSize + InitToZero,
{
    fn create_with_arg(mut data: &mut [u8], arg: A) -> CruiserResult {
        L::create_with_arg(data.try_advance(L::on_chain_static_size())?, arg)?;
        Ok(())
    }
}
impl<'a, T, R, L, const N: usize> InPlaceRead<'a, R> for StaticSizeVec<T, L, N>
where
    T: OnChainStaticSize,
    L: InPlaceRead<'a, R> + OnChainStaticSize + InitToZero,
{
    fn read_with_arg(mut data: &'a [u8], arg: R) -> CruiserResult<Self::Access> {
        let length = L::read_with_arg(data.try_advance(L::on_chain_static_size())?, arg)?;
        let data = data.try_advance(T::on_chain_static_size() * N)?;
        Ok(StaticSizeVecAccess {
            length,
            data,
            phantom_t: PhantomData,
        })
    }
}
impl<'a, T, W, L, const N: usize> InPlaceWrite<'a, W> for StaticSizeVec<T, L, N>
where
    T: OnChainStaticSize,
    L: InPlaceWrite<'a, W> + OnChainStaticSize + InitToZero,
{
    fn write_with_arg(mut data: &'a mut [u8], arg: W) -> CruiserResult<Self::AccessMut> {
        let length = L::write_with_arg(data.try_advance(L::on_chain_static_size())?, arg)?;
        let data = data.try_advance(T::on_chain_static_size() * N)?;
        Ok(StaticSizeVecAccess {
            length,
            data,
            phantom_t: PhantomData,
        })
    }
}
