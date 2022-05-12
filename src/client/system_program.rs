//! Client functions for the system program

use crate::client::HashedSigner;
use crate::prelude::InstructionSet;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction;
use solana_sdk::signature::Signer;

/// Creates a new account with an owner
pub fn create_account<'a>(
    from: impl Into<HashedSigner<'a>>,
    to: impl Into<HashedSigner<'a>>,
    lamports: u64,
    space: u64,
    owner: Pubkey,
) -> InstructionSet<'a> {
    let from = from.into();
    let to = to.into();
    InstructionSet {
        instructions: vec![system_instruction::create_account(
            &from.pubkey(),
            &to.pubkey(),
            lamports,
            space,
            &owner,
        )],
        signers: [from, to].into_iter().collect(),
    }
}

/// Transfers SOL from a system account
pub fn transfer<'a>(
    from: impl Into<HashedSigner<'a>>,
    to: Pubkey,
    lamports: u64,
) -> InstructionSet<'a> {
    let from = from.into();
    InstructionSet {
        instructions: vec![system_instruction::transfer(&from.pubkey(), &to, lamports)],
        signers: [from].into_iter().collect(),
    }
}
