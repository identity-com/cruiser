use std::ops::RangeBounds;

use array_init::try_array_init;

use crate::{
    mul_size_hint, AccountArgument, AccountInfo, AccountInfoIterator, AllAny, AllAnyRange,
    FromAccounts, GeneratorError, GeneratorResult, MultiIndexable, Pubkey, SingleIndexable,
    SystemProgram, ValidateArgument,
};
use cruiser_derive::verify_account_arg_impl;

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
                <I> (usize, (I,)) where T: MultiIndexable<I>, I: Clone;
                <I> (usize, [I; N]) where T: MultiIndexable<I>;
                AllAny where T: MultiIndexable<()>;
                <I> (AllAny, (I, )) where T: MultiIndexable<I>, I: Clone;
                <I> (AllAny, [I; N]) where T: MultiIndexable<I>;
                <R> AllAnyRange<R> where T: MultiIndexable<()>, R: RangeBounds<usize>;
                <R, I> (AllAnyRange<R>, (I,)) where T: MultiIndexable<I>, R: RangeBounds<usize>, I: Clone;
                <R, I> (AllAnyRange<R>, [I; N]) where T: MultiIndexable<I>, R: RangeBounds<usize>;
            ];
            single: [
                usize where T: SingleIndexable<()>;
                <I> (usize, I) where T: SingleIndexable<I>;
            ];
        }
    }
}

impl<T, const N: usize> AccountArgument for [T; N]
where
    T: AccountArgument,
{
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        self.into_iter()
            .try_for_each(|item| item.write_back(program_id, system_program))
    }

    fn add_keys(
        &self,
        mut add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.iter().try_for_each(|inner| inner.add_keys(&mut add))
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
    ) -> GeneratorResult<Self> {
        let mut iter = IntoIterator::into_iter(arg);
        try_array_init(|_| T::from_accounts(program_id, infos, iter.next().unwrap()))
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(), N)
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
    ) -> GeneratorResult<Self> {
        try_array_init(|_| T::from_accounts(program_id, infos, arg.0.clone()))
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(), N)
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
    ) -> GeneratorResult<Self> {
        try_array_init(|_| T::from_accounts(program_id, infos, arg))
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(), N)
    }
}
impl<T, I, const N: usize> MultiIndexable<(AllAny, I)> for [T; N]
where
    T: AccountArgument + MultiIndexable<I>,
{
    fn is_signer(&self, indexer: (AllAny, I)) -> GeneratorResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.is_signer(indexer.1.clone()))
    }

    fn is_writable(&self, indexer: (AllAny, I)) -> GeneratorResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.is_writable(indexer.1.clone()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (AllAny, I)) -> GeneratorResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.is_owner(owner, indexer.1.clone()))
    }
}
impl<T, I, const N: usize> MultiIndexable<(usize, I)> for [T; N]
where
    T: AccountArgument + MultiIndexable<I>,
{
    fn is_signer(&self, indexer: (usize, I)) -> GeneratorResult<bool> {
        self.get(indexer.0).map_or(
            Err(GeneratorError::IndexOutOfRange {
                index: indexer.0.to_string(),
                possible_range: format!("[0,{})", self.len()),
            }
            .into()),
            |val| val.is_signer(indexer.1),
        )
    }

    fn is_writable(&self, indexer: (usize, I)) -> GeneratorResult<bool> {
        self.get(indexer.0).map_or(
            Err(GeneratorError::IndexOutOfRange {
                index: indexer.0.to_string(),
                possible_range: format!("[0,{})", self.len()),
            }
            .into()),
            |val| val.is_writable(indexer.1),
        )
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (usize, I)) -> GeneratorResult<bool> {
        self.get(indexer.0).map_or(
            Err(GeneratorError::IndexOutOfRange {
                index: indexer.0.to_string(),
                possible_range: format!("[0,{})", self.len()),
            }
            .into()),
            |val| val.is_owner(owner, indexer.1),
        )
    }
}
impl<T, I, const N: usize> SingleIndexable<(usize, I)> for [T; N]
where
    T: AccountArgument + SingleIndexable<I>,
{
    fn info(&self, indexer: (usize, I)) -> GeneratorResult<&AccountInfo> {
        self[indexer.0].info(indexer.1)
    }
}
impl<T, R, I, const N: usize> MultiIndexable<(AllAnyRange<R>, (I,))> for [T; N]
where
    T: AccountArgument + MultiIndexable<I>,
    R: RangeBounds<usize>,
    I: Clone,
{
    fn is_signer(&self, indexer: (AllAnyRange<R>, (I,))) -> GeneratorResult<bool> {
        let (start, end) = crate::convert_range(&indexer.0.range, self.len())?;
        indexer
            .0
            .all_any
            .run_func(self.iter().skip(start).take(end - start + 1), |val| {
                val.is_signer(indexer.1 .0.clone())
            })
    }

    fn is_writable(&self, indexer: (AllAnyRange<R>, (I,))) -> GeneratorResult<bool> {
        let (start, end) = crate::convert_range(&indexer.0.range, self.len())?;
        indexer
            .0
            .all_any
            .run_func(self.iter().skip(start).take(end - start + 1), |val| {
                val.is_writable(indexer.1 .0.clone())
            })
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (AllAnyRange<R>, (I,))) -> GeneratorResult<bool> {
        let (start, end) = crate::convert_range(&indexer.0.range, self.len())?;
        indexer
            .0
            .all_any
            .run_func(self.iter().skip(start).take(end - start + 1), |val| {
                val.is_owner(owner, indexer.1 .0.clone())
            })
    }
}
