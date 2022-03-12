//! Indexing helpers, specifically for [`AllAny`].

use crate::CruiserResult;
use std::fmt::Debug;

/// Implementing [`MultiIndexable<AllAny>`](crate::account_argument::MultiIndexable) allows for simpler signer, writable, and owner checks with [`AccountArgument`](cruiser_derive::AccountArgument) deriving
#[derive(Copy, Clone, Debug)]
pub enum AllAny {
    /// An index that checks that all accounts return [`true`], [`true`] on empty.
    All,
    /// An index that checks that any accounts return [`false`], [`false`] on empty.
    NotAll,
    /// An index that checks that any accounts return [`true`], [`false`] on empty.
    Any,
    /// An index that checks that none of the accounts return [`true`], [`true`] on empty.
    NotAny,
}
impl AllAny {
    /// Runs a function against an iterator following the strategy determined by `self`.
    pub fn run_func<T>(
        self,
        iter: impl IntoIterator<Item = T>,
        func: impl FnMut(T) -> CruiserResult<bool>,
    ) -> CruiserResult<bool> {
        Ok(self.is_not()
            ^ if self.is_all() {
                Self::result_all(iter.into_iter(), func)?
            } else {
                Self::option_any(iter.into_iter(), func)?
            })
    }

    fn result_all<T>(
        iter: impl Iterator<Item = T>,
        mut func: impl FnMut(T) -> CruiserResult<bool>,
    ) -> CruiserResult<bool> {
        for item in iter {
            if !func(item)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn option_any<T>(
        iter: impl Iterator<Item = T>,
        mut func: impl FnMut(T) -> CruiserResult<bool>,
    ) -> CruiserResult<bool> {
        for item in iter {
            if func(item)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns [`true`] if is [`AllAny::All`] or [`AllAny::NotAll`], [`false`] otherwise
    #[must_use]
    pub const fn is_all(self) -> bool {
        match self {
            Self::All | Self::NotAll => true,
            Self::Any | Self::NotAny => false,
        }
    }

    /// Returns [`true`] if is [`AllAny::Any`] or [`AllAny::NotAny`], [`false`] otherwise
    #[must_use]
    pub const fn is_any(self) -> bool {
        match self {
            Self::All | Self::NotAll => false,
            Self::Any | Self::NotAny => true,
        }
    }

    /// Returns [`true`] if is [`AllAny::NotAll`] or [`AllAny::NotAny`], [`false`] otherwise
    #[must_use]
    pub const fn is_not(self) -> bool {
        match self {
            Self::All | Self::Any => false,
            Self::NotAll | Self::NotAny => true,
        }
    }
}
