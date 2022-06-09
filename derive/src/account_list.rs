use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DataStruct, DataUnion, DeriveInput, Fields, Generics, Ident, Type};

use easy_proc::{find_attr, ArgumentList};

use crate::get_crate_name;

#[derive(ArgumentList)]
pub struct AccountListAttribute {
    #[argument(default = syn::parse_str("u64").unwrap())]
    discriminant_type: Type,
}

impl Default for AccountListAttribute {
    fn default() -> Self {
        Self {
            discriminant_type: syn::parse_str("::std::num::NonZeroU64").unwrap(),
        }
    }
}

#[derive(ArgumentList)]
pub struct AccountListVariantAttribute {
    data: Type,
}

pub struct AccountListDerive {
    generics: Generics,
    attribute: AccountListAttribute,
    ident: Ident,
    variant_types: Vec<Type>,
    variant_discriminants: Vec<TokenStream>,
}

impl AccountListDerive {
    pub fn into_token_stream(self) -> TokenStream {
        let crate_name = get_crate_name();

        let AccountListDerive {
            generics,
            attribute,
            ident,
            variant_types,
            variant_discriminants,
        } = self;
        let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
        let discriminant_type = attribute.discriminant_type;

        let variant_impls =
            variant_types.into_iter()
            .zip(variant_discriminants.into_iter())
            .map(|(ty, dis)| {
                quote! {
                    #crate_name::static_assertions::const_assert_ne!(0, #dis);
                    #[automatically_derived]
                    unsafe impl #impl_gen #crate_name::account_list::AccountListItem<#ty> for #ident #ty_gen #where_clause {
                        fn discriminant() -> ::std::num::NonZeroU64{
                            ::std::num::NonZeroU64::new(#dis).unwrap()
                        }
                    }
                }
            }).collect::<Vec<_>>();

        quote! {
            #(#variant_impls)*

            #[automatically_derived]
            impl #impl_gen #crate_name::account_list::AccountList for #ident #ty_gen #where_clause {
                type DiscriminantCompressed = #discriminant_type;
            }
        }
    }
}

impl Parse for AccountListDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derive: DeriveInput = input.parse()?;
        let enum_data = match derive.data {
            Data::Struct(DataStruct { struct_token, .. }) => {
                abort!(struct_token, "`#[derive(AccountList)] only supports enums")
            }
            Data::Enum(data) => data,
            Data::Union(DataUnion { union_token, .. }) => {
                abort!(union_token, "`#[derive(AccountList)] only supports enums")
            }
        };

        let account_list_attribute =
            find_attr(derive.attrs, &Ident::new("account_list", Span::call_site()))
                .as_ref()
                .map(AccountListAttribute::parse_arguments)
                .unwrap_or_default();

        let mut variant_types = Vec::with_capacity(enum_data.variants.len());
        let mut variant_discriminants = Vec::with_capacity(enum_data.variants.len());
        let mut last = None;
        for variant in enum_data.variants {
            match variant.fields {
                Fields::Named(_) | Fields::Unnamed(_) => {
                    abort!(variant.ident, "Only unit variants are allowed")
                }
                Fields::Unit => {}
            }
            let attribute = find_attr(variant.attrs, &Ident::new("account", Span::call_site()))
                .as_ref()
                .map_or_else(
                    || abort!(variant.ident, "Missing `#[account]` attribute"),
                    AccountListVariantAttribute::parse_arguments,
                );
            variant_types.push(attribute.data);
            let value = if let Some(last) = last {
                quote! {
                    (#last) + 1
                }
            } else {
                quote! {
                    1
                }
            };
            variant_discriminants.push(value.clone());
            last = Some(value.clone());
        }

        Ok(Self {
            generics: derive.generics,
            ident: derive.ident,
            attribute: account_list_attribute,
            variant_types,
            variant_discriminants,
        })
    }
}
