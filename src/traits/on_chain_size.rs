//! Automatic size calculation for on-chain data. Derive not created yet, must be done manually for now.

use std::marker::PhantomData;
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
    fn on_chain_static_size() -> usize;
}
impl<T> const OnChainStaticSize for T
where
    T: ~const OnChainSize<()>,
{
    fn on_chain_static_size() -> usize {
        Self::on_chain_max_size(())
    }
}

impl<A, T> const OnChainSize<A> for Option<T>
where
    T: ~const OnChainSize<A>,
{
    fn on_chain_max_size(arg: A) -> usize {
        1 + T::on_chain_max_size(arg)
    }
}
impl<T> const OnChainSize<()> for PhantomData<T> {
    fn on_chain_max_size(_: ()) -> usize {
        0
    }
}
// This is byte length of the string, not chars
impl const OnChainSize<usize> for String {
    fn on_chain_max_size(arg: usize) -> usize {
        Vec::<u8>::on_chain_max_size(arg)
    }
}
impl<T> const OnChainSize<usize> for Vec<T>
where
    T: ~const OnChainStaticSize,
{
    fn on_chain_max_size(arg: usize) -> usize {
        u32::on_chain_static_size() + arg * T::on_chain_static_size()
    }
}
#[cfg(not(feature = "const_eval"))]
impl<T, I, A> OnChainSize<(I,)> for Vec<T>
where
    I: IntoIterator<Item = A>,
    T: OnChainSize<A>,
{
    fn on_chain_max_size(arg: (I,)) -> usize {
        4 + arg.0.into_iter().map(T::on_chain_max_size).sum::<usize>()
    }
}
#[cfg(feature = "const_eval")]
impl<T, I, A> const OnChainSize<(I,)> for Vec<T>
where
    I: ~const IntoIterator<Item = A> + ~const Drop,
    I::IntoIter: ~const Iterator + ~const Drop,
    A: ~const Drop,
    T: ~const OnChainSize<A>,
{
    fn on_chain_max_size(arg: (I,)) -> usize {
        let mut sum = 4;
        for arg in arg.0 {
            sum += T::on_chain_max_size(arg);
        }
        sum
    }
}
impl<A, T, const N: usize> OnChainSize<[A; N]> for Vec<T>
where
    T: OnChainSize<A>,
{
    fn on_chain_max_size(arg: [A; N]) -> usize {
        4 + arg
            .into_iter()
            .map(|arg| T::on_chain_max_size(arg))
            .sum::<usize>()
    }
}
impl<T, const N: usize> const OnChainSize<()> for [T; N]
where
    T: ~const OnChainStaticSize,
{
    fn on_chain_max_size(_: ()) -> usize {
        N * T::on_chain_static_size()
    }
}

impl<A, T, const N: usize> OnChainSize<[A; N]> for [T; N]
where
    T: OnChainSize<A>,
{
    fn on_chain_max_size(arg: [A; N]) -> usize {
        arg.into_iter().map(T::on_chain_max_size).sum::<usize>()
    }
}
impl<T1, T2> const OnChainSize<()> for (T1, T2)
where
    T1: ~const OnChainStaticSize,
    T2: ~const OnChainStaticSize,
{
    fn on_chain_max_size(_arg: ()) -> usize {
        T1::on_chain_static_size() + T2::on_chain_static_size()
    }
}
impl<A1, A2, T1, T2> OnChainSize<(A1, A2)> for (T1, T2)
where
    T1: OnChainSize<A1>,
    T2: OnChainSize<A2>,
{
    fn on_chain_max_size(arg: (A1, A2)) -> usize {
        T1::on_chain_max_size(arg.0) + T2::on_chain_max_size(arg.1)
    }
}
impl<T1, T2, T3> const OnChainSize<()> for (T1, T2, T3)
where
    T1: ~const OnChainStaticSize,
    T2: ~const OnChainStaticSize,
    T3: ~const OnChainStaticSize,
{
    fn on_chain_max_size(_arg: ()) -> usize {
        T1::on_chain_static_size() + T2::on_chain_static_size() + T3::on_chain_static_size()
    }
}
impl<A1, A2, A3, T1, T2, T3> OnChainSize<(A1, A2, A3)> for (T1, T2, T3)
where
    T1: OnChainSize<A1>,
    T2: OnChainSize<A2>,
    T3: OnChainSize<A3>,
{
    fn on_chain_max_size(arg: (A1, A2, A3)) -> usize {
        T1::on_chain_max_size(arg.0) + T2::on_chain_max_size(arg.1) + T3::on_chain_max_size(arg.2)
    }
}

macro_rules! impl_on_chain_size_for_prim {
    (all: $($ty:ty),+ $(,)?) => {
        $(impl_on_chain_size_for_prim!($ty);)+
    };
    ($ty:ty) => {
        impl const OnChainSize<()> for $ty{
            fn on_chain_max_size(_arg: ()) -> usize {
                size_of::<$ty>()
            }
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
