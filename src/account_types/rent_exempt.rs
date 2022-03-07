use crate::{
    AccountArgument, AccountInfo, AccountInfoIterator, FromAccounts, GeneratorError,
    GeneratorResult, MultiIndexable, SingleIndexable, SystemProgram,
};
use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

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
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        self.0.write_back(program_id, system_program)
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.0.add_keys(add)
    }
}
impl<A> FromAccounts<()> for RentExempt<A>
where
    A: FromAccounts<()> + SingleIndexable<()>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        _arg: (),
    ) -> GeneratorResult<Self> {
        Self::from_accounts(program_id, infos, Rent::default())
    }
}
impl<A> FromAccounts<Rent> for RentExempt<A>
where
    A: FromAccounts<()> + SingleIndexable<()>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: Rent,
    ) -> GeneratorResult<Self> {
        Self::from_accounts(program_id, infos, (arg, (), ()))
    }
}
impl<A, T> FromAccounts<(Rent, T)> for RentExempt<A>
where
    A: FromAccounts<T> + SingleIndexable<()>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (Rent, T),
    ) -> GeneratorResult<Self> {
        Self::from_accounts(program_id, infos, (arg.0, arg.1, ()))
    }
}
impl<A, T, I> FromAccounts<(Rent, T, I)> for RentExempt<A>
where
    A: FromAccounts<T> + SingleIndexable<I>,
    I: Debug + Clone,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (Rent, T, I),
    ) -> GeneratorResult<Self> {
        let account = A::from_accounts(program_id, infos, arg.1)?;
        let info = account.info(arg.2)?;
        let lamports = **info.lamports.borrow();
        let needed_lamports = arg.0.minimum_balance(info.data.borrow().len());
        if lamports < needed_lamports {
            Err(GeneratorError::NotEnoughLamports {
                account: info.key,
                lamports,
                needed_lamports,
            }
            .into())
        } else {
            Ok(RentExempt(account))
        }
    }
}
impl<T, A> MultiIndexable<T> for RentExempt<A>
where
    T: Debug + Clone,
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
    T: Debug + Clone,
    A: SingleIndexable<T>,
{
    #[inline]
    fn info(&self, indexer: T) -> GeneratorResult<&AccountInfo> {
        self.0.info(indexer)
    }
}
