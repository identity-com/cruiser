//! An individual instruction for a program.

use borsh::{BorshDeserialize, BorshSerialize};
use cruiser::account_argument::AccountArgument;
use solana_program::program::MAX_RETURN_DATA;
use std::ops::DerefMut;

use crate::account_argument::{FromAccounts, ValidateArgument};
use crate::on_chain_size::OnChainSize;
use crate::{CruiserResult, Pubkey};

/// An instruction for a program with it's accounts and data.
pub trait Instruction<AI>: Sized {
    /// The account argument for this instruction.
    type Accounts;
    /// The instruction data minus the instruction discriminant.
    type Data;
    /// The return type of the instruction
    type ReturnType: ReturnValue;
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
    ) -> CruiserResult<I::ReturnType>;
}

/// A return value for an instruction call
pub trait ReturnValue: Sized {
    /// Returns self to `return_func`. Must call `return_func` exactly once.
    fn return_self(self, return_func: impl FnOnce(&[u8])) -> CruiserResult;
    /// Gets self from returned data.
    /// If `data` is [`None`] then the program did not return.
    /// Borsh implementation of [`ReturnValue`] attempts to deserialize an empty array in the [`None`] case.
    /// `return_program` being [`None`] means the max size reported was 0.
    fn from_returned(
        data: Option<impl DerefMut<Target = [u8]>>,
        return_program: Option<&Pubkey>,
    ) -> CruiserResult<Self>;

    /// The maximum size of the return value.
    fn max_size() -> usize;
}

impl<T> ReturnValue for T
where
    T: BorshSerialize + BorshDeserialize + OnChainSize,
{
    fn return_self(self, return_func: impl FnOnce(&[u8])) -> CruiserResult {
        let mut buffer = [0; MAX_RETURN_DATA];
        self.serialize(&mut buffer.as_mut())?;
        return_func(&buffer);
        Ok(())
    }

    fn from_returned(
        data: Option<impl DerefMut<Target = [u8]>>,
        _return_program: Option<&Pubkey>,
    ) -> CruiserResult<Self> {
        match data {
            None => Ok(BorshDeserialize::deserialize(&mut [].as_ref())?),
            Some(buffer) => Ok(BorshDeserialize::deserialize(&mut &*buffer)?),
        }
    }

    fn max_size() -> usize {
        Self::ON_CHAIN_SIZE
    }
}
