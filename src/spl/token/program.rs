use crate::account_argument::{AccountArgument, MultiIndexable, Single, SingleIndexable};
use crate::cpi::CPI;
use crate::pda_seeds::PDASeedSet;
use crate::{AccountInfo, CruiserResult, ToSolanaAccountInfo};
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use spl_token::instruction::{close_account, set_authority, transfer, AuthorityType};

use crate::spl::token::TokenAccount;

// verify_account_arg_impl! {
//     mod token_program_check<AI>{
//         <AI> TokenProgram<AI> where AI: AccountInfo{
//             from: [()];
//             validate: [()];
//             multi: [(); AllAny];
//             single: [()];
//         };
//     }
// }

/// The SPL Token Program. Requires feature
#[derive(AccountArgument, Debug, Clone)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
pub struct TokenProgram<AI> {
    /// The program's info
    #[validate(key = &spl_token::ID)]
    pub info: AI,
}
impl<'b, AI> TokenProgram<AI>
where
    AI: ToSolanaAccountInfo<'b>,
{
    /// Calls the token program's [`set_authority`] instruction
    pub fn set_authority<'a>(
        &self,
        cpi: impl CPI,
        account: &TokenAccount<AI>,
        new_authority: &Pubkey,
        owner: &AI,
        seeds: impl IntoIterator<Item = &'a PDASeedSet<'a>>,
    ) -> ProgramResult {
        let account_info = account.info();
        PDASeedSet::invoke_signed_multiple(
            cpi,
            &set_authority(
                &spl_token::ID,
                account_info.key(),
                Some(new_authority),
                AuthorityType::AccountOwner,
                owner.key(),
                &[owner.key()],
            )?,
            &[&self.info, account_info, owner],
            seeds,
        )
    }

    /// Calls the token program's [`transfer`] instruction
    pub fn transfer<'a>(
        &self,
        cpi: impl CPI,
        from: &TokenAccount<AI>,
        to: &TokenAccount<AI>,
        authority: &AI,
        amount: u64,
        seeds: impl IntoIterator<Item = &'a PDASeedSet<'a>>,
    ) -> ProgramResult {
        let from_info = from.info();
        let to_info = to.info();
        PDASeedSet::invoke_signed_multiple(
            cpi,
            &transfer(
                &spl_token::ID,
                from_info.key(),
                to_info.key(),
                authority.key(),
                &[authority.key()],
                amount,
            )?,
            &[&self.info, from_info, to_info, authority],
            seeds,
        )
    }

    /// Calls the token program's [`close_account`] instruction
    pub fn close_account<'a>(
        &self,
        cpi: impl CPI,
        account: &TokenAccount<AI>,
        destination: &AI,
        authority: &AI,
        seeds: impl IntoIterator<Item = &'a PDASeedSet<'a>>,
    ) -> ProgramResult {
        let account_info = account.info();
        PDASeedSet::invoke_signed_multiple(
            cpi,
            &close_account(
                &spl_token::ID,
                account_info.key(),
                destination.key(),
                authority.key(),
                &[authority.key()],
            )?,
            &[&self.info, account_info, destination, authority],
            seeds,
        )
    }
}
impl<AI, T> MultiIndexable<T> for TokenProgram<AI>
where
    AI: AccountInfo + MultiIndexable<T>,
{
    fn index_is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_owner(owner, indexer)
    }
}
impl<AI, T> SingleIndexable<T> for TokenProgram<AI>
where
    AI: AccountInfo + SingleIndexable<T>,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.info.index_info(indexer)
    }
}
