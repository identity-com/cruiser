use crate::prelude::*;

/// A solana instruction paired with its account infos.
#[derive(Debug)]
pub struct InstructionAndAccounts<A> {
    /// The instruction.
    pub instruction: SolanaInstruction,
    /// The accounts for the instruction.
    pub accounts: A,
}

/// CPI client trait with static account number.
/// More efficient than [`CPIClientDynamic`] but requires statically known account length.
pub trait CPIClientStatic<'a, const N: usize>: Sized {
    /// The instruction list for this
    type InstructionList: InstructionListItem<Self::Instruction>;
    /// The instruction for this
    type Instruction: Instruction<Self::AccountInfo>;
    /// The account info this deals with
    type AccountInfo: 'a;

    /// Gets the accounts for this call.
    #[must_use]
    fn instruction(
        self,
        program_account: impl Into<MaybeOwned<'a, Self::AccountInfo>>,
    ) -> InstructionAndAccounts<[MaybeOwned<'a, Self::AccountInfo>; N]>;

    /// Invokes this cpi call on the given program.
    fn invoke<'b, 'c: 'b, 'd: 'a, P>(
        self,
        cpi: impl CPIMethod,
        program: &'d CruiserProgramAccount<Self::AccountInfo, P>,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> CruiserResult<<Self::Instruction as Instruction<Self::AccountInfo>>::ReturnType>
    where
        P: CruiserProgram<InstructionList = Self::InstructionList>,
        P::InstructionList: InstructionListItem<Self::Instruction>,
        Self::AccountInfo: ToSolanaAccountInfo<'d>,
    {
        program.invoke(cpi, self, seeds)
    }
}

/// CPI client trait with dynamic account number.
/// Less efficient than [`CPIClientStatic`] but can have dynamically sized account length.
pub trait CPIClientDynamic<'a>: Sized {
    /// The instruction list for this
    type InstructionList: InstructionListItem<Self::Instruction>;
    /// The instruction for this
    type Instruction: Instruction<Self::AccountInfo>;
    /// The account info this deals with
    type AccountInfo: 'a;

    /// Gets the accounts for this call.
    #[must_use]
    fn instruction(
        self,
        program_account: &Self::AccountInfo,
    ) -> InstructionAndAccounts<Vec<MaybeOwned<'a, Self::AccountInfo>>>;

    /// Invokes this cpi call on the given program.
    fn invoke<'b, 'c: 'b, 'd: 'a, P>(
        self,
        cpi: impl CPIMethod,
        program: &'d CruiserProgramAccount<Self::AccountInfo, P>,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> CruiserResult<<Self::Instruction as Instruction<Self::AccountInfo>>::ReturnType>
    where
        P: CruiserProgram<InstructionList = Self::InstructionList>,
        P::InstructionList: InstructionListItem<Self::Instruction>,
        Self::AccountInfo: ToSolanaAccountInfo<'d>,
    {
        program.invoke_variable_sized(cpi, self, seeds)
    }
}
