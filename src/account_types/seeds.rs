//! A single account that must come from a given set of seeds

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::pda_seeds::{PDAGenerator, PDASeedSet, PDASeeder};
use crate::{AccountInfo, CruiserResult};
use solana_program::pubkey::Pubkey;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

// verify_account_arg_impl! {
//     mod seeds_check<AI>{
//         <T, S> Seeds<T, S> where AI: AccountInfo, T: AccountArgument<AI>, S: PDASeeder{
//             from: [<Arg> Arg where T: FromAccounts<Arg>];
//             validate: [
//                 <B> (S, B) where T: ValidateArgument + SingleIndexable, B: BumpSeed;
//                 <B, V> (S, B, V) where T: ValidateArgument<V> + SingleIndexable, B: BumpSeed;
//                 <B, V, I> (S, B, V, I) where T: ValidateArgument<V> + SingleIndexable<I>, B: BumpSeed;
//             ];
//             multi: [<Arg> Arg where T: MultiIndexable<Arg>];
//             single: [<Arg> Arg where T: SingleIndexable<Arg>];
//         }
//     }
// }

/// Requires that the address comes from a given seeder. Can use a given bump seed or find the bump seed.
#[derive(Debug)]
pub struct Seeds<T, S>
where
    S: PDASeeder,
{
    /// The wrapped argument
    argument: T,
    seeds: Option<(S, u8)>,
}
impl<'a, T, S> Seeds<T, S>
where
    S: PDASeeder + 'a,
{
    /// Takes the seed set out of this.
    /// Will be [`None`] if [`validate`](ValidateArgument::validate) was never called or this function was called before.
    pub fn take_seed_set(&mut self) -> Option<PDASeedSet<'a>> {
        let seeds = self.seeds.take()?;
        Some(PDASeedSet::new(seeds.0, seeds.1))
    }
}
impl<T, S> Deref for Seeds<T, S>
where
    S: PDASeeder,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.argument
    }
}
impl<T, S> DerefMut for Seeds<T, S>
where
    S: PDASeeder,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.argument
    }
}
impl<T, S> AccountArgument for Seeds<T, S>
where
    T::AccountInfo: AccountInfo,
    T: AccountArgument,
    S: PDASeeder,
{
    type AccountInfo = T::AccountInfo;

    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        self.argument.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.argument.add_keys(add)
    }
}
impl<T, S, Arg> FromAccounts<Arg> for Seeds<T, S>
where
    T::AccountInfo: AccountInfo,
    T: FromAccounts<Arg>,
    S: PDASeeder,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: Arg,
    ) -> CruiserResult<Self> {
        Ok(Self {
            argument: T::from_accounts(program_id, infos, arg)?,
            seeds: None,
        })
    }

    fn accounts_usage_hint(arg: &Arg) -> (usize, Option<usize>) {
        T::accounts_usage_hint(arg)
    }
}
impl<T, S, B> ValidateArgument<(S, B)> for Seeds<T, S>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument + SingleIndexable,
    S: PDASeeder,
    B: BumpSeed,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (S, B)) -> CruiserResult<()> {
        self.validate(program_id, (arg.0, arg.1, (), ()))
    }
}
impl<T, S, B, V> ValidateArgument<(S, B, V)> for Seeds<T, S>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument<V> + SingleIndexable,
    S: PDASeeder,
    B: BumpSeed,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (S, B, V)) -> CruiserResult<()> {
        self.validate(program_id, (arg.0, arg.1, arg.2, ()))
    }
}
impl<T, S, B, V, I> ValidateArgument<(S, B, V, I)> for Seeds<T, S>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument<V> + SingleIndexable<I>,
    S: PDASeeder,
    B: BumpSeed,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (S, B, V, I)) -> CruiserResult<()> {
        self.argument.validate(program_id, arg.2)?;
        let bump_seed = arg
            .1
            .verify_address(&arg.0, program_id, self.index_info(arg.3)?.key())?;
        self.seeds = Some((arg.0, bump_seed));
        Ok(())
    }
}
impl<T, S, Arg> MultiIndexable<Arg> for Seeds<T, S>
where
    T::AccountInfo: AccountInfo,
    T: MultiIndexable<Arg>,
    S: PDASeeder,
{
    fn index_is_signer(&self, indexer: Arg) -> CruiserResult<bool> {
        self.argument.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: Arg) -> CruiserResult<bool> {
        self.argument.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: Arg) -> CruiserResult<bool> {
        self.argument.index_is_owner(owner, indexer)
    }
}
impl<T, S, Arg> SingleIndexable<Arg> for Seeds<T, S>
where
    T::AccountInfo: AccountInfo,
    T: SingleIndexable<Arg>,
    S: PDASeeder,
{
    fn index_info(&self, indexer: Arg) -> CruiserResult<&Self::AccountInfo> {
        self.argument.index_info(indexer)
    }
}

/// A bump seed finder, implementations for [`u8`] and [`Find`]
pub trait BumpSeed {
    /// Verifies a given address and returns the bump seed
    fn verify_address<S>(
        self,
        seeder: &S,
        program_id: &Pubkey,
        address: &Pubkey,
    ) -> CruiserResult<u8>
    where
        S: PDASeeder;
}
impl BumpSeed for u8 {
    fn verify_address<S>(
        self,
        seeder: &S,
        program_id: &Pubkey,
        address: &Pubkey,
    ) -> CruiserResult<u8>
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
        program_id: &Pubkey,
        address: &Pubkey,
    ) -> CruiserResult<u8>
    where
        S: PDASeeder,
    {
        seeder.verify_address_find_nonce(program_id, address)
    }
}
