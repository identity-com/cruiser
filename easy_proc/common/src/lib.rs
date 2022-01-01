use std::iter::Filter;
use proc_macro_error::abort;
use syn::{Attribute, Ident};

pub fn find_attr(attrs: impl IntoIterator<Item = Attribute>, ident: &Ident) -> Option<Attribute> {
    let mut attrs = find_attrs(attrs, ident);
    let found = attrs.next();
    if let Some(attr) = attrs.next() {
        abort!(attr.path, "Multiple `{}` attributes found", ident)
    }
    found
}

pub fn find_attrs<I: IntoIterator<Item = Attribute>>(
    attrs: I,
    ident: &Ident,
) -> Filter<I::IntoIter, impl FnMut(&Attribute) -> bool + '_> {
    attrs.into_iter().filter(|attr| attr.path.is_ident(ident))
}