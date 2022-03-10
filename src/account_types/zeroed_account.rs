use std::fmt::{Debug, Formatter};
use std::ops::{Deref, DerefMut};

use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use cruiser_derive::verify_account_arg_impl;

use crate::compressed_numbers::CompressedNumber;
use crate::traits::AccountArgument;
use crate::{
    assert_is_owner, AccountInfo, AccountInfoIterator, AccountListItem, AllAny,
    DiscriminantAccount, FromAccounts, GeneratorError, GeneratorResult, MultiIndexable,
    SingleIndexable, ValidateArgument, WriteDiscriminant,
};

verify_account_arg_impl! {
    mod init_account_check{
        <AL, A> ZeroedAccount<AL, A>
        where
            AL: AccountListItem<A>,
            A: BorshSerialize + BorshDeserialize{
            from: [A];
            validate: [(); CheckAll];
            multi: [(); AllAny];
            single: [()];
        }
    }
}

#[derive(AccountArgument)]
#[account_argument(no_from, no_validate)]
pub struct ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    account: DiscriminantAccount<AL, A>,
}
impl<AL, A> Deref for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    type Target = DiscriminantAccount<AL, A>;

    fn deref(&self) -> &Self::Target {
        &self.account
    }
}
impl<AL, A> DerefMut for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.account
    }
}
impl<AL, A> Debug for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("InitAccount")
            .field("account", &self.account)
            .finish()
    }
}
impl<AL, A> FromAccounts<A> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: A,
    ) -> GeneratorResult<Self> {
        Ok(Self {
            account: DiscriminantAccount::from_accounts(program_id, infos, (arg,))?,
        })
    }

    fn accounts_usage_hint(_arg: &A) -> (usize, Option<usize>) {
        DiscriminantAccount::<AL, A>::accounts_usage_hint(&())
    }
}
impl<AL, A> ValidateArgument<()> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn validate(&mut self, program_id: &'static Pubkey, _arg: ()) -> GeneratorResult<()> {
        assert_is_owner(&self.account, program_id, ())?;
        if self.account.info.data.borrow()[..AL::DiscriminantCompressed::max_bytes()]
            .iter()
            .any(|val| *val != 0)
        {
            Err(GeneratorError::NonZeroedData {
                account: self.account.info.key,
            }
            .into())
        } else {
            self.account.validate(program_id, WriteDiscriminant)
        }
    }
}
#[derive(Debug, Copy, Clone)]
pub struct CheckAll;
impl<AL, A> ValidateArgument<CheckAll> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
{
    fn validate(&mut self, program_id: &'static Pubkey, _arg: CheckAll) -> GeneratorResult<()> {
        assert_is_owner(&self.account, program_id, ())?;
        if self.account.info.data.borrow().iter().any(|val| *val != 0) {
            Err(GeneratorError::NonZeroedData {
                account: self.account.info.key,
            }
            .into())
        } else {
            self.account.validate(program_id, WriteDiscriminant)
        }
    }
}
impl<'a, AL, A, T> MultiIndexable<T> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> GeneratorResult<bool> {
        self.account.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> GeneratorResult<bool> {
        self.account.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> GeneratorResult<bool> {
        self.account.is_owner(owner, indexer)
    }
}
impl<'a, AL, A, T> SingleIndexable<T> for ZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize,
    DiscriminantAccount<AL, A>: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> GeneratorResult<&AccountInfo> {
        self.account.info(indexer)
    }
}
