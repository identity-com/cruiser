use cruiser::account_list::AccountList;
use cruiser::account_types::PhantomAccount;
use cruiser::instruction::{Instruction, InstructionProcessor};
use cruiser::instruction_list::InstructionList;
use cruiser::CruiserResult;
use solana_program::pubkey::Pubkey;

#[derive(AccountList)]
pub enum TestAccountList {}

#[derive(Copy, Clone, InstructionList)]
#[instruction_list(account_list = TestAccountList, account_info = [<AI> AI])]
pub enum TestList {
    #[instruction(instruction_type = TestInstruction1)]
    TestInstruction1,
    #[instruction(instruction_type = TestInstruction1)]
    TestInstruction2,
    #[instruction(instruction_type = TestInstruction1)]
    TestInstruction3,
}

pub struct TestInstruction1;
impl<AI> Instruction<AI> for TestInstruction1 {
    type Data = ();
    type Accounts = PhantomAccount<AI, ()>;
}
impl<AI> InstructionProcessor<AI, TestInstruction1> for TestInstruction1 {
    type FromAccountsData = ();
    type ValidateData = ();
    type InstructionData = ();

    fn data_to_instruction_arg(
        _data: <Self as Instruction<AI>>::Data,
    ) -> CruiserResult<(
        Self::FromAccountsData,
        Self::ValidateData,
        Self::InstructionData,
    )> {
        Ok(Default::default())
    }

    fn process(
        _program_id: &Pubkey,
        _data: <TestInstruction1 as Instruction<AI>>::Data,
        _accounts: &mut <TestInstruction1 as Instruction<AI>>::Accounts,
    ) -> CruiserResult<()> {
        panic!("This is never called")
    }
}
