//! Implementations of `crusier` traits for the [`Option`] type.

use crate::account_argument::{AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable};
use crate::{CruiserResult, GenericError};
use cruiser::account_argument::ValidateArgument;
use solana_program::pubkey::Pubkey;
use std::iter::once;

impl<T> AccountArgument for Option<T>
where
    T: AccountArgument,
{
    type AccountInfo = T::AccountInfo;

    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        match self {
            Some(inner) => inner.write_back(program_id),
            None => Ok(()),
        }
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        match self {
            Some(inner) => inner.add_keys(add),
            None => Ok(()),
        }
    }
}

impl<T> FromAccounts for Option<T>
where
    T: FromAccounts<()>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: (),
    ) -> CruiserResult<Self> {
        Self::from_accounts(program_id, infos, (arg,))
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        Self::accounts_usage_hint(&(*arg,))
    }
}

impl<T> FromAccounts<bool> for Option<T>
where
    T: FromAccounts<()>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: bool,
    ) -> CruiserResult<Self> {
        Self::from_accounts(program_id, infos, if arg { Some(()) } else { None })
    }

    fn accounts_usage_hint(arg: &bool) -> (usize, Option<usize>) {
        Self::accounts_usage_hint(if *arg { &Some(()) } else { &None })
    }
}

impl<T, Arg> FromAccounts<(Arg,)> for Option<T>
where
    T: FromAccounts<Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: (Arg,),
    ) -> CruiserResult<Self> {
        match infos.next() {
            Some(info) => Ok(Some(T::from_accounts(
                program_id,
                &mut once(info).chain(infos),
                arg.0,
            )?)),
            None => Ok(None),
        }
    }

    fn accounts_usage_hint(arg: &(Arg,)) -> (usize, Option<usize>) {
        (0, T::accounts_usage_hint(&arg.0).1)
    }
}

impl<T, Arg> FromAccounts<Option<Arg>> for Option<T>
where
    T: FromAccounts<Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: Option<Arg>,
    ) -> CruiserResult<Self> {
        match arg {
            Some(inner_arg) => Ok(Some(T::from_accounts(program_id, infos, inner_arg)?)),
            None => Ok(None),
        }
    }

    fn accounts_usage_hint(arg: &Option<Arg>) -> (usize, Option<usize>) {
        match arg {
            None => (0, Some(0)),
            Some(arg) => T::accounts_usage_hint(arg),
        }
    }
}

impl<T> ValidateArgument for Option<T>
where
    T: ValidateArgument,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult<()> {
        match self {
            Some(inner) => inner.validate(program_id, arg),
            None => Ok(()),
        }
    }
}

/// Requires the optionality match the argument.
#[derive(Copy, Clone, Debug)]
pub struct OptionMatch<Arg>(pub Option<Arg>);

impl<T, Arg> ValidateArgument<OptionMatch<Arg>> for Option<T>
where
    T: ValidateArgument<Arg>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: OptionMatch<Arg>) -> CruiserResult<()> {
        match (self, arg.0) {
            (Some(inner), Some(arg)) => inner.validate(program_id, arg),
            (None, None) => Ok(()),
            _ => Err(GenericError::Custom {
                error: "Option arguments must match".to_string(),
            }
            .into()),
        }
    }
}

/// Operates the arg on the option if it is [`Some`]. Otherwise returns success.
#[derive(Copy, Clone, Debug)]
pub struct IfSomeArg<Arg>(pub Arg);

impl<T, Arg> ValidateArgument<IfSomeArg<Arg>> for Option<T>
where
    T: ValidateArgument<Arg>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: IfSomeArg<Arg>) -> CruiserResult<()> {
        match self {
            Some(inner) => inner.validate(program_id, arg.0),
            None => Ok(()),
        }
    }
}

impl<T, Arg> MultiIndexable<OptionMatch<Arg>> for Option<T>
where
    T: MultiIndexable<Arg>,
{
    fn index_is_signer(&self, indexer: OptionMatch<Arg>) -> CruiserResult<bool> {
        match (self, indexer.0) {
            (Some(inner), Some(indexer)) => inner.index_is_signer(indexer),
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }

    fn index_is_writable(&self, indexer: OptionMatch<Arg>) -> CruiserResult<bool> {
        match (self, indexer.0) {
            (Some(inner), Some(indexer)) => inner.index_is_writable(indexer),
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: OptionMatch<Arg>) -> CruiserResult<bool> {
        match (self, indexer.0) {
            (Some(inner), Some(indexer)) => inner.index_is_owner(owner, indexer),
            (None, None) => Ok(true),
            _ => Ok(false),
        }
    }
}

impl<T, Arg> MultiIndexable<IfSomeArg<Arg>> for Option<T>
where
    T: MultiIndexable<Arg>,
{
    fn index_is_signer(&self, indexer: IfSomeArg<Arg>) -> CruiserResult<bool> {
        match self {
            Some(inner) => inner.index_is_signer(indexer.0),
            None => Ok(true),
        }
    }

    fn index_is_writable(&self, indexer: IfSomeArg<Arg>) -> CruiserResult<bool> {
        match self {
            Some(inner) => inner.index_is_writable(indexer.0),
            None => Ok(true),
        }
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: IfSomeArg<Arg>) -> CruiserResult<bool> {
        match self {
            Some(inner) => inner.index_is_owner(owner, indexer.0),
            None => Ok(true),
        }
    }
}

/// Runs the check if the option is [`Some`]. Otherwise returns success.
#[derive(Copy, Clone, Debug)]
pub struct IfSome;
impl<T> MultiIndexable<IfSome> for Option<T>
where
    T: MultiIndexable<()>,
{
    fn index_is_signer(&self, _indexer: IfSome) -> CruiserResult<bool> {
        Self::index_is_signer(self, IfSomeArg(()))
    }

    fn index_is_writable(&self, _indexer: IfSome) -> CruiserResult<bool> {
        Self::index_is_writable(self, IfSomeArg(()))
    }

    fn index_is_owner(&self, owner: &Pubkey, _indexer: IfSome) -> CruiserResult<bool> {
        Self::index_is_owner(self, owner, IfSomeArg(()))
    }
}
