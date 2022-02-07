mod account_argument;
mod account_list;
mod error;
#[cfg(feature = "nightly")]
mod in_place;
mod indexer;
mod instruction;
mod instruction_list;

pub use account_argument::*;
pub use account_list::*;
pub use error::*;
#[cfg(feature = "nightly")]
pub use in_place::*;
pub use indexer::*;
pub use instruction::*;
pub use instruction_list::*;

impl_indexed_for_unit!(u8[][]);
impl_indexed_for_unit!(u16[][]);
impl_indexed_for_unit!(u32[][]);
impl_indexed_for_unit!(u64[][]);
impl_indexed_for_unit!(u128[][]);
impl_indexed_for_unit!(usize[][]);
impl_indexed_for_unit!(i8[][]);
impl_indexed_for_unit!(i16[][]);
impl_indexed_for_unit!(i32[][]);
impl_indexed_for_unit!(i64[][]);
impl_indexed_for_unit!(i128[][]);
impl_indexed_for_unit!(isize[][]);
