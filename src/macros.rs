/// Builds an instruction.
/// Used to shorten [`InstructionList::build_instruction`](crate::InstructionList::build_instruction) invocations.
#[macro_export]
macro_rules! build_instruction {
    ($program_id:expr, $instruction:ty, $instruction_ident:ident($instruction_arg:expr)) => {
        <$instruction as $crate::InstructionList>::build_instruction(
            $program_id,
            <$instruction as $crate::InstructionList>::BuildEnum::$instruction_ident(
                $instruction_arg,
            ),
        )
    };
}

/// Implements indexing with [`All`], [`Any`], [`NotAll`], and [`NotAny`] for a type that indexes with [`AllAny`].
#[macro_export]
macro_rules! impl_indexed_for_all_any {
    ($ty:ty $(, <$($impl_gen:ident),*> $(, where $($(for<$($where_for:ty),*>)? $where_ty:ty: $($first_where:ty)?),*)?)? $(,)?) => {
        impl<$($($impl_gen,)*)?> MultiIndexable<$crate::All> for $ty
        where
            $($($(for<$($($where_for,)*)?> $where_ty: $($first_where)?,)*)?)?
        {
            fn is_signer(&self, indexer: $crate::All) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_signer(self, indexer.into())
            }

            fn is_writable(&self, indexer: $crate::All) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_writable(self, indexer.into())
            }

            fn is_owner(&self, owner: &$crate::Pubkey, indexer: $crate::All) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_owner(self, owner, indexer.into())
            }
        }

        impl<$($($impl_gen,)*)?> MultiIndexable<$crate::NotAll> for $ty
        where
            $($($(for<$($($where_for,)*)?> $where_ty: $($first_where)?,)*)?)?
        {
            fn is_signer(&self, indexer: $crate::NotAll) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_signer(self, indexer.into())
            }

            fn is_writable(&self, indexer: $crate::NotAll) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_writable(self, indexer.into())
            }

            fn is_owner(&self, owner: &$crate::Pubkey, indexer: $crate::NotAll) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_owner(self, owner, indexer.into())
            }
        }

        impl<$($($impl_gen,)*)?> MultiIndexable<$crate::Any> for $ty
        where
            $($($(for<$($($where_for,)*)?> $where_ty: $($first_where)?,)*)?)?
        {
            fn is_signer(&self, indexer: $crate::Any) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_signer(self, indexer.into())
            }

            fn is_writable(&self, indexer: $crate::Any) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_writable(self, indexer.into())
            }

            fn is_owner(&self, owner: &$crate::Pubkey, indexer: $crate::Any) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_owner(self, owner, indexer.into())
            }
        }

        impl<$($($impl_gen,)*)?> MultiIndexable<$crate::NotAny> for $ty
        where
            $($($(for<$($($where_for,)*)?> $where_ty: $($first_where)?,)*)?)?
        {
            fn is_signer(&self, indexer: $crate::NotAny) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_signer(self, indexer.into())
            }

            fn is_writable(&self, indexer: $crate::NotAny) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_writable(self, indexer.into())
            }

            fn is_owner(&self, owner: &$crate::Pubkey, indexer: $crate::NotAny) -> $crate::GeneratorResult<bool> {
                <$ty as $crate::MultiIndexable<$crate::AllAny>>::is_owner(self, owner, indexer.into())
            }
        }
    }
}

/// Implements [`AccountArgument`] for a type with a certain accessor.
#[macro_export]
macro_rules! delegate_account_argument {
    ($ty:ty, ($accessor:tt)$(, $($where:tt)?)?) => {
        impl $crate::AccountArgument for $ty $($($where)?)? {
            fn write_back(
                self,
                program_id: &'static $crate::Pubkey,
                system_program: Option<&$crate::SystemProgram>,
            ) -> $crate::GeneratorResult<()> {
                self.$accessor.write_back(program_id, system_program)
            }

            fn add_keys(
                &self,
                add: impl FnMut(&'static $crate::Pubkey) -> $crate::GeneratorResult<()>,
            ) -> $crate::GeneratorResult<()> {
                self.$accessor.add_keys(add)
            }
        }
    };
}
/// Implements [`MultiIndexable`] for a type with a certain accessor.
#[macro_export]
macro_rules! delegate_multi_indexable {
    ($ty:ty, $indexer:ty, ($accessor:tt)$(, $($where:tt)?)?) => {
        impl $crate::MultiIndexable<$indexer> for $ty $($($where)?)? {
            #[inline]
            fn is_signer(&self, indexer: $indexer) -> $crate::GeneratorResult<bool> {
                self.$accessor.is_signer(indexer)
            }

            #[inline]
            fn is_writable(&self, indexer: $indexer) -> $crate::GeneratorResult<bool> {
                self.$accessor.is_writable(indexer)
            }

            #[inline]
            fn is_owner(
                &self,
                owner: &$crate::Pubkey,
                indexer: $indexer,
            ) -> $crate::GeneratorResult<bool> {
                self.$accessor.is_owner(owner, indexer)
            }
        }
    };
}
/// Implements [`SingleIndexable`] for a type with a certain accessor.
#[macro_export]
macro_rules! delegate_single_indexable {
    ($ty:ty, $indexer:ty, ($accessor:tt)$(, $($where:tt)?)?) => {
        impl $crate::SingleIndexable<$indexer> for $ty $($($where)?)? {
            #[inline]
            fn info(
                &self,
                indexer: $indexer,
            ) -> $crate::GeneratorResult<&$crate::AccountInfo> {
                self.$accessor.info(indexer)
            }
        }
    };
}
