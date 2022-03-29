#![warn(unused_import_braces, unused_imports, missing_docs, clippy::pedantic)]
#![allow(clippy::similar_names, clippy::module_name_repetitions)]

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

/// If no start specified starts at `1_000_000`
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

/// Derives `AccountArgument`, `FromAccounts`, and `ValidateArgument`.
///
/// # Requirements
/// This macro is implemented for structs only. Each field must implement `AccountArgument`.
///
/// # How to use
/// This macro utilizes `from`, `validate`, and `account_argument` attributes on the struct, and `from` and `validate` on the fields.
///
/// # `account_argument`
/// Arguments for the whole struct
/// ```ignore
/// #[derive(AccountArgument)]
/// #[account_argument(
///     no_from,
///     no_validate,
///     enum_discriminant_type = <$ty:ty>,
///     account_info = <$ty:ty>,
///     generics = [$(<$($gen:gen),*>)? $(where $($clause:where_clause),*)?],
/// )]
/// struct Test;
/// ```
/// | Argument | Argument Type | Description |
/// |---|---|---|
/// | `no_from` | presence | Presence of this means all `from` attributes are ignored and no default `FromAccounts` implementation is generated. |
/// | `no_validate` | presence | Presence of this means all `validate` attributes are ignored and no default `ValidateArgument` implementation is generated. |
/// | ~~`enum_discriminant_type = <$ty:ty>`~~ | optional | Sets the serialization type for the enum discriminant. Type must implement `CompressedNumber<Num = u64>`. Defaults to [`u64`]. Not yet implemented. |
/// | `account_info` | required | Sets the type for this arguments accoutn info. Most library functions are writen with this as a generic but you an force it to be a specific type as well. |
/// | `generics` | optional | Additional generics to apply to `AccountArgument`, `FromAccounts`, and `ValidateArgument` implementations. Can include generics and a where clause. |
///
/// # `from`
/// Arguments for `FromAccounts` implementation. Multiple `from` attributes can exist, each with a different id.
/// ```ignore
/// #[derive(AccountArgument)]
/// #[from(
///     id = <$id:ident>,
///     data = (<$($data_name:ident: $data_ty:ty),*>),
///     enum_discriminant = <$dis:expr>,
///     log_level: <$log_level:ident>,
///     generics = [$(<$($gen:gen),*>)? $(where $($clause:where_clause),*)?],
/// )]
/// struct Test{
///     #[from(
///         id = <$id:ident>,
///         data = <$data:expr>,
///     )]
///     field: FieldType,
/// }
/// ```
///
/// ## Struct Attribute
/// | Argument | Argument Type | Description |
/// |---|---|---|
/// | `id = <$id:ident>` | optional | Sets the id for this attribute and for other to reference. Defaults to unique default id. |
/// | `data = (<$($data_name:ident: $data_ty:ty),*>)` | optional | Data type coming in for the `FromAccounts` implementation. `$data_name` is the name that can be referenced. `$data_ty` is the type of the data argument. Type defaults to [`()`] and maps to a tupple of the types. If a single argument is present then both `FromAccounts<$data_ty>` and `FromAccounts<($data_ty,)>` are implemented. |
/// | ~~`enum_discriminant = <$dis:expr>`~~ | optional | Sets the enum discriminant from the incoming data. Required if deriving on enum. Not yet implemented. |
/// | `log_level = $<log_level:ident>` | optional | Sets the logging level for implementation. Valid are `none`, `error`, `warn`, `info`, `debug`, or `trace` |
/// | `generics = [$(<$($gen:gen),*>)? $(where $($clause:where_clause),*)?]` | optional | Additional generics to apply to this `FromAccounts` implementation. Can include generics and a where clause. |
///
/// ## Field Attribute
/// | Argument | Argument Type | Description |
/// |---|---|---|
/// | `id = <$id:ident>` | optional | Points to the struct attribute that this references. Defaults to unique empty id. |
/// | `data = <$data:expr>` | optional | The argument to pass to the field's `FromAccounts` implementation. Defaults to [`()`] |
///
/// # `validate`
/// Arguments for `ValidateArgument` implementation. Multiple `validate` attributes can exist, each with a different id.
/// ```ignore
/// #[derive(AccountArgument)]
/// #[validate(
///     id = <$id:ident>,
///     data = (<$($data_name:ident: $data_ty:ty),*>),
///     log_level: <$log_level:ident>,
///     generics = [$(<$($gen:gen),*>)? $(where $($clause:where_clause),*)?],
/// )]
/// struct Test{
///     #[validate(
///         id = <$id:ident>,
///         data = <$data:expr>,
///         signer(<$index:expr>),
///         writable(<$index:expr>),
///         owner(<$index:expr>) = <$owner:expr>,
///         key(<$index:expr>) = <$key:expr>,
///     )]
///     field: FieldType,
/// }
/// ```
/// ## Struct Attribute
/// | Argument | Argument Type | Description |
/// |---|---|---|
/// | `id = <$id:ident>` | optional | Sets the id for this attribute and for other to reference. Defaults to unique default id. |
/// | `data = (<$($data_name:ident: $data_ty:ty),*>)` | optional | Data type coming in for the `ValidateArgument` implementation. `$data_name` is the name that can be referenced. `$data_ty` is the type of the data argument. Type defaults to [`()`] and maps to a tupple of the types. If a single argument is present then both `ValidateArgument<$data_ty>` and `ValidateArgument<($data_ty,)>` are implemented. |
/// | `log_level = $<log_level:ident>` | optional | Sets the logging level for implementation. Valid are `none`, `error`, `warn`, `info`, `debug`, or `trace` |
/// | `generics = [$(<$($gen:gen),*>)? $(where $($clause:where_clause),*)?]` | optional | Additional generics to apply to this `ValidateArgument` implementation. Can include generics and a where clause. |
///
/// ## Field Attribute
/// | Argument | Argument Type | Description |
/// |---|---|---|
/// | `id = <$id:ident>` | optional | Points to the struct attribute that this references. Defaults to unique empty id. |
/// | `data = <$data:expr>` | optional | The argument to pass to the field's `ValidateArgument` implementation. Defaults to [`()`] |
/// | `signer(<$index:expr>)` | multiple, 0+ | Checks that `MultiIndexable::is_signer($index)` is true. If indexer is omitted defaults to `AllAny::All` |
/// | `writable(<$index:expr)` | multiple, 0+ | Checks that `MultiIndexable::is_signer($index)` is true. If indexer is omitted defaults to `AllAny::All` |
/// | `owner(<$index:expr>) = <$owner:expr>` | multiple, 0+ | Checks that `MultiIndexable::is_owner($owner, $index)` is true. If indexer is omitted defaults to `AllAny::All` |
/// | `key(<$index:expr) = <$key:expr>` | multiple, 0+ | Checks that `SingleIndexable::info($index).key` is `$key`. If indexer is omitted defaults to `AllAny::All` |
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
// #[cfg(feature = "in_place")]
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

/// Macro for testing `easy_proc`
#[cfg(feature = "easy_proc_test")]
#[proc_macro_error]
#[proc_macro_attribute]
pub fn test_easy_proc(args: TokenStream, tokens: TokenStream) -> TokenStream {
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
