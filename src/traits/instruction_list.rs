//! A list of instructions serving as an interface and entrypoint for the program.

pub use cruiser_derive::InstructionList;

use crate::account_argument::AccountInfoIterator;
use crate::account_list::AccountList;
use crate::compressed_numbers::CompressedNumber;
use crate::CruiserResult;
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
