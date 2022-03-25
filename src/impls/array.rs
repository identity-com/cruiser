use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::util::{convert_range, mul_size_hint, sum_size_hints};
use crate::AllAny;
use crate::{CruiserResult, GenericError};
use array_init::try_array_init;
use cruiser_derive::verify_account_arg_impl;
use solana_program::pubkey::Pubkey;
use std::ops::RangeBounds;

verify_account_arg_impl! {
    mod array_checks<AI>{
        <AI, T, const N: usize>[T; N]
        where
            T: AccountArgument<AI>{
            from: [
                () where T: FromAccounts<AI, ()>;
                <Arg> (Arg,) where T: FromAccounts<AI, Arg>, Arg: Clone;
                <Arg> [Arg; N] where T: FromAccounts<AI, Arg>;
            ];
            validate: [
                () where T: ValidateArgument<AI, ()>;
                <Arg> (Arg,) where T: ValidateArgument<AI, Arg>, Arg: Clone;
                <Arg> [Arg; N] where T: ValidateArgument<AI, Arg>;
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
fn get_index<T, const N: usize>(array: &[T; N], index: usize) -> CruiserResult<&T> {
    array.get(index).ok_or_else(|| {
        GenericError::IndexOutOfRange {
            index: index.to_string(),
            possible_range: format!("[0,{})", array.len()),
        }
        .into()
    })
}

impl<AI, T, const N: usize> AccountArgument<AI> for [T; N]
where
    T: AccountArgument<AI>,
{
    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        self.into_iter()
            .try_for_each(|item| item.write_back(program_id))
    }

    fn add_keys(&self, mut add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.iter().try_for_each(|inner| inner.add_keys(&mut add))
    }
}
impl<AI, T, const N: usize> FromAccounts<AI, ()> for [T; N]
where
    T: FromAccounts<AI, ()>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: (),
    ) -> CruiserResult<Self> {
        try_array_init(|_| T::from_accounts(program_id, infos, arg))
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(arg), N)
    }
}
impl<AI, Arg, T, const N: usize> FromAccounts<AI, (Arg,)> for [T; N]
where
    T: FromAccounts<AI, Arg>,
    Arg: Clone,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: (Arg,),
    ) -> CruiserResult<Self> {
        try_array_init(|_| T::from_accounts(program_id, infos, arg.0.clone()))
    }

    fn accounts_usage_hint(arg: &(Arg,)) -> (usize, Option<usize>) {
        mul_size_hint(T::accounts_usage_hint(&arg.0), N)
    }
}
impl<AI, Arg, T, const N: usize> FromAccounts<AI, [Arg; N]> for [T; N]
where
    T: FromAccounts<AI, Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: [Arg; N],
    ) -> CruiserResult<Self> {
        let mut iter = IntoIterator::into_iter(arg);
        try_array_init(|_| T::from_accounts(program_id, infos, iter.next().unwrap()))
    }

    fn accounts_usage_hint(arg: &[Arg; N]) -> (usize, Option<usize>) {
        sum_size_hints(arg.iter().map(|arg| T::accounts_usage_hint(arg)))
    }
}
impl<AI, T, const N: usize> ValidateArgument<AI, ()> for [T; N]
where
    T: ValidateArgument<AI, ()>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult<()> {
        self.iter_mut()
            .try_for_each(|val| val.validate(program_id, arg))
    }
}
impl<AI, Arg, T, const N: usize> ValidateArgument<AI, (Arg,)> for [T; N]
where
    T: ValidateArgument<AI, Arg>,
    Arg: Clone,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (Arg,)) -> CruiserResult<()> {
        self.iter_mut()
            .try_for_each(|val| val.validate(program_id, arg.0.clone()))
    }
}
impl<AI, Arg, T, const N: usize> ValidateArgument<AI, [Arg; N]> for [T; N]
where
    T: ValidateArgument<AI, Arg>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: [Arg; N]) -> CruiserResult<()> {
        self.iter_mut()
            .zip(arg.into_iter())
            .try_for_each(|(val, arg)| val.validate(program_id, arg))
    }
}
impl<AI, T, const N: usize> MultiIndexable<AI, usize> for [T; N]
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
impl<AI, T, I, const N: usize> MultiIndexable<AI, (usize, I)> for [T; N]
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
impl<AI, T, const N: usize> MultiIndexable<AI, AllAny> for [T; N]
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
impl<AI, T, I, const N: usize> MultiIndexable<AI, (AllAny, I)> for [T; N]
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
impl<AI, T, R, I, const N: usize> MultiIndexable<AI, (R, AllAny, I)> for [T; N]
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
impl<AI, T, const N: usize> SingleIndexable<AI, usize> for [T; N]
where
    T: SingleIndexable<AI, ()>,
{
    fn index_info(&self, indexer: usize) -> CruiserResult<&AI> {
        get_index(self, indexer)?.index_info(())
    }
}
impl<AI, T, I, const N: usize> SingleIndexable<AI, (usize, I)> for [T; N]
where
    T: SingleIndexable<AI, I>,
{
    fn index_info(&self, indexer: (usize, I)) -> CruiserResult<&AI> {
        get_index(self, indexer.0)?.index_info(indexer.1)
    }
}
