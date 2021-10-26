use crate::solana_program::pubkey::PubkeyError;
use crate::{GeneratorError, GeneratorResult, Pubkey};
use std::fmt::Debug;
use std::iter::once;
use std::ops::Deref;

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
impl PDASeed for &[u8] {
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
impl PDASeeder for &dyn PDASeeder {
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a> {
        self.deref().seeds()
    }
}

/// Generates a PDA from a given seeder.
#[derive(Debug)]
pub struct PDAGenerator<T> {
    program_id: Pubkey,
    seeds: T,
}
impl<T> PDAGenerator<T>
where
    T: PDASeeder,
{
    /// Creates a new generator
    pub fn new(program_id: Pubkey, seeds: T) -> Self {
        Self { program_id, seeds }
    }

    /// Gets the seeds as an iterator of bytes
    pub fn seeds_to_bytes(&self) -> impl Iterator<Item = &'_ [u8]> + '_ {
        self.seeds.seeds().map(|seed| seed.as_ref())
    }

    /// Gets the seeds as an iterator of bytes with an additional nonce
    pub fn seeds_to_bytes_with_nonce<'a, 'b, 'c>(
        &'a self,
        nonce: &'b [u8; 1],
    ) -> impl Iterator<Item = &'c [u8]> + 'c
    where
        'a: 'c,
        'b: 'c,
    {
        self.seeds_to_bytes().chain(once(nonce as &[u8]))
    }

    /// Gets the seeds as an iterator of strings
    pub fn seeds_to_strings(&self) -> impl Iterator<Item = String> + '_ {
        self.seeds
            .seeds()
            .map(|seed: &dyn PDASeed| seed.to_seed_string())
    }

    /// Gets the seeds as an iterator of strings with an additional nonce
    pub fn seeds_to_strings_with_nonce(&self, nonce: u8) -> impl Iterator<Item = String> + '_ {
        self.seeds_to_strings().chain(once(nonce.to_string()))
    }

    /// Finds an address for the given seeds returning `(key, nonce)`
    pub fn find_address(&self) -> (Pubkey, u8) {
        let seed_bytes = self.seeds_to_bytes().collect::<Vec<_>>();
        Pubkey::find_program_address(&seed_bytes, &self.program_id)
    }

    /// Creates an address from the given seeds
    pub fn create_address(&self) -> GeneratorResult<Pubkey> {
        Pubkey::create_program_address(&self.seeds_to_bytes().collect::<Vec<_>>(), &self.program_id)
            .map_err(|error| match error {
                PubkeyError::InvalidSeeds => GeneratorError::NoAccountFromSeeds {
                    seeds: self.seeds_to_strings().collect(),
                }
                .into(),
                error => error.into(),
            })
    }

    /// Creates an address from given seeds and nonce, ~50% chance to error if given a random nonce
    pub fn create_address_with_nonce(&self, nonce: u8) -> GeneratorResult<Pubkey> {
        Pubkey::create_program_address(
            &self.seeds_to_bytes_with_nonce(&[nonce]).collect::<Vec<_>>(),
            &self.program_id,
        )
        .map_err(|error| match error {
            PubkeyError::InvalidSeeds => GeneratorError::NoAccountFromSeeds {
                seeds: self.seeds_to_strings_with_nonce(nonce).collect(),
            }
            .into(),
            error => error.into(),
        })
    }

    /// Verifies that a given address is derived from given seeds and finds nonce. Returns the found nonce.
    pub fn verify_address_find_nonce(&self, address: Pubkey) -> GeneratorResult<u8> {
        let (key, nonce) = self.find_address();
        if address != key {
            return Err(GeneratorError::AccountNotFromSeeds {
                account: address,
                seeds: self.seeds_to_strings().collect(),
                program_id: self.program_id,
            }
            .into());
        }
        Ok(nonce)
    }

    /// Verifies that a given address is derived from given seeds and nonce.
    pub fn verify_address_with_nonce(&self, address: Pubkey, nonce: u8) -> GeneratorResult<()> {
        let created_key = self.create_address_with_nonce(nonce);
        if created_key.is_err() || address != created_key? {
            return Err(GeneratorError::AccountNotFromSeeds {
                account: address,
                seeds: self.seeds_to_strings_with_nonce(nonce).collect(),
                program_id: self.program_id,
            }
            .into());
        }
        Ok(())
    }

    /// Verifies that a given address is derived from given seeds.
    pub fn verify_address(&self, address: Pubkey) -> GeneratorResult<()> {
        let created_key = self.create_address();
        if created_key.is_err() || address != created_key? {
            return Err(GeneratorError::AccountNotFromSeeds {
                account: address,
                seeds: self.seeds_to_strings().collect(),
                program_id: self.program_id,
            }
            .into());
        }
        Ok(())
    }
}
