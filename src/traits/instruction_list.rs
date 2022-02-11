use crate::compressed_numbers::CompressedNumber;
use crate::{AccountInfoIterator, AccountList, GeneratorResult, Pubkey, SolanaInstruction};
pub use cruiser_derive::InstructionList;
use std::num::NonZeroU64;

/// A list of possible instructions for a program.
pub trait InstructionList: Copy {
    /// The compression for the discriminant
    type DiscriminantCompressed: CompressedNumber<Num = u64>;
    /// The accounts for this list of instructions
    type AccountList: AccountList;

    /// Gets the discriminant for the instruction
    fn discriminant(self) -> NonZeroU64;
    /// Creates the instruction from a discriminant
    fn from_discriminant(discriminant: NonZeroU64) -> Option<Self>;
}

/// A Processor for a given [`InstructionList`]
pub trait InstructionListProcessor<IL: InstructionList> {
    /// Processes a given instruction. Usually delegates to [`crate::InstructionProcessor`].
    fn process_instruction(
        program_id: &'static Pubkey,
        accounts: &mut impl AccountInfoIterator,
        data: &[u8],
    ) -> GeneratorResult<()>;
}

/// Adds support for building items in the instruction list
pub trait InstructionListBuilder<IL: InstructionList, B> {
    /// Builds an instruction from [`BuildEnum`].
    fn build_instruction(
        program_id: &'static Pubkey,
        build: B,
    ) -> GeneratorResult<SolanaInstruction>;
}

/// Instruction list is an interface. Still WIP
pub trait Interface: InstructionList {
    /// The global discriminant of the developer
    const DEVELOPER_DISCRIMINANT: &'static [u8];
    /// The global discriminant of the interface
    const INTERFACE_DISCRIMINANT: u64;
}

/// Processor can process a given interface
pub trait InterfaceProcessor<I: Interface>: InstructionListProcessor<I> {}
impl<T, I> InterfaceProcessor<I> for T
where
    T: InstructionListProcessor<I>,
    I: Interface,
{
}
