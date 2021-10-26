//! Short vector implementations

use std::fmt::{Debug, Formatter};
use std::iter::{FromIterator, FusedIterator};
use std::mem::{ManuallyDrop, MaybeUninit};
use std::ops::{Deref, DerefMut, Index, IndexMut};
use std::ptr::{slice_from_raw_parts, slice_from_raw_parts_mut};
use std::slice::{Iter, IterMut};

/// A Vector on the stack whose maximum size is `N`
pub struct ShortVec<T, const N: usize> {
    values: [MaybeUninit<T>; N],
    length: usize,
}
impl<T, const N: usize> ShortVec<T, N> {
    /// Creates a 0 length [`ShortVec`]
    pub fn new() -> Self {
        Self {
            // TODO: Replace with `MaybeUninit::uninit_array()` when stabilized
            values: unsafe { MaybeUninit::<[MaybeUninit<T>; N]>::uninit().assume_init() },
            length: 0,
        }
    }

    /// Adds a value to the short vec, returning it if not enough space
    pub fn push(&mut self, value: T) -> Result<(), T> {
        if self.length < N {
            self.values[self.length] = MaybeUninit::new(value);
            self.length += 1;
            Ok(())
        } else {
            Err(value)
        }
    }

    /// Returns the short vec as a slice
    pub fn as_slice(&self) -> &[T] {
        // Safety: Valid because MaybeUninit<T> is transparent to internal T
        unsafe { &*slice_from_raw_parts(self.values.as_ptr() as *const T, self.length) }
    }

    /// Returns the short vec as a mutable slice
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        // Safety: Valid because MaybeUninit<T> is transparent to internal T
        unsafe { &mut *slice_from_raw_parts_mut(self.values.as_mut_ptr() as *mut T, self.length) }
    }

    /// Returns a shared iterator to the short vec
    pub fn iter(&self) -> Iter<'_, T> {
        self.as_slice().iter()
    }

    /// Returns a mutable iterator to the shared vec
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        self.as_mut_slice().iter_mut()
    }
}
impl<T, const N: usize> Debug for ShortVec<T, N>
where
    T: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter()).finish()
    }
}
impl<T, const N: usize> Default for ShortVec<T, N> {
    fn default() -> Self {
        Self::new()
    }
}
impl<T, const N: usize> Drop for ShortVec<T, N> {
    fn drop(&mut self) {
        for index in 0..self.length {
            // TODO: Change to `MaybeUninit::assume_init_drop` when stabilized
            unsafe { drop(self.values[index].as_ptr().read()) }
        }
    }
}
impl<T, const N: usize> Deref for ShortVec<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T, const N: usize> DerefMut for ShortVec<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
impl<T, const N: usize> Index<usize> for ShortVec<T, N> {
    type Output = T;

    fn index(&self, index: usize) -> &Self::Output {
        &self.as_slice()[index]
    }
}
impl<T, const N: usize> IndexMut<usize> for ShortVec<T, N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.as_mut_slice()[index]
    }
}
impl<T, const N: usize> IntoIterator for ShortVec<T, N> {
    type Item = T;
    type IntoIter = IntoIter<T, N>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter::new(self)
    }
}
impl<'a, T, const N: usize> IntoIterator for &'a ShortVec<T, N> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<'a, T, const N: usize> IntoIterator for &'a mut ShortVec<T, N> {
    type Item = &'a mut T;
    type IntoIter = IterMut<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
impl<T, const N: usize> FromIterator<T> for ShortVec<T, N> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut iter = iter.into_iter();
        let mut out = Self::new();
        while out.length < N {
            match iter.next() {
                Some(val) => {
                    if out.push(val).is_err() {
                        unreachable!()
                    }
                }
                None => break,
            }
        }
        out
    }
}

/// IntoIter for [`ShortVec`]
#[derive(Debug)]
pub struct IntoIter<T, const N: usize> {
    vec: ManuallyDrop<ShortVec<T, N>>,
    index: usize,
}
impl<T, const N: usize> IntoIter<T, N> {
    /// Creates a new Iterator over a [`ShortVec`]
    pub fn new(vec: ShortVec<T, N>) -> Self {
        Self {
            vec: ManuallyDrop::new(vec),
            index: 0,
        }
    }
}
impl<T, const N: usize> Iterator for IntoIter<T, N> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.vec.length {
            // TODO: Change to `MaybeUninit::assume_init_read` when stabilized
            let out = unsafe { self.vec.values[self.index].as_ptr().read() };
            self.index += 1;
            Some(out)
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let size = self.vec.length - self.index;
        (size, Some(size))
    }

    fn count(self) -> usize
    where
        Self: Sized,
    {
        self.vec.length - self.index
    }
}
impl<T, const N: usize> ExactSizeIterator for IntoIter<T, N> {}
impl<T, const N: usize> FusedIterator for IntoIter<T, N> {}
impl<T, const N: usize> Drop for IntoIter<T, N> {
    fn drop(&mut self) {
        for index in self.index..self.vec.length {
            // TODO: Change to `MaybeUninit::assume_init_drop` when stabilized
            unsafe { drop(self.vec.values[index].as_ptr().read()) }
        }
    }
}
