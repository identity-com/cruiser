use solana_generator::{
    GeneratorResult, Instruction, InstructionList, InstructionProcessor, SolanaAccountMeta,
    SystemProgram,
};
use solana_program::instruction::AccountMeta;
use solana_program::pubkey::Pubkey;

#[derive(Copy, Clone, InstructionList)]
#[instruction_list()]
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
        program_id: &'static Pubkey,
        data: <TestInstruction1 as Instruction>::Data,
        accounts: &mut <TestInstruction1 as Instruction>::Accounts,
    ) -> GeneratorResult<Option<SystemProgram>> {
        todo!()
    }
}
