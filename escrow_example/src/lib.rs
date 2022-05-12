#![feature(const_trait_impl)]

//! The escrow program from the paulx blog

#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "cpi")]
pub mod cpi;
pub mod instructions;

use cruiser::prelude::*;

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

#[derive(BorshSerialize, BorshDeserialize, OnChainSize, Default)]
pub struct EscrowAccount {
    pub initializer: Pubkey,
    pub temp_token_account: Pubkey,
    pub initializer_token_to_receive: Pubkey,
    pub expected_amount: u64,
}

#[derive(Debug)]
struct EscrowPDASeeder;

impl PDASeeder for EscrowPDASeeder {
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a> {
        Box::new([&"escrow" as &dyn PDASeed].into_iter())
    }
}
