use crate::{
    Advance, AdvanceArray, Error, GeneratorError, GeneratorResult, InPlaceBuilder, InPlaceData,
    InPlaceGet, InPlaceSet, StaticSized, StaticSizedSize,
};
use array_init::try_array_init;
use num_traits::Zero;
use std::collections::Bound;
use std::convert::{Infallible, TryFrom, TryInto};
use std::marker::PhantomData;
use std::ops::RangeBounds;

pub trait InPlaceVec<'a, T, D>: InPlaceData
where
    T: StaticSized,
    D: InPlaceBuilder,
    for<'b> D::InPlaceData<'a>: InPlaceGet<'b, usize> + InPlaceSet<'b, usize>,
{
    fn len(&self) -> usize;
    unsafe fn len_mut(&mut self) -> &mut D::InPlaceData<'a>;
    unsafe fn data(&mut self) -> &mut [u8];
    unsafe fn length_and_data(&mut self) -> (&mut D::InPlaceData<'a>, &mut [u8]);
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn max_length(&self) -> usize;
    fn get(&mut self, index: usize) -> GeneratorResult<Option<T::InPlaceData<'_>>> {
        vec_get::<T>(self.len(), index, unsafe { self.data() })
    }
    fn get_all(&mut self) -> GeneratorResult<Vec<T::InPlaceData<'_>>> {
        let length = self.len();
        vec_get_subset::<T, _>(length, 0..length, unsafe { self.data() })
    }
    fn get_subset(
        &mut self,
        range: impl RangeBounds<usize>,
    ) -> GeneratorResult<Vec<T::InPlaceData<'_>>> {
        vec_get_subset::<T, _>(self.len(), range, unsafe { self.data() })
    }
    fn get_array<const N: usize>(
        &mut self,
        start: usize,
    ) -> GeneratorResult<Option<[T::InPlaceData<'_>; N]>> {
        vec_get_array::<T, N>(self.len(), start, unsafe { self.data() })
    }
    fn replace(
        &mut self,
        index: usize,
        value: T::CreateArg,
    ) -> GeneratorResult<Result<T::InPlaceData<'_>, T::CreateArg>> {
        vec_replace::<T>(self.len(), index, value, unsafe { self.data() })
    }
    fn swap_buffer(
        &mut self,
        index1: usize,
        index2: usize,
        temp_buffer: &mut [u8; T::DATA_SIZE],
    ) -> GeneratorResult<bool> {
        vec_swap::<T>(self.len(), index1, index2, temp_buffer, unsafe {
            self.data()
        })
    }
    fn swap(&mut self, index1: usize, index2: usize) -> GeneratorResult<bool>
    where
        [(); T::DATA_SIZE]:,
    {
        self.swap_buffer(index1, index2, &mut [0; T::DATA_SIZE])
    }
    fn push<'b>(
        &'b mut self,
        value: T::CreateArg,
    ) -> GeneratorResult<Result<T::InPlaceData<'_>, T::CreateArg>>
    where
        D::InPlaceData<'a>: 'b,
    {
        let max_length = self.max_length();
        let (length, data) = unsafe { self.length_and_data() };
        vec_push::<T, _>(max_length, value, length, data)
    }
    // TODO: Add version that returns iterator when impl return without lifetime bound bug fixed
    fn push_all<'b, I>(
        &'b mut self,
        values: I,
    ) -> GeneratorResult<Result<Vec<T::InPlaceData<'b>>, I::IntoIter>>
    where
        D::InPlaceData<'a>: 'b,
        I: 'b + IntoIterator<Item = T::CreateArg>,
        I::IntoIter: ExactSizeIterator,
    {
        let max_length = self.max_length();
        let (length, data) = unsafe { self.length_and_data() };
        vec_push_all::<T, I, _>(max_length, values, length, data)
    }
    fn remove(&mut self, index: usize) -> GeneratorResult<bool> {
        let (length, data) = unsafe { self.length_and_data() };
        vec_remove::<T, _>(index, length, data)
    }
}

#[derive(Debug)]
pub struct DynamicInPlaceVec<'a, T, D>
where
    D: InPlaceBuilder,
{
    max_length: D::InPlaceData<'a>,
    length: D::InPlaceData<'a>,
    /// Start of items
    data: &'a mut [u8],
    phantom_t: PhantomData<fn() -> T>,
}
impl<T, D> InPlaceBuilder for DynamicInPlaceVec<'static, T, D>
where
    T: StaticSized,
    D: InPlaceBuilder,
    D::CreateArg: Zero,
    for<'b> usize: From<&'b D::CreateArg>,
    for<'a, 'b> D::InPlaceData<'a>: InPlaceGet<'b, usize>,
    Box<dyn Error>: From<<D as InPlaceBuilder>::SizeError>,
{
    type InPlaceData<'a> = DynamicInPlaceVec<'a, T, D>;
    type SizeError = Box<dyn Error>;
    type CreateArg = D::CreateArg;

    fn data_size(data: &[u8]) -> Result<usize, Self::SizeError> {
        let data_size = D::data_size(data)?;
        let max_length = D::InPlaceData::<'static>::read_and_get(data)?;
        Ok(data_size * 2 + max_length * T::DATA_SIZE)
    }

    fn create_size(create_arg: &Self::CreateArg) -> usize {
        D::create_size(create_arg) * 2 + usize::from(create_arg) * T::DATA_SIZE
    }

    fn create(
        mut data: &mut [u8],
        max_length: Self::CreateArg,
    ) -> GeneratorResult<Self::InPlaceData<'_>> {
        let max_length_data_size = D::create_size(&max_length);
        let length = D::CreateArg::zero();
        let max_length = D::create(data.advance(max_length_data_size), max_length)?;
        let length = D::create(data.advance(max_length_data_size), length)?;
        Ok(DynamicInPlaceVec {
            max_length,
            length,
            data,
            phantom_t: PhantomData,
        })
    }

    fn read(mut data: &mut [u8]) -> GeneratorResult<Self::InPlaceData<'_>> {
        let max_length_size = D::data_size(data)?;
        let max_length = D::read(data.advance(max_length_size))?;
        let length = D::read(data.advance(max_length_size))?;
        Ok(DynamicInPlaceVec {
            max_length,
            length,
            data,
            phantom_t: PhantomData,
        })
    }
}
impl<'a, T, D> InPlaceData for DynamicInPlaceVec<'a, T, D>
where
    T: StaticSized,
    D: InPlaceBuilder,
    for<'b> D::InPlaceData<'a>: InPlaceGet<'b, usize>,
{
    fn self_data_size(&self) -> usize {
        self.max_length.self_data_size() * 2 + self.max_length.get_value() * T::DATA_SIZE
    }
}
impl<'a, T, D> InPlaceVec<'a, T, D> for DynamicInPlaceVec<'a, T, D>
where
    T: StaticSized,
    D: InPlaceBuilder,
    for<'b> D::InPlaceData<'a>: InPlaceGet<'b, usize> + InPlaceSet<'b, usize>,
{
    fn len(&self) -> usize {
        self.length.get_value()
    }

    unsafe fn len_mut(&mut self) -> &mut D::InPlaceData<'a> {
        &mut self.length
    }

    unsafe fn data(&mut self) -> &mut [u8] {
        self.data
    }

    unsafe fn length_and_data(&mut self) -> (&mut D::InPlaceData<'a>, &mut [u8]) {
        (&mut self.length, self.data)
    }

    fn max_length(&self) -> usize {
        self.max_length.get_value()
    }
}

#[derive(Debug)]
pub struct StaticInPlaceVec<'a, T, D, const N: usize>
where
    T: StaticSized,
    D: InPlaceBuilder,
    [(); N * T::DATA_SIZE]:,
{
    length: D::InPlaceData<'a>,
    /// Start of items
    data: &'a mut [u8; N * T::DATA_SIZE],
}
impl<'a, T, D, const N: usize> StaticInPlaceVec<'a, T, D, N>
where
    T: StaticSized,
    D: StaticSized,
    [(); N * T::DATA_SIZE]:,
{
    const fn data_size() -> usize {
        D::DATA_SIZE + N * T::DATA_SIZE
    }
}
impl<T, D, const N: usize> InPlaceBuilder for StaticInPlaceVec<'static, T, D, N>
where
    T: StaticSized,
    D: StaticSized,
    D::CreateArg: Zero + From<usize>,
    for<'a, 'b> D::InPlaceData<'a>: InPlaceGet<'b, usize>,
    [(); N * T::DATA_SIZE]:,
{
    type InPlaceData<'a> = StaticInPlaceVec<'a, T, D, N>;
    type SizeError = Infallible;
    type CreateArg = ();

    fn data_size(_data: &[u8]) -> Result<usize, Self::SizeError> {
        Ok(Self::DATA_SIZE)
    }

    fn create_size(_create_arg: &Self::CreateArg) -> usize {
        Self::DATA_SIZE
    }

    fn create(
        mut data: &mut [u8],
        _create_arg: Self::CreateArg,
    ) -> GeneratorResult<Self::InPlaceData<'_>> {
        Self::create_static(data.try_advance_array()?)
    }

    fn read(mut data: &mut [u8]) -> GeneratorResult<Self::InPlaceData<'_>> {
        let length = D::read(data.advance(D::DATA_SIZE))?;
        if data.len() < N * T::DATA_SIZE {
            Err(GeneratorError::NotEnoughData {
                needed: N * T::DATA_SIZE,
                remaining: data.len(),
            }
            .into())
        } else {
            Ok(StaticInPlaceVec {
                length,
                data: (&mut data[..N * T::DATA_SIZE]).try_into().unwrap(),
            })
        }
    }
}
impl<T, D, const N: usize> StaticSizedSize for StaticInPlaceVec<'static, T, D, N>
where
    T: StaticSized,
    D: StaticSized,
    D::CreateArg: Zero + From<usize>,
    for<'a, 'b> D::InPlaceData<'a>: InPlaceGet<'b, usize>,
    [(); N * T::DATA_SIZE]:,
{
}
// impl<T, D, const N: usize> StaticSized for StaticInPlaceVec<'static, T, D, N>
// where
//     T: StaticSized,
//     D: StaticSized,
//     D::CreateArg: Zero + From<usize>,
//     for<'a, 'b> D::InPlaceData<'a>: InPlaceGet<'b, usize>,
//     [(); N * T::DATA_SIZE]:,
// {

// const DATA_SIZE: usize = Self::data_size();
//     fn create_static(
//         data: &mut [u8; Self::data_size()],
//         _create_arg: Self::CreateArg,
//     ) -> GeneratorResult<Self::InPlaceData<'_>> {
//         let [length @ 0..D::DATA_SIZE, data @ ..] = data;
//         Ok(StaticInPlaceVec {
//             length: D::create_static(length, D::CreateArg::zero()),
//             data,
//         })
//     }
//
//     fn read_static(data: &mut [u8; Self::DATA_SIZE]) -> GeneratorResult<Self::InPlaceData<'_>> {
//         todo!()
//     }
// }
impl<'a, T, D, const N: usize> InPlaceData for StaticInPlaceVec<'a, T, D, N>
where
    T: StaticSized,
    D: StaticSized,
    [(); N * T::DATA_SIZE]:,
{
    fn self_data_size(&self) -> usize {
        Self::data_size()
    }
}
impl<'a, T, D, const N: usize> InPlaceVec<'a, T, D> for StaticInPlaceVec<'a, T, D, N>
where
    T: StaticSized,
    D: StaticSized,
    for<'b> D::InPlaceData<'a>: InPlaceGet<'b, usize> + InPlaceSet<'b, usize>,
    [(); N * T::DATA_SIZE]:,
{
    fn len(&self) -> usize {
        self.length.get_value()
    }

    unsafe fn len_mut(&mut self) -> &mut D::InPlaceData<'a> {
        &mut self.length
    }

    unsafe fn data(&mut self) -> &mut [u8] {
        self.data
    }

    unsafe fn length_and_data(&mut self) -> (&mut D::InPlaceData<'a>, &mut [u8]) {
        (&mut self.length, self.data)
    }

    fn max_length(&self) -> usize {
        N
    }
}

fn vec_get<T>(
    length: usize,
    index: usize,
    data: &mut [u8],
) -> GeneratorResult<Option<T::InPlaceData<'_>>>
where
    T: StaticSized,
{
    if index >= length {
        Ok(None)
    } else {
        T::read(&mut data[index * T::DATA_SIZE..][..T::DATA_SIZE]).map(Some)
    }
}

fn vec_get_subset<T, R>(
    length: usize,
    range: R,
    data: &mut [u8],
) -> GeneratorResult<Vec<T::InPlaceData<'_>>>
where
    T: StaticSized,
    R: RangeBounds<usize>,
{
    let start_index = match range.start_bound() {
        Bound::Included(value) => *value,
        Bound::Excluded(value) => *value + 1,
        Bound::Unbounded => 0,
    };
    if start_index >= length {
        return Ok(Vec::new());
    }
    let end_index = match range.end_bound() {
        Bound::Included(value) => *value + 1,
        Bound::Excluded(value) => *value,
        Bound::Unbounded => length,
    }
    .max(length);
    if start_index > end_index {
        Err(GeneratorError::Custom {
            error: format!(
                "Start index (`{}`) before end index (`{}`)",
                start_index, end_index
            ),
        }
        .into())
    } else {
        let mut bytes = &mut data[start_index * T::DATA_SIZE..end_index * T::DATA_SIZE];
        let mut out = Vec::with_capacity(end_index - start_index);
        while !bytes.is_empty() {
            out.push(T::read(bytes.advance(T::DATA_SIZE))?);
        }
        Ok(out)
    }
}

fn vec_get_array<T, const N: usize>(
    length: usize,
    start: usize,
    data: &mut [u8],
) -> GeneratorResult<Option<[T::InPlaceData<'_>; N]>>
where
    T: StaticSized,
{
    if start + N > length {
        Ok(None)
    } else {
        let mut bytes = &mut data[start * T::DATA_SIZE..];
        Ok(Some(try_array_init(|_| {
            T::read(bytes.advance(T::DATA_SIZE))
        })?))
    }
}

fn vec_replace<T>(
    length: usize,
    index: usize,
    value: T::CreateArg,
    data: &mut [u8],
) -> GeneratorResult<Result<T::InPlaceData<'_>, T::CreateArg>>
where
    T: StaticSized,
{
    if index >= length {
        Ok(Err(value))
    } else {
        let data = &mut data[index * T::DATA_SIZE..][..T::DATA_SIZE];
        Ok(Ok(T::create(data, value)?))
    }
}

fn vec_swap<T>(
    length: usize,
    index1: usize,
    index2: usize,
    temp_buffer: &mut [u8; T::DATA_SIZE],
    data: &mut [u8],
) -> GeneratorResult<bool>
where
    T: StaticSized,
{
    if index1 >= length || index2 >= length {
        Ok(false)
    } else if index1 == index2 {
        Ok(true)
    } else {
        let (first_index, second_index) = if index1 > index2 {
            (index2, index1)
        } else {
            (index1, index2)
        };
        let (first_slice, rest) = data[first_index * T::DATA_SIZE..].split_at_mut(T::DATA_SIZE);
        let second_slice = &mut rest[(second_index - first_index) * T::DATA_SIZE..][..T::DATA_SIZE];
        temp_buffer.copy_from_slice(first_slice);
        first_slice.copy_from_slice(second_slice);
        second_slice.copy_from_slice(temp_buffer);
        Ok(true)
    }
}

fn vec_push<'a, T, L>(
    max_length: usize,
    value: T::CreateArg,
    length: &mut L,
    data: &'a mut [u8],
) -> GeneratorResult<Result<T::InPlaceData<'a>, T::CreateArg>>
where
    T: StaticSized,
    for<'b> L: InPlaceGet<'b, usize> + InPlaceSet<'b, usize>,
{
    let length_val = length.get_value();
    if length_val >= max_length {
        Ok(Err(value))
    } else {
        length.set_value(length_val + 1);
        let data = &mut data[length_val * T::DATA_SIZE..][..T::DATA_SIZE];
        Ok(Ok(T::create(data, value)?))
    }
}

/// If an error occurs in returned iter vec will be in questionable state
fn vec_push_all<'a, T, I, L>(
    max_length: usize,
    values: I,
    length: &mut L,
    mut data: &'a mut [u8],
) -> GeneratorResult<Result<Vec<T::InPlaceData<'a>>, I::IntoIter>>
where
    T: StaticSized,
    I: 'a + IntoIterator<Item = T::CreateArg>,
    I::IntoIter: ExactSizeIterator,
    for<'b> L: InPlaceGet<'b, usize> + InPlaceSet<'b, usize>,
{
    let iter = values.into_iter();
    let length_val = length.get_value();
    if iter.len() > max_length - length_val {
        Ok(Err(iter))
    } else {
        data.advance(length_val * T::DATA_SIZE);
        let out = iter
            .map(|value| T::create(data.advance(T::DATA_SIZE), value))
            .collect::<Result<Vec<_>, _>>()?;
        length.set_value(length_val + out.len() * T::DATA_SIZE);
        Ok(Ok(out))
    }
}

fn vec_remove<T, L>(index: usize, length: &mut L, data: &mut [u8]) -> GeneratorResult<bool>
where
    T: StaticSized,
    for<'b> L: InPlaceGet<'b, usize> + InPlaceSet<'b, usize>,
{
    let length_val = length.get_value();
    if index >= length_val {
        Ok(false)
    } else {
        data.copy_within(
            (index + 1) * T::DATA_SIZE..length_val * T::DATA_SIZE,
            index * T::DATA_SIZE,
        );
        length.set_value(length_val - 1);
        Ok(true)
    }
}
