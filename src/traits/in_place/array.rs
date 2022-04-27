use crate::in_place::{InPlace, InPlaceCreate, InPlaceRead, InPlaceWrite};
use crate::on_chain_size::OnChainSize;
use crate::util::{
    assert_data_len, Advance, MappableRef, MappableRefMut, TryMappableRef, TryMappableRefMut,
};
use crate::CruiserResult;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// In-place access to arrays
#[derive(Debug)]
pub struct InPlaceArray<'a, T, A, const N: usize> {
    data: A,
    phantom_t: PhantomData<fn() -> &'a T>,
}

impl<'a, T, A, const N: usize> InPlaceArray<'a, T, A, N> {
    /// Returns an iterator over all arguments in the array with args passed
    pub fn all_with_args<Arg>(
        &self,
        args: [Arg; N],
    ) -> impl Iterator<Item = CruiserResult<T::Access<'_, &'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead<Arg> + OnChainSize,
    {
        let mut data = &*self.data;
        args.into_iter()
            .map(move |arg| T::read_with_arg(data.try_advance(T::ON_CHAIN_SIZE)?, arg))
    }

    /// An iterator over all elements of the array
    pub fn all(&self) -> impl Iterator<Item = CruiserResult<T::Access<'_, &'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead + OnChainSize,
    {
        self.all_with_args([(); N])
    }

    /// An iterator over all the elements mutably with args passed in
    pub fn all_with_args_mut<Arg>(
        &mut self,
        args: [Arg; N],
    ) -> impl Iterator<Item = CruiserResult<T::AccessMut<'_, &'_ mut [u8]>>>
    where
        A: DerefMut<Target = [u8]>,
        T: InPlaceWrite<Arg> + OnChainSize,
    {
        let mut data = &mut *self.data;
        args.into_iter()
            .map(move |arg| T::write_with_arg(data.try_advance(T::ON_CHAIN_SIZE)?, arg))
    }

    /// An iterator over all the elements mutably
    pub fn all_mut(&mut self) -> impl Iterator<Item = CruiserResult<T::AccessMut<'_, &'_ mut [u8]>>>
    where
        A: DerefMut<Target = [u8]>,
        T: InPlaceWrite + OnChainSize,
    {
        self.all_with_args_mut([(); N])
    }

    /// Gets an item in the array with arg
    pub fn get_with_arg<Arg>(
        &self,
        index: usize,
        arg: Arg,
    ) -> CruiserResult<Option<T::Access<'_, &'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead<Arg> + OnChainSize,
    {
        if index < N {
            let data = &self.data[T::ON_CHAIN_SIZE * index..][..T::ON_CHAIN_SIZE];
            Ok(Some(T::read_with_arg(data, arg)?))
        } else {
            Ok(None)
        }
    }

    /// Gets an item in the array
    pub fn get(&self, index: usize) -> CruiserResult<Option<T::Access<'_, &'_ [u8]>>>
    where
        A: Deref<Target = [u8]>,
        T: InPlaceRead + OnChainSize,
    {
        self.get_with_arg(index, ())
    }
}

impl<T, const N: usize> const InPlace for [T; N] {
    type Access<'a, A>
    where
        Self: 'a,
        A: 'a + MappableRef + TryMappableRef,
    = InPlaceArray<'a, T, A, N>;
}

impl<T, Arg, const N: usize> InPlaceCreate<[Arg; N]> for [T; N]
where
    T: OnChainSize + InPlaceCreate<Arg>,
{
    fn create_with_arg<A>(mut data: A, arg: [Arg; N]) -> CruiserResult
    where
        A: DerefMut<Target = [u8]>,
    {
        let element_length = T::ON_CHAIN_SIZE;
        let mut data = &mut *data;
        for arg in arg {
            T::create_with_arg(data.try_advance(element_length)?, arg)?;
        }
        Ok(())
    }
}

impl<T, const N: usize> InPlaceCreate for [T; N]
where
    T: OnChainSize + InPlaceCreate,
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
    T: OnChainSize,
{
    fn read_with_arg<'a, A>(data: A, _arg: ()) -> CruiserResult<Self::Access<'a, A>>
    where
        Self: 'a,
        A: 'a + Deref<Target = [u8]> + MappableRef + TryMappableRef,
    {
        assert_data_len(data.len(), N * T::ON_CHAIN_SIZE)?;
        Ok(InPlaceArray {
            data,
            phantom_t: PhantomData,
        })
    }
}

impl<T, const N: usize> InPlaceWrite for [T; N]
where
    T: OnChainSize,
{
    fn write_with_arg<'a, A>(data: A, _arg: ()) -> CruiserResult<Self::AccessMut<'a, A>>
    where
        Self: 'a,
        A: 'a
            + DerefMut<Target = [u8]>
            + MappableRef
            + TryMappableRef
            + MappableRefMut
            + TryMappableRefMut,
    {
        assert_data_len(data.len(), N * T::ON_CHAIN_SIZE)?;
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
