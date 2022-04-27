#![feature(const_trait_impl)]

//! The escrow program from the paulx blog

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "cpi")]
pub mod cpi;
pub mod instructions;

use cruiser::account_list::AccountList;
use cruiser::borsh::{BorshDeserialize, BorshSerialize};
use cruiser::instruction_list::InstructionList;
use cruiser::on_chain_size::OnChainSize;
use cruiser::pda_seeds::{PDASeed, PDASeeder};
use cruiser::{borsh, Pubkey};

#[cfg(feature = "entrypoint")]
cruiser::entrypoint_list!(EscrowInstructions, EscrowInstructions);

#[derive(InstructionList, Copy, Clone)]
#[instruction_list(account_list = EscrowAccounts, account_info = [< 'a, AI > AI where AI: cruiser::ToSolanaAccountInfo < 'a >])]
pub enum EscrowInstructions {
    #[instruction(instruction_type = instructions::init_escrow::InitEscrow)]
    InitEscrow,
    #[instruction(instruction_type = instructions::exchange::Exchange)]
    Exchange,
}

#[derive(AccountList)]
pub enum EscrowAccounts {
    EscrowAccount(EscrowAccount),
}

#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct EscrowAccount {
    pub initializer: Pubkey,
    pub temp_token_account: Pubkey,
    pub initializer_token_to_receive: Pubkey,
    pub expected_amount: u64,
}

impl const OnChainSize for EscrowAccount {
    const ON_CHAIN_SIZE: usize = Pubkey::ON_CHAIN_SIZE * 3 + u64::ON_CHAIN_SIZE;
}

#[derive(Debug)]
struct EscrowPDASeeder;

impl PDASeeder for EscrowPDASeeder {
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a> {
        Box::new([&"escrow" as &dyn PDASeed].into_iter())
    }
}
