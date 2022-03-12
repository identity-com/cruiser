//! Programs as accounts support.
//! Will eventually support [`InstructionList`](crate::instruction_list::InstructionList)s and interfaces.

use crate::account_argument::AccountArgument;
use solana_program::pubkey::Pubkey;

/// A program on solana
pub trait Program: AccountArgument {
    /// The program's key
    const KEY: Pubkey;
}
