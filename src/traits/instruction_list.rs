use crate::compressed_numbers::CompressedU64;
use crate::{AccountInfoIterator, GeneratorResult, Pubkey, SolanaInstruction};
pub use solana_generator_derive::InstructionList;

/// A list of possible instructions for a program.
pub trait InstructionList: Copy {
    /// The compression for the discriminant
    type DiscriminantCompressed: CompressedU64;

    /// Gets the discriminant for the instruction
    fn discriminant(self) -> u64;
    /// Creates the instruction from a discriminant
    fn from_discriminant(discriminant: u64) -> Option<Self>;
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

pub trait InstructionListBuilder<IL: InstructionList> {
    /// The enum of instruction builders.
    type BuildEnum;

    /// Builds an instruction from [`BuildEnum`].
    fn build_instruction(
        program_id: &'static Pubkey,
        build_enum: Self::BuildEnum,
    ) -> GeneratorResult<SolanaInstruction>;
}

pub trait Interface: InstructionList {
    const INTERFACE_DISCRIMINANT: u64;
}

pub trait InterfaceProcessor<I: Interface>: InstructionListProcessor<I> {}
impl<T, I> InterfaceProcessor<I> for T
where
    T: InstructionListProcessor<I>,
    I: Interface,
{
}
