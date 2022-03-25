use solana_program::pubkey::Pubkey;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::CruiserResult;
use cruiser_derive::verify_account_arg_impl;

verify_account_arg_impl! {
    mod box_checks<AI>{
        <AI, T> Box<T> where T: AccountArgument<AI>{
            from: [<Arg> Arg where T: FromAccounts<AI, Arg>];
            validate: [<Arg> Arg where T: ValidateArgument<AI, Arg>];
            multi: [<Arg> Arg where T: MultiIndexable<AI, Arg>];
            single: [<Arg> Arg where T: SingleIndexable<AI, Arg>];
        }
    }
}

impl<AI, T> AccountArgument<AI> for Box<T>
where
    T: AccountArgument<AI>,
{
    #[inline]
    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        T::write_back(*self, program_id)
    }

    #[inline]
    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        T::add_keys(self, add)
    }
}
impl<AI, Arg, T> FromAccounts<AI, Arg> for Box<T>
where
    T: FromAccounts<AI, Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<AI>,
        arg: Arg,
    ) -> CruiserResult<Self> {
        T::from_accounts(program_id, infos, arg).map(Box::new)
    }

    fn accounts_usage_hint(arg: &Arg) -> (usize, Option<usize>) {
        T::accounts_usage_hint(arg)
    }
}
impl<AI, Arg, T> ValidateArgument<AI, Arg> for Box<T>
where
    T: ValidateArgument<AI, Arg>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: Arg) -> CruiserResult<()> {
        T::validate(self, program_id, arg)
    }
}
impl<AI, T, Arg> MultiIndexable<AI, Arg> for Box<T>
where
    T: MultiIndexable<AI, Arg>,
{
    fn index_is_signer(&self, indexer: Arg) -> CruiserResult<bool> {
        T::index_is_signer(self, indexer)
    }

    fn index_is_writable(&self, indexer: Arg) -> CruiserResult<bool> {
        T::index_is_writable(self, indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: Arg) -> CruiserResult<bool> {
        T::index_is_owner(self, owner, indexer)
    }
}
impl<AI, T, Arg> SingleIndexable<AI, Arg> for Box<T>
where
    T: SingleIndexable<AI, Arg>,
{
    fn index_info(&self, indexer: Arg) -> CruiserResult<&AI> {
        T::index_info(self, indexer)
    }
}
