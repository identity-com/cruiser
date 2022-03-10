use std::collections::{HashMap, HashSet};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parenthesized, token, Attribute, Data, DataEnum, DeriveInput, Expr, Field, Fields, Generics,
    Ident, Index, Token, Type,
};

use easy_proc::{find_attr, parse_attribute_list, ArgumentList};

use crate::get_crate_name;
use crate::log_level::LogLevel;

#[derive(ArgumentList)]
pub struct AccountArgumentAttribute {
    // TODO: Use this with enum derivation
    #[allow(dead_code)]
    #[argument(attr_ident)]
    attr_ident: Ident,
    // TODO: Use this with enum derivation
    #[allow(dead_code)]
    #[argument(default = syn::parse_str("u64").unwrap())]
    enum_discriminant_type: Type,
    #[argument(presence)]
    no_from: bool,
    #[argument(presence)]
    no_validate: bool,
}
impl AccountArgumentAttribute {
    const IDENT: &'static str = "account_argument";
}
impl Default for AccountArgumentAttribute {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("__does_not_exist__", Span::call_site()),
            enum_discriminant_type: syn::parse_str("u64").unwrap(),
            no_from: false,
            no_validate: false,
        }
    }
}

#[derive(ArgumentList)]
pub struct FromAttribute {
    #[argument(attr_ident)]
    attr_ident: Ident,
    id: Option<Ident>,
    data: NamedTupple,
    // TODO: Use this for enum derivation
    #[allow(dead_code)]
    enum_discriminant: Option<Expr>,
    //TODO: Add logging
    #[allow(dead_code)]
    #[argument(default)]
    log_level: LogLevel,
}
impl FromAttribute {
    const IDENT: &'static str = "from";

    fn to_type(&self, accessor: &TokenStream) -> Vec<(TokenStream, Vec<TokenStream>)> {
        self.data.to_type(accessor)
    }
}
impl IdAttr for FromAttribute {
    fn id(&self) -> Option<&Ident> {
        self.id.as_ref()
    }

    fn attr_ident(&self) -> &Ident {
        &self.attr_ident
    }
}
impl Default for FromAttribute {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("__does_not_exist__", Span::call_site()),
            id: None,
            data: NamedTupple::default(),
            enum_discriminant: None,
            log_level: LogLevel::default(),
        }
    }
}

#[derive(ArgumentList)]
pub struct ValidateAttribute {
    #[argument(attr_ident)]
    attr_ident: Ident,
    id: Option<Ident>,
    data: NamedTupple,
    // TODO: add logging
    #[allow(dead_code)]
    #[argument(default)]
    log_level: LogLevel,
}
impl ValidateAttribute {
    const IDENT: &'static str = "validate";

    fn to_type(&self, accessor: &TokenStream) -> Vec<(TokenStream, Vec<TokenStream>)> {
        self.data.to_type(accessor)
    }
}
impl IdAttr for ValidateAttribute {
    fn id(&self) -> Option<&Ident> {
        self.id.as_ref()
    }

    fn attr_ident(&self) -> &Ident {
        &self.attr_ident
    }
}
impl Default for ValidateAttribute {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("__does_not_exist__", Span::call_site()),
            id: None,
            data: NamedTupple::default(),
            log_level: LogLevel::default(),
        }
    }
}

#[derive(ArgumentList, Debug)]
struct ArgumentFromFieldAttribute {
    #[argument(attr_ident)]
    attr_ident: Ident,
    id: Option<Ident>,
    data: Option<Expr>,
}
impl ArgumentFromFieldAttribute {
    const IDENT: &'static str = "from";
}
impl IdAttr for ArgumentFromFieldAttribute {
    fn id(&self) -> Option<&Ident> {
        self.id.as_ref()
    }

    fn attr_ident(&self) -> &Ident {
        &self.attr_ident
    }
}
impl Default for ArgumentFromFieldAttribute {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("__invalid_identifier__", Span::call_site()),
            id: None,
            data: None,
        }
    }
}

#[derive(ArgumentList, Debug, Clone)]
struct ArgumentValidateFieldAttribute {
    #[argument(attr_ident)]
    attr_ident: Ident,
    id: Option<Ident>,
    data: Option<Expr>,
    #[argument(custom)]
    signer: Vec<Indexes>,
    #[argument(custom)]
    writable: Vec<Indexes>,
    #[argument(custom)]
    owner: Vec<IndexesValue<Expr, UnitDefault>>,
    #[argument(custom)]
    key: Option<IndexesValue<Expr, UnitDefault>>,
}
impl ArgumentValidateFieldAttribute {
    const IDENT: &'static str = "validate";
}
impl IdAttr for ArgumentValidateFieldAttribute {
    fn id(&self) -> Option<&Ident> {
        self.id.as_ref()
    }

    fn attr_ident(&self) -> &Ident {
        &self.attr_ident
    }
}
impl Default for ArgumentValidateFieldAttribute {
    fn default() -> Self {
        Self {
            attr_ident: Ident::new("__invalid_identifier__", Span::call_site()),
            id: None,
            data: None,
            signer: Vec::new(),
            writable: Vec::new(),
            owner: Vec::new(),
            key: None,
        }
    }
}

trait IdAttr: ArgumentList {
    fn id(&self) -> Option<&Ident>;
    fn attr_ident(&self) -> &Ident;
    fn read_all<'a>(
        ident: &'a Ident,
        attrs: impl IntoIterator<Item = &'a Attribute> + 'a,
    ) -> HashMap<String, Self>
    where
        Self: 'a,
    {
        let mut out = HashMap::new();
        for attr in parse_attribute_list::<Self, _>(ident, attrs) {
            if let Some(attr) =
                out.insert(attr.id().map(Ident::to_string).unwrap_or_default(), attr)
            {
                match attr.id() {
                    Some(id) => abort!(id, "Duplicate id `{}`", id),
                    None => abort!(attr.attr_ident(), "Multiple blank id `from`s"),
                }
            }
        }
        out
    }
}

#[derive(Default)]
pub struct NamedTupple {
    list: Punctuated<(Ident, Token![:], Type), Token![,]>,
}
impl NamedTupple {
    fn to_type(&self, accessor: &TokenStream) -> Vec<(TokenStream, Vec<TokenStream>)> {
        match self.list.len() {
            0 => vec![(quote! { () }, vec![])],
            1 => {
                let item = &self.list[0];
                let ident = &item.0;
                let ty = &item.2;
                vec![
                    (
                        ty.into_token_stream(),
                        vec![quote! { let #ident = #accessor; }],
                    ),
                    (
                        quote! { (#ty,) },
                        vec![quote! { let #ident = #accessor.0; }],
                    ),
                ]
            }
            x => {
                let mut types = Vec::with_capacity(x);
                let accessors = self
                    .list
                    .iter()
                    .enumerate()
                    .map(|(index, (ident, _, ty))| {
                        types.push(ty);
                        let index = Index::from(index);
                        quote! { let #ident = #accessor.#index; }
                    })
                    .collect();
                vec![(quote! { (#(#types,)*) }, accessors)]
            }
        }
    }
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

pub struct AccountArgumentDerive {
    ident: Ident,
    generics: Generics,
    derive_type: AccountArgumentDeriveType,
    // TODO: Use with enum derivation
    #[allow(dead_code)]
    account_argument_attribute: AccountArgumentAttribute,
    from_attributes: HashMap<String, FromAttribute>,
    validate_attributes: HashMap<String, ValidateAttribute>,
}
impl Parse for AccountArgumentDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let from_attribute_ident = format_ident!("{}", FromAttribute::IDENT);
        let validate_attribute_ident = format_ident!("{}", ValidateAttribute::IDENT);
        let argument_from_field_attr_ident = format_ident!("{}", ArgumentFromFieldAttribute::IDENT);
        let argument_validate_field_attr_ident =
            format_ident!("{}", ArgumentValidateFieldAttribute::IDENT);
        let derive_input: DeriveInput = input.parse()?;

        let account_argument_attribute = find_attr(
            derive_input.attrs.iter(),
            &format_ident!("{}", AccountArgumentAttribute::IDENT),
        )
        .map(AccountArgumentAttribute::parse_arguments)
        .unwrap_or_default();

        let mut from_attributes =
            FromAttribute::read_all(&from_attribute_ident, derive_input.attrs.iter());
        from_attributes.entry(String::default()).or_default();
        let mut validate_attributes =
            ValidateAttribute::read_all(&validate_attribute_ident, derive_input.attrs.iter());
        validate_attributes.entry(String::default()).or_default();

        let derive_type = AccountArgumentDeriveType::from_data(
            derive_input.data,
            &derive_input.ident,
            &argument_from_field_attr_ident,
            &argument_validate_field_attr_ident,
            from_attributes.keys().cloned().collect(),
            validate_attributes.keys().cloned().collect(),
        )?;

        Ok(Self {
            ident: derive_input.ident,
            generics: derive_input.generics,
            derive_type,
            account_argument_attribute,
            from_attributes,
            validate_attributes,
        })
    }
}
impl AccountArgumentDerive {
    pub fn into_token_stream(self) -> TokenStream {
        let account_argument = self.account_argument();

        let from_accounts = if self.account_argument_attribute.no_from {
            TokenStream::new()
        } else {
            let from_accounts = self.from_attributes.into_iter().map(|(id, attr)| {
                self.derive_type
                    .from_accounts(&self.ident, &self.generics, id, attr)
            });
            quote! { #(#from_accounts)* }
        };

        let validate_argument = if self.account_argument_attribute.no_validate {
            TokenStream::new()
        } else {
            let validate_argument = self.validate_attributes.into_iter().map(|(id, attr)| {
                self.derive_type
                    .validate_argument(&self.ident, &self.generics, id, attr)
            });
            quote! { #(#validate_argument)* }
        };

        quote! {
            #account_argument
            #from_accounts
            #validate_argument
        }
    }

    fn account_argument(&self) -> TokenStream {
        let crate_name = get_crate_name();
        let ident = &self.ident;
        let (impl_gen, ty_gen, where_clause) = self.generics.split_for_impl();
        let write_back = self.derive_type.write_back();
        let add_keys = self.derive_type.add_keys();

        quote! {
            #[automatically_derived]
            impl #impl_gen #crate_name::AccountArgument for #ident #ty_gen #where_clause {
                fn write_back(
                    self,
                    program_id: &'static #crate_name::Pubkey,
                ) -> #crate_name::GeneratorResult<()>{
                    #write_back
                    Ok(())
                }

                fn add_keys(
                    &self,
                    mut add__: impl ::core::ops::FnMut(&'static #crate_name::solana_program::pubkey::Pubkey) -> #crate_name::GeneratorResult<()>
                ) -> #crate_name::GeneratorResult<()>{
                    #add_keys
                    Ok(())
                }
            }
        }
    }
}

enum AccountArgumentDeriveType {
    Enum(AccountArgumentDeriveEnum),
    Struct(AccountArgumentDeriveStruct),
}
impl AccountArgumentDeriveType {
    fn from_data(
        data: Data,
        ident: &Ident,
        argument_from_field_attr_ident: &Ident,
        argument_validate_field_attr_ident: &Ident,
        from_ids: HashSet<String>,
        validate_ids: HashSet<String>,
    ) -> syn::Result<Self> {
        match data {
            Data::Struct(data_struct) => {
                Ok(Self::Struct(AccountArgumentDeriveStruct::from_fields(
                    data_struct.fields,
                    argument_from_field_attr_ident,
                    argument_validate_field_attr_ident,
                    from_ids,
                    validate_ids,
                )?))
            }
            Data::Enum(data_enum) => Ok(Self::Enum(AccountArgumentDeriveEnum::from_enum(
                data_enum,
                argument_from_field_attr_ident,
                argument_validate_field_attr_ident,
                from_ids,
                validate_ids,
            )?)),
            Data::Union(union) => {
                abort!(
                    union.union_token.span.join(ident.span()).unwrap(),
                    "Cannot derive `AccountArgument` for union {}",
                    ident
                )
            }
        }
    }

    fn write_back(&self) -> TokenStream {
        match self {
            AccountArgumentDeriveType::Enum(data) => data.write_back(),
            AccountArgumentDeriveType::Struct(data) => data.write_back(&quote! { self. }),
        }
    }

    fn add_keys(&self) -> TokenStream {
        match self {
            AccountArgumentDeriveType::Enum(data) => data.add_keys(),
            AccountArgumentDeriveType::Struct(data) => data.add_keys(&quote! { self. }),
        }
    }

    //noinspection RsSelfConvention
    fn from_accounts(
        &self,
        ident: &Ident,
        generics: &Generics,
        id: String,
        attr: FromAttribute,
    ) -> TokenStream {
        let crate_name = get_crate_name();
        let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
        let ty_accessors = attr.to_type(&quote! { __arg });
        let program_id = quote! { program_id };
        let infos = quote! { __infos };
        let mut out = Vec::with_capacity(ty_accessors.len());
        for (ty, accessors) in ty_accessors {
            let inner = match self {
                AccountArgumentDeriveType::Enum(_) => todo!(),
                AccountArgumentDeriveType::Struct(data) => {
                    data.from_accounts(&id, &program_id, &infos)
                }
            };
            out.push(quote! {
                #[automatically_derived]
                impl #impl_gen #crate_name::FromAccounts<#ty> for #ident #ty_gen #where_clause{
                    fn from_accounts(
                        program_id: &'static #crate_name::Pubkey,
                        __infos: &mut impl #crate_name::AccountInfoIterator,
                        __arg: #ty,
                    ) -> #crate_name::GeneratorResult<Self>{
                        #(#accessors)*
                        #inner
                    }

                    #[must_use]
                    fn accounts_usage_hint(_arg: &#ty) -> (usize, ::std::option::Option<usize>){
                        (0, ::std::option::Option::None)
                    }
                }
            });
        }
        quote! {
            #(#out)*
        }
    }

    fn validate_argument(
        &self,
        ident: &Ident,
        generics: &Generics,
        id: String,
        attr: ValidateAttribute,
    ) -> TokenStream {
        let crate_name = get_crate_name();
        let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
        let ty_accessors = attr.to_type(&quote! { __arg });
        let program_id = quote! { program_id };
        let mut out = Vec::with_capacity(ty_accessors.len());
        for (ty, accessors) in ty_accessors {
            let inner = match self {
                AccountArgumentDeriveType::Enum(_) => todo!(),
                AccountArgumentDeriveType::Struct(data) => {
                    data.validate_argument(&id, &program_id, &quote! { self. })
                }
            };
            out.push(quote! {
                #[automatically_derived]
                impl #impl_gen #crate_name::ValidateArgument<#ty> for #ident #ty_gen #where_clause{
                    fn validate(&mut self, program_id: &'static #crate_name::Pubkey, __arg: #ty) -> #crate_name::GeneratorResult<()>{
                        #(#accessors)*
                        #inner
                        ::std::result::Result::Ok(())
                    }
                }
            });
        }
        quote! {
            #(#out)*
        }
    }
}

#[derive(Debug)]
struct AccountArgumentDeriveEnum(Vec<AccountArgumentEnumVariant>);
impl AccountArgumentDeriveEnum {
    fn from_enum(
        value: DataEnum,
        argument_from_field_attr_ident: &Ident,
        argument_validate_field_attr_ident: &Ident,
        from_ids: HashSet<String>,
        validate_ids: HashSet<String>,
    ) -> syn::Result<Self> {
        let mut variants = Vec::with_capacity(value.variants.len());
        for variant in value.variants {
            variants.push(AccountArgumentEnumVariant {
                ident: variant.ident,
                data: AccountArgumentDeriveStruct::from_fields(
                    variant.fields,
                    argument_from_field_attr_ident,
                    argument_validate_field_attr_ident,
                    from_ids.clone(),
                    validate_ids.clone(),
                )?,
                discriminant: variant.discriminant.map(|(_, discriminant)| discriminant),
            })
        }
        Ok(Self(variants))
    }

    fn write_back(&self) -> TokenStream {
        let write_back = self.0.iter().map(AccountArgumentEnumVariant::write_back);
        quote! {
            match self {#(
                #write_back
            )*}
        }
    }

    fn add_keys(&self) -> TokenStream {
        let add_keys = self.0.iter().map(AccountArgumentEnumVariant::add_keys);
        quote! {
            match self {#(
                #add_keys
            )*}
        }
    }
}

#[derive(Debug)]
struct AccountArgumentEnumVariant {
    ident: Ident,
    data: AccountArgumentDeriveStruct,
    // TODO: Use this with enum derivation
    #[allow(dead_code)]
    discriminant: Option<Expr>,
}
impl AccountArgumentEnumVariant {
    fn do_fields(
        &self,
        on_named: impl FnOnce(&[NamedField]) -> TokenStream,
        on_unnamed: impl FnOnce(&[UnnamedField]) -> TokenStream,
        on_unit: impl FnOnce() -> TokenStream,
    ) -> TokenStream {
        let ident = &self.ident;
        let self_data = match &self.data {
            AccountArgumentDeriveStruct::Named(fields) => {
                let field_names = fields.iter().map(|field| &field.ident);
                let field_construction = quote! { {#(#field_names,)*} };
                let named_action = on_named(fields);
                quote! { #field_construction => { #named_action } }
            }
            AccountArgumentDeriveStruct::Unnamed(fields) => {
                let field_names: Vec<_> = (0..fields.len())
                    .map(|index| format_ident!("val{}", index))
                    .collect();
                let field_construction = quote! { (#(#field_names,)*) };
                let unnamed_action = on_unnamed(fields);
                quote! { #field_construction => { #unnamed_action } }
            }
            AccountArgumentDeriveStruct::Unit => {
                let unit_action = on_unit();
                quote! { => { #unit_action } }
            }
        };
        quote! {
            Self::#ident #self_data
        }
    }

    fn write_back(&self) -> TokenStream {
        self.do_fields(
            |fields| {
                let write_back = fields
                    .iter()
                    .map(|field| field.write_back(&TokenStream::new()));
                quote! { #(#write_back)* }
            },
            |fields| {
                let field_names: Vec<_> = (0..fields.len())
                    .map(|index| format_ident!("val{}", index))
                    .collect();
                let write_back = fields
                    .iter()
                    .zip(field_names.iter())
                    .map(|(field, ident)| field.write_back(&ident.into_token_stream()));
                quote! { #(#write_back)* }
            },
            TokenStream::new,
        )
    }

    fn add_keys(&self) -> TokenStream {
        self.do_fields(
            |fields| {
                let add_keys = fields
                    .iter()
                    .map(|field| field.add_keys(&TokenStream::new()));
                quote! { #(#add_keys)* }
            },
            |fields| {
                let field_names: Vec<_> = (0..fields.len())
                    .map(|index| format_ident!("val{}", index))
                    .collect();
                let add_keys = fields
                    .iter()
                    .zip(field_names.iter())
                    .map(|(field, ident)| field.add_keys(&ident.into_token_stream()));
                quote! { #(#add_keys)* }
            },
            TokenStream::new,
        )
    }
}

#[derive(Debug)]
enum AccountArgumentDeriveStruct {
    Named(Vec<NamedField>),
    Unnamed(Vec<UnnamedField>),
    Unit,
}
impl AccountArgumentDeriveStruct {
    fn from_fields(
        value: Fields,
        argument_from_field_attr_ident: &Ident,
        argument_validate_field_attr_ident: &Ident,
        from_ids: HashSet<String>,
        validate_ids: HashSet<String>,
    ) -> Result<Self, syn::Error> {
        match value {
            Fields::Named(named) => Ok(Self::Named(
                Self::from_named(
                    named.named.into_iter(),
                    argument_from_field_attr_ident,
                    argument_validate_field_attr_ident,
                    from_ids,
                    validate_ids,
                )
                .collect(),
            )),
            Fields::Unnamed(unnamed) => Ok(Self::Unnamed(
                Self::from_unnamed(
                    unnamed.unnamed.into_iter(),
                    argument_from_field_attr_ident,
                    argument_validate_field_attr_ident,
                    from_ids,
                    validate_ids,
                )
                .collect(),
            )),
            Fields::Unit => Ok(Self::Unit),
        }
    }

    fn from_named<'a>(
        value: impl Iterator<Item = Field> + Clone + 'a,
        argument_from_field_attr_ident: &'a Ident,
        argument_validate_field_attr_ident: &'a Ident,
        from_ids: HashSet<String>,
        validate_ids: HashSet<String>,
    ) -> impl Iterator<Item = NamedField> + 'a {
        Self::from_unnamed(
            value.clone(),
            argument_from_field_attr_ident,
            argument_validate_field_attr_ident,
            from_ids,
            validate_ids,
        )
        .zip(value)
        .map(|(unnamed, field)| NamedField {
            ident: field.ident.unwrap(),
            field: unnamed,
        })
    }

    fn from_unnamed<'a>(
        value: impl Iterator<Item = Field> + 'a,
        argument_from_field_attr_ident: &'a Ident,
        argument_validate_field_attr_ident: &'a Ident,
        from_ids: HashSet<String>,
        validate_ids: HashSet<String>,
    ) -> impl Iterator<Item = UnnamedField> + 'a {
        value.map(move |field| {
            let from_attrs = ArgumentFromFieldAttribute::read_all(
                argument_from_field_attr_ident,
                field.attrs.iter(),
            );
            let validate_attrs = ArgumentValidateFieldAttribute::read_all(
                argument_validate_field_attr_ident,
                field.attrs.iter(),
            );

            for (key, value) in from_attrs.iter() {
                if !from_ids.contains(key) {
                    match &value.id {
                        Some(id) => abort!(id, "Unknown id `{}`", id),
                        None => unreachable!(),
                    }
                }
            }
            for (key, value) in validate_attrs.iter() {
                if !validate_ids.contains(key) {
                    match &value.id {
                        Some(id) => abort!(id, "Unknown id `{}`", id),
                        None => unreachable!(),
                    }
                }
            }

            UnnamedField {
                from_attrs,
                validate_attrs,
                ty: field.ty,
            }
        })
    }

    fn write_back(&self, self_access: &TokenStream) -> TokenStream {
        match self {
            AccountArgumentDeriveStruct::Named(named) => Self::write_back_named(named, self_access),
            AccountArgumentDeriveStruct::Unnamed(unnamed) => {
                Self::write_back_unnamed(unnamed, self_access)
            }
            AccountArgumentDeriveStruct::Unit => TokenStream::new(),
        }
    }

    fn write_back_named(named: &[NamedField], self_access: &TokenStream) -> TokenStream {
        let write_back = named.iter().map(|field| field.write_back(self_access));

        quote! { #(#write_back)* }
    }

    fn write_back_unnamed(unnamed: &[UnnamedField], self_access: &TokenStream) -> TokenStream {
        let write_back = unnamed.iter().enumerate().map(|(index, field)| {
            field.write_back({
                let index = Index::from(index);
                &quote! { #self_access #index }
            })
        });

        quote! { #(#write_back)* }
    }

    fn add_keys(&self, self_access: &TokenStream) -> TokenStream {
        match self {
            AccountArgumentDeriveStruct::Named(named) => Self::add_keys_named(named, self_access),
            AccountArgumentDeriveStruct::Unnamed(unnamed) => {
                Self::add_keys_unnamed(unnamed, self_access)
            }
            AccountArgumentDeriveStruct::Unit => TokenStream::new(),
        }
    }

    fn add_keys_named(named: &[NamedField], self_access: &TokenStream) -> TokenStream {
        let add_keys = named.iter().map(|field| field.add_keys(self_access));

        quote! { #(#add_keys)* }
    }

    fn add_keys_unnamed(unnamed: &[UnnamedField], self_access: &TokenStream) -> TokenStream {
        let add_keys = unnamed.iter().enumerate().map(|(index, field)| {
            field.add_keys({
                let index = Index::from(index);
                &quote! { #self_access #index }
            })
        });

        quote! { #(#add_keys)* }
    }

    //noinspection RsSelfConvention
    fn from_accounts(
        &self,
        id: &str,
        program_id: &TokenStream,
        infos: &TokenStream,
    ) -> TokenStream {
        match self {
            AccountArgumentDeriveStruct::Named(named) => {
                Self::from_accounts_named(named, id, program_id, infos)
            }
            AccountArgumentDeriveStruct::Unnamed(unnamed) => {
                Self::from_accounts_unnamed(unnamed, id, program_id, infos)
            }
            AccountArgumentDeriveStruct::Unit => quote! { ::std::result::Result::Ok(Self) },
        }
    }

    //noinspection RsSelfConvention
    fn from_accounts_named(
        named: &[NamedField],
        id: &str,
        program_id: &TokenStream,
        infos: &TokenStream,
    ) -> TokenStream {
        let tokens = named
            .iter()
            .map(|field| field.from_accounts(id, program_id, infos));
        quote! {
            ::std::result::Result::Ok(Self{
                #(#tokens,)*
            })
        }
    }

    //noinspection RsSelfConvention
    fn from_accounts_unnamed(
        unnamed: &[UnnamedField],
        id: &str,
        program_id: &TokenStream,
        infos: &TokenStream,
    ) -> TokenStream {
        let tokens = unnamed
            .iter()
            .map(|field| field.from_accounts(id, program_id, infos));
        quote! {
            ::std::result::Result::Ok(Self(#(#tokens,)*))
        }
    }

    fn validate_argument(
        &self,
        id: &str,
        program_id: &TokenStream,
        accessor: &TokenStream,
    ) -> TokenStream {
        match self {
            AccountArgumentDeriveStruct::Named(named) => {
                Self::validate_argument_named(named, id, program_id, accessor)
            }
            AccountArgumentDeriveStruct::Unnamed(unnamed) => {
                Self::validate_argument_unnamed(unnamed, id, program_id, accessor)
            }
            AccountArgumentDeriveStruct::Unit => TokenStream::new(),
        }
    }

    fn validate_argument_named(
        named: &[NamedField],
        id: &str,
        program_id: &TokenStream,
        accessor: &TokenStream,
    ) -> TokenStream {
        let tokens = named
            .iter()
            .map(|field| field.validate_argument(id, program_id, accessor));
        quote! {
            #(#tokens)*
        }
    }

    fn validate_argument_unnamed(
        unnamed: &[UnnamedField],
        id: &str,
        program_id: &TokenStream,
        accessor: &TokenStream,
    ) -> TokenStream {
        let tokens = unnamed.iter().enumerate().map(|(index, field)| {
            let index = Index::from(index);
            field.validate_argument(id, program_id, &quote! { #accessor #index })
        });
        quote! {
            #(#tokens)*
        }
    }
}

#[derive(Debug)]
struct NamedField {
    ident: Ident,
    field: UnnamedField,
}
impl NamedField {
    fn write_back(&self, self_access: &TokenStream) -> TokenStream {
        let ident = &self.ident;
        self.field.write_back(&quote! { #self_access #ident })
    }

    fn add_keys(&self, self_access: &TokenStream) -> TokenStream {
        let ident = &self.ident;
        self.field.add_keys(&quote! { #self_access #ident })
    }

    //noinspection RsSelfConvention
    fn from_accounts(
        &self,
        id: &str,
        program_id: &TokenStream,
        infos: &TokenStream,
    ) -> TokenStream {
        let ident = &self.ident;
        let expr = self.field.from_accounts(id, program_id, infos);
        quote! { #ident: #expr }
    }

    fn validate_argument(
        &self,
        id: &str,
        program_id: &TokenStream,
        accessor: &TokenStream,
    ) -> TokenStream {
        let ident = &self.ident;
        self.field
            .validate_argument(id, program_id, &quote! { #accessor #ident })
    }
}
impl Deref for NamedField {
    type Target = UnnamedField;

    fn deref(&self) -> &Self::Target {
        &self.field
    }
}
impl DerefMut for NamedField {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.field
    }
}

#[derive(Debug)]
struct UnnamedField {
    from_attrs: HashMap<String, ArgumentFromFieldAttribute>,
    validate_attrs: HashMap<String, ArgumentValidateFieldAttribute>,
    ty: Type,
}
impl UnnamedField {
    fn write_back(&self, accessor: &TokenStream) -> TokenStream {
        let crate_name = get_crate_name();
        let ty = &self.ty;
        quote! {
            <#ty as #crate_name::AccountArgument>::write_back(#accessor, program_id)?;
        }
    }

    fn add_keys(&self, accessor: &TokenStream) -> TokenStream {
        let crate_name = get_crate_name();
        let ty = &self.ty;
        quote! {
            <#ty as #crate_name::AccountArgument>::add_keys(&#accessor, &mut add__)?;
        }
    }

    //noinspection RsSelfConvention
    fn from_accounts(
        &self,
        id: &str,
        program_id: &TokenStream,
        infos: &TokenStream,
    ) -> TokenStream {
        let crate_name = get_crate_name();
        let expr = self
            .from_attrs
            .get(id)
            .and_then(|attr| attr.data.clone())
            .unwrap_or_else(|| syn::parse_str("()").unwrap());
        quote! { #crate_name::FromAccounts::from_accounts(#program_id, #infos, #expr)? }
    }

    fn validate_argument(
        &self,
        id: &str,
        program_id: &TokenStream,
        accessor: &TokenStream,
    ) -> TokenStream {
        let crate_name = get_crate_name();
        let attr = self.validate_attrs.get(id).cloned().unwrap_or_default();
        let validate = attr.data.unwrap_or_else(|| syn::parse_str("()").unwrap());
        let signer = attr.signer.into_iter().map(|signer| {
            let indexer = signer.to_tokens();
            quote! { #crate_name::assert_is_signer(&#accessor, #indexer)?; }
        });
        let writable = attr.writable.into_iter().map(|writable| {
            let indexer = writable.to_tokens();
            quote! { #crate_name::assert_is_writable(&#accessor, #indexer)?; }
        });
        let owner = attr.owner.into_iter().map(|owner| {
            let indexer = owner.indexes.to_tokens();
            let owner = owner.value;
            quote! { #crate_name::assert_is_owner(&#accessor, #owner, #indexer)?; }
        });
        let key = attr.key.into_iter().map(|key| {
            let indexer = key.indexes.to_tokens();
            let key = key.value;
            quote! { #crate_name::assert_is_key(&#accessor, #key, #indexer)?; }
        });

        quote! {
            #crate_name::ValidateArgument::validate(&mut #accessor, #program_id, #validate)?;
            #(#signer)*
            #(#writable)*
            #(#owner)*
            #(#key)*
        }
    }
}

pub trait DefaultIndex: Sized {
    fn default_index() -> Indexes<Self>;
}
#[derive(Debug, Clone)]
pub struct AllDefault;
impl DefaultIndex for AllDefault {
    fn default_index() -> Indexes<Self> {
        Indexes::All(kw::all::default())
    }
}
#[derive(Debug, Clone)]
pub struct UnitDefault;
impl DefaultIndex for UnitDefault {
    fn default_index() -> Indexes<Self> {
        Indexes::Expr(syn::parse_str("()").unwrap(), PhantomData)
    }
}

#[derive(Debug, Clone)]
pub struct IndexesValue<T, D: DefaultIndex = AllDefault> {
    indexes: Indexes<D>,
    value: T,
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

#[derive(Clone, Debug)]
pub enum Indexes<D: DefaultIndex = AllDefault> {
    All(kw::all),
    NotAll(kw::not_all),
    Any(kw::any),
    NotAny(kw::not_any),
    Expr(Box<Expr>, PhantomData<fn() -> D>),
}
impl<D: DefaultIndex> Indexes<D> {
    fn to_tokens(&self) -> TokenStream {
        let crate_name = get_crate_name();
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
