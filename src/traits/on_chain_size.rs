use std::mem::size_of;

use solana_program::pubkey::Pubkey;

pub trait OnChainSize {
    fn on_chain_size() -> usize;
}

impl<T> OnChainSize for Option<T>
where
    T: OnChainSize,
{
    fn on_chain_size() -> usize {
        1 + T::on_chain_size()
    }
}
macro_rules! impl_on_chain_size_for_prim {
    (all: $($ty:ty),+ $(,)?) => {
        $(impl_on_chain_size_for_prim!($ty);)+
    };
    ($ty:ty) => {
        impl OnChainSize for $ty{
            fn on_chain_size() -> usize {
                size_of::<$ty>()
            }
        }
    };
}
impl_on_chain_size_for_prim!(
    all: bool,
    u8,
    u16,
    u32,
    u64,
    u128,
    i8,
    i16,
    i32,
    i64,
    i128,
    Pubkey
);
