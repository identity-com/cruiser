use crate::account_argument::{combine_generics, AdditionalGenerics};
use crate::get_crate_name;
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{Data, DataEnum, DataStruct, DataUnion, DeriveInput, Fields, Generics, Ident};

#[derive(ArgumentList, Default)]
pub struct OnChainSizeAttribute {
    generics: Option<AdditionalGenerics>,
}
impl OnChainSizeAttribute {
    const IDENT: &'static str = "on_chain_size";
}

pub struct OnChainSizeDerive {
    ident: Ident,
    generics: Generics,
    data: Data,
    attribute: OnChainSizeAttribute,
}

impl Parse for OnChainSizeDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derive: DeriveInput = input.parse()?;
        let attribute = find_attr(
            derive.attrs.iter(),
            &format_ident!("{}", OnChainSizeAttribute::IDENT),
        )
        .map(OnChainSizeAttribute::parse_arguments)
        .unwrap_or_default();

        Ok(Self {
            ident: derive.ident,
            data: derive.data,
            generics: derive.generics,
            attribute,
        })
    }
}

impl OnChainSizeDerive {
    pub fn into_token_stream(self) -> TokenStream {
        let ident = self.ident;
        let (impl_generics, ty_generics, where_clause) =
            combine_generics(&self.generics, [self.attribute.generics.as_ref()]);
        let crate_name = get_crate_name();

        match self.data {
            Data::Struct(DataStruct { fields, .. }) => {
                let mut field_types = fields.into_iter().map(|field| field.ty);
                let first_field = field_types.next();

                match first_field {
                    Some(first_field) => quote! {
                        impl #impl_generics #crate_name::on_chain_size::OnChainSize for #ident #ty_generics #where_clause {
                            const ON_CHAIN_SIZE: usize = <#first_field as #crate_name::on_chain_size::OnChainSize>::ON_CHAIN_SIZE
                                #(+ <#field_types as #crate_name::on_chain_size::OnChainSize>::ON_CHAIN_SIZE)*;
                        }
                    },
                    None => quote! { 0 },
                }
            }
            Data::Enum(DataEnum { variants, .. }) => {
                let variants = variants.into_iter().map(|variant| {
                    let mut field_types = match variant.fields {
                        Fields::Named(fields) => fields
                            .named
                            .into_iter()
                            .map(|field| field.ty)
                            .collect::<Vec<_>>(),
                        Fields::Unnamed(fields) => {
                            fields.unnamed.into_iter().map(|field| field.ty).collect()
                        }
                        Fields::Unit => vec![],
                    }
                    .into_iter();
                    let first_field = field_types.next();
                    match first_field {
                        Some(first_field) => quote! {
                            <#first_field as #crate_name::on_chain_size::OnChainSize>::ON_CHAIN_SIZE
                                #(+ <#field_types as #crate_name::on_chain_size::OnChainSize>::ON_CHAIN_SIZE)*
                        },
                        None => quote! { 0 },
                    }

                });

                quote! {
                    impl #impl_generics #crate_name::on_chain_size::OnChainSize for #ident #ty_generics #where_clause {
                        const ON_CHAIN_SIZE: usize = 1 + #crate_name::util::usize_array_max([#(#variants),*]);
                    }
                }
            }
            Data::Union(DataUnion { fields, .. }) => {
                let field_types = fields.named.into_iter().map(|field| field.ty);

                quote! {
                    impl #impl_generics #crate_name::on_chain_size::OnChainSize for #ident #ty_generics #where_clause {
                        const ON_CHAIN_SIZE: usize = #crate_name::util::usize_array_max([#(<#field_types as #crate_name::on_chain_size::OnChainSize>::ON_CHAIN_SIZE,)*]);
                    }
                }
            }
        }
    }
}
