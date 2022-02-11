use cruiser::{
    AccountList, GeneratorResult, Instruction, InstructionList, InstructionProcessor, SystemProgram,
};
use solana_program::pubkey::Pubkey;

#[derive(AccountList)]
pub enum TestAccountList {}

#[derive(Copy, Clone, InstructionList)]
#[instruction_list(account_list = TestAccountList)]
pub enum TestList {
    #[instruction(instruction_type = TestInstruction1)]
    TestInstruction1,
    #[instruction(instruction_type = TestInstruction1)]
    TestInstruction2,
    #[instruction(instruction_type = TestInstruction1)]
    TestInstruction3,
}

::static_assertions::const_assert_ne!(0, 1);

pub struct TestInstruction1;
impl Instruction for TestInstruction1 {
    type Data = ();
    type FromAccountsData = ();
    type Accounts = ();

    fn data_to_instruction_arg(_data: &mut Self::Data) -> GeneratorResult<Self::FromAccountsData> {
        Ok(())
    }
}
impl InstructionProcessor<TestInstruction1> for TestInstruction1 {
    fn process(
        _program_id: &'static Pubkey,
        _data: <TestInstruction1 as Instruction>::Data,
        _accounts: &mut <TestInstruction1 as Instruction>::Accounts,
    ) -> GeneratorResult<Option<SystemProgram>> {
        todo!()
    }
}
