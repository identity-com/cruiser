#![warn(unused_import_braces, unused_imports, missing_docs)]

//! The proc macros of [`cruiser`](https://docs.rs/cruiser/latest/cruiser/)

extern crate proc_macro;

use proc_macro::TokenStream;

#[cfg(feature = "easy_proc_test")]
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::proc_macro_error;
use quote::{format_ident, quote};
#[cfg(feature = "easy_proc_test")]
use syn::parse::{Parse, ParseStream};
use syn::parse_macro_input;
#[cfg(feature = "easy_proc_test")]
use syn::{Ident, LitInt, LitStr};

#[cfg(feature = "easy_proc_test")]
use easy_proc::ArgumentList;

use crate::account_argument::AccountArgumentDerive;
use crate::account_list::AccountListDerive;
use crate::error::ErrorDerive;
use crate::instruction_list::InstructionListDerive;
use crate::verify_account_arg_impl::VerifyAccountArgs;

mod account_argument;
mod account_list;
mod error;
mod in_place;
mod instruction_list;
mod log_level;
mod verify_account_arg_impl;

/// If no start specified starts at `300`
#[proc_macro_error]
#[proc_macro_derive(Error, attributes(error, error_msg))]
pub fn derive_error(ts: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(ts as ErrorDerive).into_token_stream();
    #[cfg(feature = "debug_error")]
    {
        println!("{}", stream);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    stream.into()
}

/// The derive macro is implemented for structs only. Each field must implement `AccountArgument`.
///
/// TODO: Write docs for this
#[proc_macro_error]
#[proc_macro_derive(AccountArgument, attributes(from, account_argument, validate))]
pub fn derive_account_argument(ts: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(ts as AccountArgumentDerive).into_token_stream();
    #[cfg(feature = "debug_account_argument")]
    {
        println!("{}", stream);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    stream.into()
}

/// Derives the `InstructionList` trait.
///
/// TODO: Write docs for this
#[proc_macro_error]
#[proc_macro_derive(InstructionList, attributes(instruction_list, instruction))]
pub fn derive_instruction_list(ts: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(ts as InstructionListDerive).into_token_stream();
    #[cfg(feature = "debug_instruction_list")]
    {
        println!("{}", stream);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    stream.into()
}

/// Derives the `AccountList` trait
///
/// TODO: Write docs for this
#[proc_macro_error]
#[proc_macro_derive(AccountList)]
pub fn derive_account_list(ts: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(ts as AccountListDerive).into_token_stream();
    #[cfg(feature = "debug_account_list")]
    {
        println!("{}", stream);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    stream.into()
}

/// Verifies a given type implements the proper traits
///
/// TODO: Write docs for this
#[proc_macro_error]
#[proc_macro]
pub fn verify_account_arg_impl(tokens: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(tokens as VerifyAccountArgs).into_token_stream();
    stream.into()
}

fn get_crate_name() -> proc_macro2::TokenStream {
    let generator_crate = crate_name("cruiser").expect("Could not find `cruiser`");
    match generator_crate {
        FoundCrate::Itself => quote! { ::cruiser },
        FoundCrate::Name(name) => {
            let ident = format_ident!("{}", name);
            quote! { ::#ident }
        }
    }
}

// /// Sets up an in-place struct
// #[cfg(feature = "nightly")]
// #[proc_macro_error]
// #[proc_macro_attribute]
// pub fn derive_in_place(args: TokenStream, tokens: TokenStream) -> TokenStream {
//     let stream = parse_macro_input!(tokens as AccountListDerive).into_token_stream();
//     #[cfg(feature = "debug_in_place")]
//     {
//         println!("{}", stream);
//         std::thread::sleep(std::time::Duration::from_millis(100));
//     }
//     stream.into()
// }

#[cfg(feature = "easy_proc_test")]
#[proc_macro_error]
#[proc_macro_attribute]
fn test_easy_proc(args: TokenStream, tokens: TokenStream) -> TokenStream {
    println!("ts1: {}", args);
    println!("ts2: {}", tokens);

    let tokens = parse_macro_input!(tokens as TestStruct);
    tokens.into_token_stream()
}

#[cfg(feature = "easy_proc_test")]
struct TestStruct {
    cool: Cool,
}
#[cfg(feature = "easy_proc_test")]
impl TestStruct {
    fn into_token_stream(self) -> TokenStream {
        if self.cool.boolean_value {
            (quote::quote! {
                fn cool(){
                    println!("Success!");
                }
            })
            .into()
        } else {
            proc_macro_error::abort_call_site!("Oh No!");
        }
    }
}
#[cfg(feature = "easy_proc_test")]
impl Parse for TestStruct {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![fn]>()?;
        input.parse::<syn::Ident>()?;
        let _content;
        syn::parenthesized!(_content in input);
        let content;
        syn::braced!(content in input);
        let function = content.parse::<syn::ItemFn>()?;
        let cool = Cool::parse_arguments(&function.attrs[0]);
        Ok(Self { cool })
    }
}

#[cfg(feature = "easy_proc_test")]
#[derive(ArgumentList)]
#[allow(dead_code)]
struct Cool {
    /// The ident of the whole attribute, not required and can only be one
    #[argument(attr_ident)]
    pub attr_ident: Ident,
    /// [`true`] if arg is present
    #[argument(presence)]
    pub boolean_value: bool,
    /// Required argument of form `count = 10`
    pub count: LitInt,
    /// Optional argument, if present of form `size = 3`
    pub size: Option<LitInt>,
    /// Custom parsing, including equals. Uses parse function.
    /// Ex: `custom_parse cool`
    #[argument(custom)]
    pub custom_parse: Ident,
    /// Optional with default value. Also implies `raw_type`
    #[argument(default = Ident::new("default", Span::call_site()))]
    pub default: Ident,
    /// Many, 0 or more
    pub many: Vec<LitStr>,
}
