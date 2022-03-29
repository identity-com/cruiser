use crate::ToSolanaAccountInfo;
use solana_program::account_info::AccountInfo as SolanaAccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::instruction::Instruction as SolanaInstruction;
use solana_program::program::{
    invoke_signed as solana_invoke_signed,
    invoke_signed_unchecked as solana_invoke_signed_unchecked,
};

/// A way of executing CPI calls
pub trait CPI: Sized {
    /// The raw execution function.
    /// Usually ends up at either [`solana_program::program::invoke_signed`] or [`solana_program::program::invoke_signed_unchecked`]
    fn raw_invoke_signed(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[SolanaAccountInfo],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult;

    /// Invokes another solana program.
    fn invoke<'a, AI, const N: usize>(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[&AI; N],
    ) -> ProgramResult
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        self.invoke_signed(instruction, account_infos, &[])
    }

    /// Invokes another solana program, signing with seeds.
    fn invoke_signed<'a, AI, const N: usize>(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[&AI; N],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        self.raw_invoke_signed(
            instruction,
            &array_init::array_init::<_, _, N>(|x| unsafe {
                account_infos[x].to_solana_account_info()
            }),
            signer_seeds,
        )
    }

    /// Invokes another solana program with a variable number of accounts.
    /// Less efficient than [`CPI::invoke`].
    fn invoke_variable_size<'a, 'b, AI, I>(
        self,
        instruction: &SolanaInstruction,
        account_infos: I,
    ) -> ProgramResult
    where
        AI: 'a + ToSolanaAccountInfo<'b>,
        I: IntoIterator<Item = &'a AI>,
    {
        self.invoke_signed_variable_size(instruction, account_infos, &[])
    }

    /// Invokes another solana program with a variable number of accounts, signing with seeds.
    /// Less efficient than [`CPI::invoke_signed`].
    fn invoke_signed_variable_size<'a, 'b, AI, I>(
        self,
        instruction: &SolanaInstruction,
        account_infos: I,
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult
    where
        AI: 'a + ToSolanaAccountInfo<'b>,
        I: IntoIterator<Item = &'a AI>,
    {
        self.raw_invoke_signed(
            instruction,
            &account_infos
                .into_iter()
                .map(|info| unsafe { info.to_solana_account_info() })
                .collect::<Vec<_>>(),
            signer_seeds,
        )
    }
}

/// CPI functions that check each account for outstanding usages.
/// Less efficient than [`CPIUnchecked`] but will avoid unsafe situations.
/// Suggested to use this for validation and then swap to [`CPIUnchecked`].
/// Uses [`solana_program::program::invoke_signed`]
#[derive(Copy, Clone, Debug)]
pub struct CPIChecked;
impl CPI for CPIChecked {
    #[inline]
    fn raw_invoke_signed(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[SolanaAccountInfo],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        solana_invoke_signed(instruction, account_infos, signer_seeds)
    }
}

/// CPI functions that doesn't check each account for outstanding usages.
/// Can result in unsafe situations but is more efficient than [`CPIChecked`].
/// Uses [`solana_program::program::invoke_signed_unchecked`]
#[derive(Copy, Clone, Debug)]
pub struct CPIUnchecked;
impl CPI for CPIUnchecked {
    #[inline]
    fn raw_invoke_signed(
        self,
        instruction: &SolanaInstruction,
        account_infos: &[SolanaAccountInfo],
        signer_seeds: &[&[&[u8]]],
    ) -> ProgramResult {
        solana_invoke_signed_unchecked(instruction, account_infos, signer_seeds)
    }
}
