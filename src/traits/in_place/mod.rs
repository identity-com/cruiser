//! Types for manipulating account dat in-place (aka zero-copy)

mod array;
mod prim;
mod pubkey;
mod static_size_vec;
mod unit;

pub use array::*;
pub use prim::*;
pub use pubkey::*;
pub use static_size_vec::*;
pub use unit::*;

use crate::CruiserResult;

/// In-place account data access
pub trait InPlace<'a> {
    /// The type accessed
    type Access;
    /// The type accessed mutably
    type AccessMut;
}
/// In place item can be created with arg `C`
pub trait InPlaceCreate<'a, C>: InPlace<'a> {
    /// Create a new instance of `Self::Access` with the given argument
    fn create_with_arg(data: &mut [u8], arg: C) -> CruiserResult;
}
/// In place item can be read with arg `R`
pub trait InPlaceRead<'a, R>: InPlace<'a> {
    /// Reads the access type from data and an arg
    fn read_with_arg(data: &'a [u8], arg: R) -> CruiserResult<Self::Access>;
}
/// In place item can be written with arg `W`
pub trait InPlaceWrite<'a, W>: InPlace<'a> {
    /// Writes the access type to data and an arg
    fn write_with_arg(data: &'a mut [u8], arg: W) -> CruiserResult<Self::AccessMut>;
}

/// An access that can get a value out
pub trait InPlaceGet<V> {
    /// Gets a value out
    fn get(&self) -> CruiserResult<V>;
}
/// An access that can be set to a value
pub trait InPlaceSet<V> {
    /// Sets this to a value
    fn set(&mut self, val: V) -> CruiserResult;
}
