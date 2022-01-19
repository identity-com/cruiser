pub extern crate self as easy_proc;

pub use easy_proc_common::{find_attr, find_attrs};
pub use easy_proc_derive::ArgumentList;
use proc_macro2::Ident;
pub use proc_macro_error;
use syn::Attribute;

pub trait ArgumentList: Sized {
    fn parse_arguments(attr: &Attribute) -> Self;
}

pub fn parse_attribute_list<'a, T: ArgumentList>(
    ident: &'a Ident,
    attrs: impl IntoIterator<Item = &'a Attribute> + 'a,
) -> impl Iterator<Item = T> + 'a
where
    T: 'a,
{
    attrs
        .into_iter()
        .filter(|attr| attr.path.is_ident(ident))
        .map(T::parse_arguments)
}
