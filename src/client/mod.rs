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
use solana_sdk::commitment_config::CommitmentConfig;
use solana_sdk::signature::{Keypair, Signature, SignerError};
use solana_sdk::signer::Signer;
use solana_sdk::transaction::Transaction;
use std::collections::HashSet;
use std::fmt::{Debug, Formatter};
use std::hash::Hasher;
use std::iter::once;
use std::ops::Deref;

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
    /// Executes this using the given client and config
    pub async fn send_transaction(
        &self,
        client: &RpcClient,
        commitment: CommitmentConfig,
        config: RpcSendTransactionConfig,
    ) -> ClientResult<Signature> {
        let transaction = self.to_transaction(
            client
                .get_latest_blockhash_with_commitment(commitment)
                .await?
                .0,
        );
        client
            .send_and_confirm_transaction_with_spinner_and_config(&transaction, commitment, config)
            .await
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
