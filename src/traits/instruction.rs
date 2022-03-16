//! An individual instruction for a program.

use borsh::{BorshDeserialize, BorshSerialize};

use crate::account_argument::FromAccounts;
use crate::{CruiserResult, Pubkey, SolanaAccountMeta};

/// An instruction for a program with it's accounts and data.
pub trait Instruction: Sized {
    /// The instruction data minus the instruction discriminant.
    type Data: BorshDeserialize;
    /// The data passed to [`FromAccounts::from_accounts`].
    type FromAccountsData;
    /// The data passed to [`ValidateArgument::validate`].
    type ValidateData;
    /// The data passed to [`InstructionProcessor::process`].
    type InstructionData;
    /// The account argument for this instruction.
    type Accounts: FromAccounts<Self::FromAccountsData>;

    /// Turns the [`Self::Data`] into the instruction arg for [`Self::Accounts`].
    fn data_to_instruction_arg(
        data: Self::Data,
    ) -> CruiserResult<(
        Self::FromAccountsData,
        Self::ValidateData,
        Self::InstructionData,
    )>;
}

/// A processor for a given instruction `I`
pub trait InstructionProcessor<I: Instruction> {
    /// Processes the instruction
    fn process(
        program_id: &'static Pubkey,
        data: I::InstructionData,
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
