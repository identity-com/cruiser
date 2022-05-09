//! Types for manipulating account dat in-place (aka zero-copy)

mod array;
mod prim;
mod properties;
mod pubkey;
// mod static_size_vec;
mod unit;

pub use array::*;
pub use prim::*;
pub use pubkey::*;
// pub use static_size_vec::*;
pub use properties::*;
pub use unit::*;

pub use cruiser_derive::InPlace;

#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
use crate::util::AdvanceArray;
use crate::util::{MappableRef, MappableRefMut, TryMappableRef, TryMappableRefMut};
use crate::CruiserResult;
#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
use cruiser::on_chain_size::OnChainSize;
use std::ops::{Deref, DerefMut};

/// In-place account data access
pub trait InPlace {
    /// The type accessed
    type Access<'a, A>
    where
        Self: 'a,
        A: 'a + MappableRef + TryMappableRef;
    /// The type accessed mutably
    type AccessMut<'a, A>
    where
        Self: 'a,
        A: 'a + MappableRef + TryMappableRef + MappableRefMut + TryMappableRefMut,
    = Self::Access<'a, A>;
}

/// In place item can be created with arg `C`
pub trait InPlaceCreate<C = ()>: InPlace {
    /// Create a new instance with the given argument
    fn create_with_arg<A: DerefMut<Target = [u8]>>(data: A, arg: C) -> CruiserResult;
}

#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
/// In place item that is statically sized and can be created with arg `C`
pub trait InPlaceCreateSized<C = ()>: InPlace + OnChainSize {
    /// Create a new instance with the given argument
    fn create_with_arg_sized<A: DerefMut<Target = [u8; Self::ON_CHAIN_SIZE]>>(
        data: A,
        arg: C,
    ) -> CruiserResult;
}
#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
impl<T, C> InPlaceCreate<C> for T
where
    T: InPlaceCreateSized<C>,
    [(); Self::ON_CHAIN_SIZE]:,
{
    default fn create_with_arg<A: DerefMut<Target = [u8]>>(mut data: A, arg: C) -> CruiserResult {
        let data: &mut [_; T::ON_CHAIN_SIZE] = <&mut [u8]>::try_advance_array(&mut &mut *data)?;
        T::create_with_arg_sized(data, arg)
    }
}

/// In place item can be read with arg `R`
pub trait InPlaceRead<R = ()>: InPlace {
    /// Reads the access type from data and an arg
    fn read_with_arg<'a, A>(data: A, arg: R) -> CruiserResult<Self::Access<'a, A>>
    where
        Self: 'a,
        A: 'a + Deref<Target = [u8]> + MappableRef + TryMappableRef;
}
#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
/// In place item that is statically sized and can be read with arg `R`
pub trait InPlaceReadSized<R = ()>: InPlace + OnChainSize {
    /// Reads the access type from data and an arg
    fn read_with_arg_sized<'a, A>(data: A, arg: R) -> CruiserResult<Self::Access<'a, A>>
    where
        A: Deref<Target = [u8; Self::ON_CHAIN_SIZE]> + MappableRef + TryMappableRef;
}
/// In place item can be written with arg `W`
pub trait InPlaceWrite<W = ()>: InPlace {
    /// Writes the access type to data and an arg
    fn write_with_arg<'a, A>(data: A, arg: W) -> CruiserResult<Self::AccessMut<'a, A>>
    where
        Self: 'a,
        A: 'a
            + DerefMut<Target = [u8]>
            + MappableRef
            + TryMappableRef
            + MappableRefMut
            + TryMappableRefMut;
}

#[cfg(all(feature = "unstable", VERSION_GREATER_THAN_59))]
/// In place item that is statically sized and can be written with arg `W`
pub trait InPlaceWriteSized<W = ()>: InPlace + OnChainSize {
    /// Writes the access type to data and an arg
    fn write_with_arg_sized<'a, A>(data: A, arg: W) -> CruiserResult<Self::AccessMut<'a, A>>
    where
        A: DerefMut<Target = [u8; Self::ON_CHAIN_SIZE]>
            + MappableRef
            + TryMappableRef
            + MappableRefMut
            + TryMappableRefMut;
}
