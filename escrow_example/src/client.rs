use crate::cpi::InitEscrowCPI;
use crate::Pubkey;
use cruiser::client::InstructionSet;
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
) -> Result<InstructionSet<'a>, E>
where
    F: Future<Output = Result<u64, E>>,
{
    let initializer_token_account = Keypair::new();
    let initializer_token_account_key = initializer_token_account.pubkey();
    let escrow_account = Keypair::new();

    let mut out = token::create_token_account(
        funder,
        initializer_token_account,
        receive_mint,
        initializer,
        rent,
    )
    .await?;
    out.add_set(InstructionSet {
        instructions: vec![
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
            .instruction(&SolanaAccountMeta::new_readonly(program_id, false))
            .instruction,
        ],
        signers: once(escrow_account.into()).collect(),
    });
    Ok(out)
}
