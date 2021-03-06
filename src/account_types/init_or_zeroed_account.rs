//! Accepts both forms of initialize-able account

use std::iter::once;
use std::ops::{Deref, DerefMut};

use crate::cpi::CPI;
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::pubkey::Pubkey;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::account_list::AccountListItem;
use crate::account_types::discriminant_account::DiscriminantAccount;
use crate::account_types::init_account::{InitAccount, InitArgs};
use crate::account_types::zeroed_account::{CheckAll, ZeroedAccount};
use crate::AccountInfo;
use crate::{CruiserResult, ToSolanaAccountInfo};

// verify_account_arg_impl! {
//     mod init_account_check<AI>{
//         <AI, AL, D> InitOrZeroedAccount<AI, AL, D>
//         where
//             AI: AccountInfo,
//             AL: AccountListItem<D>,
//             D: BorshSerialize + BorshDeserialize,
//         {
//             from: [
//                 /// The initial value of this account
//                 D;
//             ];
//             validate: [
//                 <'a, 'b, C> InitArgs<'a, AI, C> where AI: 'a + ToSolanaAccountInfo<'b>, C: CPI;
//                 <'a, 'b, C> (InitArgs<'a, AI, C>, CheckAll) where AI: 'a + ToSolanaAccountInfo<'b>, C: CPI;
//             ];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// A combination of [`InitAccount`] and [`ZeroedAccount`] accepting either based on owner.
// TODO: impl Debug for this
#[allow(missing_debug_implementations)]
// TODO: use AccountArgument trait for impl when enums supported
pub enum InitOrZeroedAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    /// Is an [`InitAccount`]
    Init(InitAccount<AI, AL, D>),
    /// Is a [`ZeroedAccount`]
    Zeroed(ZeroedAccount<AI, AL, D>),
}
impl<AI, AL, D> Deref for InitOrZeroedAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    type Target = DiscriminantAccount<AI, AL, D>;

    fn deref(&self) -> &Self::Target {
        match self {
            InitOrZeroedAccount::Init(init) => init,
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed,
        }
    }
}
impl<AI, AL, D> DerefMut for InitOrZeroedAccount<AI, AL, D>
where
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            InitOrZeroedAccount::Init(init) => init,
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed,
        }
    }
}
impl<AI, AL, D> AccountArgument for InitOrZeroedAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    type AccountInfo = AI;

    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        match self {
            InitOrZeroedAccount::Init(init) => init.write_back(program_id),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.write_back(program_id),
        }
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        match self {
            InitOrZeroedAccount::Init(init) => init.add_keys(add),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.add_keys(add),
        }
    }
}
impl<'a, AI, AL, D> FromAccounts<D> for InitOrZeroedAccount<AI, AL, D>
where
    AI: AccountInfo,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = AI>,
        arg: D,
    ) -> CruiserResult<Self> {
        let info = AI::from_accounts(program_id, infos, ())?;
        if &*info.owner() == program_id {
            Ok(Self::Zeroed(ZeroedAccount::from_accounts(
                program_id,
                &mut once(info),
                arg,
            )?))
        } else {
            Ok(Self::Init(InitAccount::from_accounts(
                program_id,
                &mut once(info),
                arg,
            )?))
        }
    }

    fn accounts_usage_hint(_arg: &D) -> (usize, Option<usize>) {
        AI::accounts_usage_hint(&())
    }
}
impl<'a, 'b, AI, AL, D, C> ValidateArgument<InitArgs<'a, AI, C>> for InitOrZeroedAccount<AI, AL, D>
where
    AI: ToSolanaAccountInfo<'b>,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    C: CPI,
{
    fn validate(&mut self, program_id: &Pubkey, arg: InitArgs<'a, AI, C>) -> CruiserResult<()> {
        match self {
            InitOrZeroedAccount::Init(init) => init.validate(program_id, arg),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.validate(program_id, ()),
        }
    }
}
impl<'a, 'b, AI, AL, D, C> ValidateArgument<(InitArgs<'a, AI, C>, CheckAll)>
    for InitOrZeroedAccount<AI, AL, D>
where
    AI: ToSolanaAccountInfo<'b>,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
    C: CPI,
{
    fn validate(
        &mut self,
        program_id: &Pubkey,
        arg: (InitArgs<'a, AI, C>, CheckAll),
    ) -> CruiserResult<()> {
        match self {
            InitOrZeroedAccount::Init(init) => init.validate(program_id, arg.0),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.validate(program_id, arg.1),
        }
    }
}
impl<AI, AL, D, T> MultiIndexable<T> for InitOrZeroedAccount<AI, AL, D>
where
    AI: AccountInfo + MultiIndexable<T>,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn index_is_signer(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: T) -> CruiserResult<bool> {
        self.info.index_is_owner(owner, indexer)
    }
}
impl<AI, AL, D, T> SingleIndexable<T> for InitOrZeroedAccount<AI, AL, D>
where
    AI: AccountInfo + SingleIndexable<T>,
    AL: AccountListItem<D>,
    D: BorshSerialize + BorshDeserialize,
{
    fn index_info(&self, indexer: T) -> CruiserResult<&AI> {
        self.info.index_info(indexer)
    }
}
