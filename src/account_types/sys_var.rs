use crate::{
    AccountArgument, AccountInfoIterator, FromAccounts, GeneratorError, GeneratorResult,
    SystemProgram,
};
use cruiser::AccountInfo;
use solana_program::pubkey::Pubkey;
use solana_program::sysvar::Sysvar;
use std::marker::PhantomData;
use std::ops::Deref;

/// A sysvar, checks the address is the same.
#[derive(Debug)]
pub struct SysVar<T>(pub AccountInfo, PhantomData<fn() -> T>)
where
    T: Sysvar;
impl<T> SysVar<T>
where
    T: Sysvar,
{
    /// Gets the sysvar, may be unsupported for large sys vars
    pub fn get(&self) -> GeneratorResult<T> {
        unsafe { Ok(T::from_account_info(&self.0.to_solana_account_info())?) }
    }
}
impl<T> Deref for SysVar<T>
where
    T: Sysvar,
{
    type Target = AccountInfo;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<T> AccountArgument for SysVar<T>
where
    T: Sysvar,
{
    fn write_back(
        self,
        program_id: &'static Pubkey,
        system_program: Option<&SystemProgram>,
    ) -> GeneratorResult<()> {
        self.0.write_back(program_id, system_program)
    }

    fn add_keys(
        &self,
        add: impl FnMut(&'static Pubkey) -> GeneratorResult<()>,
    ) -> GeneratorResult<()> {
        self.0.add_keys(add)
    }
}
impl<T> FromAccounts<()> for SysVar<T>
where
    T: Sysvar,
{
    fn from_accounts(
        program_id: &'static Pubkey,
        infos: &mut impl AccountInfoIterator,
        _arg: (),
    ) -> GeneratorResult<Self> {
        let account = AccountInfo::from_accounts(program_id, infos, ())?;
        if T::check_id(account.key) {
            Ok(Self(account, PhantomData))
        } else {
            Err(GeneratorError::InvalidSysVar {
                actual: account.key,
            }
            .into())
        }
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        AccountInfo::accounts_usage_hint(arg)
    }
}
