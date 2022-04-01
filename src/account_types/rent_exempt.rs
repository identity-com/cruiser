//! A single account that must be rent exempt

use std::fmt::Debug;
use std::ops::{Deref, DerefMut};

use solana_program::pubkey::Pubkey;
use solana_program::rent::Rent;
use solana_program::sysvar::Sysvar;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::{AccountInfo, AccountInfoAccess, CruiserResult, GenericError};

// verify_account_arg_impl! {
//     mod rent_exempt_check<AI>{
//         <T> RentExempt<T> where T: AccountArgument<AI>{
//             from: [
//                 <Arg> Arg where T: FromAccounts<Arg>;
//             ];
//             validate: [
//                 /// Uses [`Rent::get`] to determine the required rent.
//                 () where T::AccountInfo: AccountInfo, T: ValidateArgument<()> + SingleIndexable<()>;
//                 /// Uses the passed rent to determine the required rent.
//                 Rent where T::AccountInfo: AccountInfo, T: ValidateArgument<()> + SingleIndexable<()>;
//                 /// Uses [`Rent::get`] to determine the required rent.
//                 <Arg> (Arg,) where T::AccountInfo: AccountInfo, T: ValidateArgument<Arg> + SingleIndexable<()>;
//                 /// Uses [`Rent::get`] to determine the required rent.
//                 <Arg, I> (Arg, I) where T::AccountInfo: AccountInfo, T: ValidateArgument<Arg> + SingleIndexable<I>;
//                 /// Uses the passed rent to determine the required rent.
//                 <Arg, I> (Arg, I, Rent) where T::AccountInfo: AccountInfo, T: ValidateArgument<Arg> + SingleIndexable<I>;
//             ];
//             multi: [<I> I where T: MultiIndexable<I>];
//             single: [<I> I where T: SingleIndexable<I>];
//         }
//     }
// }

/// A single account wrapper that ensures the account is rent exempt. Used commonly with [`ZeroedAccount`](crate::account_types::zeroed_account::ZeroedAccount).
///
/// - `A` the Account argument to wrap. Must implement [`SingleIndexable<()>`].
#[derive(Debug)]
pub struct RentExempt<T>(pub T);
impl<T> Deref for RentExempt<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<A> DerefMut for RentExempt<A> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
impl<T> AccountArgument for RentExempt<T>
where
    T: AccountArgument,
{
    type AccountInfo = T::AccountInfo;

    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        self.0.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.0.add_keys(add)
    }
}
impl<T, Arg> FromAccounts<Arg> for RentExempt<T>
where
    T: FromAccounts<Arg>,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = T::AccountInfo>,
        arg: Arg,
    ) -> CruiserResult<Self> {
        Ok(Self(T::from_accounts(program_id, infos, arg)?))
    }

    fn accounts_usage_hint(arg: &Arg) -> (usize, Option<usize>) {
        T::accounts_usage_hint(arg)
    }
}
impl<T> ValidateArgument<()> for RentExempt<T>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument<()> + SingleIndexable<()>,
{
    fn validate(&mut self, program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        self.validate(program_id, Rent::get()?)
    }
}
impl<T> ValidateArgument<Rent> for RentExempt<T>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument<()> + SingleIndexable<()>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: Rent) -> CruiserResult<()> {
        self.validate(program_id, ((), (), arg))
    }
}
impl<T, Arg> ValidateArgument<(Arg,)> for RentExempt<T>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument<Arg> + SingleIndexable<()>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (Arg,)) -> CruiserResult<()> {
        self.validate(program_id, (arg.0, (), Rent::get()?))
    }
}
impl<T, Arg, I> ValidateArgument<(Arg, I)> for RentExempt<T>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument<Arg> + SingleIndexable<I>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (Arg, I)) -> CruiserResult<()> {
        self.validate(program_id, (arg.0, arg.1, Rent::get()?))
    }
}
impl<T, Arg, I> ValidateArgument<(Arg, I, Rent)> for RentExempt<T>
where
    T::AccountInfo: AccountInfo,
    T: ValidateArgument<Arg> + SingleIndexable<I>,
{
    fn validate(&mut self, program_id: &Pubkey, arg: (Arg, I, Rent)) -> CruiserResult<()> {
        self.0.validate(program_id, arg.0)?;
        let info = self.0.index_info(arg.1)?;
        let lamports = *info.lamports();
        let needed_lamports = arg.2.minimum_balance(info.data().len());
        if lamports < needed_lamports {
            Err(GenericError::NotEnoughLamports {
                account: *info.key(),
                lamports,
                needed_lamports,
            }
            .into())
        } else {
            Ok(())
        }
    }
}
impl<T, Arg> MultiIndexable<Arg> for RentExempt<T>
where
    T: MultiIndexable<Arg>,
{
    #[inline]
    fn index_is_signer(&self, indexer: Arg) -> CruiserResult<bool> {
        self.0.index_is_signer(indexer)
    }

    #[inline]
    fn index_is_writable(&self, indexer: Arg) -> CruiserResult<bool> {
        self.0.index_is_writable(indexer)
    }

    #[inline]
    fn index_is_owner(&self, owner: &Pubkey, indexer: Arg) -> CruiserResult<bool> {
        self.0.index_is_owner(owner, indexer)
    }
}
impl<T, Arg> SingleIndexable<Arg> for RentExempt<T>
where
    T: SingleIndexable<Arg>,
{
    #[inline]
    fn index_info(&self, indexer: Arg) -> CruiserResult<&Self::AccountInfo> {
        self.0.index_info(indexer)
    }
}
