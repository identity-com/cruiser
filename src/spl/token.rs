//! Implementations for spl tokens

use crate::{
    AccountArgument, AccountInfo, AccountInfoIterator, AllAny, FromAccounts, GeneratorError,
    GeneratorResult, RentExempt,
};
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;
use std::ops::{Deref, DerefMut};

/// Account owned by the token program
#[derive(AccountArgument, Debug)]
pub struct TokenProgramAccount(#[account_argument(owner = &spl_token::ID)] pub AccountInfo);
impl Deref for TokenProgramAccount {
    type Target = AccountInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for TokenProgramAccount {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
delegate_multi_indexable!(TokenProgramAccount, AllAny, (0));
delegate_multi_indexable!(TokenProgramAccount, (), (0));
delegate_single_indexable!(TokenProgramAccount, (), (0));

/// A token account owned by the token program
#[derive(Debug)]
pub struct TokenAccount {
    data: spl_token::state::Account,
    /// The account associated
    pub account: RentExempt<TokenProgramAccount>,
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
    ) -> GeneratorResult<Self> {
        let account: RentExempt<TokenProgramAccount> =
            FromAccounts::from_accounts(program_id, infos, arg)?;
        let rent = Rent::get().unwrap();
        if !rent.is_exempt(**account.lamports.borrow(), account.data.borrow().len()) {
            return Err(GeneratorError::NotEnoughLamports {
                account: account.key,
                lamports: **account.lamports.borrow(),
                needed_lamports: rent.minimum_balance(account.data.borrow().len()),
            }
            .into());
        }
        let data = spl_token::state::Account::unpack(&**account.data.borrow())?;
        Ok(Self { data, account })
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        TokenProgramAccount::accounts_usage_hint()
    }
}
delegate_multi_indexable!(TokenAccount, AllAny, (account));
delegate_multi_indexable!(TokenAccount, (), (account));
delegate_single_indexable!(TokenAccount, (), (account));

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

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        TokenProgramAccount::accounts_usage_hint()
    }
}
delegate_multi_indexable!(MintAccount, AllAny, (account));
delegate_multi_indexable!(MintAccount, (), (account));
delegate_single_indexable!(MintAccount, (), (account));
