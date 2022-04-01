//! Client functions for the system program

use crate::client::HashedSigner;
use crate::SolanaInstruction;
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
) -> (
    impl IntoIterator<Item = SolanaInstruction>,
    impl IntoIterator<Item = HashedSigner<'a>>,
) {
    let from = from.into();
    let to = to.into();
    (
        [system_instruction::create_account(
            &from.pubkey(),
            &to.pubkey(),
            lamports,
            space,
            &owner,
        )],
        [from, to],
    )
}

/// Transfers SOL from a system account
pub fn transfer<'a>(
    from: impl Into<HashedSigner<'a>>,
    to: Pubkey,
    lamports: u64,
) -> (
    impl IntoIterator<Item = SolanaInstruction>,
    impl IntoIterator<Item = HashedSigner<'a>>,
) {
    let from = from.into();
    (
        [system_instruction::transfer(&from.pubkey(), &to, lamports)],
        [from],
    )
}
