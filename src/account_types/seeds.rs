use crate::{
    AccountArgument, AccountInfoIterator, FromAccounts, GeneratorResult, PDAGenerator, PDASeeder,
    SingleIndexableAccountArgument, SystemProgram,
};
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

/// Requires that the address comes from a given seeder. Can use a given bump seed or find the bump seed.
#[derive(Debug)]
pub struct Seeds<A, S>
where
    A: AccountArgument,
    S: PDASeeder,
{
    /// The wrapped argument
    pub argument: A,
    /// The bump seed of the account
    pub bump_seed: u8,
    phantom_s: PhantomData<fn() -> S>,
}
impl<A, S> Deref for Seeds<A, S>
where
    A: AccountArgument,
    S: PDASeeder,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        &self.argument
    }
}
impl<A, S> DerefMut for Seeds<A, S>
where
    A: AccountArgument,
    S: PDASeeder,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.argument
    }
}
impl<A, S> AccountArgument for Seeds<A, S>
where
    A: AccountArgument,
    S: PDASeeder,
{
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        self.argument.write_back(program_id, system_program)
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.argument.add_keys(add)
    }
}
/// A bump seed finder, implementations for [`u8`] and [`Find`]
pub trait BumpSeed {
    /// Verifies a given address and returns the bump seed
    fn verify_address<S>(
        self,
        seeder: &S,
        program_id: &'static Pubkey,
        address: &Pubkey,
    ) -> GeneratorResult<u8>
    where
        S: PDASeeder;
}
impl BumpSeed for u8 {
    fn verify_address<S>(
        self,
        seeder: &S,
        program_id: &'static Pubkey,
        address: &Pubkey,
    ) -> GeneratorResult<u8>
    where
        S: PDASeeder,
    {
        seeder.verify_address_with_nonce(program_id, address, self)?;
        Ok(self)
    }
}
/// Finds a bump seed rather than using a given one. Can be very compute intensive for specific seeds.
#[derive(Copy, Clone, Debug)]
pub struct Find;
impl BumpSeed for Find {
    fn verify_address<S>(
        self,
        seeder: &S,
        program_id: &'static Pubkey,
        address: &Pubkey,
    ) -> GeneratorResult<u8>
    where
        S: PDASeeder,
    {
        seeder.verify_address_find_nonce(program_id, address)
    }
}
impl<'a, A, S, B> FromAccounts<(&'a S, B)> for Seeds<A, S>
where
    A: FromAccounts<()> + SingleIndexableAccountArgument<()>,
    S: PDASeeder,
    B: BumpSeed,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (&'a S, B),
    ) -> GeneratorResult<Self> {
        Self::from_accounts(program_id, infos, (arg.0, arg.1, ()))
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        A::accounts_usage_hint()
    }
}
impl<'a, A, S, B, D> FromAccounts<(&'a S, B, D)> for Seeds<A, S>
where
    A: FromAccounts<D> + SingleIndexableAccountArgument<()>,
    S: PDASeeder,
    B: BumpSeed,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (&'a S, B, D),
    ) -> GeneratorResult<Self> {
        Self::from_accounts(program_id, infos, (arg.0, arg.1, arg.2, ()))
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        A::accounts_usage_hint()
    }
}

impl<'a, A, S, B, D, I> FromAccounts<(&'a S, B, D, I)> for Seeds<A, S>
where
    A: FromAccounts<D> + SingleIndexableAccountArgument<I>,
    S: PDASeeder,
    B: BumpSeed,
    I: Debug + Clone,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: (&'a S, B, D, I),
    ) -> GeneratorResult<Self> {
        let argument = A::from_accounts(program_id, infos, arg.2)?;
        let account_key = argument.info(arg.3)?.key;
        let bump_seed = arg.1.verify_address(arg.0, program_id, account_key)?;
        Ok(Self {
            argument,
            bump_seed,
            phantom_s: PhantomData,
        })
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        A::accounts_usage_hint()
    }
}
