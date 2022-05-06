//! A list of instructions serving as an interface and entrypoint for the program.

pub use cruiser_derive::InstructionList;

use crate::account_argument::AccountInfoIterator;
use crate::account_list::AccountList;
use crate::compressed_numbers::CompressedNumber;
use crate::instruction::Instruction;
use crate::{CruiserResult, SolanaInstruction};
use solana_program::pubkey::Pubkey;

/// A list of possible instructions for a program.
pub trait InstructionList: Copy {
    /// The compression for the discriminant
    type DiscriminantCompressed: CompressedNumber<u64>;
    /// The accounts for this list of instructions
    type AccountList: AccountList;

    /// Creates the instruction from a discriminant
    fn from_discriminant(discriminant: u64) -> Option<Self>;
}
/// Allows an instruction list to support an instruction type
///
/// # Safety
/// Implementor must guarantee that no two discriminates match
pub unsafe trait InstructionListItem<I>: Sized + InstructionList {
    /// Gets the discriminant for the instruction
    #[must_use]
    fn discriminant() -> u64;
    /// Gets the discriminant in compressed form
    #[must_use]
    fn discriminant_compressed() -> Self::DiscriminantCompressed {
        Self::DiscriminantCompressed::from_number(Self::discriminant())
    }
}

/// A Processor for a given [`InstructionList`].
pub trait InstructionListProcessor<AI, IL: InstructionList> {
    /// Processes a given instruction. Usually delegates to [`InstructionProcessor`](crate::instruction::InstructionProcessor).
    fn process_instruction(
        program_id: &Pubkey,
        accounts: &mut impl AccountInfoIterator<Item = AI>,
        data: &[u8],
    ) -> CruiserResult<()>;
}

/// The basic client function, should have a version of this for each thing you want to be able to cpi.
/// Also should implement either [`InstructionListCPIStatic`] or [`InstructionListCPIDynamic`].
pub trait InstructionListCPI {
    /// The instruction list for this
    type InstructionList: InstructionListItem<Self::Instruction>;
    /// The instruction for this
    type Instruction: Instruction<Self::AccountInfo>;
    /// The account info this deals with
    type AccountInfo;

    /// Gets this as a solana instruction
    #[must_use]
    fn instruction(&mut self, program_id: &Pubkey) -> SolanaInstruction;
}

/// Extension to [`InstructionListCPI`]. More efficient than [`InstructionListCPIDynamic`] but requires statically known account length.
pub trait InstructionListCPIStatic<const N: usize>: InstructionListCPI {
    /// Gets the accounts for this call.
    #[must_use]
    fn to_accounts_static<'a>(
        &'a self,
        program_account: &'a Self::AccountInfo,
    ) -> [&'a Self::AccountInfo; N];
}

/// Extension to [`InstructionListCPI`].
/// Less efficient than [`InstructionListCPIStatic`] but can have dynamically sized account length.
pub trait InstructionListCPIDynamic: InstructionListCPI {
    /// The iterator returned by [`InstructionListCPIDynamic::to_accounts_dynamic`].
    type Iter<'a>: Iterator<Item = &'a Self::AccountInfo>
    where
        Self: 'a;

    /// Gets the accounts for this call.
    #[must_use]
    fn to_accounts_dynamic(&self) -> Self::Iter<'_>;
}

/// Instruction list is an interface. Still Experimental.
#[cfg(feature = "interface")]
pub trait Interface: InstructionList {
    /// The global discriminant of the developer
    const DEVELOPER_DISCRIMINANT: &'static [u8];
    /// The global discriminant of the interface
    const INTERFACE_DISCRIMINANT: u64;
}

/// Processor can process a given interface. Still Experimental.
#[cfg(feature = "interface")]
pub trait InterfaceProcessor<AI, I: Interface>: InstructionListProcessor<AI, I> {}
