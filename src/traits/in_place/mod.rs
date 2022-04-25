//! Types for manipulating account dat in-place (aka zero-copy)

mod array;
// mod prim;
mod pubkey;
// mod static_size_vec;
// mod unit;
mod properties;

pub use array::*;
// pub use prim::*;
pub use pubkey::*;
// pub use static_size_vec::*;

pub use properties::*;
// pub use unit::*;

use crate::CruiserResult;
use std::ops::{Deref, DerefMut};

/// In-place account data access
pub trait InPlace {
    /// The type accessed
    type Access<A>;
}
/// In place item can be created with arg `C`
pub trait InPlaceCreate<C = ()>: InPlace {
    /// Create a new instance of `Self::Access` with the given argument
    fn create_with_arg<A: DerefMut<Target = [u8]>>(data: A, arg: C) -> CruiserResult;
}
/// In place item can be read with arg `R`
pub trait InPlaceRead<R = ()>: InPlace {
    /// Reads the access type from data and an arg
    fn read_with_arg<A: Deref<Target = [u8]>>(data: A, arg: R) -> CruiserResult<Self::Access<A>>;
}
/// In place item can be written with arg `W`
pub trait InPlaceWrite<W = ()>: InPlace {
    /// Writes the access type to data and an arg
    fn write_with_arg<A: DerefMut<Target = [u8]>>(
        data: A,
        arg: W,
    ) -> CruiserResult<Self::Access<A>>;
}

/// Access the data of an in-place item
pub trait InPlaceGetData {
    /// The accessor, usually is `A`
    type Accessor;

    /// Gets the data
    fn get_raw_data(&self) -> &[u8]
    where
        Self::Accessor: Deref<Target = [u8]>;
    /// Gets the data mutably
    fn get_raw_data_mut(&mut self) -> &mut [u8]
    where
        Self::Accessor: DerefMut<Target = [u8]>;
}
impl<T> const InPlaceGetData for T
where
    T: Deref<Target = [u8]>,
{
    type Accessor = T;

    fn get_raw_data(&self) -> &[u8]
    where
        Self::Accessor: ~const Deref<Target = [u8]>,
    {
        &*self
    }
    fn get_raw_data_mut(&mut self) -> &mut [u8]
    where
        Self::Accessor: ~const DerefMut<Target = [u8]>,
    {
        &mut *self
    }
}
