use std::ops::Deref;

use crate::account_argument::{
    AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable, ValidateArgument,
};
use crate::{AccountInfo, CruiserResult, GenericError};
use cruiser_derive::verify_account_arg_impl;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use crate::spl::token::TokenProgramAccount;

verify_account_arg_impl! {
    mod token_account_check{
        TokenAccount{
            from: [()];
            validate: [(); <'a> Owner<'a>];
            multi: [<I> I where TokenProgramAccount: MultiIndexable<I>];
            single: [<I> I where TokenProgramAccount: SingleIndexable<I>];
        }
    }
}

/// A token account owned by the token program
#[derive(Debug)]
pub struct TokenAccount {
    data: spl_token::state::Account,
    /// The account associated
    pub account: TokenProgramAccount,
}
impl Deref for TokenAccount {
    type Target = spl_token::state::Account;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
delegate_account_argument!(TokenAccount, (account));
impl FromAccounts<()> for TokenAccount {
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (),
    ) -> CruiserResult<Self> {
        let account: TokenProgramAccount = FromAccounts::from_accounts(program_id, infos, arg)?;
        let data = spl_token::state::Account::unpack(&**account.data.borrow())?;
        Ok(Self { data, account })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        TokenProgramAccount::accounts_usage_hint(arg)
    }
}
impl ValidateArgument<()> for TokenAccount {
    fn validate(&mut self, program_id: &'static Pubkey, arg: ()) -> CruiserResult<()> {
        self.account.validate(program_id, arg)?;
        Ok(())
    }
}
/// Validates that the given key is the owner of the [`TokenAccount`]
#[derive(Debug)]
pub struct Owner<'a>(pub &'a Pubkey);
impl ValidateArgument<Owner<'_>> for TokenAccount {
    fn validate(&mut self, program_id: &'static Pubkey, arg: Owner) -> CruiserResult<()> {
        self.validate(program_id, ())?;
        if &self.data.owner == arg.0 {
            Ok(())
        } else {
            Err(GenericError::InvalidAccount {
                account: self.data.owner,
                expected: *arg.0,
            }
            .into())
        }
    }
}
impl<I> MultiIndexable<I> for TokenAccount
where
    TokenProgramAccount: MultiIndexable<I>,
{
    fn is_signer(&self, indexer: I) -> CruiserResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: I) -> CruiserResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: I) -> CruiserResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<I> SingleIndexable<I> for TokenAccount
where
    TokenProgramAccount: SingleIndexable<I>,
{
    fn info(&self, indexer: I) -> CruiserResult<&AccountInfo> {
        self.account.info(indexer)
    }
}
