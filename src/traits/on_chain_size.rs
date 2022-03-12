//! Automatic size calculation for on-chain data. Derive not created yet, must be done manually for now.

use std::mem::size_of;

use solana_program::pubkey::Pubkey;

/// This value can be sized on-chain using arg `A`
pub trait OnChainSize<A> {
    /// Gets the on-chain size of this value
    #[must_use]
    fn on_chain_max_size(arg: A) -> usize;
}
/// This value can be statically sized
pub trait OnChainStaticSize: OnChainSize<()> {
    /// Gets the on-chain size of this value
    #[must_use]
    fn on_chain_static_size() -> usize {
        Self::on_chain_max_size(())
    }
}
impl<T> OnChainStaticSize for T where T: OnChainSize<()> {}

impl<A, T> OnChainSize<A> for Option<T>
where
    T: OnChainSize<A>,
{
    fn on_chain_max_size(arg: A) -> usize {
        1 + T::on_chain_max_size(arg)
    }
}
impl<T> OnChainSize<usize> for Vec<T>
where
    T: OnChainStaticSize,
{
    fn on_chain_max_size(arg: usize) -> usize {
        4 + arg * T::on_chain_static_size()
    }
}
impl<A, T, const N: usize> OnChainSize<[A; N]> for Vec<T>
where
    T: OnChainSize<A>,
{
    fn on_chain_max_size(arg: [A; N]) -> usize {
        arg.into_iter().map(|arg| T::on_chain_max_size(arg)).sum()
    }
}

macro_rules! impl_on_chain_size_for_prim {
    (all: $($ty:ty),+ $(,)?) => {
        $(impl_on_chain_size_for_prim!($ty);)+
    };
    ($ty:ty) => {
        impl OnChainSize<()> for $ty{
            fn on_chain_max_size(_arg: ()) -> usize {
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
