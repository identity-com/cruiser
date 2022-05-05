use cruiser::client::token::{create_mint, create_token_account, mint_to};
use cruiser::client::{ConfirmationResult, TransactionBuilder};
use cruiser::solana_client::nonblocking::rpc_client::RpcClient;
use cruiser::solana_client::rpc_config::{RpcSendTransactionConfig, RpcTransactionConfig};
use cruiser::solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use cruiser::solana_sdk::native_token::LAMPORTS_PER_SOL;
use cruiser::solana_sdk::signature::{Keypair, Signer};
use escrow_example::client::init_escrow;
use futures::executor::block_on;
use futures::try_join;
use futures::{select_biased, FutureExt};
use reqwest::Client;
use std::error::Error;
use std::panic;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::process::Command;
use tokio::sync::Mutex;
use tokio::time::sleep;

#[tokio::test]
async fn main_flow() -> Result<(), Box<dyn Error>> {
    let deploy_dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .parent()
        .unwrap()
        .join("deploy");
    let build = Command::new("cargo")
        .env("RUSTFLAGS", "-D warnings")
        .arg("build-bpf")
        .arg("--workspace")
        .spawn()?
        .wait()
        .await?;
    if !build.success() {
        return Err(build.to_string().into());
    }
    let program_id = Keypair::new().pubkey();
    println!("Program ID: `{}`", program_id);

    let mut local_validator = Command::new("solana-test-validator");
    local_validator
        .arg("-r")
        .arg("--bpf-program")
        .arg(program_id.to_string())
        .arg(deploy_dir.join("escrow_example.so"))
        .arg("--deactivate-feature")
        .arg("5ekBxc8itEnPv4NzGJtr8BVVQLNMQuLMNQQj7pHoLNZ9") // transaction wide compute cap
        .arg("--deactivate-feature")
        .arg("75m6ysz33AfLA5DDEzWM1obBrnPQRSsdVQ2nRmc8Vuu1") // support account data reallocation
        .arg("--ledger")
        .arg(Path::new(env!("CARGO_TARGET_TMPDIR")).join("test_ledger_8899"));
    println!("Running {:?}", local_validator);
    let local_validator = Arc::new(Mutex::new(local_validator.spawn()?));

    let local_validator_clone = local_validator.clone();
    let hook = panic::take_hook();
    panic::set_hook(Box::new(move |panic_info| {
        println!("{}", panic_info);
        let local_validator = local_validator_clone.lock();
        if let Err(error) = block_on(async move { local_validator.await.kill().await }) {
            eprintln!("Error killing validator: {}", error);
        }
        hook(panic_info);
    }));

    let test_func = {
        let local_validator = local_validator.clone();
        async move {
            let run_local_validator = async {
                let client = Client::new();
                loop {
                    if let Some(exit_status) = local_validator.lock().await.try_wait()? {
                        return Result::<_, Box<dyn Error>>::Err(
                            format!("Local validator exited early: {}", exit_status).into(),
                        );
                    }
                    if client
                        .get("http://localhost:8899/health")
                        .send()
                        .await
                        .map_or(false, |res| res.status().is_success())
                    {
                        break;
                    }
                    sleep(Duration::from_millis(500)).await;
                }
                Ok(())
            };
            (select_biased! {
                res = run_local_validator.fuse() => res,
                _ = sleep(Duration::from_secs(5)).fuse() => Err("Local Validator Timed out!".into())
            })?;

            let rpc = RpcClient::new_with_commitment(
                "http://localhost:8899".to_string(),
                CommitmentConfig::confirmed(),
            );

            let funder = Keypair::new();
            let blockhash = rpc.get_latest_blockhash().await?;
            let sig = rpc
                .request_airdrop_with_blockhash(&funder.pubkey(), LAMPORTS_PER_SOL * 2, &blockhash)
                .await?;
            rpc.confirm_transaction_with_spinner(&sig, &blockhash, CommitmentConfig::confirmed())
                .await?;

            let send_mint = Keypair::new();
            let receive_mint = Keypair::new();
            let send_token_account = Keypair::new();

            let rent = |size: usize| rpc.get_minimum_balance_for_rent_exemption(size);
            let send_config = RpcSendTransactionConfig {
                skip_preflight: false,
                preflight_commitment: Some(CommitmentLevel::Confirmed),
                encoding: None,
                max_retries: None,
            };

            let (create_send_mint, create_receive_mint, create_send_token_account) = try_join!(
                create_mint(&funder, &send_mint, funder.pubkey(), None, 0, rent),
                create_mint(&funder, &receive_mint, funder.pubkey(), None, 0, rent),
                create_token_account(
                    &funder,
                    &send_token_account,
                    send_mint.pubkey(),
                    funder.pubkey(),
                    rent,
                ),
            )?;

            let (sig, result) = TransactionBuilder::new(&funder)
                .signed_instructions(create_send_mint)
                .signed_instructions(create_receive_mint)
                .signed_instructions(create_send_token_account)
                .signed_instructions(mint_to(
                    send_mint.pubkey(),
                    send_token_account.pubkey(),
                    &funder,
                    100,
                ))
                .send_and_confirm_transaction(
                    &rpc,
                    send_config,
                    CommitmentConfig::confirmed(),
                    Duration::from_millis(500),
                )
                .await?;
            match result {
                ConfirmationResult::Success => {}
                ConfirmationResult::Failure(error) => return Err(error.into()),
                ConfirmationResult::Dropped => return Err("Transaction dropped".into()),
            }
            println!(
                "Initialize logs: {:#?}",
                rpc.get_transaction_with_config(
                    &sig,
                    RpcTransactionConfig {
                        encoding: None,
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: None
                    }
                )
                .await?
                .transaction
                .meta
                .unwrap()
                .log_messages
            );

            let (sig, result) = TransactionBuilder::new(&funder)
                .signed_instructions(
                    init_escrow(
                        program_id,
                        100,
                        &funder,
                        funder.pubkey(),
                        send_token_account.pubkey(),
                        receive_mint.pubkey(),
                        |size| rpc.get_minimum_balance_for_rent_exemption(size),
                    )
                    .await?,
                )
                .send_and_confirm_transaction(
                    &rpc,
                    send_config,
                    CommitmentConfig::confirmed(),
                    Duration::from_millis(500),
                )
                .await?;
            match result {
                ConfirmationResult::Success => {}
                ConfirmationResult::Failure(error) => return Err(error.into()),
                ConfirmationResult::Dropped => return Err("Transaction dropped".into()),
            }

            println!(
                "Initialize logs: {:#?}",
                rpc.get_transaction_with_config(
                    &sig,
                    RpcTransactionConfig {
                        encoding: None,
                        commitment: Some(CommitmentConfig::confirmed()),
                        max_supported_transaction_version: None
                    }
                )
                .await?
                .transaction
                .meta
                .unwrap()
                .log_messages
            );
            Ok(())
        }
    };

    let out = test_func.await;

    let mut local = local_validator.lock().await;
    local.start_kill()?;
    local.wait().await?;

    out
}
