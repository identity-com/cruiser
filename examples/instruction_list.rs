use solana_generator::InstructionList;

#[derive(InstructionList)]
#[instruction_list()]
pub enum TestList {
    #[instruction_list(instruction_type = TestInstruction1)]
    TestInstruction1,
    TestInstruction2,
    TestInstruction3,
}

pub struct TestInstruction1;
