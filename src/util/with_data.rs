use std::iter::FusedIterator;
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

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.iter.count()
    }
}
impl<I, D, F, O> DoubleEndedIterator for WithDataIter<I, D, F>
where
    I: DoubleEndedIterator,
    F: FnMut(I::Item, &mut D) -> O,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        Some((self.func)(self.iter.next_back()?, &mut self.data))
    }
}
impl<I, D, F, O> ExactSizeIterator for WithDataIter<I, D, F>
where
    I: ExactSizeIterator,
    F: FnMut(I::Item, &mut D) -> O,
{
}
impl<I, D, F, O> FusedIterator for WithDataIter<I, D, F>
where
    I: FusedIterator,
    F: FnMut(I::Item, &mut D) -> O,
{
}
