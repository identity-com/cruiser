use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::util::{convert_range, mul_size_hint, sum_size_hints};
use crate::AllAny;
use crate::CruiserResult;
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;
use std::ops::RangeBounds;

verify_account_arg_impl! {
    mod vec_checks<AI> {
        <AI, T> Vec<T>
        where
            T: AccountArgument<AI>{
            from: [
                usize where T: FromAccounts<AI, ()>;
                <Arg> (usize, (Arg,)) where T: FromAccounts<AI, Arg>, Arg: Clone;
                <Arg, F> (usize, F, ()) where T: FromAccounts<AI, Arg>, F: FnMut(usize) -> Arg;
                <Arg, const N: usize> [Arg; N] where T: FromAccounts<AI, Arg>;
                <Arg> Vec<Arg> where T: FromAccounts<AI, Arg>;
            ];
            validate: [
                () where T: ValidateArgument<AI, ()>;
                <Arg> (Arg,) where T: ValidateArgument<AI, Arg>, Arg: Clone;
                <Arg, F> (F, ()) where T: ValidateArgument<AI, Arg>, F: FnMut(usize) -> Arg;
            ];
            multi: [
                usize where T: MultiIndexable<AI, ()>;
                <I> (usize, I) where T: MultiIndexable<AI, I>;
                AllAny where T: MultiIndexable<AI, ()>;
                <I> (AllAny, I) where T: MultiIndexable<AI, I>, I: Clone;
                <R, I> (R, AllAny, I) where T: MultiIndexable<AI, I>, R: RangeBounds<usize>, I: Clone;
            ];
            single: [
                usize where T: SingleIndexable<AI, ()>;
                <I> (usize, I) where T: SingleIndexable<AI, I>;
            ];
        }
    }
}

impl<AI, T> AccountArgument<AI> for Vec<T>
where
    T: AccountArgument<AI>,
{
    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        for item in self {
            item.write_back(program_id)?;
        }
        Ok(())
    }

    fn add_keys(&self, mut add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.iter().try_for_each(|inner| inner.add_keys(&mut add))
    }
}
impl<AI, T> FromAccounts<AI, usize> for Vec<T>
where
    T: FromAccounts<AI, ()>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: usize,
    ) -> CruiserResult<Self> {
        (0..arg)
            .map(|_| T::from_accounts(program_id, infos, ()))
            .collect()
    }

    fn accounts_usage_hint(arg: &usize) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(&()), *arg)
    }
}
impl<AI, Arg, T> FromAccounts<AI, (usize, (Arg,))> for Vec<T>
where
    T: FromAccounts<AI, Arg>,
    Arg: Clone,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: (usize, (Arg,)),
    ) -> CruiserResult<Self> {
        let mut out = Vec::with_capacity(arg.0);
        if arg.0 != 0 {
            for _ in 0..arg.0 - 1 {
                out.push(T::from_accounts(program_id, infos, arg.1 .0.clone())?);
            }
            out.push(T::from_accounts(program_id, infos, arg.1 .0)?);
        }
        Ok(out)
    }

    fn accounts_usage_hint(arg: &(usize, (Arg,))) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(&arg.1 .0), arg.0)
    }
}
impl<AI, Arg, T, F> FromAccounts<AI, (usize, F, ())> for Vec<T>
where
    T: FromAccounts<AI, Arg>,
    F: FnMut(usize) -> Arg,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        mut arg: (usize, F, ()),
    ) -> CruiserResult<Self> {
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
impl<AI, Arg, T, const N: usize> FromAccounts<AI, [Arg; N]> for Vec<T>
where
    T: FromAccounts<AI, Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: [Arg; N],
    ) -> CruiserResult<Self> {
        Ok(IntoIterator::into_iter(<[T; N]>::from_accounts(program_id, infos, arg)?).collect())
    }

    fn accounts_usage_hint(arg: &[Arg; N]) -> (usize, Option<usize>) {
        sum_size_hints(arg.iter().map(|arg| T::accounts_usage_hint(arg)))
    }
}
impl<AI, Arg, T> FromAccounts<AI, Vec<Arg>> for Vec<T>
where
    T: FromAccounts<AI, Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: Vec<Arg>,
    ) -> CruiserResult<Self> {
        arg.into_iter()
            .map(|arg| T::from_accounts(program_id, infos, arg))
            .collect()
    }

    fn accounts_usage_hint(arg: &Vec<Arg>) -> (usize, Option<usize>) {
        sum_size_hints(arg.iter().map(|arg| T::accounts_usage_hint(arg)))
    }
}
impl<AI, T> ValidateArgument<AI, ()> for Vec<T>
where
    T: ValidateArgument<AI, ()>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult<()> {
        self.validate(program_id, (arg,))
    }
}
impl<AI, T, Arg> ValidateArgument<AI, (Arg,)> for Vec<T>
where
    T: ValidateArgument<AI, Arg>,
    Arg: Clone,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (Arg,)) -> CruiserResult<()> {
        self.iter_mut()
            .try_for_each(|val| val.validate(program_id, arg.0.clone()))
    }
}
impl<AI, T, Arg, F> ValidateArgument<AI, (F, ())> for Vec<T>
where
    T: ValidateArgument<AI, Arg>,
    F: FnMut(usize) -> Arg,
{
    fn validate(&mut self, program_id: &Pubkey, mut arg: (F, ())) -> CruiserResult<()> {
        self.iter_mut()
            .enumerate()
            .try_for_each(|(index, val)| val.validate(program_id, arg.0(index)))
    }
}
impl<AI, T> MultiIndexable<AI, usize> for Vec<T>
where
    T: MultiIndexable<AI, ()>,
{
    fn index_is_signer(&self, indexer: usize) -> CruiserResult<bool> {
        self.index_is_signer((indexer, ()))
    }

    fn index_is_writable(&self, indexer: usize) -> CruiserResult<bool> {
        self.index_is_writable((indexer, ()))
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: usize) -> CruiserResult<bool> {
        self.index_is_owner(owner, (indexer, ()))
    }
}
impl<AI, T, I> MultiIndexable<AI, (usize, I)> for Vec<T>
where
    T: MultiIndexable<AI, I>,
{
    fn index_is_signer(&self, indexer: (usize, I)) -> CruiserResult<bool> {
        self[indexer.0].index_is_signer(indexer.1)
    }

    fn index_is_writable(&self, indexer: (usize, I)) -> CruiserResult<bool> {
        self[indexer.0].index_is_writable(indexer.1)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: (usize, I)) -> CruiserResult<bool> {
        self[indexer.0].index_is_owner(owner, indexer.1)
    }
}
impl<AI, T> MultiIndexable<AI, AllAny> for Vec<T>
where
    T: MultiIndexable<AI, ()>,
{
    fn index_is_signer(&self, indexer: AllAny) -> CruiserResult<bool> {
        self.index_is_signer((indexer, ()))
    }

    fn index_is_writable(&self, indexer: AllAny) -> CruiserResult<bool> {
        self.index_is_writable((indexer, ()))
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: AllAny) -> CruiserResult<bool> {
        self.index_is_owner(owner, (indexer, ()))
    }
}
impl<AI, T, I> MultiIndexable<AI, (AllAny, I)> for Vec<T>
where
    T: MultiIndexable<AI, I>,
    I: Clone,
{
    fn index_is_signer(&self, indexer: (AllAny, I)) -> CruiserResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.index_is_signer(indexer.1.clone()))
    }

    fn index_is_writable(&self, indexer: (AllAny, I)) -> CruiserResult<bool> {
        indexer
            .0
            .run_func(self.iter(), |val| val.index_is_writable(indexer.1.clone()))
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: (AllAny, I)) -> CruiserResult<bool> {
        indexer.0.run_func(self.iter(), |val| {
            val.index_is_owner(owner, indexer.1.clone())
        })
    }
}
impl<AI, T, R, I> MultiIndexable<AI, (R, AllAny, I)> for Vec<T>
where
    T: MultiIndexable<AI, I>,
    R: RangeBounds<usize>,
    I: Clone,
{
    fn index_is_signer(&self, indexer: (R, AllAny, I)) -> CruiserResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer.1.run_func(&self[start..=end], |val| {
            val.index_is_signer(indexer.2.clone())
        })
    }

    fn index_is_writable(&self, indexer: (R, AllAny, I)) -> CruiserResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer.1.run_func(&self[start..=end], |val| {
            val.index_is_writable(indexer.2.clone())
        })
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: (R, AllAny, I)) -> CruiserResult<bool> {
        let (start, end) = convert_range(&indexer.0, self.len())?;
        indexer.1.run_func(&self[start..=end], |val| {
            val.index_is_owner(owner, indexer.2.clone())
        })
    }
}
impl<AI, T> SingleIndexable<AI, usize> for Vec<T>
where
    T: SingleIndexable<AI, ()>,
{
    fn index_info(&self, indexer: usize) -> CruiserResult<&AI> {
        self.index_info((indexer, ()))
    }
}
impl<AI, T, I> SingleIndexable<AI, (usize, I)> for Vec<T>
where
    T: SingleIndexable<AI, I>,
{
    fn index_info(&self, indexer: (usize, I)) -> CruiserResult<&AI> {
        self[indexer.0].index_info(indexer.1)
    }
}
