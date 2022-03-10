use solana_program::pubkey::Pubkey;

use cruiser_derive::verify_account_arg_impl;

use crate::{
    AccountArgument, AccountInfo, AccountInfoIterator, FromAccounts, GeneratorResult,
    MultiIndexable, SingleIndexable, ValidateArgument,
};

verify_account_arg_impl! {
    mod box_checks{
        <A> Box<A> where A: AccountArgument{
            from: [<T> T where A: FromAccounts<T>];
            validate: [<T> T where A: ValidateArgument<T>];
            multi: [<T> T where A: MultiIndexable<T>];
            single: [<T> T where A: SingleIndexable<T>];
        }
    }
}

impl<A> AccountArgument for Box<A>
where
    A: AccountArgument,
{
    #[inline]
    fn write_back(self, program_id: &'static Pubkey) -> GeneratorResult<()> {
        A::write_back(*self, program_id)
    }

    #[inline]
    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        A::add_keys(self, add)
    }
}
impl<A, T> FromAccounts<T> for Box<A>
where
    A: FromAccounts<T>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: T,
    ) -> GeneratorResult<Self> {
        A::from_accounts(program_id, infos, arg).map(Box::new)
    }

    fn accounts_usage_hint(arg: &T) -> (usize, Option<usize>) {
        A::accounts_usage_hint(arg)
    }
}
impl<A, T> ValidateArgument<T> for Box<A>
where
    A: ValidateArgument<T>,
{
    fn validate(&mut self, program_id: &'static Pubkey, arg: T) -> GeneratorResult<()> {
        A::validate(self, program_id, arg)
    }
}
impl<A, T> MultiIndexable<T> for Box<A>
where
    A: MultiIndexable<T>,
{
    fn is_signer(&self, indexer: T) -> GeneratorResult<bool> {
        A::is_signer(self, indexer)
    }

    fn is_writable(&self, indexer: T) -> GeneratorResult<bool> {
        A::is_writable(self, indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: T) -> GeneratorResult<bool> {
        A::is_owner(self, owner, indexer)
    }
}
impl<A, T> SingleIndexable<T> for Box<A>
where
    A: SingleIndexable<T>,
{
    fn info(&self, indexer: T) -> GeneratorResult<&AccountInfo> {
        A::info(self, indexer)
    }
}
