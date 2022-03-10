use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use cruiser_derive::verify_account_arg_impl;

use crate::{
    AccountArgument, AccountInfo, AccountInfoIterator, FromAccounts, GeneratorError,
    GeneratorResult, MultiIndexable, SingleIndexable, ValidateArgument,
};

verify_account_arg_impl! {
    mod rent_exempt_check{
        <A> RentExempt<A> where A: AccountArgument{
            from: [
                <T> T where A: FromAccounts<T>;
            ];
            validate: [
                () where A: ValidateArgument<()> + SingleIndexable<()>;
                Rent where A: ValidateArgument<()> + SingleIndexable<()>;
                <T> (T,) where A: ValidateArgument<T> + SingleIndexable<()>;
                <T, I> (T, I) where A: ValidateArgument<T> + SingleIndexable<I>;
                <T, I> (T, I, Rent) where A: ValidateArgument<T> + SingleIndexable<I>;
            ];
            multi: [<I> I where A: MultiIndexable<I>];
            single: [<I> I where A: SingleIndexable<I>];
        }
    }
}

#[derive(Debug)]
pub struct RentExempt<A>(pub A);
impl<A> Deref for RentExempt<A> {
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A> DerefMut for RentExempt<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<A> AccountArgument for RentExempt<A>
where
    A: AccountArgument,
{
    fn write_back(self, program_id: &'static Pubkey) -> GeneratorResult<()> {
        self.0.write_back(program_id)
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.0.add_keys(add)
    }
}
impl<A, T> FromAccounts<T> for RentExempt<A>
where
    A: FromAccounts<T>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: T,
    ) -> GeneratorResult<Self> {
        Ok(Self(A::from_accounts(program_id, infos, arg)?))
    }

    fn accounts_usage_hint(arg: &T) -> (usize, Option<usize>) {
        A::accounts_usage_hint(arg)
    }
}
impl<A> ValidateArgument<()> for RentExempt<A>
where
    A: ValidateArgument<()> + SingleIndexable<()>,
{
    fn validate(&mut self, program_id: &'static Pubkey, _arg: ()) -> GeneratorResult<()> {
        self.validate(program_id, Rent::get()?)
    }
}
impl<A> ValidateArgument<Rent> for RentExempt<A>
where
    A: ValidateArgument<()> + SingleIndexable<()>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: Rent) -> GeneratorResult<()> {
        self.validate(program_id, ((), (), arg))
    }
}
impl<A, T> ValidateArgument<(T,)> for RentExempt<A>
where
    A: ValidateArgument<T> + SingleIndexable<()>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (T,)) -> GeneratorResult<()> {
        self.validate(program_id, (arg.0, (), Rent::get()?))
    }
}
impl<A, T, I> ValidateArgument<(T, I)> for RentExempt<A>
where
    A: ValidateArgument<T> + SingleIndexable<I>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (T, I)) -> GeneratorResult<()> {
        self.validate(program_id, (arg.0, arg.1, Rent::get()?))
    }
}
impl<A, T, I> ValidateArgument<(T, I, Rent)> for RentExempt<A>
where
    A: ValidateArgument<T> + SingleIndexable<I>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (T, I, Rent)) -> GeneratorResult<()> {
        self.0.validate(program_id, arg.0)?;
        let info = self.0.info(arg.1)?;
        let lamports = **info.lamports.borrow();
        let needed_lamports = arg.2.minimum_balance(info.data.borrow().len());
        if lamports < needed_lamports {
            Err(GeneratorError::NotEnoughLamports {
                account: info.key,
                lamports,
                needed_lamports,
            }
            .into())
        } else {
            Ok(())
        }
    }
}
impl<T, A> MultiIndexable<T> for RentExempt<A>
where
    A: MultiIndexable<T>,
{
    #[inline]
    fn is_signer(&self, indexer: T) -> GeneratorResult<bool> {
        self.0.is_signer(indexer)
    }

    #[inline]
    fn is_writable(&self, indexer: T) -> GeneratorResult<bool> {
        self.0.is_writable(indexer)
    }

    #[inline]
    fn is_owner(&self, owner: &Pubkey, indexer: T) -> GeneratorResult<bool> {
        self.0.is_owner(owner, indexer)
    }
}
impl<T, A> SingleIndexable<T> for RentExempt<A>
where
    A: SingleIndexable<T>,
{
    #[inline]
    fn info(&self, indexer: T) -> GeneratorResult<&AccountInfo> {
        self.0.info(indexer)
    }
}
