use crate::solana_program::entrypoint::ProgramResult;
use crate::solana_program::pubkey::PubkeyError;
use crate::{
    invoke_signed, invoke_signed_variable_size, AccountInfo, GeneratorError, GeneratorResult,
    Pubkey, SolanaInstruction,
};
use std::fmt::Debug;
use std::iter::{once, Chain, Map, Once};

/// A set of seeds for a pda
#[derive(Debug)]
pub struct PDASeedSet<'a> {
    /// The seeder for these seeds
    pub seeder: Box<dyn PDASeeder + 'a>,
    /// The nonce of the account
    pub nonce: [u8; 1],
}
impl<'a> PDASeedSet<'a> {
    /// Creates a new set of seeds
    pub fn new(seeder: impl PDASeeder + 'a, nonce: u8) -> Self {
        Self::from_boxed(Box::new(seeder), nonce)
    }

    /// Creates a new set of seeds from an already boxed seeder
    pub fn from_boxed(seeder: Box<dyn PDASeeder + 'a>, nonce: u8) -> Self {
        PDASeedSet {
            seeder,
            nonce: [nonce],
        }
    }

    /// Gets an iterator of the seeds
    pub fn seeds(&self) -> impl Iterator<Item = &'_ dyn PDASeed> {
        self.seeder.seeds().chain(once(&self.nonce as &dyn PDASeed))
    }

    /// Invokes an instruction with these seeds
    pub fn invoke_signed<const N: usize>(
        &self,
        instruction: &SolanaInstruction,
        accounts: &[&AccountInfo; N],
    ) -> ProgramResult {
        let seeds = self.seeds().map(|seed| seed.as_ref()).collect::<Vec<_>>();

        invoke_signed(instruction, accounts, &[&seeds])
    }

    /// Invokes an instruction of variable account size with these seeds
    pub fn invoke_signed_variable_size(
        &self,
        instruction: &SolanaInstruction,
        accounts: &[&AccountInfo],
    ) -> ProgramResult {
        let seeds = self.seeds().map(|seed| seed.as_ref()).collect::<Vec<_>>();

        invoke_signed_variable_size(instruction, accounts, &[&seeds])
    }

    /// Invokes an instruction with given seed sets
    pub fn invoke_signed_multiple<const N: usize>(
        instruction: &SolanaInstruction,
        accounts: &[&AccountInfo; N],
        seed_sets: &[Self],
    ) -> ProgramResult {
        let seeds_array = seed_sets
            .iter()
            .map(|seed_set| {
                seed_set
                    .seeds()
                    .map(|seed| seed.as_ref())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let seeds = seeds_array
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<_>>();

        invoke_signed(instruction, accounts, seeds.as_slice())
    }

    /// Invokes an instruction of variable account size with given seed sets
    pub fn invoke_signed_variable_size_multiple(
        instruction: &SolanaInstruction,
        accounts: &[&AccountInfo],
        seed_sets: &[Self],
    ) -> ProgramResult {
        let seeds_array = seed_sets
            .iter()
            .map(|seed_set| {
                seed_set
                    .seeds()
                    .map(|seed| seed.as_ref())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        let seeds = seeds_array
            .iter()
            .map(|seed| seed.as_slice())
            .collect::<Vec<_>>();

        invoke_signed_variable_size(instruction, accounts, seeds.as_slice())
    }
}

/// A possible seed to a PDA.
pub trait PDASeed: AsRef<[u8]> {
    /// Turns the seed into a human readable string.
    fn to_seed_string(&self) -> String;
}
impl PDASeed for Pubkey {
    fn to_seed_string(&self) -> String {
        format!("{}", self)
    }
}
impl PDASeed for &str {
    fn to_seed_string(&self) -> String {
        String::from(*self)
    }
}
impl PDASeed for String {
    fn to_seed_string(&self) -> String {
        self.clone()
    }
}
impl PDASeed for &[u8] {
    fn to_seed_string(&self) -> String {
        format!("{:?}", self)
    }
}
impl<const N: usize> PDASeed for [u8; N] {
    fn to_seed_string(&self) -> String {
        format!("{:?}", self)
    }
}
impl PDASeed for Vec<u8> {
    fn to_seed_string(&self) -> String {
        format!("{:?}", self)
    }
}

/// A set of seeds for a given PDA type.
pub trait PDASeeder: Debug {
    /// Gets an iterator of seeds for this address.
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a>;
}
impl<'b, T: ?Sized> PDASeeder for &'b T
where
    T: PDASeeder,
{
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a> {
        T::seeds(self)
    }
}
impl<'b, T: ?Sized> PDASeeder for &'b mut T
where
    T: PDASeeder,
{
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a> {
        T::seeds(self)
    }
}
impl<T: ?Sized> PDASeeder for Box<T>
where
    T: PDASeeder,
{
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a> {
        T::seeds(self)
    }
}

/// Generates a PDA from a given seeder.
pub trait PDAGenerator<'a, 'b, 'c>
where
    'a: 'c,
    'b: 'c,
{
    /// Return type of [`PDAGenerator::seeds_to_bytes`]
    type SeedsToBytesIter: Iterator<Item = &'a [u8]> + 'a;
    /// Return type of [`PDAGenerator::seeds_to_bytes_with_nonce`]
    type SeedsToBytesWithNonceIter: Iterator<Item = &'c [u8]> + 'c;
    /// Return type of [`PDAGenerator::seeds_to_strings`]
    type SeedsToStringsIter: Iterator<Item = String> + 'a;
    /// Return type of [`PDAGenerator::seeds_to_strings_with_nonce`]
    type SeedsToStringsWithNonceIter: Iterator<Item = String> + 'a;

    /// Gets the seeds as an iterator of bytes
    fn seeds_to_bytes(&'a self) -> Self::SeedsToBytesIter;
    /// Gets the seeds as an iterator of bytes with an additional nonce
    fn seeds_to_bytes_with_nonce(&'a self, nonce: &'b [u8; 1]) -> Self::SeedsToBytesWithNonceIter;
    /// Gets the seeds as an iterator of strings
    fn seeds_to_strings(&'a self) -> Self::SeedsToStringsIter;
    /// Gets the seeds as an iterator of strings with an additional nonce
    fn seeds_to_strings_with_nonce(&'a self, nonce: u8) -> Self::SeedsToStringsWithNonceIter;
    /// Finds an address for the given seeds returning `(key, nonce)`
    fn find_address(&self, program_id: &'static Pubkey) -> (Pubkey, u8);
    /// Creates an address from given seeds and nonce, ~50% chance to error if given a random nonce
    fn create_address(&self, program_id: &'static Pubkey, nonce: u8) -> GeneratorResult<Pubkey>;
    /// Verifies that a given address is derived from given seeds and finds nonce. Returns the found nonce.
    fn verify_address_find_nonce(
        &self,
        program_id: &'static Pubkey,
        address: &Pubkey,
    ) -> GeneratorResult<u8>;
    /// Verifies that a given address is derived from given seeds and nonce.
    fn verify_address_with_nonce(
        &self,
        program_id: &'static Pubkey,
        address: &Pubkey,
        nonce: u8,
    ) -> GeneratorResult<()>;
    /// Verifies that a given address is derived from given seeds.
    fn verify_address(&self, program_id: &'static Pubkey, address: &Pubkey) -> GeneratorResult<()>;
}
#[allow(clippy::type_complexity)]
impl<'a, 'b, 'c, T: ?Sized> PDAGenerator<'a, 'b, 'c> for T
where
    T: PDASeeder,
    'a: 'c,
    'b: 'c,
{
    type SeedsToBytesIter =
        Map<Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a>, fn(&dyn PDASeed) -> &[u8]>;
    type SeedsToBytesWithNonceIter = Chain<
        Map<Box<dyn Iterator<Item = &'c dyn PDASeed> + 'c>, fn(&dyn PDASeed) -> &[u8]>,
        Once<&'c [u8]>,
    >;
    type SeedsToStringsIter =
        Map<Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a>, fn(&dyn PDASeed) -> String>;
    type SeedsToStringsWithNonceIter = Chain<
        Map<Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a>, fn(&dyn PDASeed) -> String>,
        Once<String>,
    >;

    fn seeds_to_bytes(&'a self) -> Self::SeedsToBytesIter {
        self.seeds().map(|seed| seed.as_ref())
    }

    fn seeds_to_bytes_with_nonce(&'a self, nonce: &'b [u8; 1]) -> Self::SeedsToBytesWithNonceIter {
        self.seeds_to_bytes().chain(once(nonce as &[u8]))
    }

    fn seeds_to_strings(&'a self) -> Self::SeedsToStringsIter {
        self.seeds().map(|seed| seed.to_seed_string())
    }

    fn seeds_to_strings_with_nonce(&'a self, nonce: u8) -> Self::SeedsToStringsWithNonceIter {
        self.seeds_to_strings().chain(once(nonce.to_string()))
    }

    fn find_address(&self, program_id: &'static Pubkey) -> (Pubkey, u8) {
        let seed_bytes = self.seeds_to_bytes().collect::<Vec<_>>();
        Pubkey::find_program_address(&seed_bytes, program_id)
    }

    fn create_address(&self, program_id: &'static Pubkey, nonce: u8) -> GeneratorResult<Pubkey> {
        Pubkey::create_program_address(
            &self.seeds_to_bytes_with_nonce(&[nonce]).collect::<Vec<_>>(),
            program_id,
        )
        .map_err(|error| match error {
            PubkeyError::InvalidSeeds => GeneratorError::NoAccountFromSeeds {
                seeds: self.seeds_to_strings_with_nonce(nonce).collect(),
            }
            .into(),
            error => error.into(),
        })
    }

    fn verify_address_find_nonce(
        &self,
        program_id: &'static Pubkey,
        address: &Pubkey,
    ) -> GeneratorResult<u8> {
        let (key, nonce) = self.find_address(program_id);
        if address != &key {
            return Err(GeneratorError::AccountNotFromSeeds {
                account: *address,
                seeds: self.seeds_to_strings().collect(),
                program_id,
            }
            .into());
        }
        Ok(nonce)
    }

    fn verify_address_with_nonce(
        &self,
        program_id: &'static Pubkey,
        address: &Pubkey,
        nonce: u8,
    ) -> GeneratorResult<()> {
        let created_key = self.create_address(program_id, nonce);
        if created_key.is_err() || address != &created_key? {
            Err(GeneratorError::AccountNotFromSeeds {
                account: *address,
                seeds: self.seeds_to_strings_with_nonce(nonce).collect(),
                program_id,
            }
            .into())
        } else {
            Ok(())
        }
    }

    fn verify_address(&self, program_id: &'static Pubkey, address: &Pubkey) -> GeneratorResult<()> {
        let created_key = self.find_address(program_id).0;
        if address != &created_key {
            return Err(GeneratorError::AccountNotFromSeeds {
                account: *address,
                seeds: self.seeds_to_strings().collect(),
                program_id,
            }
            .into());
        }
        Ok(())
    }
}
