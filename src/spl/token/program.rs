use crate::account_argument::{AccountArgument, MultiIndexable, Single, SingleIndexable};
use crate::pda_seeds::PDASeedSet;
use crate::{AccountInfo, AllAny, CruiserResult};
use cruiser_derive::verify_account_arg_impl;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use spl_token::instruction::{close_account, set_authority, transfer, AuthorityType};

use crate::spl::token::TokenAccount;

verify_account_arg_impl! {
    mod token_program_check{
        TokenProgram{
            from: [()];
            validate: [()];
            multi: [(); AllAny];
            single: [()];
        };
    }
}

/// The SPL Token Program. Requires feature
#[derive(AccountArgument, Debug, Clone)]
pub struct TokenProgram {
    /// The program's info
    #[validate(key = &spl_token::ID)]
    pub info: AccountInfo,
}
impl TokenProgram {
    /// Calls the token program's [`set_authority`] instruction
    pub fn set_authority<'a>(
        &self,
        account: &TokenAccount,
        new_authority: &Pubkey,
        owner: &AccountInfo,
        seeds: impl IntoIterator<Item = &'a PDASeedSet<'a>>,
    ) -> ProgramResult {
        let account_info = account.get_info();
        PDASeedSet::invoke_signed_multiple(
            &set_authority(
                &spl_token::ID,
                account_info.key,
                Some(new_authority),
                AuthorityType::AccountOwner,
                owner.key,
                &[owner.key],
            )?,
            &[&self.info, account_info, owner],
            seeds,
        )
    }

    /// Calls the token program's [`transfer`] instruction
    pub fn transfer<'a>(
        &self,
        from: &TokenAccount,
        to: &TokenAccount,
        authority: &AccountInfo,
        amount: u64,
        seeds: impl IntoIterator<Item = &'a PDASeedSet<'a>>,
    ) -> ProgramResult {
        let from_info = from.get_info();
        let to_info = to.get_info();
        PDASeedSet::invoke_signed_multiple(
            &transfer(
                &spl_token::ID,
                from_info.key,
                to_info.key,
                authority.key,
                &[authority.key],
                amount,
            )?,
            &[&self.info, from_info, to_info, authority],
            seeds,
        )
    }

    /// Calls the token program's [`close_account`] instruction
    pub fn close_account<'a>(
        &self,
        account: &TokenAccount,
        destination: &AccountInfo,
        authority: &AccountInfo,
        seeds: impl IntoIterator<Item = &'a PDASeedSet<'a>>,
    ) -> ProgramResult {
        let account_info = account.get_info();
        PDASeedSet::invoke_signed_multiple(
            &close_account(
                &spl_token::ID,
                account_info.key,
                destination.key,
                authority.key,
                &[authority.key],
            )?,
            &[&self.info, account_info, destination, authority],
            seeds,
        )
    }
}
impl<T> MultiIndexable<T> for TokenProgram
where
    AccountInfo: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<T> SingleIndexable<T> for TokenProgram
where
    AccountInfo: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.info.info(indexer)
    }
}
