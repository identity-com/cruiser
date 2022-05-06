use crate::cpi::InitEscrowCPI;
use crate::Pubkey;
use cruiser::prelude::*;
use std::future::Future;
use std::iter::once;

pub async fn init_escrow<'a, E, F>(
    program_id: Pubkey,
    amount: u64,
    funder: impl Into<HashedSigner<'a>>,
    initializer: Pubkey,
    send_account: Pubkey,
    receive_mint: Pubkey,
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
    let initializer_token_account = Keypair::new();
    let initializer_token_account_key = initializer_token_account.pubkey();
    let escrow_account = Keypair::new();

    let out = token::create_token_account(
        funder,
        initializer_token_account,
        receive_mint,
        initializer,
        rent,
    )
    .await?;
    Ok((
        out.0.into_iter().chain(once(
            InitEscrowCPI::new(
                SolanaAccountMeta::new_readonly(initializer, true),
                SolanaAccountMeta::new(send_account, false),
                SolanaAccountMeta::new_readonly(initializer_token_account_key, false),
                SolanaAccountMeta::new(escrow_account.pubkey(), true),
                SolanaAccountMeta::new_readonly(TokenProgram::<()>::KEY, false),
                SolanaAccountMeta::new_readonly(SystemProgram::<()>::KEY, false),
                amount,
            )
            .unwrap()
            .instruction(&program_id),
        )),
        out.1.into_iter().chain(once(escrow_account.into())),
    ))
}
