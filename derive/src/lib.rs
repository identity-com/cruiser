#![warn(unused_import_braces, unused_imports)]

extern crate proc_macro;

mod account_argument;
mod error;
mod instruction_list;
mod log_level;
// mod instruction_list_processor;

use crate::account_argument::AccountArgumentDerive;
use crate::error::ErrorDerive;
use crate::instruction_list::InstructionListDerive;
// use crate::instruction_list_processor::InstructionListProcessorDerive;
use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::parse_macro_input;

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

/// The derive macro is implemented for structs only. Each field must implement [`AccountArgument`].
///
/// ## Struct Macros
/// The struct macro is `account_argument` and contains a comma seperated list of arguments.
/// ex:
/// ```
/// use solana_generator::AccountArgument;
/// #[derive(AccountArgument)]
/// #[account_argument(instruction_data = (size: usize))]
///  pub struct ArgumentAccounts{}
/// ```
/// ### `instruction_data`
/// format: `instruction_data = ($($name:ident: $ty:ty,)*)`
///
/// This is the types (`$ty`) that the [`InstructionArg`](AccountArgument::InstructionArg) tuple will be created from and the names (`$name`) that can be used to access them.
///
/// ## Field Macros
/// The field macro is `account_argument` and contains a comma seperated list of arguments.
/// These arguments can access the top level `instruction_data` by name.
/// ex:
/// ```
/// use solana_generator::{AccountInfo, AccountArgument};
/// #[derive(AccountArgument)]
///  pub struct ArgumentAccounts{
///      #[account_argument(signer, writable)]
///      account: AccountInfo,
///  }
/// ```
///
/// ### `signer`, `writable`, and `owner`
/// format: `$(signer|writable|owner)$(($optional_index:expr))? $(= $owner:expr)?
///
/// Requires the argument implement [`MultiIndexableAccountArgument`].
/// These allow restrictions to be added to the arguments they are added to.
/// `signer` verifies that the index is a signer
/// `writable` verifies that the index is writable
/// `owner` verifies that the index's owner is `$owner`. This is the only valid argument with `$owner`
///
/// `$optional_index` is an optional index (type `T`) where the argument must implement [`MultiIndexableAccountArgument<T>`].
/// Defaults to [`All`](crate::All)
///
/// ### `instruction_data`
/// format: `instruction_data = $data:expr`
///
/// This is optional and allows the setting of the [`InstructionArg`](AccountArgument::InstructionArg) passed to this field.
/// If not used calls [`Default::default`] instead.
#[proc_macro_error]
#[proc_macro_derive(AccountArgument, attributes(from, account_argument))]
pub fn derive_account_argument(ts: TokenStream) -> TokenStream {
    let stream = parse_macro_input!(ts as AccountArgumentDerive).into_token_stream();
    #[cfg(feature = "debug_account_argument")]
    {
        println!("{}", stream);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
    stream.into()
}

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

// #[proc_macro_error]
// #[proc_macro_derive(InstructionListProcessor, attributes(instruction_list_processor))]
// pub fn derive_instruction_list_processor(ts: TokenStream) -> TokenStream{
//     let stream = parse_macro_input!(ts as InstructionListProcessorDerive).into_token_stream();
//     #[cfg(feature = "debug_instruction_list_processor")]
//     {
//         println!("{}", stream);
//         std::thread::sleep(std::time::Duration::from_millis(100));
//     }
//     stream.into()
// }

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
use easy_proc::ArgumentList;
#[cfg(feature = "easy_proc_test")]
use proc_macro2::Span;
#[cfg(feature = "easy_proc_test")]
use syn::parse::{Parse, ParseStream};
#[cfg(feature = "easy_proc_test")]
use syn::{Ident, LitInt, LitStr};

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
