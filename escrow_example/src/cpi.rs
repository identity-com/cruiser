use crate::EscrowInstructions;
use cruiser::account_argument::ToSolanaAccountMeta;
use cruiser::borsh::BorshSerialize;
use cruiser::instruction_list::{InstructionList, InstructionListCPI, InstructionListCPIStatic};
use cruiser::util::MaybeOwned;
use cruiser::{CruiserResult, Pubkey, SolanaInstruction};

pub struct InitEscrow<'a, AI> {
    accounts: [MaybeOwned<'a, AI>; 6],
    data: Option<Vec<u8>>,
}
impl<'a, AI> InitEscrow<'a, AI> {
    pub fn new(
        initializer: impl Into<MaybeOwned<'a, AI>>,
        temp_token_account: impl Into<MaybeOwned<'a, AI>>,
        initializer_token_account: impl Into<MaybeOwned<'a, AI>>,
        escrow_account: impl Into<MaybeOwned<'a, AI>>,
        token_program: impl Into<MaybeOwned<'a, AI>>,
        system_program: impl Into<MaybeOwned<'a, AI>>,
        amount: u64,
    ) -> CruiserResult<Self> {
        let mut data = Vec::with_capacity(8 + 8);
        EscrowInstructions::InitEscrow
            .discriminant_compressed()
            .serialize(&mut data)?;
        amount.serialize(&mut data)?;
        Ok(Self {
            accounts: [
                initializer.into(),
                temp_token_account.into(),
                initializer_token_account.into(),
                escrow_account.into(),
                token_program.into(),
                system_program.into(),
            ],
            data: Some(data),
        })
    }
}
impl<'a, AI> InstructionListCPI<EscrowInstructions> for InitEscrow<'a, AI>
where
    AI: ToSolanaAccountMeta,
{
    type AccountInfo = AI;

    fn instruction(&mut self, program_id: &Pubkey) -> SolanaInstruction {
        SolanaInstruction {
            program_id: *program_id,
            accounts: self
                .accounts
                .iter()
                .map(MaybeOwned::as_ref)
                .map(AI::to_solana_account_meta)
                .collect(),
            data: self.data.take().unwrap(),
        }
    }
}
impl<'a, AI> InstructionListCPIStatic<EscrowInstructions, 7> for InitEscrow<'a, AI>
where
    AI: ToSolanaAccountMeta,
{
    fn to_accounts_static<'b>(&'b self, program_account: &'b AI) -> [&'b AI; 7] {
        // TODO: Replace this with a const push operation when willing to go to const generics
        [
            self.accounts[0].as_ref(),
            self.accounts[1].as_ref(),
            self.accounts[2].as_ref(),
            self.accounts[3].as_ref(),
            self.accounts[4].as_ref(),
            self.accounts[5].as_ref(),
            program_account,
        ]
    }
}

pub struct Exchange<'a, AI> {
    accounts: [MaybeOwned<'a, AI>; 9],
    data: Option<Vec<u8>>,
}
impl<'a, AI> Exchange<'a, AI> {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        taker: impl Into<MaybeOwned<'a, AI>>,
        taker_send_token_account: impl Into<MaybeOwned<'a, AI>>,
        taker_receive_token_account: impl Into<MaybeOwned<'a, AI>>,
        temp_token_account: impl Into<MaybeOwned<'a, AI>>,
        initializer: impl Into<MaybeOwned<'a, AI>>,
        initializer_token_account: impl Into<MaybeOwned<'a, AI>>,
        escrow_account: impl Into<MaybeOwned<'a, AI>>,
        token_program: impl Into<MaybeOwned<'a, AI>>,
        pda_account: impl Into<MaybeOwned<'a, AI>>,
        amount: u64,
    ) -> CruiserResult<Self> {
        let mut data = Vec::with_capacity(8 + 8);
        EscrowInstructions::Exchange
            .discriminant_compressed()
            .serialize(&mut data)?;
        amount.serialize(&mut data)?;
        Ok(Self {
            accounts: [
                taker.into(),
                taker_send_token_account.into(),
                taker_receive_token_account.into(),
                temp_token_account.into(),
                initializer.into(),
                initializer_token_account.into(),
                escrow_account.into(),
                token_program.into(),
                pda_account.into(),
            ],
            data: Some(data),
        })
    }
}
impl<'a, AI> InstructionListCPI<EscrowInstructions> for Exchange<'a, AI>
where
    AI: ToSolanaAccountMeta,
{
    type AccountInfo = AI;

    fn instruction(&mut self, program_id: &Pubkey) -> SolanaInstruction {
        SolanaInstruction {
            program_id: *program_id,
            accounts: self
                .accounts
                .iter()
                .map(MaybeOwned::as_ref)
                .map(AI::to_solana_account_meta)
                .collect(),
            data: self.data.take().unwrap(),
        }
    }
}
impl<'a, AI> InstructionListCPIStatic<EscrowInstructions, 10> for Exchange<'a, AI>
where
    AI: ToSolanaAccountMeta,
{
    fn to_accounts_static<'b>(&'b self, program_account: &'b AI) -> [&'b AI; 10] {
        // TODO: Replace this with a const push operation when willing to go to const generics
        [
            self.accounts[0].as_ref(),
            self.accounts[1].as_ref(),
            self.accounts[2].as_ref(),
            self.accounts[3].as_ref(),
            self.accounts[4].as_ref(),
            self.accounts[5].as_ref(),
            self.accounts[6].as_ref(),
            self.accounts[7].as_ref(),
            self.accounts[8].as_ref(),
            program_account,
        ]
    }
}
