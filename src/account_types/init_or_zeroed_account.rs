use super::SYSTEM_PROGRAM_ID;
use crate::solana_program::program_error::ProgramError;
use crate::{
    combine_hints_branch, AccountArgument, AccountInfo, AccountInfoIterator, AccountListItem,
    AllAny, FromAccounts, GeneratorError, GeneratorResult, InitAccount, InitSize,
    MultiIndexableAccountArgument, PDASeedSet, Pubkey, SingleIndexableAccountArgument,
    SystemProgram, ZeroedAccount,
};
use borsh::{BorshDeserialize, BorshSerialize};
use std::cell::RefCell;
use std::iter::once;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

/// A combination of [`InitAccount`] and [`ZeroedAccount`] accepting either based on owner.
/// Should call [`InitOrZeroedAccount::set_funder`] unless guaranteed not [`InitAccount`]
#[derive(Debug)]
pub enum InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    /// Is an [`InitAccount`]
    Init(InitAccount<AL, A>),
    /// Is an [`ZeroedAccount`]
    Zeroed(ZeroedAccount<AL, A>),
}
impl<AL, A> InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    /// Sets the init size, no-op if zeroed
    pub fn set_init_size(&mut self, init_size: InitSize) {
        if let Self::Init(init) = self {
            init.init_size = init_size;
        }
    }

    /// Sets the funder, no-op if zeroed
    pub fn set_funder(&mut self, funder: AccountInfo) {
        if let Self::Init(init) = self {
            init.funder = Some(funder);
        }
    }

    /// Sets the account seeds if init, no-op if zeroed
    pub fn set_account_seeds(&mut self, account_seeds: PDASeedSet<'static>) {
        if let Self::Init(init) = self {
            init.account_seeds = Some(account_seeds);
        }
    }

    /// Sets the funder seeds if init, no-op if zeroed
    pub fn set_funder_seeds(&mut self, funder_seeds: PDASeedSet<'static>) {
        if let Self::Init(init) = self {
            init.funder_seeds = Some(funder_seeds);
        }
    }

    /// Gets the account info
    pub fn info(&self) -> &AccountInfo {
        match self {
            InitOrZeroedAccount::Init(init) => &init.info,
            InitOrZeroedAccount::Zeroed(zeroed) => &zeroed.info,
        }
    }
}
impl<AL, A> Deref for InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    type Target = A;

    fn deref(&self) -> &Self::Target {
        match self {
            InitOrZeroedAccount::Init(init) => init.deref(),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.deref(),
        }
    }
}
impl<AL, A> DerefMut for InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            InitOrZeroedAccount::Init(init) => init.deref_mut(),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.deref_mut(),
        }
    }
}
impl<AL, A> AccountArgument for InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        match self {
            InitOrZeroedAccount::Init(init) => init.write_back(program_id, system_program),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.write_back(program_id, system_program),
        }
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        match self {
            InitOrZeroedAccount::Init(init) => init.add_keys(add),
            InitOrZeroedAccount::Zeroed(zeroed) => zeroed.add_keys(add),
        }
    }
}
impl<AL, A, Arg> FromAccounts<Arg> for InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize + BorshDeserialize + Default,
    InitAccount<AL, A>: FromAccounts<Arg>,
    ZeroedAccount<AL, A>: FromAccounts<Arg>,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        arg: Arg,
    ) -> GeneratorResult<Self> {
        let info = infos.next().ok_or(ProgramError::NotEnoughAccountKeys)?;
        let owner = info.owner.borrow();
        if *owner == program_id {
            drop(owner);
            Ok(Self::Zeroed(ZeroedAccount::from_accounts(
                program_id,
                &mut once(info),
                arg,
            )?))
        } else if *owner == &SYSTEM_PROGRAM_ID {
            drop(owner);
            Ok(Self::Init(InitAccount::from_accounts(
                program_id,
                &mut once(info),
                arg,
            )?))
        } else {
            Err(GeneratorError::AccountOwnerNotEqual {
                account: info.key,
                owner: **owner,
                expected_owner: vec![*program_id, SYSTEM_PROGRAM_ID],
            }
            .into())
        }
    }

    fn accounts_usage_hint() -> (usize, Option<usize>) {
        combine_hints_branch(IntoIterator::into_iter([
            <InitAccount<AL, A> as FromAccounts<Arg>>::accounts_usage_hint(),
            <ZeroedAccount<AL, A> as FromAccounts<Arg>>::accounts_usage_hint(),
        ]))
    }
}
impl<AL, A> MultiIndexableAccountArgument<()> for InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn is_signer(&self, indexer: ()) -> GeneratorResult<bool> {
        self.info().is_signer(indexer)
    }

    fn is_writable(&self, indexer: ()) -> GeneratorResult<bool> {
        self.info().is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: ()) -> GeneratorResult<bool> {
        self.info().is_owner(owner, indexer)
    }
}
impl<AL, A> MultiIndexableAccountArgument<AllAny> for InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn is_signer(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.info().is_signer(indexer)
    }

    fn is_writable(&self, indexer: AllAny) -> GeneratorResult<bool> {
        self.info().is_writable(indexer)
    }

    fn is_owner(&self, owner: &Pubkey, indexer: AllAny) -> GeneratorResult<bool> {
        self.info().is_owner(owner, indexer)
    }
}
impl<AL, A> SingleIndexableAccountArgument<()> for InitOrZeroedAccount<AL, A>
where
    AL: AccountListItem<A>,
    A: BorshSerialize,
{
    fn owner(&self, indexer: ()) -> GeneratorResult<&Rc<RefCell<&'static mut Pubkey>>> {
        self.info().owner(indexer)
    }

    fn key(&self, indexer: ()) -> GeneratorResult<&'static Pubkey> {
        self.info().key(indexer)
    }
}
