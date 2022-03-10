use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use solana_program::pubkey::Pubkey;

use cruiser::PDASeedSet;
use cruiser_derive::verify_account_arg_impl;

use crate::{
    AccountArgument, AccountInfo, AccountInfoIterator, FromAccounts, GeneratorResult,
    MultiIndexable, PDAGenerator, PDASeeder, SingleIndexable, ValidateArgument,
};

verify_account_arg_impl! {
    mod seeds_check{
        <A, S> Seeds<A, S> where A: AccountArgument, S: PDASeeder{
            from: [<T> T where A: FromAccounts<T>];
            validate: [
                <B> (S, B) where A: ValidateArgument<()> + SingleIndexable<()>, B: BumpSeed;
                <B, V> (S, B, V) where A: ValidateArgument<V> + SingleIndexable<()>, B: BumpSeed;
                <B, V, I> (S, B, V, I) where A: ValidateArgument<V> + SingleIndexable<I>, B: BumpSeed;
            ];
            multi: [<T> T where A: MultiIndexable<T>];
            single: [<T> T where A: SingleIndexable<T>];
        }
    }
}

/// Requires that the address comes from a given seeder. Can use a given bump seed or find the bump seed.
#[derive(Debug)]
pub struct Seeds<A, S>
where
    A: AccountArgument,
    S: PDASeeder,
{
    /// The wrapped argument
    argument: A,
    seeds: Option<(S, u8)>,
}
impl<'a, A, S> Seeds<A, S>
where
    A: AccountArgument,
    S: PDASeeder + 'a,
{
    pub fn take_seed_set(&mut self) -> Option<PDASeedSet<'a>> {
        let seeds = self.seeds.take()?;
        Some(PDASeedSet::new(seeds.0, seeds.1))
    }
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
    fn write_back(self, program_id: &'static Pubkey) -> GeneratorResult<()> {
        self.argument.write_back(program_id)
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.argument.add_keys(add)
    }
}
impl<A, S, T> FromAccounts<T> for Seeds<A, S>
where
    A: FromAccounts<T>,
    S: PDASeeder,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: T,
    ) -> GeneratorResult<Self> {
        Ok(Self {
            argument: A::from_accounts(program_id, infos, arg)?,
            seeds: None,
        })
    }

    fn accounts_usage_hint(arg: &T) -> (usize, Option<usize>) {
        A::accounts_usage_hint(arg)
    }
}
impl<A, S, B> ValidateArgument<(S, B)> for Seeds<A, S>
where
    A: ValidateArgument<()> + SingleIndexable<()>,
    S: PDASeeder,
    B: BumpSeed,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (S, B)) -> GeneratorResult<()> {
        self.validate(program_id, (arg.0, arg.1, (), ()))
    }
}
impl<A, S, B, V> ValidateArgument<(S, B, V)> for Seeds<A, S>
where
    A: ValidateArgument<V> + SingleIndexable<()>,
    S: PDASeeder,
    B: BumpSeed,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (S, B, V)) -> GeneratorResult<()> {
        self.validate(program_id, (arg.0, arg.1, arg.2, ()))
    }
}
impl<A, S, B, V, I> ValidateArgument<(S, B, V, I)> for Seeds<A, S>
where
    A: ValidateArgument<V> + SingleIndexable<I>,
    S: PDASeeder,
    B: BumpSeed,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: (S, B, V, I)) -> GeneratorResult<()> {
        self.argument.validate(program_id, arg.2)?;
        let bump_seed = arg
            .1
            .verify_address(&arg.0, program_id, self.info(arg.3)?.key)?;
        self.seeds = Some((arg.0, bump_seed));
        Ok(())
    }
}
impl<A, S, T> MultiIndexable<T> for Seeds<A, S>
where
    A: MultiIndexable<T>,
    S: PDASeeder,
{
    fn is_signer(&self, indexer: T) -> GeneratorResult<bool> {
        self.argument.is_signer(indexer)
    }

    fn is_writable(&self, indexer: T) -> GeneratorResult<bool> {
        self.argument.is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> GeneratorResult<bool> {
        self.argument.is_owner(owner, indexer)
    }
}
impl<A, S, T> SingleIndexable<T> for Seeds<A, S>
where
    A: SingleIndexable<T>,
    S: PDASeeder,
{
    fn info(&self, indexer: T) -> GeneratorResult<&AccountInfo> {
        self.argument.info(indexer)
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
