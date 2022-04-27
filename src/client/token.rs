//! Client functions for the [`spl-token`] program.

use crate::client::HashedSigner;
use crate::on_chain_size::OnChainSize;
use crate::program::ProgramKey;
use crate::spl::token::{MintAccount, TokenAccount, TokenProgram};
use cruiser::SolanaInstruction;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::create_account;
use solana_sdk::signature::Signer;
use spl_token::instruction;
use std::future::Future;

/// Creates a new token account
#[allow(clippy::missing_panics_doc)]
pub async fn create_token_account<'a, F, E>(
    funder: impl Into<HashedSigner<'a>>,
    account: impl Into<HashedSigner<'a>>,
    mint: Pubkey,
    owner: Pubkey,
    rent: impl FnOnce(usize) -> F,
) -> Result<
    (
        impl IntoIterator<Item = SolanaInstruction>,
        impl IntoIterator<Item = HashedSigner<'a>>,
    ),
    E,
>
where
    F: Future<Output = Result<u64, E>>,
{
    const SPACE: usize = TokenAccount::<()>::ON_CHAIN_SIZE;

    let funder = funder.into();
    let account = account.into();
    let rent = rent(SPACE).await?;
    Ok((
        [
            create_account(
                &funder.pubkey(),
                &account.pubkey(),
                rent,
                SPACE as u64,
                &TokenProgram::<()>::KEY,
            ),
            instruction::initialize_account(
                &TokenProgram::<()>::KEY,
                &account.pubkey(),
                &mint,
                &owner,
            )
            .unwrap(),
        ],
        [funder, account],
    ))
}

/// Creates a new mint
#[allow(clippy::missing_panics_doc)]
pub async fn create_mint<'a, F, E>(
    funder: impl Into<HashedSigner<'a>>,
    account: impl Into<HashedSigner<'a>>,
    mint_authority: Pubkey,
    freeze_authority: Option<Pubkey>,
    decimals: u8,
    rent: impl FnOnce(usize) -> F,
) -> Result<
    (
        impl IntoIterator<Item = SolanaInstruction>,
        impl IntoIterator<Item = HashedSigner<'a>>,
    ),
    E,
>
where
    F: Future<Output = Result<u64, E>>,
{
    const SPACE: usize = MintAccount::<()>::ON_CHAIN_SIZE;

    let funder = funder.into();
    let account = account.into();
    let rent = rent(SPACE).await?;
    Ok((
        [
            create_account(
                &funder.pubkey(),
                &account.pubkey(),
                rent,
                SPACE as u64,
                &TokenProgram::<()>::KEY,
            ),
            instruction::initialize_mint(
                &TokenProgram::<()>::KEY,
                &account.pubkey(),
                &mint_authority,
                freeze_authority.as_ref(),
                decimals,
            )
            .unwrap(),
        ],
        [funder, account],
    ))
}

/// Mints tokens to an account
#[allow(clippy::missing_panics_doc)]
pub fn mint_to<'a>(
    mint: Pubkey,
    token_account_to: Pubkey,
    mint_authority: impl Into<HashedSigner<'a>>,
    amount: u64,
) -> (
    impl IntoIterator<Item = SolanaInstruction>,
    impl IntoIterator<Item = HashedSigner<'a>>,
) {
    let mint_authority = mint_authority.into();
    (
        [instruction::mint_to(
            &TokenProgram::<()>::KEY,
            &mint,
            &token_account_to,
            &mint_authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        [mint_authority],
    )
}

/// Transfers tokens between accounts
#[allow(clippy::missing_panics_doc)]
pub fn transfer<'a>(
    source_account: Pubkey,
    destination_account: Pubkey,
    authority: impl Into<HashedSigner<'a>>,
    amount: u64,
) -> (
    impl IntoIterator<Item = SolanaInstruction>,
    impl IntoIterator<Item = HashedSigner<'a>>,
) {
    let authority = authority.into();
    (
        [instruction::transfer(
            &TokenProgram::<()>::KEY,
            &source_account,
            &destination_account,
            &authority.pubkey(),
            &[],
            amount,
        )
        .unwrap()],
        [authority],
    )
}
