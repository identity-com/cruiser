//! Helpers for writing rust tests

use crate::instruction_list::{InstructionList, InstructionListCPI};
use crate::program::CruiserProgram;
use crate::SolanaAccountMeta;
use async_trait::async_trait;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::{RpcProgramAccountsConfig, RpcSendTransactionConfig};
use solana_client::rpc_response::Response;
use solana_program::hash::Hash;
use solana_program::pubkey::Pubkey;
use solana_sdk::account::Account;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::Signature;
use solana_sdk::signers::Signers;
use solana_sdk::transaction::Transaction;
use solana_transaction_status::TransactionStatus;
use std::error::Error;

/// Client trait for generalizing across debuggable rust tests and tests against localnet/devnet/mainnet
#[async_trait]
pub trait TestingClient {
    /// Sends a cruiser transaction
    async fn send_cruiser_transaction<'a, 'b, IL, P>(
        &'a self,
        instructions: impl Iterator<Item = &'b mut dyn InstructionListCPI<IL, AccountInfo = SolanaAccountMeta>>
            + Send
            + 'b,
        payer: &Pubkey,
        signers: &(impl 'a + Signers + Send + Sync),
        blockhash_commitment: CommitmentLevel,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, Box<dyn Error>>
    where
        IL: InstructionList + 'b,
        P: CruiserProgram<InstructionList = IL>,
    {
        let transaction = Transaction::new_signed_with_payer(
            &instructions
                .map(|i| i.instruction(&P::KEY))
                .collect::<Vec<_>>(),
            Some(payer),
            signers,
            self.get_latest_blockhash(blockhash_commitment).await?,
        );
        self.send_transaction(&transaction, config).await
    }

    /// Sends a transaction
    async fn send_transaction(
        &self,
        transaction: &Transaction,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, Box<dyn Error>>;

    /// Gets the status of a set of signatures
    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> Result<Response<Vec<Option<TransactionStatus>>>, Box<dyn Error>>;

    /// Gets an individual account
    async fn get_account(
        &self,
        account: &Pubkey,
        commitment: CommitmentLevel,
    ) -> Result<Response<Option<Account>>, Box<dyn Error>>;

    /// Gets the latest blockhash
    async fn get_latest_blockhash(
        &self,
        commitment: CommitmentLevel,
    ) -> Result<Hash, Box<dyn Error>>;

    /// Gets some program accounts
    async fn get_program_accounts(
        &self,
        program: &Pubkey,
        config: RpcProgramAccountsConfig,
    ) -> Result<Vec<(Pubkey, Account)>, Box<dyn Error>>;

    /// Returns whether a blockhash is valid
    async fn is_blockhash_valid(
        &self,
        hash: &Hash,
        commitment: CommitmentLevel,
    ) -> Result<bool, Box<dyn Error>>;
}
#[async_trait]
impl TestingClient for RpcClient {
    async fn send_transaction(
        &self,
        transaction: &Transaction,
        config: RpcSendTransactionConfig,
    ) -> Result<Signature, Box<dyn Error>> {
        Ok(self
            .send_transaction_with_config(transaction, config)
            .await?)
    }

    async fn get_signature_statuses(
        &self,
        signatures: &[Signature],
    ) -> Result<Response<Vec<Option<TransactionStatus>>>, Box<dyn Error>> {
        Ok(self.get_signature_statuses(signatures).await?)
    }

    async fn get_account(
        &self,
        account: &Pubkey,
        commitment: CommitmentLevel,
    ) -> Result<Response<Option<Account>>, Box<dyn Error>> {
        Ok(self
            .get_account_with_commitment(account, CommitmentConfig { commitment })
            .await?)
    }

    async fn get_latest_blockhash(
        &self,
        commitment: CommitmentLevel,
    ) -> Result<Hash, Box<dyn Error>> {
        Ok(self
            .get_latest_blockhash_with_commitment(CommitmentConfig { commitment })
            .await?
            .0)
    }

    async fn get_program_accounts(
        &self,
        program: &Pubkey,
        config: RpcProgramAccountsConfig,
    ) -> Result<Vec<(Pubkey, Account)>, Box<dyn Error>> {
        Ok(self
            .get_program_accounts_with_config(program, config)
            .await?)
    }

    async fn is_blockhash_valid(
        &self,
        hash: &Hash,
        commitment: CommitmentLevel,
    ) -> Result<bool, Box<dyn Error>> {
        Ok(self
            .is_blockhash_valid(hash, CommitmentConfig { commitment })
            .await?)
    }
}
