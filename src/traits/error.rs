//! Custom Error support.

pub use cruiser_derive::Error;

use solana_program::program_error::ProgramError;
use std::fmt::{Debug, Display, Formatter};
use std::ops::{Deref, DerefMut};

use crate::msg;
use crate::solana_program::pubkey::PubkeyError;

/// A version of [`Result`] returned by many [`cruiser`] functions.
pub type CruiserResult<T = ()> = Result<T, CruiserError>;

/// An error that is returned by many [`cruiser`] functions
#[derive(Debug)]
pub struct CruiserError(Box<dyn Error>);
impl Deref for CruiserError {
    type Target = Box<dyn Error>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for CruiserError {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl Display for CruiserError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0.message())
    }
}
impl std::error::Error for CruiserError {}
impl<T> From<T> for CruiserError
where
    T: Error + 'static,
{
    fn from(from: T) -> Self {
        Self(Box::new(from))
    }
}
impl From<std::io::Error> for CruiserError {
    fn from(from: std::io::Error) -> Self {
        ProgramError::from(from).into()
    }
}
impl From<PubkeyError> for CruiserError {
    fn from(from: PubkeyError) -> Self {
        match from {
            PubkeyError::MaxSeedLengthExceeded => msg!("PubkeyError::MaxSeedLengthExceeded"),
            PubkeyError::InvalidSeeds => msg!("PubkeyError::InvalidSeeds"),
            PubkeyError::IllegalOwner => msg!("PubkeyError::IllegalOwner"),
        };
        ProgramError::InvalidSeeds.into()
    }
}

/// An error that can be returned on the chain
pub trait Error: Debug {
    /// The message the error represents
    fn message(&self) -> String;
    /// Turns this into a returnable error
    fn to_program_error(&self) -> ProgramError;
}
impl Error for ProgramError {
    fn message(&self) -> String {
        format!("{}", self)
    }

    fn to_program_error(&self) -> ProgramError {
        self.clone()
    }
}
