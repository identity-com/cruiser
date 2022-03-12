use crate::account_argument::AccountArgument;
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;

/// Validates this argument using data `A`. The seconds step in the instruction lifecycle.
pub trait ValidateArgument<A>: Sized + AccountArgument {
    /// Runs validation on this account with data `A`.
    ///
    /// Ordering for wrapping should be to call `validate` on the wrapped type first.
    fn validate(&mut self, program_id: &'static Pubkey, arg: A) -> CruiserResult<()>;
}
