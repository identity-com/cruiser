use borsh::{BorshDeserialize, BorshSerialize};

use crate::{FromAccounts, GeneratorResult, Pubkey, SolanaAccountMeta};

/// An instruction for a program with it's accounts and data.
pub trait Instruction: Sized {
    /// The data type minus the discriminant.
    type Data: BorshSerialize + BorshDeserialize;
    /// The type used for passing data to accounts.
    type FromAccountsData;
    /// The list of accounts for this instruction.
    type Accounts: FromAccounts<Self::FromAccountsData>;

    /// Turns the [`Self::Data`] into the instruction arg for [`Self::Accounts`].
    fn data_to_instruction_arg(data: &mut Self::Data) -> GeneratorResult<Self::FromAccountsData>;
}

/// A processor for a given instruction `I`
pub trait InstructionProcessor<I: Instruction> {
    /// Processes the instruction, writing back after this instruction.
    fn process(
        program_id: &'static Pubkey,
        data: I::Data,
        accounts: &mut I::Accounts,
    ) -> GeneratorResult<()>;
}

/// This instruction supports building from a given build arg
pub trait InstructionBuilder<I: Instruction, B> {
    /// Creates this instruction from a given argument
    fn build_instruction(
        program_id: &'static Pubkey,
        arg: B,
    ) -> GeneratorResult<(Vec<SolanaAccountMeta>, I::Data)>;
}
