use std::iter::FusedIterator;

/// Extension for chaining iterators while maintaining exact size.
/// Panics if size is larger than `usize::MAX` unlike [`Iterator::chain`].
pub trait ChainExactSizeExt: Sized + Iterator {
    /// Chain two iterators together while maintaining exact size.
    /// Panics if size is larger than `usize::MAX` unlike [`Iterator::chain`].
    fn chain_exact_size<I>(self, other: I) -> ChainExactSize<Self, I::IntoIter>
    where
        I: IntoIterator<Item = Self::Item>,
    {
        ChainExactSize {
            iter1: Some(self),
            iter2: Some(other.into_iter()),
        }
    }
}
impl<T> ChainExactSizeExt for T where T: Sized + Iterator {}
/// Two chained iterators maintaining exact size.
#[derive(Debug, Clone)]
pub struct ChainExactSize<I1, I2> {
    iter1: Option<I1>,
    iter2: Option<I2>,
}
impl<I1, I2> Iterator for ChainExactSize<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
    type Item = I1::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self.iter1.as_mut() {
            None => self.iter2.as_mut().and_then(Iterator::next),
            Some(iter) => match iter.next() {
                None => {
                    self.iter1 = None;
                    self.iter2.as_mut().and_then(Iterator::next)
                }
                Some(val) => Some(val),
            },
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let first = self
            .iter1
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        let second = self
            .iter2
            .as_ref()
            .map_or((0, Some(0)), Iterator::size_hint);
        (
            first.0 + second.0,
            first.1.and_then(|n| {
                second.1.map(|m| {
                    n.checked_add(m)
                        .expect("Size of iterator larger than `usize::MAX`")
                })
            }),
        )
    }
}
impl<I1, I2> DoubleEndedIterator for ChainExactSize<I1, I2>
where
    I1: DoubleEndedIterator,
    I2: DoubleEndedIterator<Item = I1::Item>,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        match self.iter2.as_mut() {
            None => self.iter1.as_mut().and_then(DoubleEndedIterator::next_back),
            Some(iter) => match iter.next_back() {
                None => {
                    self.iter2 = None;
                    self.iter1.as_mut().and_then(DoubleEndedIterator::next_back)
                }
                Some(val) => Some(val),
            },
        }
    }
}
impl<I1, I2> ExactSizeIterator for ChainExactSize<I1, I2>
where
    I1: ExactSizeIterator,
    I2: ExactSizeIterator<Item = I1::Item>,
{
}
impl<I1, I2> FusedIterator for ChainExactSize<I1, I2>
where
    I1: Iterator,
    I2: Iterator<Item = I1::Item>,
{
}
