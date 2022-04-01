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
/// Also should implement either [`InstructionListCPIStatic`] or [`InstructionListCPIDynamic`].
pub trait InstructionListCPI<IL: InstructionList> {
    /// The account info this deals with
    type AccountInfo;

    /// Gets this as a solana instruction
    #[must_use]
    fn instruction(&mut self, program_id: &Pubkey) -> SolanaInstruction;
}

/// Extension to [`InstructionListCPI`]. More efficient than [`InstructionListCPIDynamic`] but requires statically known account length.
pub trait InstructionListCPIStatic<IL: InstructionList, const N: usize>:
    InstructionListCPI<IL>
{
    /// Gets the accounts for this call.
    #[must_use]
    fn to_accounts_static<'a>(
        &'a self,
        program_account: &'a Self::AccountInfo,
    ) -> [&'a Self::AccountInfo; N];
}

/// Extension to [`InstructionListCPI`].
/// Less efficient than [`InstructionListCPIStatic`] but can have dynamically sized account length.
pub trait InstructionListCPIDynamic<IL: InstructionList>:
    for<'a> InstructionListCPIDynamicAccess<'a, IL>
{
}
impl<IL: InstructionList, T> InstructionListCPIDynamic<IL> for T where
    T: for<'a> InstructionListCPIDynamicAccess<'a, IL>
{
}

/// Extension to [`InstructionListCPI`].
/// Less efficient than [`InstructionListCPIStatic`] but can have dynamically sized account length.
/// Use [`InstructionListCPIDynamic`].
pub trait InstructionListCPIDynamicAccess<'a, IL: InstructionList>: InstructionListCPI<IL>
where
    Self::AccountInfo: 'a,
{
    /// The iterator returned by [`InstructionListClientDynamic::to_accounts_dynamic`].
    type Iter: Iterator<Item = &'a Self::AccountInfo>;

    /// Gets the accounts for this call.
    #[must_use]
    #[allow(clippy::needless_lifetimes)]
    fn to_accounts_dynamic(&'a self) -> Self::Iter;
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
