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
