//! A stack allocated iterator for variable but short iterations

use array_init::array_init;
use std::array::IntoIter;
use std::iter::{Map, Take};
use std::mem::MaybeUninit;
use std::slice::{Iter, IterMut};

/// A stack allocated iterator of `T` with max size `N`
#[derive(Debug)]
pub struct ShortIter<T, const N: usize> {
    data: [MaybeUninit<T>; N],
    length: usize,
}
impl<T, const N: usize> ShortIter<T, N> {
    /// Creates a new 0 length [`ShortIter`]
    #[must_use]
    pub fn new() -> Self {
        Self {
            data: array_init(|_| MaybeUninit::uninit()),
            length: 0,
        }
    }

    /// Creates a new [`ShortIter`] whose length is the min of `N` and `N2`, filled with the first length values from `array`
    #[must_use]
    pub fn from_array<const N2: usize>(array: [T; N2]) -> Self {
        let mut out = Self::new();
        out.data
            .iter_mut()
            .zip(array)
            .for_each(|(out_val, in_val)| *out_val = MaybeUninit::new(in_val));
        out
    }

    /// Pushes a value into self
    /// # Panics
    /// If self is full when trying to push
    pub fn push(&mut self, value: T) {
        assert!(self.try_push(value).is_ok(), "Cannot add to `ShortIter`");
    }

    /// Tries to push a value into self, returns an error if self is full
    pub fn try_push(&mut self, value: T) -> Result<(), T> {
        if self.length >= N {
            Err(value)
        } else {
            self.data[self.length] = MaybeUninit::new(value);
            self.length += 1;
            Ok(())
        }
    }

    #[allow(clippy::iter_not_returning_iterator)]
    /// Gets an iterator of shared references to self
    pub fn iter(&self) -> <&Self as IntoIterator>::IntoIter {
        self.into_iter()
    }

    #[allow(clippy::iter_not_returning_iterator)]
    /// Gets an iterator of mutable references to self
    pub fn iter_mut(&mut self) -> <&mut Self as IntoIterator>::IntoIter {
        self.into_iter()
    }
}
impl<T, const N: usize> Default for ShortIter<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> IntoIterator for ShortIter<T, N> {
    type Item = T;
    type IntoIter = Map<Take<IntoIter<MaybeUninit<T>, N>>, fn(MaybeUninit<T>) -> T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            self.data
                .into_iter()
                .take(self.length)
                .map(|val| val.assume_init())
        }
    }
}
impl<'a, T, const N: usize> IntoIterator for &'a ShortIter<T, N> {
    type Item = &'a T;
    type IntoIter = Map<Take<Iter<'a, MaybeUninit<T>>>, fn(&'a MaybeUninit<T>) -> &'a T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            self.data
                .iter()
                .take(self.length)
                .map(|val| val.assume_init_ref())
        }
    }
}

impl<'a, T, const N: usize> IntoIterator for &'a mut ShortIter<T, N> {
    type Item = &'a mut T;
    type IntoIter = Map<Take<IterMut<'a, MaybeUninit<T>>>, fn(&'a mut MaybeUninit<T>) -> &'a mut T>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            self.data
                .iter_mut()
                .take(self.length)
                .map(|val| val.assume_init_mut())
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn short_iter_test() {
        let mut iter = ShortIter::<_, 3>::from_array([100, 200, 300]);
        assert_eq!(iter.try_push(400), Err(400));
        assert_eq!(iter.into_iter().collect::<Vec<_>>(), vec![100, 200, 300]);

        let mut iter = ShortIter::<_, 4>::from_array([1, 2]);
        iter.push(3);
        assert_eq!(iter.iter().collect::<Vec<_>>(), vec![&1, &2, &3]);
        assert_eq!(
            iter.iter_mut().collect::<Vec<_>>(),
            vec![&mut 1, &mut 2, &mut 3]
        );
        assert_eq!(iter.into_iter().collect::<Vec<_>>(), vec![1, 2, 3]);
    }
}
