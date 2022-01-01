use crate::{AccountInfoIterator, GeneratorResult, Pubkey, SolanaInstruction};
pub use solana_generator_derive::InstructionList;
use crate::discriminant::Discriminant;

/// A list of possible instructions for a program.
pub trait InstructionList: Copy {
    /// The enum of instruction builders.
    type BuildEnum;

    /// Builds an instruction from [`BuildEnum`].
    fn build_instruction(
        program_id: Pubkey,
        build_enum: Self::BuildEnum,
    ) -> GeneratorResult<SolanaInstruction>;
    /// Gets the discriminant for the instruction
    fn discriminant(self) -> Discriminant;
    fn from_discriminant(discriminant: Discriminant) -> Option<Self>;
}

/// A Processor for a given [`InstructionList`]
pub trait InstructionListProcessor<IL: InstructionList>{
    /// Processes a given instruction. Usually delegates to [`crate::InstructionProcessor`].
    fn process_instruction(
        program_id: Pubkey,
        accounts: &mut impl AccountInfoIterator,
        data: &[u8],
    ) -> GeneratorResult<()>;
}

pub trait Interface: InstructionList{
    const INTERFACE_DISCRIMINANT: Discriminant;
}

pub trait InterfaceProcessor<I: Interface>: InstructionListProcessor<I>{}
impl<T, I> InterfaceProcessor<I> for T where T: InstructionListProcessor<I>, I: Interface{}
