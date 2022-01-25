use proc_macro_error::abort;
use std::iter::Filter;
use syn::{Attribute, Ident};

pub fn find_attr<T: PathIsIdent>(attrs: impl IntoIterator<Item = T>, ident: &Ident) -> Option<T> {
    let mut attrs = find_attrs(attrs, ident);
    let found = attrs.next();
    if let Some(attr) = attrs.next() {
        attr.abort_with_span(format!("Multiple `{}` attributes found", ident))
    }
    found
}
pub trait PathIsIdent {
    fn path_is_ident(&self, ident: &Ident) -> bool;
    fn abort_with_span(self, message: String) -> !;
}
impl PathIsIdent for Attribute {
    fn path_is_ident(&self, ident: &Ident) -> bool {
        self.path.is_ident(ident)
    }

    fn abort_with_span(self, message: String) -> ! {
        abort!(self, "{}", message)
    }
}
impl PathIsIdent for &Attribute {
    fn path_is_ident(&self, ident: &Ident) -> bool {
        self.path.is_ident(ident)
    }

    fn abort_with_span(self, message: String) -> ! {
        abort!(self, "{}", message)
    }
}
impl PathIsIdent for &mut Attribute {
    fn path_is_ident(&self, ident: &Ident) -> bool {
        self.path.is_ident(ident)
    }

    fn abort_with_span(self, message: String) -> ! {
        abort!(self, "{}", message)
    }
}

pub fn find_attrs<'a, T: PathIsIdent, I: 'a + IntoIterator<Item = T>>(
    attrs: I,
    ident: &'a Ident,
) -> Filter<I::IntoIter, impl FnMut(&T) -> bool + 'a> {
    attrs
        .into_iter()
        .filter(move |attr| attr.path_is_ident(ident))
}
