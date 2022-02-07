use crate::{GeneratorError, GeneratorResult};
pub use short_vec::ShortVec;
use std::borrow::Cow;
use std::cmp::{max, min};
use std::num::NonZeroU64;
use std::ops::{Bound, Deref, RangeBounds};
use std::ptr::slice_from_raw_parts_mut;

pub(crate) mod bytes_ext;
pub mod short_vec;

/// (start, end), inclusive
pub fn convert_range(
    range: &impl RangeBounds<usize>,
    length: usize,
) -> GeneratorResult<(usize, usize)> {
    let start = match range.start_bound() {
        Bound::Included(val) => *val,
        Bound::Excluded(val) => val + 1,
        Bound::Unbounded => 0,
    };
    let end = match range.end_bound() {
        Bound::Included(val) => *val,
        Bound::Excluded(val) => val - 1,
        Bound::Unbounded => length - 1,
    };
    let (start, end) = if start <= end {
        (start, end)
    } else {
        (end, start)
    };
    if end >= length {
        Err(GeneratorError::IndexOutOfRange {
            index: format!(
                "{},{}",
                match range.start_bound() {
                    Bound::Included(val) => Cow::Owned(format!("[{}", val)),
                    Bound::Excluded(val) => Cow::Owned(format!("({}", val)),
                    Bound::Unbounded => Cow::Borrowed("["),
                },
                match range.end_bound() {
                    Bound::Included(val) => Cow::Owned(format!("{}]", val)),
                    Bound::Excluded(val) => Cow::Owned(format!("{})", val)),
                    Bound::Unbounded => Cow::Borrowed("]"),
                }
            ),
            possible_range: format!("[0, {})", length),
        }
        .into())
    } else {
        Ok((start, end))
    }
}

/// Helper function to combine multiple size hints with a branch strategy, where the minimum lower bound and maximum upper bound are returned
pub fn combine_hints_branch(
    mut hints: impl Iterator<Item = (usize, Option<usize>)>,
) -> (usize, Option<usize>) {
    let (mut lower, mut upper) = match hints.next() {
        None => return (0, None),
        Some(hint) => hint,
    };
    for (hint_lower, hint_upper) in hints {
        lower = min(lower, hint_lower);
        upper = match (upper, hint_upper) {
            (Some(upper), Some(hint_upper)) => Some(max(upper, hint_upper)),
            _ => None,
        }
    }
    (lower, upper)
}

/// Helper function to combine multiple size hints with a chain strategy, where the sum of lower and upper bounds are returned
pub fn combine_hints_chain(
    mut hints: impl Iterator<Item = (usize, Option<usize>)>,
) -> (usize, Option<usize>) {
    let (mut lower, mut upper) = match hints.next() {
        None => return (0, None),
        Some(hint) => hint,
    };
    for (hint_lower, hint_upper) in hints {
        lower += hint_lower;
        upper = match (upper, hint_upper) {
            (Some(upper), Some(hint_upper)) => upper.checked_add(hint_upper),
            _ => None,
        }
    }
    (lower, upper)
}

/// Helper function to multiply a size hint by a number
pub fn mul_size_hint(hint: (usize, Option<usize>), mul: usize) -> (usize, Option<usize>) {
    (
        hint.0 * mul,
        match hint.1 {
            Some(upper) => upper.checked_mul(mul),
            None => None,
        },
    )
}

/// Length grabbing functions
pub trait Length {
    /// Gets the length
    fn len(&self) -> usize;
    /// Tells whether the length is 0
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}
impl<T> Length for [T] {
    fn len(&self) -> usize {
        self.len()
    }
}
impl<'a, T> Length for &'a [T] {
    fn len(&self) -> usize {
        self.deref().len()
    }
}
impl<'a, T> Length for &'a mut [T] {
    fn len(&self) -> usize {
        self.deref().len()
    }
}
impl<T, const N: usize> Length for [T; N] {
    fn len(&self) -> usize {
        N
    }
}
impl<'a, T, const N: usize> Length for &'a [T; N] {
    fn len(&self) -> usize {
        N
    }
}
impl<'a, T, const N: usize> Length for &'a mut [T; N] {
    fn len(&self) -> usize {
        N
    }
}

/// Advances a given slice while maintaining lifetimes
pub trait Advance<'a>: Length {
    /// The output of advancing
    type AdvanceOut;

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Panics if not enough data.
    fn advance(&'a mut self, amount: usize) -> Self::AdvanceOut {
        assert!(amount <= self.len());
        // Safety: amount is not greater than the length of self
        unsafe { self.advance_unchecked(amount) }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Errors if not enough data.
    fn try_advance(&'a mut self, amount: usize) -> GeneratorResult<Self::AdvanceOut> {
        if self.len() < amount {
            Err(GeneratorError::NotEnoughData {
                needed: amount,
                remaining: self.len(),
            }
            .into())
        } else {
            // Safety: amount is not greater than the length of self
            Ok(unsafe { self.advance_unchecked(amount) })
        }
    }

    /// Advances self forward by `amount`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `amount` is not greater than the length of self.
    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut;
}
/// Advances a given slice giving back an array
pub trait AdvanceArray<'a, const N: usize>: Length {
    /// The output of advancing
    type AdvanceOut;

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Panics if not enough data.
    fn advance_array(&'a mut self) -> Self::AdvanceOut {
        assert!(N <= self.len());
        // Safety: N is not greater than the length of self
        unsafe { self.advance_array_unchecked() }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Errors if not enough data.
    fn try_advance_array(&'a mut self) -> GeneratorResult<Self::AdvanceOut> {
        if self.len() < N {
            Err(GeneratorError::NotEnoughData {
                needed: N,
                remaining: self.len(),
            }
            .into())
        } else {
            // Safety: N is not greater than the length of self
            Ok(unsafe { self.advance_array_unchecked() })
        }
    }

    /// Advances self forward by `N`, returning the advanced over portion.
    /// Does not error if not enough data.
    ///
    /// # Safety
    /// Caller must guarantee that `N` is not greater than the length of self.
    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut;
}
impl<'a, 'b, T> Advance<'a> for &'b mut [T] {
    type AdvanceOut = &'b mut [T];

    unsafe fn advance_unchecked(&'a mut self, amount: usize) -> Self::AdvanceOut {
        // Safety neither slice overlaps and points to valid r/w data
        let len = self.len();
        let ptr = self.as_mut_ptr();
        *self = &mut *slice_from_raw_parts_mut(ptr.add(amount), len - amount);
        &mut *slice_from_raw_parts_mut(ptr, amount)
    }
}
impl<'a, 'b, T, const N: usize> AdvanceArray<'a, N> for &'b mut [T] {
    type AdvanceOut = &'b mut [T; N];

    unsafe fn advance_array_unchecked(&'a mut self) -> Self::AdvanceOut {
        // Safe conversion because returned array will always be same size as value passed in (`N`)
        &mut *(
            // Safety: Same requirements as this function
            self.advance_unchecked(N) as *mut [T] as *mut [T; N]
        )
    }
}

/// Number can become non-zero, panicking if can't
pub trait ToNonZero {
    /// The non-zero type
    type NonZero;

    /// Converts to non-zero
    fn to_non_zero(self) -> Self::NonZero;
}
impl ToNonZero for u64 {
    type NonZero = NonZeroU64;

    fn to_non_zero(self) -> Self::NonZero {
        NonZeroU64::new(self).unwrap()
    }
}
impl ToNonZero for NonZeroU64 {
    type NonZero = NonZeroU64;

    fn to_non_zero(self) -> Self::NonZero {
        self
    }
}
