use std::ops::Deref;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::CruiserResult;
use cruiser::AccountInfo;
use cruiser_derive::verify_account_arg_impl;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use crate::spl::token::TokenProgramAccount;

verify_account_arg_impl! {
    mod mint_account_check<AI>{
        <AI> MintAccount<AI> where AI: AccountInfo{
            from: [()];
            validate: [()];
            multi: [<I> I where TokenProgramAccount<AI>: MultiIndexable<AI, I>];
            single: [<I> I where TokenProgramAccount<AI>: SingleIndexable<AI, I>];
        }
    }
}

/// A Mint account owned by the token program
#[derive(Debug)]
pub struct MintAccount<AI>
where
    AI: AccountInfo,
{
    data: spl_token::state::Mint,
    /// The account associated
    pub account: TokenProgramAccount<AI>,
}
impl<AI> Deref for MintAccount<AI>
where
    AI: AccountInfo,
{
    type Target = spl_token::state::Mint;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
impl<AI> AccountArgument<AI> for MintAccount<AI>
where
    AI: AccountInfo,
{
    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        self.account.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.account.add_keys(add)
    }
}
impl<AI> FromAccounts<AI, ()> for MintAccount<AI>
where
    AI: AccountInfo,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: (),
    ) -> CruiserResult<Self> {
        let account: TokenProgramAccount<AI> = FromAccounts::from_accounts(program_id, infos, arg)?;
        let data = spl_token::state::Mint::unpack(&*account.0.data())?;
        Ok(Self { data, account })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        TokenProgramAccount::<AI>::accounts_usage_hint(arg)
    }
}
impl<AI> ValidateArgument<AI, ()> for MintAccount<AI>
where
    AI: AccountInfo,
{
    fn validate(&mut self, _program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}
impl<AI, I> MultiIndexable<AI, I> for MintAccount<AI>
where
    AI: AccountInfo,
    TokenProgramAccount<AI>: MultiIndexable<AI, I>,
{
    fn index_is_signer(&self, indexer: I) -> CruiserResult<bool> {
        self.account.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: I) -> CruiserResult<bool> {
        self.account.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: I) -> CruiserResult<bool> {
        self.account.index_is_owner(owner, indexer)
    }
}
impl<AI, I> SingleIndexable<AI, I> for MintAccount<AI>
where
    AI: AccountInfo,
    TokenProgramAccount<AI>: SingleIndexable<AI, I>,
{
    fn index_info(&self, indexer: I) -> CruiserResult<&AI> {
        self.account.index_info(indexer)
    }
}
