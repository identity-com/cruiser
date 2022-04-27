//! Automatic size calculation for on-chain data. Derive not created yet, must be done manually for now.

use std::marker::PhantomData;
use std::mem::size_of;

use solana_program::pubkey::Pubkey;

/// This value has as static size on-chain
pub trait OnChainSize {
    /// The size on-chain
    const ON_CHAIN_SIZE: usize;
}

/// Theis value can be sized with a given argument
pub trait OnChainSizeWithArg<Arg> {
    /// Gets the size with an argument
    fn on_chain_size_with_arg(arg: Arg) -> usize;
}

impl<T> const OnChainSizeWithArg<()> for T
where
    T: OnChainSize,
{
    fn on_chain_size_with_arg(_arg: ()) -> usize {
        Self::ON_CHAIN_SIZE
    }
}

impl<T> const OnChainSize for Option<T>
where
    T: ~const OnChainSize,
{
    const ON_CHAIN_SIZE: usize = 1 + T::ON_CHAIN_SIZE;
}

impl<T> const OnChainSize for PhantomData<T> {
    const ON_CHAIN_SIZE: usize = 0;
}

impl<T, const N: usize> const OnChainSize for [T; N]
where
    T: ~const OnChainSize,
{
    const ON_CHAIN_SIZE: usize = T::ON_CHAIN_SIZE * N;
}

/// String byte length as arg
impl const OnChainSizeWithArg<usize> for String {
    fn on_chain_size_with_arg(arg: usize) -> usize {
        u32::ON_CHAIN_SIZE + arg
    }
}

impl<T> const OnChainSizeWithArg<usize> for Vec<T>
where
    T: OnChainSize,
{
    fn on_chain_size_with_arg(arg: usize) -> usize {
        u32::ON_CHAIN_SIZE + arg * T::ON_CHAIN_SIZE
    }
}

impl<T1, T2> const OnChainSize for (T1, T2)
where
    T1: ~const OnChainSize,
    T2: ~const OnChainSize,
{
    const ON_CHAIN_SIZE: usize = T1::ON_CHAIN_SIZE + T2::ON_CHAIN_SIZE;
}

impl<T1, T2, A1, A2> const OnChainSizeWithArg<(A1, A2)> for (T1, T2)
where
    T1: ~const OnChainSizeWithArg<A1>,
    T2: ~const OnChainSizeWithArg<A2>,
    A1: ~const Drop,
    A2: ~const Drop,
{
    fn on_chain_size_with_arg((arg1, arg2): (A1, A2)) -> usize {
        T1::on_chain_size_with_arg(arg1) + T2::on_chain_size_with_arg(arg2)
    }
}

impl<T1, T2, T3> const OnChainSize for (T1, T2, T3)
where
    T1: ~const OnChainSize,
    T2: ~const OnChainSize,
    T3: ~const OnChainSize,
{
    const ON_CHAIN_SIZE: usize = T1::ON_CHAIN_SIZE + T2::ON_CHAIN_SIZE + T3::ON_CHAIN_SIZE;
}

impl<T1, T2, T3, A1, A2, A3> const OnChainSizeWithArg<(A1, A2, A3)> for (T1, T2, T3)
where
    T1: ~const OnChainSizeWithArg<A1>,
    T2: ~const OnChainSizeWithArg<A2>,
    T3: ~const OnChainSizeWithArg<A3>,
    A1: ~const Drop,
    A2: ~const Drop,
    A3: ~const Drop,
{
    fn on_chain_size_with_arg((arg1, arg2, arg3): (A1, A2, A3)) -> usize {
        T1::on_chain_size_with_arg(arg1)
            + T2::on_chain_size_with_arg(arg2)
            + T3::on_chain_size_with_arg(arg3)
    }
}

macro_rules! impl_on_chain_size_for_prim {
    (all: $($ty:ty),+ $(,)?) => {
        $(impl_on_chain_size_for_prim!($ty);)+
    };
    ($ty:ty) => {
        impl const OnChainSize for $ty{
            const ON_CHAIN_SIZE: usize = size_of::<$ty>();
        }
    };
}
impl_on_chain_size_for_prim!(
    all: (),
    bool,
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
    Pubkey,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
);
