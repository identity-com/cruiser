//! Helpers for writing rust tests

use solana_client::nonblocking::rpc_client::RpcClient as NonBlockingRpcClient;
use solana_client::rpc_client::RpcClient as BlockingRpcClient;
use solana_program_test::BanksClient;

/// Client trait for generalizing across debuggable rust tests and tests against localnet/devnet/mainnet
pub trait TestingClient {}
impl TestingClient for BanksClient {}
impl TestingClient for NonBlockingRpcClient {}
impl TestingClient for BlockingRpcClient {}
