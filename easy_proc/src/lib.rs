pub extern crate self as easy_proc;

pub use easy_proc_common::find_attrs;
pub use easy_proc_derive::ArgEnum;
pub use proc_macro_error;
use syn::Attribute;

pub trait Arguments: Sized {
    fn parse_arguments(attr: &Attribute) -> Self;
}
