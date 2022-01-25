use easy_proc::{parse_attribute_list, ArgumentList};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, quote_spanned, ToTokens};
use std::convert::{TryFrom, TryInto};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parenthesized, parse_str, token, Attribute, Data, DataEnum, DeriveInput, Expr, Fields,
    Generics, Ident, Index, Token, Type,
};

#[derive(ArgumentList)]
pub struct FromAttribute {
    id: Option<Ident>,
    data: NamedTupple,
}
impl FromAttribute {
    const IDENT: &'static str = "from";
}

pub struct NamedTupple {
    paren: token::Paren,
    list: Punctuated<(Ident, Token![=], Type), Token![,]>,
}
impl Parse for NamedTupple {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let paren = parenthesized!(content in input);
        let list = content
            .parse_terminated(|stream| Ok((stream.parse()?, stream.parse()?, stream.parse()?)))?;
        Ok(Self { paren, list })
    }
}

pub struct AccountArgumentDerive {
    ident: Ident,
    generics: Generics,
    derive_type: AccountArgumentDeriveType,
    from_attributes: Vec<FromAttribute>,
}
impl Parse for AccountArgumentDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let from_attribute_ident = Ident::new(FromAttribute::IDENT, Span::call_site());
        let derive_input: DeriveInput = input.parse()?;

        let from_attributes =
            parse_attribute_list(&from_attribute_ident, derive_input.attrs.iter()).collect();
        let derive_type =
            AccountArgumentDeriveType::from_data(derive_input.data, &derive_input.ident)?;

        Ok(Self {
            ident: derive_input.ident,
            generics: derive_input.generics,
            derive_type,
            from_attributes,
        })
    }
}
impl AccountArgumentDerive {
    pub fn into_token_stream(self) -> TokenStream {
        let generator_crate =
            crate_name("solana_generator").expect("Could not find `solana_generator`");
        let crate_name = match generator_crate {
            FoundCrate::Itself => quote! { ::solana_generator },
            FoundCrate::Name(name) => {
                let ident = Ident::new(&name, Span::call_site());
                quote! { ::#ident }
            }
        };

        let ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let (fields_and_creation, write_back, keys) = {
            let AccountArgumentDeriveType::Struct(derive_struct) = self.derive_type.clone();
            match derive_struct {
                AccountArgumentDeriveStruct::Named { fields } => {
                    let mut idents = Vec::with_capacity(fields.len());
                    let mut instruction_data = Vec::with_capacity(fields.len());
                    let mut signers = Vec::with_capacity(fields.len());
                    let mut writables = Vec::with_capacity(fields.len());
                    let mut owners_index = Vec::with_capacity(fields.len());
                    let mut owners = Vec::with_capacity(fields.len());
                    let mut verification = Vec::with_capacity(fields.len());
                    let mut types = Vec::with_capacity(fields.len());
                    for field in fields {
                        idents.push(field.0);
                        instruction_data.push(field.1.instruction_data);
                        signers.push(field.1.signer);
                        writables.push(field.1.writable);
                        owners_index.push(Vec::with_capacity(field.1.owner.len()));
                        owners.push(Vec::with_capacity(field.1.owner.len()));
                        for (index, owner) in field.1.owner {
                            owners_index.last_mut().unwrap().push(index);
                            owners.last_mut().unwrap().push(owner);
                        }
                        types.push(field.2);
                    }
                    let map_func = |val: Indexes| match val {
                        Indexes::All(span) => {
                            quote_spanned! { span.span() => #crate_name::All }
                        }
                        Indexes::NotAll(span) => {
                            quote_spanned! { span.span() => #crate_name::NotAll }
                        }
                        Indexes::Any(span) => {
                            quote_spanned! { span.span() => #crate_name::Any }
                        }
                        Indexes::NotAny(span) => {
                            quote_spanned! { span.span() => #crate_name::NotAny }
                        }
                        Indexes::Expr(expr) => quote! { #expr },
                    };
                    let signers = signers.into_iter().map(|val| val.into_iter().map(map_func));
                    let writables = writables
                        .into_iter()
                        .map(|val| val.into_iter().map(map_func));
                    let owners_index = owners_index
                        .into_iter()
                        .map(|val| val.into_iter().map(map_func));

                    for ((((ident, signer), writable), owner), owner_index) in idents
                        .clone()
                        .into_iter()
                        .zip(signers)
                        .zip(writables)
                        .zip(owners)
                        .zip(owners_index)
                    {
                        verification.push(quote! {
                            #(#crate_name::assert_is_signer(&#ident, #signer)?;)*
                            #(#crate_name::assert_is_writable(&#ident, #writable)?;)*
                            #(#crate_name::assert_is_owner(&#ident, #owner, #owner_index)?;)*
                        })
                    }

                    (
                        quote! {
                            #(let #idents = <#types as #crate_name::FromAccounts<_>>::from_accounts(program_id, infos__, #instruction_data)?;)*
                            #(#verification)*
                            Ok(Self{
                                #(#idents,)*
                            })
                        },
                        quote! {
                            #(<#types as #crate_name::AccountArgument>::write_back(self.#idents, program_id, system_program)?;)*
                        },
                        quote! {
                            #(<#types as #crate_name::AccountArgument>::add_keys(&self.#idents, &mut add__)?;)*
                        },
                    )
                }
                AccountArgumentDeriveStruct::Unnamed { fields } => {
                    let index = (0..fields.len()).map(Index::from);
                    let mut instruction_data = Vec::with_capacity(fields.len());
                    let mut signers = Vec::with_capacity(fields.len());
                    let mut writables = Vec::with_capacity(fields.len());
                    let mut owners_index = Vec::with_capacity(fields.len());
                    let mut owners = Vec::with_capacity(fields.len());
                    let mut verification = Vec::with_capacity(fields.len());
                    let mut types = Vec::with_capacity(fields.len());
                    for field in fields {
                        instruction_data.push(field.0.instruction_data);
                        signers.push(field.0.signer);
                        writables.push(field.0.writable);
                        owners_index.push(Vec::with_capacity(field.0.owner.len()));
                        owners.push(Vec::with_capacity(field.0.owner.len()));
                        for (index, owner) in field.0.owner {
                            owners_index.last_mut().unwrap().push(index);
                            owners.last_mut().unwrap().push(owner);
                        }
                        types.push(field.1);
                    }
                    let map_func = |val: Indexes| match val {
                        Indexes::All(span) => {
                            quote_spanned! { span.span() => #crate_name::All }
                        }
                        Indexes::NotAll(span) => {
                            quote_spanned! { span.span() => #crate_name::NotAll }
                        }
                        Indexes::Any(span) => {
                            quote_spanned! { span.span() => #crate_name::Any }
                        }
                        Indexes::NotAny(span) => {
                            quote_spanned! { span.span() => #crate_name::NotAny }
                        }
                        Indexes::Expr(expr) => quote! { #expr },
                    };
                    let signers = signers.into_iter().map(|val| val.into_iter().map(map_func));
                    let writables = writables
                        .into_iter()
                        .map(|val| val.into_iter().map(map_func));
                    let owners_index = owners_index
                        .into_iter()
                        .map(|val| val.into_iter().map(map_func));

                    for ((((index, signer), writable), owner), owner_index) in index
                        .clone()
                        .into_iter()
                        .zip(signers)
                        .zip(writables)
                        .zip(owners)
                        .zip(owners_index)
                    {
                        verification.push(quote! {
                            #(#crate_name::assert_is_signer(&out.#index, #signer)?;)*
                            #(#crate_name::assert_is_writable(&out.#index, #writable)?;)*
                            #(#crate_name::assert_is_owner(&out.#index, #owner, #owner_index)?;)*
                        })
                    }
                    let index_clone = index.clone();
                    (
                        quote! {
                            let out = Self(
                                #(<#types as #crate_name::FromAccounts<_>>::from_accounts(program_id, infos__, #instruction_data)?,)*
                            );
                            #(#verification)*
                            Ok(out)
                        },
                        quote! {
                            #(<#types as #crate_name::AccountArgument>::write_back(self.#index, program_id, system_program)?;)*
                        },
                        quote! {
                            #(<#types as #crate_name::AccountArgument>::add_keys(&self.#index_clone, &mut add__)?;)*
                        },
                    )
                }
                AccountArgumentDeriveStruct::Unit => (quote! { Ok(Self) }, quote! {}, quote! {}),
            }
        };

        quote! {
            #[automatically_derived]
            impl #impl_generics AccountArgument for #ident #ty_generics #where_clause{
                fn write_back(
                    self,
                    program_id: #crate_name::solana_program::pubkey::Pubkey,
                    system_program: Option<&#crate_name::SystemProgram>,
                ) -> #crate_name::GeneratorResult<()>{
                    #write_back
                    Ok(())
                }

                fn add_keys(
                    &self,
                    mut add__: impl ::core::ops::FnMut(#crate_name::solana_program::pubkey::Pubkey) -> #crate_name::GeneratorResult<()>
                ) -> #crate_name::GeneratorResult<()>{
                    #keys
                    Ok(())
                }
            }

            #[automatically_derived]
            impl #impl_generics #crate_name::FromAccounts<#instruction_arg> for #ident #where_clause{
                fn from_accounts(
                    program_id: #crate_name::solana_program::pubkey::Pubkey,
                    infos__: &mut impl #crate_name::AccountInfoIterator,
                    arg__: #instruction_arg,
                ) -> #crate_name::GeneratorResult<Self>{
                    #instruction_naming
                    #fields_and_creation
                }
            }
        }
    }
}

#[derive(Clone)]
enum AccountArgumentDeriveType {
    // Enum(AccountArgumentDeriveEnum),
    Struct(AccountArgumentDeriveStruct),
}
impl AccountArgumentDeriveType {
    fn from_data(data: Data, ident: &Ident) -> syn::Result<Self> {
        match data {
            Data::Struct(data_struct) => Ok(Self::Struct(data_struct.fields.try_into()?)),
            Data::Enum(_data_enum) => {
                abort_call_site!("Cannot derive `AccountArgument` for enum {}", ident)
                // Ok(Self::Enum(data_enum.try_into()?))
            }
            Data::Union(_) => {
                abort_call_site!("Cannot derive `AccountArgument` for union {}", ident)
            }
        }
    }
}

struct AccountArgumentDeriveEnum {
    _variants: Vec<(
        Ident,
        AccountArgumentEnumAttribute,
        AccountArgumentDeriveStruct,
        Option<Expr>,
    )>,
}
impl TryFrom<DataEnum> for AccountArgumentDeriveEnum {
    type Error = syn::Error;

    fn try_from(value: DataEnum) -> Result<Self, Self::Error> {
        let mut variants = Vec::with_capacity(value.variants.len());
        for variant in value.variants {
            let attribute = variant.attrs.try_into()?;

            variants.push((
                variant.ident,
                attribute,
                variant.fields.try_into()?,
                variant.discriminant.map(|(_, discriminant)| discriminant),
            ))
        }
        Ok(Self {
            _variants: variants,
        })
    }
}

#[derive(Clone)]
enum AccountArgumentDeriveStruct {
    Named {
        fields: Vec<(Ident, Vec<AccountArgumentFieldAttribute>, Type)>,
    },
    Unnamed {
        fields: Vec<(Vec<AccountArgumentFieldAttribute>, Type)>,
    },
    Unit,
}
impl AccountArgumentDeriveStruct {
    fn from_fields(
        value: Fields,
        account_argument_field_attr_ident: &Ident,
    ) -> Result<Self, syn::Error> {
        match value {
            Fields::Named(named) => Ok(Self::Named {
                fields: named
                    .named
                    .into_iter()
                    .map(|field| {
                        Ok((
                            field.ident.unwrap(),
                            parse_attribute_list(
                                account_argument_field_attr_ident,
                                field.attrs.iter(),
                            )
                            .collect(),
                            field.ty,
                        ))
                    })
                    .collect::<Result<_, Self::Error>>()?,
            }),
            Fields::Unnamed(unnamed) => Ok(Self::Unnamed {
                fields: unnamed
                    .unnamed
                    .into_iter()
                    .map(|field| {
                        Ok((
                            parse_attribute_list(
                                account_argument_field_attr_ident,
                                field.attrs.iter(),
                            )
                            .collect(),
                            field.ty,
                        ))
                    })
                    .collect::<Result<_, Self::Error>>()?,
            }),
            Fields::Unit => Ok(Self::Unit),
        }
    }
}

struct AccountArgumentEnumAttribute {}
impl TryFrom<Vec<Attribute>> for AccountArgumentEnumAttribute {
    type Error = syn::Error;

    fn try_from(_value: Vec<Attribute>) -> Result<Self, Self::Error> {
        todo!()
    }
}

#[derive(ArgumentList, Clone)]
struct AccountArgumentFieldAttribute {
    id: Ident,
    from_data: Expr,
    #[argument(custom)]
    signer: Vec<Indexes>,
    #[argument(custom)]
    writable: Vec<Indexes>,
    #[argument(custom)]
    owner: Vec<IndexesValue<Expr>>,
}
impl AccountArgumentFieldAttribute {
    const IDENT: &'static str = "account_argument";
}

pub struct IndexesValue<T> {
    indexes: Indexes,
    equals: Token![=],
    value: T,
}
impl<T> Parse for IndexesValue<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            indexes: input.parse()?,
            equals: input.parse()?,
            value: input.parse()?,
        })
    }
}

#[derive(Clone)]
pub enum Indexes {
    All(Ident),
    NotAll(Ident),
    Any(Ident),
    NotAny(Ident),
    Expr(Box<Expr>),
}
impl Indexes {
    pub const ALL_IDENT: &'static str = "all";
    pub const NOT_ALL_IDENT: &'static str = "not_all";
    pub const ANY_IDENT: &'static str = "any";
    pub const NOT_ANY_IDENT: &'static str = "not_any";
}
impl Parse for Indexes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Paren) {
            let lookahead = input.lookahead1();
            if lookahead.peek(Ident) {
                let fork = input.fork();
                let ident: Ident = fork.parse()?;
                if ident == Self::ALL_IDENT {
                    Ok(Self::All(input.parse()?))
                } else if ident == Self::NOT_ALL_IDENT {
                    Ok(Self::NotAll(input.parse()?))
                } else if ident == Self::ANY_IDENT {
                    Ok(Self::Any(input.parse()?))
                } else if ident == Self::NOT_ANY_IDENT {
                    Ok(Self::NotAny(input.parse()?))
                } else {
                    Ok(Self::Expr(Box::new(input.parse()?)))
                }
            } else {
                Ok(Self::Expr(Box::new(input.parse()?)))
            }
        } else {
            Ok(Self::All(Ident::new(Self::ALL_IDENT, Span::call_site())))
        }
    }
}
