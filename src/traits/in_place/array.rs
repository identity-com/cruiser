use crate::in_place::{InPlace, InPlaceCreate, InPlaceGetData, InPlaceRead, InPlaceWrite};
use crate::on_chain_size::OnChainStaticSize;
use crate::util::{assert_data_len, Advance};
use crate::CruiserResult;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// In-place access to arrays
#[derive(Debug)]
pub struct InPlaceArray<T, A, const N: usize> {
    data: A,
    phantom_t: PhantomData<fn() -> T>,
}
impl<T, A, const N: usize> InPlaceArray<T, A, N> {
    /// Returns an iterator over all arguments in the array with args passed
    pub fn all_with_args<Arg>(
        &self,
        args: [Arg; N],
    ) -> impl Iterator<Item = CruiserResult<T::Access<&'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead<Arg> + OnChainStaticSize,
    {
        let mut data = &*self.data;
        args.into_iter()
            .map(move |arg| T::read_with_arg(data.try_advance(T::on_chain_static_size())?, arg))
    }

    /// An iterator over all elements of the array
    pub fn all(&self) -> impl Iterator<Item = CruiserResult<T::Access<&'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead + OnChainStaticSize,
    {
        self.all_with_args([(); N])
    }

    /// An iterator over all the elements mutably with args passed in
    pub fn all_with_args_mut<Arg>(
        &mut self,
        args: [Arg; N],
    ) -> impl Iterator<Item = CruiserResult<T::Access<&'_ mut [u8]>>>
    where
        A: DerefMut<Target = [u8]>,
        T: InPlaceWrite<Arg> + OnChainStaticSize,
    {
        let mut data = &mut *self.data;
        args.into_iter()
            .map(move |arg| T::write_with_arg(data.try_advance(T::on_chain_static_size())?, arg))
    }

    /// An iterator over all the elements mutably
    pub fn all_mut(&mut self) -> impl Iterator<Item = CruiserResult<T::Access<&'_ mut [u8]>>>
    where
        A: DerefMut<Target = [u8]>,
        T: InPlaceWrite + OnChainStaticSize,
    {
        self.all_with_args_mut([(); N])
    }

    /// Gets an item in the array with arg
    pub fn get_with_arg<Arg>(
        &self,
        index: usize,
        arg: Arg,
    ) -> CruiserResult<Option<T::Access<&'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead<Arg> + OnChainStaticSize,
    {
        if index < N {
            let data = &self.data[T::on_chain_static_size() * index..][..T::on_chain_static_size()];
            Ok(Some(T::read_with_arg(data, arg)?))
        } else {
            Ok(None)
        }
    }

    /// Gets an item in the array
    pub fn get(&self, index: usize) -> CruiserResult<Option<T::Access<&'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead + OnChainStaticSize,
    {
        self.get_with_arg(index, ())
    }
}
impl<T, A, const N: usize> const InPlaceGetData for InPlaceArray<T, A, N> {
    type Accessor = A;

    fn get_raw_data(&self) -> &[u8]
    where
        Self::Accessor: ~const Deref<Target = [u8]>,
    {
        &*self.data
    }

    fn get_raw_data_mut(&mut self) -> &mut [u8]
    where
        Self::Accessor: ~const DerefMut<Target = [u8]>,
    {
        &mut self.data
    }
}
impl<T, const N: usize> const InPlace for [T; N] {
    type Access<A> = InPlaceArray<T, A, N>;
}
impl<T, Arg, const N: usize> InPlaceCreate<[Arg; N]> for [T; N]
where
    T: OnChainStaticSize + InPlaceCreate<Arg>,
{
    fn create_with_arg<A>(mut data: A, arg: [Arg; N]) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        let element_length = T::on_chain_static_size();
        let mut data = &mut *data;
        for arg in arg {
            T::create_with_arg(data.try_advance(element_length)?, arg)?;
        }
        Ok(())
    }
}
impl<T, const N: usize> InPlaceCreate for [T; N]
where
    T: OnChainStaticSize + InPlaceCreate,
{
    fn create_with_arg<A>(data: A, arg: ()) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        Self::create_with_arg(data, [arg; N])
    }
}
impl<T, const N: usize> InPlaceRead for [T; N]
where
    T: OnChainStaticSize,
{
    fn read_with_arg<A>(data: A, _arg: ()) -> CruiserResult<Self::Access<A>>
    where
        A: Deref<Target = [u8]>,
    {
        assert_data_len(data.len(), N * T::on_chain_static_size())?;
        Ok(InPlaceArray {
            data,
            phantom_t: PhantomData,
        })
    }
}
impl<T, const N: usize> InPlaceWrite for [T; N]
where
    T: OnChainStaticSize,
{
    fn write_with_arg<A>(data: A, _arg: ()) -> CruiserResult<Self::Access<A>>
    where
        A: DerefMut<Target = [u8]>,
    {
        assert_data_len(data.len(), N * T::on_chain_static_size())?;
        Ok(InPlaceArray {
            data,
            phantom_t: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::account_types::system_program::SystemProgram;
    use crate::in_place::{InPlaceCreate, InPlaceRead, InPlaceWrite};
    use crate::program::ProgramKey;
    use crate::util::VoidCollect;
    use crate::{CruiserResult, Pubkey};
    use rand::{thread_rng, Rng};

    #[test]
    fn array_test() -> CruiserResult {
        let mut rng = thread_rng();
        let values = (0..1024)
            .map(|_| rng.gen::<[u8; 32]>())
            .map(Pubkey::new_from_array)
            .collect::<Vec<_>>();
        let mut data = vec![0u8; 1024 * 32];

        <[Pubkey; 1024]>::create_with_arg(data.as_mut_slice(), ())?;
        let in_place = <[Pubkey; 1024]>::read_with_arg(data.as_slice(), ())?;
        for value in in_place.all() {
            assert_eq!(SystemProgram::<()>::KEY, *value?);
        }
        let mut in_place = <[Pubkey; 1024]>::write_with_arg(data.as_mut_slice(), ())?;
        in_place
            .all_mut()
            .zip(values.iter())
            .map(|(write, value)| write.map(|mut v| *v = *value))
            .collect::<Result<VoidCollect, _>>()?;

        for (i, value) in values.iter().enumerate() {
            assert_eq!(*in_place.get_with_arg(i, ())?.unwrap(), *value);
        }
        Ok(())
    }
}
