//! A list of instructions serving as an interface and entrypoint for the program.

pub use cruiser_derive::InstructionList;

use crate::account_argument::AccountInfoIterator;
use crate::account_list::AccountList;
use crate::compressed_numbers::CompressedNumber;
use crate::{CruiserResult, SolanaInstruction};
use solana_program::pubkey::Pubkey;

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
pub trait InstructionListProcessor<AI, IL: InstructionList> {
    /// Processes a given instruction. Usually delegates to [`InstructionProcessor`](crate::instruction::InstructionProcessor).
    fn process_instruction(
        program_id: &Pubkey,
        accounts: &mut impl AccountInfoIterator<Item = AI>,
        data: &[u8],
    ) -> CruiserResult<()>;
}

/// The basic client function, should have a version of this for each thing you want to be able to cpi.
/// Also should implement either [`InstructionListClientStatic`] or [`InstructionListClientDynamic`].
pub trait InstructionListClient<IL: InstructionList> {
    /// The account info this deals with
    type AccountInfo;

    /// Gets this as a solana instruction
    #[must_use]
    fn instruction(&mut self, program_id: &Pubkey) -> SolanaInstruction;
}

/// Extension to [`InstructionListClient`]. More efficient than [`InstructionListClientDynamic`] but requires statically known account length.
pub trait InstructionListClientStatic<IL: InstructionList, const N: usize>:
    InstructionListClient<IL>
{
    /// Gets the accounts for this call.
    #[must_use]
    fn to_accounts_static<'a>(
        &'a self,
        program_account: &'a Self::AccountInfo,
    ) -> [&'a Self::AccountInfo; N];
}

/// Extension to [`InstructionListClient`]. Less efficient than [`InstructionListClientStatic`] but can have dynamically sized account length.
pub trait InstructionListClientDynamic<IL: InstructionList>: InstructionListClient<IL> {
    /// The iterator returned by [`InstructionListClientDynamic::to_accounts_dynamic`].
    type Iter<'a>: Iterator<Item = &'a Self::AccountInfo>
    where
        Self::AccountInfo: 'a,
        Self: 'a;

    /// Gets the accounts for this call.
    #[must_use]
    #[allow(clippy::needless_lifetimes)]
    fn to_accounts_dynamic<'a>(&'a self) -> Self::Iter<'a>;
}

/// Instruction list is an interface. Still Experimental.
pub trait Interface: InstructionList {
    /// The global discriminant of the developer
    const DEVELOPER_DISCRIMINANT: &'static [u8];
    /// The global discriminant of the interface
    const INTERFACE_DISCRIMINANT: u64;
}

/// Processor can process a given interface. Still Experimental.
pub trait InterfaceProcessor<AI, I: Interface>: InstructionListProcessor<AI, I> {}
