use crate::{
    convert_range, mul_size_hint, sum_size_hints, verify_account_arg_impl, AccountArgument,
    AccountInfo, AccountInfoIterator, AllAny, FromAccounts, GeneratorResult, MultiIndexable,
    Pubkey, SingleIndexable, ValidateArgument,
};
use std::ops::RangeBounds;

verify_account_arg_impl! {
    mod vec_checks{
        <T> Vec<T>
        where
            T: AccountArgument{
            from: [
                usize where T: FromAccounts<()>;
                <A> (usize, (A,)) where T: FromAccounts<A>, A: Clone;
                <A, F> (usize, F, ()) where T: FromAccounts<A>, F: FnMut(usize) -> A;
                <A, const N: usize> [A; N] where T: FromAccounts<A>;
                <A> Vec<A> where T: FromAccounts<A>;
            ];
            validate: [
                () where T: ValidateArgument<()>;
                <A> (A,) where T: ValidateArgument<A>, A: Clone;
                <A, F> (F, ()) where T: ValidateArgument<A>, F: FnMut(usize) -> A;
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

impl<T> AccountArgument for Vec<T>
where
    T: AccountArgument,
{
    fn write_back(self, program_id: &'static Pubkey) -> GeneratorResult<()> {
        for item in self {
            item.write_back(program_id)?;
        }
        Ok(())
    }

    fn add_keys(
        &self,
        mut add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.iter().try_for_each(|inner| inner.add_keys(&mut add))
    }
}
impl<T> FromAccounts<usize> for Vec<T>
where
    T: FromAccounts<()>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: usize,
    ) -> GeneratorResult<Self> {
        (0..arg)
            .map(|_| T::from_accounts(program_id, infos, ()))
            .collect()
    }

    fn accounts_usage_hint(arg: &usize) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(&()), *arg)
    }
}
impl<A, T> FromAccounts<(usize, (A,))> for Vec<T>
where
    T: FromAccounts<A>,
    A: Clone,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (usize, (A,)),
    ) -> GeneratorResult<Self> {
        let mut out = Vec::with_capacity(arg.0);
        if arg.0 != 0 {
            for _ in 0..arg.0 - 1 {
                out.push(T::from_accounts(program_id, infos, arg.1 .0.clone())?);
            }
            out.push(T::from_accounts(program_id, infos, arg.1 .0)?);
        }
        Ok(out)
    }

    fn accounts_usage_hint(arg: &(usize, (A,))) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(&arg.1 .0), arg.0)
    }
}
impl<A, T, F> FromAccounts<(usize, F, ())> for Vec<T>
where
    T: FromAccounts<A>,
    F: FnMut(usize) -> A,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        mut arg: (usize, F, ()),
    ) -> GeneratorResult<Self> {
        let mut out = Vec::with_capacity(arg.0);
        for index in 0..arg.0 {
            out.push(T::from_accounts(program_id, infos, arg.1(index))?);
        }
        Ok(out)
    }

    //TODO: Make this better
    fn accounts_usage_hint(_arg: &(usize, F, ())) -> (usize, Option<usize>) {
        (0, None)
    }
}
impl<A, T, const N: usize> FromAccounts<[A; N]> for Vec<T>
where
    T: FromAccounts<A>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: [A; N],
    ) -> GeneratorResult<Self> {
        Ok(IntoIterator::into_iter(<[T; N]>::from_accounts(program_id, infos, arg)?).collect())
    }

    fn accounts_usage_hint(arg: &[A; N]) -> (usize, Option<usize>) {
        sum_size_hints(arg.iter().map(|arg| T::accounts_usage_hint(arg)))
    }
}
impl<A, T> FromAccounts<Vec<A>> for Vec<T>
where
    T: FromAccounts<A>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: Vec<A>,
    ) -> GeneratorResult<Self> {
        arg.into_iter()
            .map(|arg| T::from_accounts(program_id, infos, arg))
            .collect()
    }

    fn accounts_usage_hint(arg: &Vec<A>) -> (usize, Option<usize>) {
        sum_size_hints(arg.iter().map(|arg| T::accounts_usage_hint(arg)))
    }
}
impl<T> ValidateArgument<()> for Vec<T>
where
    T: ValidateArgument<()>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: ()) -> GeneratorResult<()> {
        self.validate(program_id, (arg,))
    }
}
impl<T, A> ValidateArgument<(A,)> for Vec<T>
where
    T: ValidateArgument<A>,
    A: Clone,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (A,)) -> GeneratorResult<()> {
        self.iter_mut()
            .try_for_each(|val| val.validate(program_id, arg.0.clone()))
    }
}
impl<T, A, F> ValidateArgument<(F, ())> for Vec<T>
where
    T: ValidateArgument<A>,
    F: FnMut(usize) -> A,
{
    fn validate(&mut self, program_id: &'static Pubkey, mut arg: (F, ())) -> GeneratorResult<()> {
        self.iter_mut()
            .enumerate()
            .try_for_each(|(index, val)| val.validate(program_id, arg.0(index)))
    }
}
impl<T> MultiIndexable<usize> for Vec<T>
where
    T: MultiIndexable<()>,
{
    fn is_signer(&self, indexer: usize) -> GeneratorResult<bool> {
        self.is_signer((indexer, ()))
    }

    fn is_writable(&self, indexer: usize) -> GeneratorResult<bool> {
        self.is_writable((indexer, ()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: usize) -> GeneratorResult<bool> {
        self.is_owner(owner, (indexer, ()))
    }
}
impl<T, I> MultiIndexable<(usize, I)> for Vec<T>
where
    T: MultiIndexable<I>,
{
    fn is_signer(&self, indexer: (usize, I)) -> GeneratorResult<bool> {
        self[indexer.0].is_signer(indexer.1)
    }

    fn is_writable(&self, indexer: (usize, I)) -> GeneratorResult<bool> {
        self[indexer.0].is_writable(indexer.1)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (usize, I)) -> GeneratorResult<bool> {
        self[indexer.0].is_owner(owner, indexer.1)
    }
}
impl<T> MultiIndexable<AllAny> for Vec<T>
where
    T: MultiIndexable<()>,
{
    fn is_signer(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.is_signer((indexer, ()))
    }

    fn is_writable(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.is_writable((indexer, ()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> GeneratorResult<bool> {
        self.is_owner(owner, (indexer, ()))
    }
}
impl<T, I> MultiIndexable<(AllAny, I)> for Vec<T>
where
    T: MultiIndexable<I>,
    I: Clone,
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
impl<T, R, I> MultiIndexable<(R, AllAny, I)> for Vec<T>
where
    T: AccountArgument + MultiIndexable<I>,
    R: RangeBounds<usize>,
    I: Clone,
{
    fn is_signer(&self, indexer: (R, AllAny, I)) -> GeneratorResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer
            .1
            .run_func(&self[start..=end], |val| val.is_signer(indexer.2.clone()))
    }

    fn is_writable(&self, indexer: (R, AllAny, I)) -> GeneratorResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer
            .1
            .run_func(&self[start..=end], |val| val.is_writable(indexer.2.clone()))
    }

    fn is_owner(&self, owner: &Pubkey, indexer: (R, AllAny, I)) -> GeneratorResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer.1.run_func(&self[start..=end], |val| {
            val.is_owner(owner, indexer.2.clone())
        })
    }
}
impl<T> SingleIndexable<usize> for Vec<T>
where
    T: SingleIndexable<()>,
{
    fn info(&self, indexer: usize) -> GeneratorResult<&AccountInfo> {
        self.info((indexer, ()))
    }
}
impl<T, I> SingleIndexable<(usize, I)> for Vec<T>
where
    T: AccountArgument + SingleIndexable<I>,
{
    fn info(&self, indexer: (usize, I)) -> GeneratorResult<&AccountInfo> {
        self[indexer.0].info(indexer.1)
    }
}
