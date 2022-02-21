use cruiser::borsh::{BorshDeserialize, BorshSerialize};
use cruiser::spl::token::TokenAccount;
use cruiser::{
    entrypoint_list, AccountArgument, AccountInfo, AccountList, GeneratorResult,
    InitOrZeroedAccount, Instruction, InstructionList, InstructionProcessor, ProgramAccount,
    Pubkey, RentExempt, SystemProgram, ZeroedAccount,
};

entrypoint_list!(EscrowInstructions, EscrowInstructions);

#[derive(InstructionList, Copy, Clone)]
#[instruction_list(account_list = EscrowAccounts)]
pub enum EscrowInstructions {
    #[instruction(instruction_type = InitEscrow)]
    InitEscrow,
    #[instruction(instruction_type = InitEscrow)]
    Exchange,
}

#[derive(AccountList)]
pub enum EscrowAccounts {
    EscrowAccount(EscrowAccount),
}

#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct EscrowAccount {
    pub is_initialized: bool,
    pub initializer: Pubkey,
    pub temp_token_account: Pubkey,
    pub initializer_token_to_receive: Pubkey,
    pub expected_amount: u64,
}

pub struct InitEscrow;
impl Instruction for InitEscrow {
    type Data = InitEscrowData;
    type FromAccountsData = ();
    type Accounts = ();

    fn data_to_instruction_arg(_data: &mut Self::Data) -> GeneratorResult<Self::FromAccountsData> {
        Ok(())
    }
}
impl InstructionProcessor<InitEscrow> for InitEscrow {
    fn process(
        program_id: &'static Pubkey,
        data: <Self as Instruction>::Data,
        accounts: &mut <Self as Instruction>::Accounts,
    ) -> GeneratorResult<Option<SystemProgram>> {
        todo!()
    }
}
#[derive(BorshSerialize, BorshDeserialize)]
pub struct InitEscrowData {
    amount: u64,
}
#[derive(AccountArgument)]
#[from(data = (), log_level = trace)]
pub struct InitEscrowAccounts {
    #[account_argument(signer)]
    initializer: AccountInfo,
    #[account_argument(writable)]
    temp_token_account: TokenAccount,
    initializer_token_account: TokenAccount,
    #[account_argument(writable)]
    escrow_account: RentExempt<ZeroedAccount<EscrowAccounts, EscrowAccount>>,
    #[account_argument(key = &spl_token::ID)]
    token_program: AccountInfo,
}
