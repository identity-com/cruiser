use crate::get_crate_name;
use crate::log_level::LogLevel;
use easy_proc::{parse_attribute_list, ArgumentList};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use std::collections::HashSet;
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parenthesized, token, Attribute, Data, DataEnum, DeriveInput, Expr, Fields, Generics, Ident,
    Index, LitStr, Token, Type,
};

#[derive(ArgumentList)]
pub struct FromAttribute {
    #[argument(attr_ident)]
    attr_ident: Ident,
    id: Option<Ident>,
    data: NamedTupple,
    #[argument(default)]
    log_level: LogLevel,
}
impl FromAttribute {
    const IDENT: &'static str = "from";
}
impl Default for FromAttribute {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("__does_not_exist__", Span::call_site()),
            id: None,
            data: NamedTupple::default(),
            log_level: LogLevel::default(),
        }
    }
}

#[derive(ArgumentList)]
struct AccountArgumentFieldAttribute {
    #[argument(attr_ident)]
    attr_ident: Ident,
    id: Option<Ident>,
    from_data: Option<Expr>,
    #[argument(custom)]
    signer: Vec<Indexes>,
    #[argument(custom)]
    writable: Vec<Indexes>,
    #[argument(custom)]
    owner: Vec<IndexesValue<Expr>>,
    #[argument(custom)]
    key: Option<IndexesValue<Expr, UnitDefault>>,
}
impl AccountArgumentFieldAttribute {
    const IDENT: &'static str = "account_argument";
}
impl Default for AccountArgumentFieldAttribute {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("__invalid_identifier__", Span::call_site()),
            id: None,
            from_data: None,
            signer: Vec::new(),
            writable: Vec::new(),
            owner: Vec::new(),
            key: None,
        }
    }
}

pub struct NamedTupple {
    list: Punctuated<(Ident, Token![:], Type), Token![,]>,
}
impl Parse for NamedTupple {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        parenthesized!(content in input);
        let list = content
            .parse_terminated(|stream| Ok((stream.parse()?, stream.parse()?, stream.parse()?)))?;
        Ok(Self { list })
    }
}
impl Default for NamedTupple {
    fn default() -> Self {
        Self {
            list: Punctuated::new(),
        }
    }
}

pub struct AccountArgumentDerive {
    ident: Ident,
    generics: Generics,
    derive_type: AccountArgumentDeriveType,
    from_attributes: Vec<FromAttribute>,
    has_ids_for_from: bool,
}
impl Parse for AccountArgumentDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let from_attribute_ident = Ident::new(FromAttribute::IDENT, Span::call_site());
        let account_argument_field_attr_ident =
            Ident::new(AccountArgumentFieldAttribute::IDENT, Span::call_site());
        let derive_input: DeriveInput = input.parse()?;

        let from_attributes: Vec<FromAttribute> =
            parse_attribute_list(&from_attribute_ident, derive_input.attrs.iter()).collect();
        let derive_type = AccountArgumentDeriveType::from_data(
            derive_input.data,
            &derive_input.ident,
            &account_argument_field_attr_ident,
        )?;

        let mut attr_idents = HashSet::with_capacity(from_attributes.len());
        if from_attributes.len() == 1 {
            if let Some(id) = &from_attributes[0].id {
                attr_idents.insert(id.to_string());
            }
        } else {
            for from in &from_attributes {
                match &from.id {
                    Some(id) => {
                        if !attr_idents.insert(id.to_string()) {
                            abort!(id, "Multiple from implementations with same `id`");
                        }
                    }
                    None => abort!(
                        from.attr_ident,
                        "No `id` with multiple `from` implementations"
                    ),
                }
            }
        }
        let field_attrs: Vec<_> = match &derive_type {
            AccountArgumentDeriveType::Struct(AccountArgumentDeriveStruct::Named { fields }) => {
                fields.iter().flat_map(|(_, attrs, _)| attrs).collect()
            }
            AccountArgumentDeriveType::Struct(AccountArgumentDeriveStruct::Unnamed { fields }) => {
                fields.iter().flat_map(|(attrs, _)| attrs).collect()
            }
            AccountArgumentDeriveType::Struct(AccountArgumentDeriveStruct::Unit) => Vec::new(),
        };

        for attr in field_attrs {
            match &attr.id {
                Some(id) => {
                    if !attr_idents.contains(&id.to_string()) {
                        abort!(id, "Unknown `id` on field");
                    }
                }
                None => {
                    if !attr_idents.is_empty() {
                        abort!(
                            attr.attr_ident,
                            "No `id` field when id fields present inf `from`"
                        )
                    }
                }
            }
        }

        Ok(Self {
            ident: derive_input.ident,
            generics: derive_input.generics,
            derive_type,
            from_attributes,
            has_ids_for_from: !attr_idents.is_empty(),
        })
    }
}
impl AccountArgumentDerive {
    pub fn into_token_stream(mut self) -> TokenStream {
        let crate_name = get_crate_name();

        let ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let (field_accessors, field_init, field_attrs, field_types) = match self.derive_type {
            AccountArgumentDeriveType::Struct(AccountArgumentDeriveStruct::Unit) => (
                Vec::<TokenStream>::new(),
                Box::new(|_| TokenStream::new()) as Box<dyn Fn(Vec<TokenStream>) -> TokenStream>,
                Vec::<Vec<AccountArgumentFieldAttribute>>::new(),
                Vec::<Type>::new(),
            ),
            AccountArgumentDeriveType::Struct(AccountArgumentDeriveStruct::Unnamed { fields }) => {
                let accessors = (0..fields.len())
                    .map(Index::from)
                    .map(|index| {
                        quote! {
                            .#index
                        }
                    })
                    .collect();
                let init = Box::new(|streams: Vec<TokenStream>| {
                    quote! {
                        (#(#streams,)*)
                    }
                });
                let mut attrs = Vec::with_capacity(fields.len());
                let mut types = Vec::with_capacity(fields.len());
                for field in fields {
                    attrs.push(field.0);
                    types.push(field.1);
                }
                (
                    accessors,
                    init as Box<dyn Fn(Vec<TokenStream>) -> TokenStream>,
                    attrs,
                    types,
                )
            }
            AccountArgumentDeriveType::Struct(AccountArgumentDeriveStruct::Named { fields }) => {
                let mut idents = Vec::with_capacity(fields.len());
                let mut attrs = Vec::with_capacity(fields.len());
                let mut types = Vec::with_capacity(fields.len());
                for field in fields {
                    idents.push(field.0);
                    attrs.push(field.1);
                    types.push(field.2);
                }

                let accessors = idents.iter().map(|ident| quote! { .#ident }).collect();
                let init = Box::new(move |streams: Vec<TokenStream>| {
                    quote! {
                        {
                            #(#idents: #streams,)*
                        }
                    }
                });
                (
                    accessors,
                    init as Box<dyn Fn(Vec<TokenStream>) -> TokenStream>,
                    attrs,
                    types,
                )
            }
        };

        let from_impls = if self.has_ids_for_from {
            todo!()
        } else {
            let from_attribute = self.from_attributes.pop().unwrap_or_default();
            let field_attrs = field_attrs
                .into_iter()
                .map(|mut attrs| {
                    assert!(attrs.len() <= 1);
                    attrs.pop().unwrap_or_default()
                })
                .collect::<Vec<_>>();

            let logging = from_attribute.log_level.if_level(LogLevel::Debug, |_| {
                let message = LitStr::new(&format!("Reading `{}`", ident), Span::call_site());
                quote! {
                    #crate_name::msg!(#message);
                }
            });

            let mut from_idents = Vec::with_capacity(from_attribute.data.list.len());
            let mut from_types = Vec::with_capacity(from_attribute.data.list.len());
            for (ident, _, ty) in from_attribute.data.list {
                from_idents.push(ident);
                from_types.push(ty);
            }

            let mut impl_types = vec![quote! {
                (#(#from_types,)*)
            }];
            let mut impl_create = vec![{
                let indexes = (0..from_idents.len()).map(Index::from);
                quote! {
                    #(let #from_idents = arg__.#indexes;)*
                }
            }];
            if from_types.len() == 1 {
                let from_ident = &from_idents[0];
                let from_type = &from_types[0];
                impl_types.push(quote! {
                    #from_type
                });
                impl_create.push(quote! {
                    let #from_ident = arg__;
                });
            }

            // #(#crate_name::assert_is_signer(&#ident, #signer)?;)*
            // #(#crate_name::assert_is_writable(&#ident, #writable)?;)*
            // #(#crate_name::assert_is_owner(&#ident, #owner, #owner_index)?;)*

            let creation = field_init(field_types.iter().zip(field_attrs.iter()).map(|(ty, field_attr)|{
                let from_data = field_attr.from_data.as_ref().cloned().unwrap_or_else(||syn::parse_str("()").unwrap());
                quote!{
                    <#ty as #crate_name::FromAccounts<_>>::from_accounts(program_id, infos__, #from_data)?
                }
            }).collect());
            let size_hint: Vec<_> = field_types
                .iter()
                .zip(field_attrs.iter())
                .map(|(ty, field_attr)| {
                    let from_data = field_attr
                        .from_data
                        .as_ref()
                        .cloned()
                        .unwrap_or_else(|| syn::parse_str("()").unwrap());
                    quote! {
                        <#ty as #crate_name::FromAccounts<_>>::accounts_usage_hint(&#from_data)
                    }
                })
                .collect();
            let size_hint = quote! {
                #(#size_hint,)*
            };

            let verifications: Vec<_> = field_attrs
                .iter()
                .zip(field_accessors.iter())
                .map(|(field_attr, accessor)| {
                    let signers = field_attr
                        .signer
                        .iter()
                        .map(|signer| signer.to_tokens(&crate_name))
                        .collect::<Vec<_>>();
                    let writables = &field_attr
                        .writable
                        .iter()
                        .map(|writable| writable.to_tokens(&crate_name))
                        .collect::<Vec<_>>();
                    let owners = &field_attr
                        .owner
                        .iter()
                        .map(|owner| {
                            let index = owner.indexes.to_tokens(&crate_name);
                            let value = &owner.value;
                            quote! {
                                #crate_name::assert_is_owner(&accounts #accessor, #value, #index)?;
                            }
                        })
                        .collect::<Vec<_>>();
                    let key = match &field_attr.key {
                        None => quote! {},
                        Some(key) => {
                            let index = key.indexes.to_tokens(&crate_name);
                            let value = &key.value;
                            quote! {
                                #crate_name::assert_is_key(&accounts #accessor, #index, #value)?;
                            }
                        }
                    };

                    quote! {
                        #(#crate_name::assert_is_signer(&accounts #accessor, #signers)?;)*
                        #(#crate_name::assert_is_writable(&accounts #accessor, #writables)?;)*
                        #(#owners)*
                        #key
                    }
                })
                .collect();
            let verifications = quote! {
                #(#verifications)*
            };

            quote! {
                #(
                    #[automatically_derived]
                    impl #impl_generics #crate_name::FromAccounts<#impl_types> for #ident #ty_generics #where_clause{
                        fn from_accounts(
                            program_id: &'static #crate_name::solana_program::pubkey::Pubkey,
                            infos__: &mut impl #crate_name::AccountInfoIterator,
                            arg__: #impl_types,
                        ) -> #crate_name::GeneratorResult<Self>{
                            #logging
                            #impl_create
                            let accounts = Self #creation;
                            #verifications
                            Ok(accounts)
                        }

                        fn accounts_usage_hint(arg: &#impl_types) -> (usize, Option<usize>) {
                            #impl_create
                            #crate_name::util::sum_size_hints([
                                #size_hint
                            ].into_iter())
                        }
                    }
                )*
            }
        };

        quote! {
            #[automatically_derived]
            impl #impl_generics AccountArgument for #ident #ty_generics #where_clause{
                fn write_back(
                    self,
                    program_id: &'static #crate_name::solana_program::pubkey::Pubkey,
                    system_program: ::std::option::Option<&#crate_name::SystemProgram>,
                ) -> #crate_name::GeneratorResult<()>{
                    #(<#field_types as #crate_name::AccountArgument>::write_back(self #field_accessors, program_id, system_program)?;)*
                    Ok(())
                }

                fn add_keys(
                    &self,
                    mut add__: impl ::core::ops::FnMut(&'static #crate_name::solana_program::pubkey::Pubkey) -> #crate_name::GeneratorResult<()>
                ) -> #crate_name::GeneratorResult<()>{
                    #(<#field_types as #crate_name::AccountArgument>::add_keys(&self #field_accessors, &mut add__)?;)*
                    Ok(())
                }
            }

            #from_impls
        }
    }
}

enum AccountArgumentDeriveType {
    // Enum(AccountArgumentDeriveEnum),
    Struct(AccountArgumentDeriveStruct),
}
impl AccountArgumentDeriveType {
    fn from_data(
        data: Data,
        ident: &Ident,
        account_argument_field_attr_ident: &Ident,
    ) -> syn::Result<Self> {
        match data {
            Data::Struct(data_struct) => {
                Ok(Self::Struct(AccountArgumentDeriveStruct::from_fields(
                    data_struct.fields,
                    account_argument_field_attr_ident,
                )?))
            }
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

//TODO: Use this
#[allow(dead_code)]
struct AccountArgumentDeriveEnum {
    variants: Vec<(
        Ident,
        AccountArgumentEnumAttribute,
        AccountArgumentDeriveStruct,
        Option<Expr>,
    )>,
}
impl AccountArgumentDeriveEnum {
    //TODO: Use this
    #[allow(dead_code)]
    fn from_enum(value: DataEnum, account_argument_field_attr_ident: &Ident) -> syn::Result<Self> {
        let mut variants = Vec::with_capacity(value.variants.len());
        for variant in value.variants {
            let attribute = variant.attrs.try_into()?;

            variants.push((
                variant.ident,
                attribute,
                AccountArgumentDeriveStruct::from_fields(
                    variant.fields,
                    account_argument_field_attr_ident,
                )?,
                variant.discriminant.map(|(_, discriminant)| discriminant),
            ))
        }
        Ok(Self { variants })
    }
}

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
                    .collect::<syn::Result<_>>()?,
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
                    .collect::<syn::Result<_>>()?,
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

pub trait DefaultIndex: Sized {
    fn default_index() -> Indexes<Self>;
}
pub struct AllDefault;
impl DefaultIndex for AllDefault {
    fn default_index() -> Indexes<Self> {
        Indexes::All(kw::all::default())
    }
}
pub struct UnitDefault;
impl DefaultIndex for UnitDefault {
    fn default_index() -> Indexes<Self> {
        Indexes::Expr(syn::parse_str("()").unwrap(), PhantomData)
    }
}

pub struct IndexesValue<T, D: DefaultIndex = AllDefault> {
    indexes: Indexes<D>,
    value: T,
    phantom_d: PhantomData<fn() -> D>,
}
impl<T, D> Parse for IndexesValue<T, D>
where
    T: Parse,
    D: DefaultIndex,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let indexes = input.parse()?;
        input.parse::<Token![=]>()?;
        Ok(Self {
            indexes,
            value: input.parse()?,
            phantom_d: PhantomData,
        })
    }
}

mod kw {
    use syn::custom_keyword;
    custom_keyword!(all);
    custom_keyword!(not_all);
    custom_keyword!(any);
    custom_keyword!(not_any);
}

#[derive(Clone)]
pub enum Indexes<D: DefaultIndex = AllDefault> {
    All(kw::all),
    NotAll(kw::not_all),
    Any(kw::any),
    NotAny(kw::not_any),
    Expr(Box<Expr>, PhantomData<fn() -> D>),
}
impl<D: DefaultIndex> Indexes<D> {
    fn to_tokens(&self, crate_name: &TokenStream) -> TokenStream {
        match self {
            Indexes::All(_) => quote! { #crate_name::AllAny::from(#crate_name::All) },
            Indexes::NotAll(_) => quote! { #crate_name::AllAny::from(#crate_name::NotAll) },
            Indexes::Any(_) => quote! { #crate_name::AllAny::from(#crate_name::Any) },
            Indexes::NotAny(_) => quote! { #crate_name::AllAny::from(#crate_name::NotAny) },
            Indexes::Expr(expr, _) => quote! { #expr },
        }
    }
}
impl<D: DefaultIndex> Parse for Indexes<D> {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let lookahead = content.lookahead1();
            if lookahead.peek(kw::all) {
                Ok(Self::All(content.parse()?))
            } else if lookahead.peek(kw::not_all) {
                Ok(Self::NotAll(content.parse()?))
            } else if lookahead.peek(kw::any) {
                Ok(Self::Any(content.parse()?))
            } else if lookahead.peek(kw::not_any) {
                Ok(Self::NotAny(content.parse()?))
            } else {
                Ok(Self::Expr(Box::new(content.parse()?), PhantomData))
            }
        } else {
            Ok(D::default_index())
        }
    }
}
