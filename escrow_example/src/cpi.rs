use crate::instructions::exchange::Exchange;
use crate::instructions::init_escrow::InitEscrow;
use crate::EscrowInstructions;
use cruiser::prelude::*;

pub struct InitEscrowCPI<'a, AI> {
    accounts: [MaybeOwned<'a, AI>; 6],
    data: Vec<u8>,
}

impl<'a, AI> InitEscrowCPI<'a, AI> {
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
        <EscrowInstructions as InstructionListItem<InitEscrow>>::discriminant_compressed()
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
            data,
        })
    }
}

impl<'a, AI> CPIClientStatic<'a, 7> for InitEscrowCPI<'a, AI>
where
    AI: ToSolanaAccountMeta,
{
    type InstructionList = EscrowInstructions;
    type Instruction = InitEscrow;
    type AccountInfo = AI;

    fn instruction(
        self,
        program_account: &'a AI,
    ) -> InstructionAndAccounts<[MaybeOwned<'a, Self::AccountInfo>; 7]> {
        let instruction = SolanaInstruction {
            program_id: *program_account.meta_key(),
            accounts: self
                .accounts
                .iter()
                .map(MaybeOwned::as_ref)
                .map(AI::to_solana_account_meta)
                .collect(),
            data: self.data,
        };
        let mut accounts = self.accounts.into_iter();
        InstructionAndAccounts {
            instruction,
            // TODO: Replace this with a const push operation when willing to go to const generics
            accounts: [
                accounts.next().unwrap(),
                accounts.next().unwrap(),
                accounts.next().unwrap(),
                accounts.next().unwrap(),
                accounts.next().unwrap(),
                accounts.next().unwrap(),
                program_account.into(),
            ],
        }
    }
}

pub struct ExchangeCPI<'a, AI> {
    accounts: [MaybeOwned<'a, AI>; 9],
    data: Vec<u8>,
}

impl<'a, AI> ExchangeCPI<'a, AI> {
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
        <EscrowInstructions as InstructionListItem<Exchange>>::discriminant_compressed()
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
            data,
        })
    }
}
impl<'a, AI> CPIClientStatic<'a, 10> for ExchangeCPI<'a, AI>
where
    AI: ToSolanaAccountMeta,
{
    type InstructionList = EscrowInstructions;
    type Instruction = InitEscrow;
    type AccountInfo = AI;

    fn instruction(
        self,
        program_account: &'a AI,
    ) -> InstructionAndAccounts<[MaybeOwned<'a, Self::AccountInfo>; 10]> {
        let instruction = SolanaInstruction {
            program_id: *program_account.meta_key(),
            accounts: self
                .accounts
                .iter()
                .map(MaybeOwned::as_ref)
                .map(AI::to_solana_account_meta)
                .collect(),
            data: self.data,
        };
        let mut accounts = self.accounts.into_iter();
        InstructionAndAccounts {
            instruction,
            // TODO: Replace this with a const push operation when willing to go to const generics
            accounts: [
                accounts.next().unwrap(), // 0
                accounts.next().unwrap(), // 1
                accounts.next().unwrap(), // 2
                accounts.next().unwrap(), // 3
                accounts.next().unwrap(), // 4
                accounts.next().unwrap(), // 5
                accounts.next().unwrap(), // 6
                accounts.next().unwrap(), // 7
                accounts.next().unwrap(), // 8
                program_account.into(),
            ],
        }
    }
}
