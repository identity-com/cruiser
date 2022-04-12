/// Iterator method for
pub trait WithData: Sized + Iterator {
    /// Runs map function with data stored in the iterator
    fn map_with_data<D, F>(self, data: D, func: F) -> WithDataIter<Self, D, F>;
}
impl<T> WithData for T
where
    T: Iterator,
{
    fn map_with_data<D, F>(self, data: D, func: F) -> WithDataIter<Self, D, F> {
        WithDataIter {
            iter: self,
            func,
            data,
        }
    }
}

/// The iterator for [`WithData::map_with_data`]
#[derive(Debug, Copy, Clone)]
pub struct WithDataIter<I, D, F> {
    iter: I,
    func: F,
    data: D,
}
impl<I, D, F, O> Iterator for WithDataIter<I, D, F>
where
    I: Iterator,
    F: FnMut(I::Item, &mut D) -> O,
{
    type Item = O;

    fn next(&mut self) -> Option<Self::Item> {
        Some((self.func)(self.iter.next()?, &mut self.data))
    }
}
