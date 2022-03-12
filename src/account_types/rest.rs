//! All remaining accounts to a certain type

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::{AccountInfo, CruiserResult};
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;
use std::iter::once;
use std::ops::{Deref, DerefMut};

verify_account_arg_impl! {
    mod init_account_check{
        <A> Rest<A>
        where
            A: AccountArgument{
            from: [
                () where A: FromAccounts<()>;
                <T> (T,) where A: FromAccounts<T>, T: Clone;
                <T, F> (F, ()) where A: FromAccounts<T>, F: FnMut(usize) -> T;
            ];
            validate: [<T> T where Vec<A>: ValidateArgument<T>];
            multi: [<T> T where Vec<A>: MultiIndexable<T>];
            single: [<T> T where Vec<A>: SingleIndexable<T>];
        }
    }
}

/// An account argument that takes the rest of the accounts as type `A`
#[derive(Debug)]
pub struct Rest<A>(pub Vec<A>);
impl<A> AccountArgument for Rest<A>
where
    A: AccountArgument,
{
    fn write_back(self, program_id: &'static Pubkey) -> CruiserResult<()> {
        self.0.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(&'static Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.0.add_keys(add)
    }
}
impl<A> FromAccounts<()> for Rest<A>
where
    A: FromAccounts<()>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (),
    ) -> CruiserResult<Self> {
        Self::from_accounts(program_id, infos, (arg,))
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        Self::accounts_usage_hint(&(*arg,))
    }
}
impl<A, T> FromAccounts<(T,)> for Rest<A>
where
    A: FromAccounts<T>,
    T: Clone,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        mut infos: &mut impl AccountInfoIterator,
        arg: (T,),
    ) -> CruiserResult<Self> {
        let mut out = match A::accounts_usage_hint(&arg.0).1 {
            Some(0) | None => Vec::new(),
            Some(upper) => Vec::with_capacity(infos.size_hint().0 / upper),
        };
        let mut next = infos.next();
        while let Some(info) = next {
            let mut iter = once(info).chain(&mut infos);
            out.push(A::from_accounts(program_id, &mut iter, arg.0.clone())?);
            next = iter.next();
        }
        Ok(Self(out))
    }

    fn accounts_usage_hint(_arg: &(T,)) -> (usize, Option<usize>) {
        (0, None)
    }
}
impl<A, T, F> FromAccounts<(F, ())> for Rest<A>
where
    A: FromAccounts<T>,
    F: FnMut(usize) -> T,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        mut infos: &mut impl AccountInfoIterator,
        mut arg: (F, ()),
    ) -> CruiserResult<Self> {
        let mut out = Vec::new();
        let mut next = infos.next();
        let mut index = 0;
        while let Some(info) = next {
            let mut iter = once(info).chain(&mut infos);
            out.push(A::from_accounts(program_id, &mut iter, arg.0(index))?);
            next = iter.next();
            index += 1;
        }
        Ok(Self(out))
    }

    fn accounts_usage_hint(_arg: &(F, ())) -> (usize, Option<usize>) {
        (0, None)
    }
}
impl<A, T> ValidateArgument<T> for Rest<A>
where
    A: AccountArgument,
    Vec<A>: ValidateArgument<T>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: T) -> CruiserResult<()> {
        self.0.validate(program_id, arg)
    }
}
impl<A, T> MultiIndexable<T> for Rest<A>
where
    A: AccountArgument,
    Vec<A>: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.0.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.0.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.0.is_owner(owner, indexer)
    }
}
impl<A, T> SingleIndexable<T> for Rest<A>
where
    A: AccountArgument,
    Vec<A>: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> CruiserResult<&AccountInfo> {
        self.0.info(indexer)
    }
}
impl<A> Deref for Rest<A> {
    type Target = Vec<A>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A> DerefMut for Rest<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<A> IntoIterator for Rest<A> {
    type Item = <std::vec::Vec<A> as IntoIterator>::Item;
    type IntoIter = <std::vec::Vec<A> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a, A> IntoIterator for &'a Rest<A>
where
    A: 'a,
{
    type Item = <&'a std::vec::Vec<A> as IntoIterator>::Item;
    type IntoIter = <&'a std::vec::Vec<A> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a, A> IntoIterator for &'a mut Rest<A>
where
    A: 'a,
{
    type Item = <&'a mut std::vec::Vec<A> as IntoIterator>::Item;
    type IntoIter = <&'a mut std::vec::Vec<A> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
