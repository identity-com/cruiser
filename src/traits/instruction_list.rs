//! A list of instructions serving as an interface and entrypoint for the program.

pub use cruiser_derive::InstructionList;

use crate::account_argument::AccountInfoIterator;
use solana_program::pubkey::Pubkey;

use crate::account_list::AccountList;
use crate::compressed_numbers::CompressedNumber;
use crate::{CruiserResult, SolanaInstruction};

/// A list of possible instructions for a program.
pub trait InstructionList: Copy {
    /// The compression for the discriminant
    type DiscriminantCompressed: CompressedNumber<Num = u64>;
    /// The accounts for this list of instructions
    type AccountList: AccountList;

    /// Gets the discriminant for the instruction
    fn discriminant(self) -> u64;
    /// Gets the discriminant in compressed form
    fn discriminant_compressed(self) -> Self::DiscriminantCompressed {
        Self::DiscriminantCompressed::from_number(self.discriminant())
    }
    /// Creates the instruction from a discriminant
    fn from_discriminant(discriminant: u64) -> Option<Self>;
}

/// A Processor for a given [`InstructionList`].
pub trait InstructionListProcessor<IL: InstructionList> {
    /// Processes a given instruction. Usually delegates to [`InstructionProcessor`](crate::instruction::InstructionProcessor).
    fn process_instruction(
        program_id: &'static Pubkey,
        accounts: &mut impl AccountInfoIterator,
        data: &[u8],
    ) -> CruiserResult<()>;
}

/// Adds support for building items in the instruction list.
pub trait InstructionListBuilder<IL: InstructionList, B> {
    /// Builds an instruction from `B`
    fn build_instruction(program_id: &'static Pubkey, build: B)
        -> CruiserResult<SolanaInstruction>;
}

/// Instruction list is an interface. Still Experimental.
pub trait Interface: InstructionList {
    /// The global discriminant of the developer
    const DEVELOPER_DISCRIMINANT: &'static [u8];
    /// The global discriminant of the interface
    const INTERFACE_DISCRIMINANT: u64;
}

/// Processor can process a given interface. Still Experimental.
pub trait InterfaceProcessor<I: Interface>: InstructionListProcessor<I> {}
impl<T, I> InterfaceProcessor<I> for T
where
    T: InstructionListProcessor<I>,
    I: Interface,
{
}
