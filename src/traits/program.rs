//! Programs as accounts support.
//! Will eventually support [`InstructionList`](crate::instruction_list::InstructionList)s and interfaces.

use crate::account_argument::AccountArgument;
use crate::instruction_list::InstructionList;
use solana_program::pubkey::Pubkey;

/// The key of a program
pub trait ProgramKey {
    /// The program's key
    const KEY: Pubkey;
}
/// A program written with `cruiser`
pub trait CruiserProgram: ProgramKey {
    /// The instruction list for this program
    type InstructionList: InstructionList;
}

/// A program on solana
pub trait Program: ProgramKey + AccountArgument {}
