use cruiser::account_list::AccountList;
use cruiser::instruction::{Instruction, InstructionProcessor};
use cruiser::instruction_list::InstructionList;
use cruiser::CruiserResult;
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
    type Accounts = ();
}
impl InstructionProcessor<TestInstruction1> for TestInstruction1 {
    type FromAccountsData = ();
    type ValidateData = ();
    type InstructionData = ();

    fn data_to_instruction_arg(
        _data: <Self as Instruction>::Data,
    ) -> CruiserResult<(
        Self::FromAccountsData,
        Self::ValidateData,
        Self::InstructionData,
    )> {
        Ok(Default::default())
    }

    fn process(
        _program_id: &'static Pubkey,
        _data: <TestInstruction1 as Instruction>::Data,
        _accounts: &mut <TestInstruction1 as Instruction>::Accounts,
    ) -> CruiserResult<()> {
        panic!("This is never called")
    }
}
