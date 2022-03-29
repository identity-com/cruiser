use solana_program::pubkey::Pubkey;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::CruiserResult;

// verify_account_arg_impl! {
//     mod box_checks<AI>{
//         <T> Box<T> where T: AccountArgument<AI>{
//             from: [<Arg> Arg where T: FromAccounts<Arg>];
//             validate: [<Arg> Arg where T: ValidateArgument<Arg>];
//             multi: [<Arg> Arg where T: MultiIndexable<Arg>];
//             single: [<Arg> Arg where T: SingleIndexable<Arg>];
//         }
//     }
// }

impl<T> AccountArgument for Box<T>
where
    T: AccountArgument,
{
    type AccountInfo = T::AccountInfo;

    #[inline]
    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        T::write_back(*self, program_id)
    }

    #[inline]
    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        T::add_keys(self, add)
    }
}
impl<Arg, T> FromAccounts<Arg> for Box<T>
where
    T: FromAccounts<Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
        arg: Arg,
    ) -> CruiserResult<Self> {
        T::from_accounts(program_id, infos, arg).map(Box::new)
    }

    fn accounts_usage_hint(arg: &Arg) -> (usize, Option<usize>) {
        T::accounts_usage_hint(arg)
    }
}
impl<Arg, T> ValidateArgument<Arg> for Box<T>
where
    T: ValidateArgument<Arg>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: Arg) -> CruiserResult<()> {
        T::validate(self, program_id, arg)
    }
}
impl<T, Arg> MultiIndexable<Arg> for Box<T>
where
    T: MultiIndexable<Arg>,
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
impl<T, Arg> SingleIndexable<Arg> for Box<T>
where
    T: SingleIndexable<Arg>,
{
    fn index_info(&self, indexer: Arg) -> CruiserResult<&Self::AccountInfo> {
        T::index_info(self, indexer)
    }
}
