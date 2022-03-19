use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::util::{convert_range, mul_size_hint, sum_size_hints};
use crate::AllAny;
use crate::{AccountInfo, CruiserResult, GenericError};
use array_init::try_array_init;
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;
use std::ops::RangeBounds;

verify_account_arg_impl! {
    mod array_checks{
        <T, const N: usize>[T; N]
        where
            T: AccountArgument{
            from: [
                () where T: FromAccounts<()>;
                <A> (A,) where T: FromAccounts<A>, A: Clone;
                <A> [A; N] where T: FromAccounts<A>;
            ];
            validate: [
                () where T: ValidateArgument<()>;
                <A> (A,) where T: ValidateArgument<A>, A: Clone;
                <A> [A; N] where T: ValidateArgument<A>;
            ];
            multi: [
                usize where T: MultiIndexable<()>;
                <I> (usize, I) where T: MultiIndexable<I>;
                AllAny where T: MultiIndexable<()>;
                <I> (AllAny, I) where T: MultiIndexable<I>, I: Clone;
                <R, I> (R, AllAny, I) where T: MultiIndexable<I>, R: RangeBounds<usize>, I: Clone;
            ];
            single: [
                usize where T: SingleIndexable<()>;
                <I> (usize, I) where T: SingleIndexable<I>;
            ];
        }
    }
}
fn get_index<T, const N: usize>(array: &[T; N], index: usize) -> CruiserResult<&T> {
    array.get(index).ok_or_else(|| {
        GenericError::IndexOutOfRange {
            index: index.to_string(),
            possible_range: format!("[0,{})", array.len()),
        }
        .into()
    })
}

impl<T, const N: usize> AccountArgument for [T; N]
where
    T: AccountArgument,
{
    fn write_back(self, program_id: &'static Pubkey) -> CruiserResult<()> {
        self.into_iter()
            .try_for_each(|item| item.write_back(program_id))
    }

    fn add_keys(
        &self,
        mut add: impl FnMut(&'static Pubkey) -> CruiserResult<()>,
    ) -> CruiserResult<()> {
        self.iter().try_for_each(|inner| inner.add_keys(&mut add))
    }
}
impl<T, const N: usize> FromAccounts<()> for [T; N]
where
    T: FromAccounts<()>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (),
    ) -> CruiserResult<Self> {
        try_array_init(|_| T::from_accounts(program_id, infos, arg))
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(arg), N)
    }
}
impl<A, T, const N: usize> FromAccounts<(A,)> for [T; N]
where
    T: FromAccounts<A>,
    A: Clone,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (A,),
    ) -> CruiserResult<Self> {
        try_array_init(|_| T::from_accounts(program_id, infos, arg.0.clone()))
    }

    fn accounts_usage_hint(arg: &(A,)) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(&arg.0), N)
    }
}
impl<A, T, const N: usize> FromAccounts<[A; N]> for [T; N]
where
    T: FromAccounts<A>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: [A; N],
    ) -> CruiserResult<Self> {
        let mut iter = IntoIterator::into_iter(arg);
        try_array_init(|_| T::from_accounts(program_id, infos, iter.next().unwrap()))
    }

    fn accounts_usage_hint(arg: &[A; N]) -> (usize, Option<usize>) {
        sum_size_hints(arg.iter().map(|arg| T::accounts_usage_hint(arg)))
    }
}
impl<T, const N: usize> ValidateArgument<()> for [T; N]
where
    T: ValidateArgument<()>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: ()) -> CruiserResult<()> {
        self.iter_mut()
            .try_for_each(|val| val.validate(program_id, arg))
    }
}
impl<A, T, const N: usize> ValidateArgument<(A,)> for [T; N]
where
    T: ValidateArgument<A>,
    A: Clone,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (A,)) -> CruiserResult<()> {
        self.iter_mut()
            .try_for_each(|val| val.validate(program_id, arg.0.clone()))
    }
}
impl<A, T, const N: usize> ValidateArgument<[A; N]> for [T; N]
where
    T: ValidateArgument<A>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: [A; N]) -> CruiserResult<()> {
        self.iter_mut()
            .zip(arg.into_iter())
            .try_for_each(|(val, arg)| val.validate(program_id, arg))
    }
}
impl<T, const N: usize> MultiIndexable<usize> for [T; N]
where
    T: MultiIndexable<()>,
{
    fn is_signer(&self, indexer: usize) -> CruiserResult<bool> {
        self.is_signer((indexer, ()))
    }

    fn is_writable(&self, indexer: usize) -> CruiserResult<bool> {
        self.is_writable((indexer, ()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: usize) -> CruiserResult<bool> {
        self.is_owner(owner, (indexer, ()))
    }
}
impl<T, I, const N: usize> MultiIndexable<(usize, I)> for [T; N]
where
    T: MultiIndexable<I>,
{
    fn is_signer(&self, indexer: (usize, I)) -> CruiserResult<bool> {
        self[indexer.0].is_signer(indexer.1)
    }

    fn is_writable(&self, indexer: (usize, I)) -> CruiserResult<bool> {
        self[indexer.0].is_writable(indexer.1)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (usize, I)) -> CruiserResult<bool> {
        self[indexer.0].is_owner(owner, indexer.1)
    }
}
impl<T, const N: usize> MultiIndexable<AllAny> for [T; N]
where
    T: MultiIndexable<()>,
{
    fn is_signer(&self, indexer: AllAny) -> CruiserResult<bool> {
        self.is_signer((indexer, ()))
    }

    fn is_writable(&self, indexer: AllAny) -> CruiserResult<bool> {
        self.is_writable((indexer, ()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> CruiserResult<bool> {
        self.is_owner(owner, (indexer, ()))
    }
}
impl<T, I, const N: usize> MultiIndexable<(AllAny, I)> for [T; N]
where
    T: AccountArgument + MultiIndexable<I>,
    I: Clone,
{
    fn is_signer(&self, indexer: (AllAny, I)) -> CruiserResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.is_signer(indexer.1.clone()))
    }

    fn is_writable(&self, indexer: (AllAny, I)) -> CruiserResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.is_writable(indexer.1.clone()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (AllAny, I)) -> CruiserResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.is_owner(owner, indexer.1.clone()))
    }
}
impl<T, R, I, const N: usize> MultiIndexable<(R, AllAny, I)> for [T; N]
where
    T: AccountArgument + MultiIndexable<I>,
    R: RangeBounds<usize>,
    I: Clone,
{
    fn is_signer(&self, indexer: (R, AllAny, I)) -> CruiserResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer
            .1
            .run_func(&self[start..=end], |val| val.is_signer(indexer.2.clone()))
    }

    fn is_writable(&self, indexer: (R, AllAny, I)) -> CruiserResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer
            .1
            .run_func(&self[start..=end], |val| val.is_writable(indexer.2.clone()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (R, AllAny, I)) -> CruiserResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer.1.run_func(&self[start..=end], |val| {
            val.is_owner(owner, indexer.2.clone())
        })
    }
}
impl<T, const N: usize> SingleIndexable<usize> for [T; N]
where
    T: SingleIndexable<()>,
{
    fn info(&self, indexer: usize) -> CruiserResult<&AccountInfo> {
        get_index(self, indexer)?.info(())
    }
}
impl<T, I, const N: usize> SingleIndexable<(usize, I)> for [T; N]
where
    T: SingleIndexable<I>,
{
    fn info(&self, indexer: (usize, I)) -> CruiserResult<&AccountInfo> {
        get_index(self, indexer.0)?.info(indexer.1)
    }
}
