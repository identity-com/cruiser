#![warn(missing_docs, unused_import_braces)]

//! Helpers for making proc macro crates

pub extern crate self as easy_proc;

pub use easy_proc_common::{find_attr, find_attrs};
pub use easy_proc_derive::ArgumentList;
use proc_macro2::Ident;
pub use proc_macro_error;
use syn::Attribute;

/// A parsable list of arguments
pub trait ArgumentList: Sized {
    /// Parses the arguments of an attribute
    fn parse_arguments(attr: &Attribute) -> Self;
}

/// Parses a list of attributes for a given ident and type
pub fn parse_attribute_list<'a, T: ArgumentList>(
    ident: &'a Ident,
    attrs: impl IntoIterator<Item = &'a Attribute> + 'a,
) -> impl Iterator<Item = T> + 'a
where
    T: 'a,
{
    attrs
        .into_iter()
        .filter(move |attr| attr.path.is_ident(ident))
        .map(T::parse_arguments)
}
