//! TODO: Write big docs here

mod from_accounts;
mod multi_indexable;
mod single_indexable;
mod validate_argument;

pub use from_accounts::*;
pub use multi_indexable::*;
pub use single_indexable::*;
pub use validate_argument::*;

pub use cruiser_derive::AccountArgument;

use solana_program::pubkey::Pubkey;

use crate::CruiserResult;

/// An argument that can come from [`AccountInfo`](crate::AccountInfo)s and data using [`FromAccounts`].
/// Can be automatically derived.
pub trait AccountArgument: Sized {
    /// The final step in the instruction lifecycle, performing any cleanup operations or writes back.
    fn write_back(self, program_id: &'static Pubkey) -> CruiserResult<()>;
    /// Passes all the account keys to a given function.
    fn add_keys(&self, add: impl FnMut(&'static Pubkey) -> CruiserResult<()>) -> CruiserResult<()>;
    /// Collects all the account keys into a [`Vec`].
    fn keys(&self) -> CruiserResult<Vec<&'static Pubkey>> {
        let mut out = Vec::new();
        self.add_keys(|key| {
            out.push(key);
            Ok(())
        })?;
        Ok(out)
    }
}
