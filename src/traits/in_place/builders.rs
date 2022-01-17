pub type DynamicInPlaceVec<T, D> = super::DynamicInPlaceVec<'static, T, D>;
pub type StaticInPlaceVec<T, D, const N: usize> = super::StaticInPlaceVec<'static, T, D, N>;
