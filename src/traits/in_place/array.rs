use crate::in_place::{
    InPlace, InPlaceCreate, InPlaceRead, InPlaceUnitRead, InPlaceUnitWrite, InPlaceWrite,
};
use crate::on_chain_size::OnChainStaticSize;
use crate::util::{range_bounds_to_range, Advance};
use crate::CruiserResult;
use std::marker::PhantomData;
use std::ops::RangeBounds;

/// In-place access to arrays
#[derive(Debug)]
pub struct InPlaceArray<T, D, const N: usize> {
    element_length: usize,
    data: D,
    phantom_t: PhantomData<fn() -> T>,
}
impl<T, D, const N: usize> InPlaceArray<T, D, N> {
    /// Gets an item in the array with a read arg
    pub fn get_with_arg<'b, R>(&'b self, index: usize, arg: R) -> CruiserResult<Option<T::Access>>
    where
        T: InPlaceRead<'b, R>,
        D: AsRef<[u8]>,
    {
        if index < N {
            Ok(Some(T::read_with_arg(
                &self.data.as_ref()[self.element_length * index..],
                arg,
            )?))
        } else {
            Ok(None)
        }
    }

    /// Gets an item in the array with a write arg
    pub fn get_with_arg_mut<'b, W>(
        &'b mut self,
        index: usize,
        arg: W,
    ) -> CruiserResult<Option<T::AccessMut>>
    where
        T: InPlaceWrite<'b, W>,
        D: AsMut<[u8]>,
    {
        if index < N {
            Ok(Some(T::write_with_arg(
                &mut self.data.as_mut()[self.element_length * index..],
                arg,
            )?))
        } else {
            Ok(None)
        }
    }

    /// Gets an item in the array
    pub fn get<'b>(&'b self, index: usize) -> CruiserResult<Option<T::Access>>
    where
        T: InPlaceUnitRead<'b>,
        D: AsRef<[u8]>,
    {
        self.get_with_arg(index, ())
    }

    /// Gets an item in the array mutably
    pub fn get_mut<'b>(&'b mut self, index: usize) -> CruiserResult<Option<T::AccessMut>>
    where
        T: InPlaceUnitWrite<'b>,
        D: AsMut<[u8]>,
    {
        self.get_with_arg_mut(index, ())
    }

    /// Gets an iterator over the array in the range cloning the arg
    // TODO: Replace with impl return when able to use `T::Access` in that return
    pub fn range_with_clone_arg<'b, R>(
        &'b self,
        range: impl RangeBounds<usize>,
        arg: R,
    ) -> Box<dyn Iterator<Item = CruiserResult<T::Access>> + 'b>
    where
        T: InPlaceRead<'b, R>,
        R: Clone + 'b,
        D: AsRef<[u8]>,
    {
        let (lower, upper) = range_bounds_to_range(range, 0, N);
        let mut data = self.data.as_ref();
        data.advance(self.element_length * lower);
        Box::new(
            (lower..upper)
                .map(move |_| T::read_with_arg(data.advance(self.element_length), arg.clone())),
        )
    }
    /// Gets an iterator over the array in the range cloning the arg
    // TODO: Replace with impl return when able to use `T::AccessMut` in that return
    pub fn range_with_clone_arg_mut<'b, W>(
        &'b mut self,
        range: impl RangeBounds<usize>,
        arg: W,
    ) -> Box<dyn Iterator<Item = CruiserResult<T::AccessMut>> + 'b>
    where
        T: InPlaceWrite<'b, W>,
        D: AsMut<[u8]>,
        W: Clone + 'b,
    {
        let (lower, upper) = range_bounds_to_range(range, 0, N);
        let mut data = self.data.as_mut();
        data.advance(self.element_length * lower);
        let element_length = self.element_length;
        Box::new(
            (lower..upper)
                .map(move |_| T::write_with_arg(data.advance(element_length), arg.clone())),
        )
    }
    /// Gets an iterator over the array in the range
    // TODO: Replace with impl return when able to use T::Access in that return
    pub fn range<'b>(
        &'b self,
        range: impl RangeBounds<usize>,
    ) -> Box<dyn Iterator<Item = CruiserResult<T::Access>> + 'b>
    where
        T: InPlaceRead<'b, ()>,
        D: AsRef<[u8]>,
    {
        self.range_with_clone_arg(range, ())
    }
    /// Gets an iterator over the array in the range
    // TODO: Replace with impl return when able to use T::AccessMut in that return
    pub fn range_mut<'b>(
        &'b mut self,
        range: impl RangeBounds<usize>,
    ) -> Box<dyn Iterator<Item = CruiserResult<T::AccessMut>> + 'b>
    where
        T: InPlaceWrite<'b, ()>,
        D: AsMut<[u8]>,
    {
        self.range_with_clone_arg_mut(range, ())
    }

    /// Gets an iterator over the array cloning the arg
    // TODO: Replace with impl return when able to use T::Access in that return
    pub fn all_with_clone_arg<'b, R>(
        &'b self,
        arg: R,
    ) -> Box<dyn Iterator<Item = CruiserResult<T::Access>> + 'b>
    where
        T: InPlaceRead<'b, R>,
        R: Clone + 'b,
        D: AsRef<[u8]>,
    {
        self.range_with_clone_arg(.., arg)
    }

    /// Gets an iterator over the array cloning the arg mutably
    // TODO: Replace with impl return when able to use T::AccessMut in that return
    pub fn all_with_clone_arg_mut<'b, W>(
        &'b mut self,
        arg: W,
    ) -> Box<dyn Iterator<Item = CruiserResult<T::AccessMut>> + 'b>
    where
        T: InPlaceWrite<'b, W>,
        D: AsMut<[u8]>,
        W: Clone + 'b,
    {
        self.range_with_clone_arg_mut(.., arg)
    }

    /// Gets an iterator over all the elements with an argument array
    // TODO: Replace with impl return when able to use T::Access in that return
    pub fn all_with_args<'b, R>(
        &'b self,
        args: [R; N],
    ) -> Box<dyn Iterator<Item = CruiserResult<T::Access>> + 'b>
    where
        T: InPlaceRead<'b, R>,
        D: AsRef<[u8]>,
        R: 'b,
    {
        let mut data = self.data.as_ref();
        Box::new(
            args.into_iter()
                .map(move |arg| T::read_with_arg(data.advance(self.element_length), arg)),
        )
    }

    /// Gets an iterator over all the elements with an argument array
    // TODO: Replace with impl return when able to use T::AccessMut in that return
    pub fn all_with_args_mut<'b, W>(
        &'b mut self,
        args: [W; N],
    ) -> Box<dyn Iterator<Item = CruiserResult<T::AccessMut>> + 'b>
    where
        T: InPlaceWrite<'b, W>,
        D: AsMut<[u8]>,
        W: 'b,
    {
        let mut data = self.data.as_mut();
        let element_length = self.element_length;
        Box::new(
            args.into_iter()
                .map(move |arg| T::write_with_arg(data.advance(element_length), arg)),
        )
    }

    /// Gets an iterator over all the elements
    #[allow(clippy::type_complexity)]
    pub fn all<'b>(&'b self) -> Box<dyn Iterator<Item = CruiserResult<T::Access>> + 'b>
    where
        T: InPlaceRead<'b, ()>,
        D: AsRef<[u8]>,
    {
        self.all_with_clone_arg(())
    }

    /// Gets an iterator over all the elements mutably
    #[allow(clippy::type_complexity)]
    pub fn all_mut<'b>(&'b mut self) -> Box<dyn Iterator<Item = CruiserResult<T::AccessMut>> + 'b>
    where
        T: InPlaceWrite<'b, ()>,
        D: AsMut<[u8]>,
    {
        self.all_with_clone_arg_mut(())
    }
}
impl<'a, T, const N: usize> InPlace<'a> for [T; N] {
    type Access = InPlaceArray<T, &'a [u8], N>;
    type AccessMut = InPlaceArray<T, &'a mut [u8], N>;
}
impl<'a, T, A, const N: usize> InPlaceCreate<'a, [A; N]> for [T; N]
where
    T: OnChainStaticSize + InPlaceCreate<'a, A>,
{
    fn create_with_arg(mut data: &mut [u8], arg: [A; N]) -> CruiserResult {
        let element_length = T::on_chain_static_size();
        let mut data = data.try_advance(element_length * N)?;
        for arg in arg {
            T::create_with_arg(data.try_advance(element_length)?, arg)?;
        }
        Ok(())
    }
}
impl<'a, T, const N: usize> InPlaceCreate<'a, ()> for [T; N]
where
    T: OnChainStaticSize + InPlaceCreate<'a, ()>,
{
    fn create_with_arg(data: &mut [u8], arg: ()) -> CruiserResult {
        Self::create_with_arg(data, [arg; N])
    }
}
impl<'a, T, const N: usize> InPlaceRead<'a, ()> for [T; N]
where
    T: OnChainStaticSize,
{
    fn read_with_arg(mut data: &'a [u8], _arg: ()) -> CruiserResult<Self::Access> {
        let element_length = T::on_chain_static_size();
        Ok(InPlaceArray {
            element_length,
            data: data.try_advance(element_length * N)?,
            phantom_t: PhantomData,
        })
    }
}
impl<'a, T, const N: usize> InPlaceWrite<'a, ()> for [T; N]
where
    T: OnChainStaticSize,
{
    fn write_with_arg(mut data: &'a mut [u8], _arg: ()) -> CruiserResult<Self::AccessMut> {
        let element_length = T::on_chain_static_size();
        Ok(InPlaceArray {
            element_length,
            data: data.try_advance(element_length * N)?,
            phantom_t: PhantomData,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::in_place::{InPlaceCreate, InPlaceGet, InPlaceRead, InPlaceSet, InPlaceWrite};
    use crate::CruiserResult;
    use rand::{thread_rng, Rng};

    #[test]
    fn array_test() -> CruiserResult {
        let mut rng = thread_rng();
        let values = (0..1024).map(|_| rng.gen::<u128>()).collect::<Vec<_>>();
        let mut data = vec![0u8; 1024 * 16];

        <[u128; 1024]>::create_with_arg(&mut data, ())?;
        let in_place = <[u128; 1024]>::read_with_arg(&data, ())?;
        for value in in_place.all() {
            assert_eq!(0, value?.get()?);
        }
        let mut in_place = <[u128; 1024]>::write_with_arg(&mut data, ())?;
        in_place
            .all_mut()
            .zip(values.iter())
            .map(|(write, value)| write?.set(*value))
            .collect::<Result<Vec<_>, _>>()?;

        for (i, value) in values.iter().enumerate() {
            assert_eq!(in_place.get(i)?.unwrap().get()?, *value);
        }
        Ok(())
    }
}
