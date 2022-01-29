use solana_generator::{GeneratorResult, Instruction, InstructionList, SolanaAccountMeta};
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

pub struct TestInstruction1;
impl Instruction for TestInstruction1 {
    type Data = ();
    type FromAccountsData = ();
    type Accounts = ();
    type BuildArg = ();

    fn data_to_instruction_arg(_data: &mut Self::Data) -> GeneratorResult<Self::FromAccountsData> {
        Ok(())
    }

    fn build_instruction(
        program_id: &Pubkey,
        arg: Self::BuildArg,
    ) -> GeneratorResult<(Vec<SolanaAccountMeta>, Self::Data)> {
        Ok((vec![], ()))
    }
}
