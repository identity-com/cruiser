//! An individual instruction for a program.

use borsh::BorshDeserialize;
use cruiser::account_argument::AccountArgument;

use crate::account_argument::{FromAccounts, ValidateArgument};
use crate::{CruiserResult, Pubkey};

/// An instruction for a program with it's accounts and data.
pub trait Instruction<AI>: Sized {
    /// The account argument for this instruction.
    type Accounts;
    /// The instruction data minus the instruction discriminant.
    type Data;
}

/// A processor for a given instruction `I`
pub trait InstructionProcessor<AI, I: Instruction<AI>>
where
    I::Data: BorshDeserialize,
    I::Accounts: AccountArgument<AccountInfo = AI>
        + FromAccounts<Self::FromAccountsData>
        + ValidateArgument<Self::ValidateData>,
{
    /// The data passed to [`FromAccounts::from_accounts`].
    type FromAccountsData;
    /// The data passed to [`ValidateArgument::validate`].
    type ValidateData;
    /// The data passed to [`InstructionProcessor::process`].
    type InstructionData;

    /// Turns the [`Instruction::Data`] into the sub-data types.
    fn data_to_instruction_arg(
        data: I::Data,
    ) -> CruiserResult<(
        Self::FromAccountsData,
        Self::ValidateData,
        Self::InstructionData,
    )>;

    /// Processes the instruction
    fn process(
        program_id: &Pubkey,
        data: Self::InstructionData,
        accounts: &mut I::Accounts,
    ) -> CruiserResult<()>;
}
