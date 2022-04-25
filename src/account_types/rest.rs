//! All remaining accounts to a certain type

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::CruiserResult;
use solana_program::pubkey::Pubkey;
use std::iter::once;
use std::ops::{Deref, DerefMut};

// verify_account_arg_impl! {
//     mod init_account_check<AI>{
//         <T> Rest<T>
//         where
//             T: AccountArgument<AI>{
//             from: [
//                 () where T: FromAccounts;
//                 <Arg> (Arg,) where T: FromAccounts<Arg>, Arg: Clone;
//                 <Arg, F> (F, ()) where T: FromAccounts<Arg>, F: FnMut(usize) -> Arg;
//             ];
//             validate: [<Arg> Arg where Vec<T>: ValidateArgument<Arg>];
//             multi: [<Arg> Arg where Vec<T>: MultiIndexable<Arg>];
//             single: [<Arg> Arg where Vec<T>: SingleIndexable<Arg>];
//         }
//     }
// }

/// An account argument that takes the rest of the accounts as type `A`
#[derive(Debug)]
pub struct Rest<T>(pub Vec<T>);
impl<T> AccountArgument for Rest<T>
where
    T: AccountArgument,
{
    type AccountInfo = T::AccountInfo;

    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        self.0.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.0.add_keys(add)
    }
}
impl<T> FromAccounts for Rest<T>
where
    T: FromAccounts,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: (),
    ) -> CruiserResult<Self> {
        Self::from_accounts(program_id, infos, (arg,))
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        Self::accounts_usage_hint(&(*arg,))
    }
}
impl<T, Arg> FromAccounts<(Arg,)> for Rest<T>
where
    T: FromAccounts<Arg>,
    Arg: Clone,
{
    fn from_accounts(
        program_id: &Pubkey,
        mut infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: (Arg,),
    ) -> CruiserResult<Self> {
        let mut out = match T::accounts_usage_hint(&arg.0).1 {
            Some(0) | None => Vec::new(),
            Some(upper) => Vec::with_capacity(infos.size_hint().0 / upper),
        };
        let mut next = infos.next();
        while let Some(info) = next {
            let mut iter = once(info).chain(&mut infos);
            out.push(T::from_accounts(program_id, &mut iter, arg.0.clone())?);
            next = iter.next();
        }
        Ok(Self(out))
    }

    fn accounts_usage_hint(_arg: &(Arg,)) -> (usize, Option<usize>) {
        (0, None)
    }
}
impl<T, Arg, F> FromAccounts<(F, ())> for Rest<T>
where
    T: FromAccounts<Arg>,
    F: FnMut(usize) -> Arg,
{
    fn from_accounts(
        program_id: &Pubkey,
        mut infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        mut arg: (F, ()),
    ) -> CruiserResult<Self> {
        let mut out = Vec::new();
        let mut next = infos.next();
        let mut index = 0;
        while let Some(info) = next {
            let mut iter = once(info).chain(&mut infos);
            out.push(T::from_accounts(program_id, &mut iter, arg.0(index))?);
            next = iter.next();
            index += 1;
        }
        Ok(Self(out))
    }

    fn accounts_usage_hint(_arg: &(F, ())) -> (usize, Option<usize>) {
        (0, None)
    }
}
impl<T, Arg> ValidateArgument<Arg> for Rest<T>
where
    T: AccountArgument,
    Vec<T>: ValidateArgument<Arg>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: Arg) -> CruiserResult<()> {
        self.0.validate(program_id, arg)
    }
}
impl<T, Arg> MultiIndexable<Arg> for Rest<T>
where
    T: AccountArgument,
    Vec<T>: MultiIndexable<Arg>,
{
    fn index_is_signer(&self, indexer: Arg) -> CruiserResult<bool> {
        self.0.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: Arg) -> CruiserResult<bool> {
        self.0.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: Arg) -> CruiserResult<bool> {
        self.0.index_is_owner(owner, indexer)
    }
}
impl<T, Arg> SingleIndexable<Arg> for Rest<T>
where
    T: AccountArgument,
    Vec<T>: SingleIndexable<Arg, AccountInfo = T::AccountInfo>,
{
    fn index_info(&self, indexer: Arg) -> CruiserResult<&Self::AccountInfo> {
        self.0.index_info(indexer)
    }
}
impl<T> Deref for Rest<T> {
    type Target = Vec<T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> DerefMut for Rest<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> IntoIterator for Rest<T> {
    type Item = <std::vec::Vec<T> as IntoIterator>::Item;
    type IntoIter = <std::vec::Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
impl<'a, T> IntoIterator for &'a Rest<T>
where
    T: 'a,
{
    type Item = <&'a std::vec::Vec<T> as IntoIterator>::Item;
    type IntoIter = <&'a std::vec::Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}
impl<'a, T> IntoIterator for &'a mut Rest<T>
where
    T: 'a,
{
    type Item = <&'a mut std::vec::Vec<T> as IntoIterator>::Item;
    type IntoIter = <&'a mut std::vec::Vec<T> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter_mut()
    }
}
