use solana_generator::InstructionList;

#[derive(InstructionList)]
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
