/// Implements [`AccountArgument`](crate::account_argument::AccountArgument) for a type with a certain accessor.
#[macro_export]
macro_rules! delegate_account_argument {
    ($ty:ty, ($accessor:tt)$(, $($where:tt)?)?) => {
        impl $crate::account_argument::AccountArgument for $ty $($($where)?)? {
            fn write_back(
                self,
                program_id: &'static $crate::Pubkey,
            ) -> $crate::CruiserResult<()> {
                self.$accessor.write_back(program_id)
            }

            fn add_keys(
                &self,
                add: impl FnMut(&'static $crate::Pubkey) -> $crate::CruiserResult<()>,
            ) -> $crate::CruiserResult<()> {
                self.$accessor.add_keys(add)
            }
        }
    };
}

/// Implementations for a type that impls [`AccountInfo`](crate::AccountInfo).
#[macro_export]
macro_rules! impl_account_info {
    ($account_info:ty $(, <$gen:tt>)?) => {
        impl$(<$gen>)? AccountArgument for $account_info {
            type AccountInfo = $account_info;
            fn write_back(self, _program_id: &Pubkey) -> CruiserResult<()> {
                Ok(())
            }

            fn add_keys(
                &self,
                mut add: impl FnMut(Pubkey) -> CruiserResult<()>,
            ) -> CruiserResult<()> {
                add(*$crate::AccountInfo::key(self))
            }
        }
        impl$(<$gen>)? FromAccounts<()> for $account_info {
            fn from_accounts(
                _program_id: &Pubkey,
                infos: &mut impl AccountInfoIterator<Item = Self::AccountInfo>,
                _arg: (),
            ) -> CruiserResult<Self> {
                match infos.next() {
                    None => Err(ProgramError::NotEnoughAccountKeys.into()),
                    Some(info) => Ok(info),
                }
            }

            fn accounts_usage_hint(_arg: &()) -> (usize, Option<usize>) {
                (1, Some(1))
            }
        }
        impl$(<$gen>)? ValidateArgument<()> for $account_info {
            fn validate(&mut self, _program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
                Ok(())
            }
        }
        impl$(<$gen>)? MultiIndexable<()> for $account_info {
            fn index_is_signer(&self, _indexer: ()) -> CruiserResult<bool> {
                Ok($crate::AccountInfo::is_signer(self))
            }

            fn index_is_writable(&self, _indexer: ()) -> CruiserResult<bool> {
                Ok($crate::AccountInfo::is_writable(self))
            }

            fn index_is_owner(&self, owner: &Pubkey, _indexer: ()) -> CruiserResult<bool> {
                Ok(&*$crate::AccountInfo::owner(self) == owner)
            }
        }
        impl$(<$gen>)? MultiIndexable<AllAny> for $account_info {
            fn index_is_signer(&self, indexer: AllAny) -> CruiserResult<bool> {
                Ok(indexer.is_not() ^ MultiIndexable::index_is_signer(self, ())?)
            }

            fn index_is_writable(&self, indexer: AllAny) -> CruiserResult<bool> {
                Ok(indexer.is_not() ^ MultiIndexable::index_is_writable(self, ())?)
            }

            fn index_is_owner(&self, owner: &Pubkey, indexer: AllAny) -> CruiserResult<bool> {
                Ok(indexer.is_not() ^ self.index_is_owner(owner, ())?)
            }
        }
        impl$(<$gen>)? SingleIndexable<()> for $account_info {
            fn index_info(&self, _indexer: ()) -> CruiserResult<&$account_info> {
                Ok(self)
            }
        }
    };
}
