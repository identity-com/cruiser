use crate::GeneratorResult;
use std::fmt::Debug;

/// An index that checks that all accounts return [`true`], [`true`] on empty.
/// See [`AllAny`].
#[derive(Copy, Clone, Default, Debug)]
pub struct All;
/// An index that checks that any accounts return [`false`], [`false`] on empty.
/// See [`AllAny`].
#[derive(Copy, Clone, Default, Debug)]
pub struct NotAll;
/// An index that checks that any accounts return [`true`], [`false`] on empty.
/// See [`AllAny`].
#[derive(Copy, Clone, Default, Debug)]
pub struct Any;
/// An index that checks that none of the accounts return [`true`], [`true`] on empty.
/// See [`AllAny`].
#[derive(Copy, Clone, Default, Debug)]
pub struct NotAny;

/// Returns [`None`] if any in range return [`None`] unless already short circuited.
/// If you want to implement for `MultiIndexableAccountArgument<AllAny>`, implement for `MultiIndexableAccountArgument<(AllAny, ())>` instead so blankets work better.
/// Implementing for `MultiIndexableAccountArgument<(AllAny, I)>` also implements for `MultiIndexableAccountArgument<(X, I)>` where X is [`All`], [`NotAll`], [`Any`], and [`NotAny`].
#[derive(Copy, Clone, Debug)]
pub enum AllAny {
    /// Returns [`true`] on empty
    All,
    /// Returns [`false`] on empty
    NotAll,
    /// Returns [`false`] on empty
    Any,
    /// Returns [`true`] on empty
    NotAny,
}
impl AllAny {
    /// Runs a function against an iterator following the strategy determined by `self`.
    pub fn run_func<T>(
        self,
        iter: impl IntoIterator<Item = T>,
        func: impl FnMut(T) -> GeneratorResult<bool>,
    ) -> GeneratorResult<bool> {
        Ok(self.is_not()
            ^ if self.is_all() {
                Self::result_all(iter.into_iter(), func)?
            } else {
                Self::option_any(iter.into_iter(), func)?
            })
    }

    fn result_all<T>(
        iter: impl Iterator<Item = T>,
        mut func: impl FnMut(T) -> GeneratorResult<bool>,
    ) -> GeneratorResult<bool> {
        for item in iter {
            if !func(item)? {
                return Ok(false);
            }
        }
        Ok(true)
    }
    fn option_any<T>(
        iter: impl Iterator<Item = T>,
        mut func: impl FnMut(T) -> GeneratorResult<bool>,
    ) -> GeneratorResult<bool> {
        for item in iter {
            if func(item)? {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// Returns [`true`] if is [`All`] or [`NotAll`], [`false`] otherwise
    #[must_use]
    pub const fn is_all(self) -> bool {
        match self {
            Self::All | Self::NotAll => true,
            Self::Any | Self::NotAny => false,
        }
    }

    /// Returns [`true`] if is [`Any`] or [`NotAny`], [`false`] otherwise
    #[must_use]
    pub const fn is_any(self) -> bool {
        match self {
            Self::All | Self::NotAll => false,
            Self::Any | Self::NotAny => true,
        }
    }

    /// Returns [`true`] if is [`NotAll`] or [`NotAny`], [`false`] otherwise
    #[must_use]
    pub const fn is_not(self) -> bool {
        match self {
            Self::All | Self::Any => false,
            Self::NotAll | Self::NotAny => true,
        }
    }
}
impl From<AllAny> for All {
    fn from(_: AllAny) -> Self {
        Self
    }
}
impl From<All> for AllAny {
    fn from(_: All) -> Self {
        Self::All
    }
}
impl From<AllAny> for NotAll {
    fn from(_: AllAny) -> Self {
        Self
    }
}
impl From<NotAll> for AllAny {
    fn from(_: NotAll) -> Self {
        Self::NotAll
    }
}
impl From<AllAny> for Any {
    fn from(_: AllAny) -> Self {
        Self
    }
}
impl From<Any> for AllAny {
    fn from(_: Any) -> Self {
        Self::Any
    }
}
impl From<AllAny> for NotAny {
    fn from(_: AllAny) -> Self {
        Self
    }
}
impl From<NotAny> for AllAny {
    fn from(_: NotAny) -> Self {
        Self::NotAny
    }
}
