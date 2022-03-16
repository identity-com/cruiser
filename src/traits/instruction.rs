//! An individual instruction for a program.

use borsh::{BorshDeserialize, BorshSerialize};

use crate::account_argument::{AccountArgument, FromAccounts, ValidateArgument};
use crate::{CruiserResult, Pubkey, SolanaAccountMeta};

/// An instruction for a program with it's accounts and data.
pub trait Instruction: Sized {
    /// The instruction data minus the instruction discriminant.
    type Data: BorshDeserialize;
    /// The account argument for this instruction.
    type Accounts: AccountArgument;
}

/// A processor for a given instruction `I`
pub trait InstructionProcessor<I: Instruction>
where
    I::Accounts: FromAccounts<Self::FromAccountsData> + ValidateArgument<Self::ValidateData>,
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
        program_id: &'static Pubkey,
        data: Self::InstructionData,
        accounts: &mut I::Accounts,
    ) -> CruiserResult<()>;
}

/// This instruction supports building from a given build arg
pub trait InstructionBuilder<I: Instruction, B>
where
    I::Data: BorshSerialize,
{
    /// Creates this instruction from a given argument
    fn build_instruction(
        program_id: &'static Pubkey,
        arg: B,
    ) -> CruiserResult<(Vec<SolanaAccountMeta>, I::Data)>;
}
