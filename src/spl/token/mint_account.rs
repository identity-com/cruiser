use std::ops::Deref;

use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use crate::spl::token::TokenProgramAccount;
use crate::{
    verify_account_arg_impl, AccountInfo, AccountInfoIterator, FromAccounts, GeneratorResult,
    MultiIndexable, SingleIndexable, ValidateArgument,
};

verify_account_arg_impl! {
    mod mint_account_check{
        MintAccount{
            from: [()];
            validate: [()];
            multi: [<I> I where TokenProgramAccount: MultiIndexable<I>];
            single: [<I> I where TokenProgramAccount: SingleIndexable<I>];
        }
    }
}

/// A Mint account owned by the token program
#[derive(Debug)]
pub struct MintAccount {
    data: spl_token::state::Mint,
    /// The account associated
    pub account: TokenProgramAccount,
}
impl Deref for MintAccount {
    type Target = spl_token::state::Mint;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}
delegate_account_argument!(MintAccount, (account));
impl FromAccounts<()> for MintAccount {
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (),
    ) -> GeneratorResult<Self> {
        let account: TokenProgramAccount = FromAccounts::from_accounts(program_id, infos, arg)?;
        let data = spl_token::state::Mint::unpack(&**account.0.data.borrow())?;
        Ok(Self { data, account })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        TokenProgramAccount::accounts_usage_hint(arg)
    }
}
impl ValidateArgument<()> for MintAccount {
    fn validate(&mut self, _program_id: &'static Pubkey, _arg: ()) -> GeneratorResult<()> {
        Ok(())
    }
}
impl<I> MultiIndexable<I> for MintAccount
where
    TokenProgramAccount: MultiIndexable<I>,
{
    fn is_signer(&self, indexer: I) -> GeneratorResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: I) -> GeneratorResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: I) -> GeneratorResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<I> SingleIndexable<I> for MintAccount
where
    TokenProgramAccount: SingleIndexable<I>,
{
    fn info(&self, indexer: I) -> GeneratorResult<&AccountInfo> {
        self.account.info(indexer)
    }
}
