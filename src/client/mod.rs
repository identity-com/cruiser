//! Functions to make client building easier

pub mod system_program;
#[cfg(feature = "spl-token")]
pub mod token;

use crate::SolanaInstruction;
use solana_client::client_error::Result as ClientResult;
use solana_client::nonblocking::rpc_client::RpcClient;
use solana_client::rpc_config::RpcSendTransactionConfig;
use solana_program::hash::Hash;
use solana_program::pubkey::Pubkey;
use solana_sdk::commitment_config::{CommitmentConfig, CommitmentLevel};
use solana_sdk::signature::{Keypair, Signature, SignerError};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::{Transaction, TransactionError};
use solana_transaction_status::TransactionConfirmationStatus;
use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hasher;
use std::iter::once;
use std::ops::Deref;
use std::time::Duration;
use tokio::time::sleep;

/// Transaction building helper
#[derive(Debug)]
pub struct TransactionBuilder<'a> {
    /// The instructions for this transaction
    pub instructions: Vec<SolanaInstruction>,
    /// The signers for this transaction
    pub signers: HashSet<HashedSigner<'a>>,
    /// The payer for this transaction
    pub payer: Pubkey,
}
impl<'a> TransactionBuilder<'a> {
    /// Creates a new [`TransactionBuilder`] with a payer
    #[must_use]
    pub fn new<S>(payer: S) -> Self
    where
        HashedSigner<'a>: From<S>,
    {
        let payer = HashedSigner::from(payer);
        Self {
            instructions: Vec::new(),
            payer: payer.pubkey(),
            signers: once(payer).collect(),
        }
    }

    /// Adds an instruction to this [`TransactionBuilder`]
    pub fn instruction(&mut self, instruction: SolanaInstruction) -> &mut Self {
        self.instructions.push(instruction);
        self
    }
    /// Adds many instructions to this [`TransactionBuilder`]
    pub fn instructions(
        &mut self,
        instructions: impl IntoIterator<Item = SolanaInstruction>,
    ) -> &mut Self {
        self.instructions.extend(instructions);
        self
    }

    /// Adds a signer to this [`TransactionBuilder`]. Can add the same signer twice, will only sign once.
    pub fn signer<S>(&mut self, signer: S) -> &mut Self
    where
        HashedSigner<'a>: From<S>,
    {
        self.signers.insert(signer.into());
        self
    }
    /// Adds many signers to this [`TransactionBuilder`]. Can add the same signer twice, will only sign once.
    pub fn signers<S>(&mut self, signers: impl IntoIterator<Item = S>) -> &mut Self
    where
        HashedSigner<'a>: From<S>,
    {
        self.signers
            .extend(signers.into_iter().map(HashedSigner::from));
        self
    }

    /// Adds instructions and signers to this [`TransactionBuilder`].
    /// Designed to be used with client functions.
    pub fn signed_instructions<S>(
        &mut self,
        instructions: (
            impl IntoIterator<Item = SolanaInstruction>,
            impl IntoIterator<Item = S>,
        ),
    ) -> &mut Self
    where
        HashedSigner<'a>: From<S>,
    {
        self.instructions(instructions.0).signers(instructions.1)
    }

    /// Turns this into a transaction
    #[must_use]
    pub fn to_transaction(&self, recent_blockhash: Hash) -> Transaction {
        Transaction::new_signed_with_payer(
            &self.instructions,
            Some(&self.payer),
            &self.signers.iter().collect::<Vec<_>>(),
            recent_blockhash,
        )
    }

    /// Sends and confirms this transaction
    pub async fn send_and_confirm_transaction(
        &self,
        client: &RpcClient,
        config: RpcSendTransactionConfig,
        commitment: CommitmentConfig,
        loop_rate: Duration,
    ) -> ClientResult<(Signature, ConfirmationResult)> {
        let (sig, last_valid_block_height) = self.send_transaction(client, config).await?;
        Self::confirm_transaction(sig, last_valid_block_height, client, commitment, loop_rate)
            .await
            .map(|result| (sig, result))
    }

    /// Executes this using the given client and config
    pub async fn send_transaction(
        &self,
        client: &RpcClient,
        config: RpcSendTransactionConfig,
    ) -> ClientResult<(Signature, u64)> {
        let (block_hash, last_valid_block_height) = client
            .get_latest_blockhash_with_commitment(CommitmentConfig::processed())
            .await?;
        let transaction = self.to_transaction(block_hash);
        client
            .send_transaction_with_config(&transaction, config)
            .await
            .map(|sig| (sig, last_valid_block_height))
    }

    /// Confirms a given transaction signature
    #[allow(clippy::missing_panics_doc)]
    pub async fn confirm_transaction(
        signature: Signature,
        last_valid_block_height: u64,
        client: &RpcClient,
        commitment: CommitmentConfig,
        loop_rate: Duration,
    ) -> ClientResult<ConfirmationResult> {
        let mut found_block = false;
        loop {
            sleep(loop_rate).await;
            let mut status = client.get_signature_statuses(&[signature]).await?;
            assert_eq!(status.value.len(), 1, "Expected one status");
            let status = status.value.remove(0).unwrap();
            if let Some(confirmation_status) = status.confirmation_status {
                found_block = true;
                if OrderedConfirmationStatus(confirmation_status) >= commitment {
                    return Ok(match status.err {
                        None => ConfirmationResult::Success,
                        Some(error) => ConfirmationResult::Failure(error),
                    });
                }
            }
            if client
                .get_block_height_with_commitment(if found_block {
                    commitment
                } else {
                    CommitmentConfig::processed()
                })
                .await?
                >= last_valid_block_height
            {
                return Ok(ConfirmationResult::Dropped);
            }
        }
    }
}

/// The result of confirming a transaction
#[must_use]
#[derive(Debug, Clone)]
pub enum ConfirmationResult {
    /// Transaction succeeded
    Success,
    /// Transaction failed
    Failure(TransactionError),
    /// Transaction was dropped
    Dropped,
}

trait ToConfirmationStatus {
    fn to_confirmation_status(&self) -> TransactionConfirmationStatus;
}
impl ToConfirmationStatus for CommitmentConfig {
    fn to_confirmation_status(&self) -> TransactionConfirmationStatus {
        #[allow(clippy::wildcard_in_or_patterns)]
        match self.commitment {
            CommitmentLevel::Processed => TransactionConfirmationStatus::Processed,
            CommitmentLevel::Confirmed => TransactionConfirmationStatus::Confirmed,
            CommitmentLevel::Finalized | _ => TransactionConfirmationStatus::Finalized,
        }
    }
}
#[derive(Clone)]
struct OrderedConfirmationStatus(TransactionConfirmationStatus);
impl From<OrderedConfirmationStatus> for u8 {
    fn from(from: OrderedConfirmationStatus) -> Self {
        match from {
            OrderedConfirmationStatus(TransactionConfirmationStatus::Processed) => 0,
            OrderedConfirmationStatus(TransactionConfirmationStatus::Confirmed) => 1,
            OrderedConfirmationStatus(TransactionConfirmationStatus::Finalized) => 2,
        }
    }
}
impl PartialEq<CommitmentConfig> for OrderedConfirmationStatus {
    fn eq(&self, other: &CommitmentConfig) -> bool {
        u8::from(self.clone()).eq(&u8::from(OrderedConfirmationStatus(
            other.to_confirmation_status(),
        )))
    }
}
impl PartialOrd<CommitmentConfig> for OrderedConfirmationStatus {
    fn partial_cmp(&self, other: &CommitmentConfig) -> Option<Ordering> {
        u8::from(self.clone()).partial_cmp(&u8::from(OrderedConfirmationStatus(
            other.to_confirmation_status(),
        )))
    }
}

/// A [`Signer`] with hash based on the pubkey.
pub struct HashedSigner<'a>(SignerCow<'a>);
impl<'a> Debug for HashedSigner<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HashedSigner")
            .field(&self.0.pubkey())
            .finish()
    }
}
impl<'a> PartialEq for HashedSigner<'a> {
    fn eq(&self, other: &Self) -> bool {
        self.0.pubkey().eq(&other.0.pubkey())
    }
}
impl<'a> Eq for HashedSigner<'a> {}
impl<'a> std::hash::Hash for HashedSigner<'a> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.pubkey().hash(state);
    }
}
impl<'a> From<&'a dyn Signer> for HashedSigner<'a> {
    fn from(from: &'a dyn Signer) -> Self {
        Self(SignerCow::Borrowed(from))
    }
}
impl<'a> From<Box<dyn Signer>> for HashedSigner<'a> {
    fn from(from: Box<dyn Signer>) -> Self {
        Self(SignerCow::Owned(from))
    }
}
impl<'a> From<Keypair> for HashedSigner<'a> {
    fn from(from: Keypair) -> Self {
        Self(SignerCow::Owned(Box::new(from)))
    }
}
impl<'a> From<&'a Keypair> for HashedSigner<'a> {
    fn from(from: &'a Keypair) -> Self {
        Self(SignerCow::Borrowed(from))
    }
}

impl<'a> Signer for HashedSigner<'a> {
    #[inline]
    fn pubkey(&self) -> Pubkey {
        self.0.pubkey()
    }

    #[inline]
    fn try_pubkey(&self) -> Result<Pubkey, SignerError> {
        self.0.try_pubkey()
    }

    #[inline]
    fn sign_message(&self, message: &[u8]) -> Signature {
        self.0.sign_message(message)
    }

    #[inline]
    fn try_sign_message(&self, message: &[u8]) -> Result<Signature, SignerError> {
        self.0.try_sign_message(message)
    }

    #[inline]
    fn is_interactive(&self) -> bool {
        self.0.is_interactive()
    }
}

enum SignerCow<'a> {
    Borrowed(&'a dyn Signer),
    Owned(Box<dyn Signer + 'a>),
}
impl<'a> Deref for SignerCow<'a> {
    type Target = dyn Signer + 'a;

    fn deref(&self) -> &Self::Target {
        match self {
            SignerCow::Borrowed(signer) => *signer,
            SignerCow::Owned(signer) => &**signer,
        }
    }
}
