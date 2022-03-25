use crate::account_argument::AccountArgument;
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;

/// Validates this argument using data `Arg`. The seconds step in the instruction lifecycle.
pub trait ValidateArgument<AI, Arg>: Sized + AccountArgument<AI> {
    /// Runs validation on this account with data `Arg`.
    ///
    /// Ordering for wrapping should be to call `validate` on the wrapped type first.
    fn validate(&mut self, program_id: &Pubkey, arg: Arg) -> CruiserResult<()>;
}
