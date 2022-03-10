use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use spl_token::instruction::AuthorityType;

use cruiser_derive::verify_account_arg_impl;

use crate::{
    AccountArgument, AccountInfo, AllAny, GeneratorResult, invoke, MultiIndexable, PDASeedSet,
    Single, SingleIndexable,
};
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

#[derive(AccountArgument, Debug, Clone)]
pub struct TokenProgram {
    #[validate(key = &spl_token::ID)]
    pub info: AccountInfo,
}
impl TokenProgram {
    pub fn invoke_set_authority(
        &self,
        account: &TokenAccount,
        new_authority: &Pubkey,
        owner: &AccountInfo,
    ) -> ProgramResult {
        let account_info = account.get_info();
        invoke(
            &spl_token::instruction::set_authority(
                &spl_token::ID,
                account_info.key,
                Some(new_authority),
                AuthorityType::AccountOwner,
                owner.key,
                &[owner.key],
            )?,
            &[&self.info, account_info, owner],
        )
    }

    pub fn invoke_transfer(
        &self,
        from: &TokenAccount,
        to: &TokenAccount,
        authority: &AccountInfo,
        amount: u64,
    ) -> ProgramResult {
        let from_info = from.get_info();
        let to_info = to.get_info();
        invoke(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                from_info.key,
                to_info.key,
                authority.key,
                &[authority.key],
                amount,
            )?,
            &[&self.info, from_info, to_info, authority],
        )
    }

    pub fn invoke_signed_transfer(
        &self,
        seeds: &PDASeedSet,
        from: &TokenAccount,
        to: &TokenAccount,
        authority: &AccountInfo,
        amount: u64,
    ) -> ProgramResult {
        let from_info = from.get_info();
        let to_info = to.get_info();
        seeds.invoke_signed(
            &spl_token::instruction::transfer(
                &spl_token::ID,
                from_info.key,
                to_info.key,
                authority.key,
                &[authority.key],
                amount,
            )?,
            &[&self.info, from_info, to_info, authority],
        )
    }

    pub fn invoke_close_account(
        &self,
        account: &TokenAccount,
        destination: &AccountInfo,
        authority: &AccountInfo,
    ) -> ProgramResult {
        let account_info = account.get_info();
        invoke(
            &spl_token::instruction::close_account(
                &spl_token::ID,
                account_info.key,
                destination.key,
                authority.key,
                &[authority.key],
            )?,
            &[&self.info, account_info, destination, authority],
        )
    }

    pub fn invoke_signed_close_account(
        &self,
        seeds: &PDASeedSet,
        account: &TokenAccount,
        destination: &AccountInfo,
        authority: &AccountInfo,
    ) -> ProgramResult {
        let account_info = account.get_info();
        seeds.invoke_signed(
            &spl_token::instruction::close_account(
                &spl_token::ID,
                account_info.key,
                destination.key,
                authority.key,
                &[authority.key],
            )?,
            &[&self.info, account_info, destination, authority],
        )
    }
}
impl<T> MultiIndexable<T> for TokenProgram
where
    AccountInfo: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> GeneratorResult<bool> {
        self.info.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> GeneratorResult<bool> {
        self.info.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> GeneratorResult<bool> {
        self.info.is_owner(owner, indexer)
    }
}
impl<T> SingleIndexable<T> for TokenProgram
where
    AccountInfo: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> GeneratorResult<&AccountInfo> {
        self.info.info(indexer)
    }
}
