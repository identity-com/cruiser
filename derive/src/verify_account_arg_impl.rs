use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    braced, bracketed, custom_keyword, token, Attribute, GenericParam, Generics, Ident, Token,
    Type, Visibility, WhereClause,
};

use crate::get_crate_name;

pub struct VerifyAccountArgs {
    vis: Visibility,
    mod_token: Token![mod],
    mod_ident: Ident,
    #[allow(dead_code)]
    lt: Token![<],
    account_info_gen: Ident,
    #[allow(dead_code)]
    gt: Token![>],
    brace: token::Brace,
    args: Punctuated<VerifyAccountArg, Token![;]>,
}
impl VerifyAccountArgs {
    pub fn into_token_stream(self) -> TokenStream {
        let crate_name = get_crate_name();

        let vis = self.vis;
        let mod_token = self.mod_token;
        let mod_ident = self.mod_ident;

        let sub_mods = self
            .args
            .into_iter()
            .map(|arg| arg.into_token_stream(&crate_name, &self.account_info_gen))
            .enumerate()
            .map(|(index, ts)| {
                let sub_name = format_ident!("sub_mod{}", index);
                quote! {
                    mod #sub_name {
                        use super::*;
                        #ts
                    }
                }
            })
            .collect::<Vec<_>>();

        quote! {
            #vis #mod_token #mod_ident {
                use super::*;
                #(#sub_mods)*
            }
        }
    }
}
impl Parse for VerifyAccountArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let vis = input.parse()?;
        let mod_token = input.parse()?;
        let mod_ident = input.parse()?;
        let lt = input.parse()?;
        let account_info_gen = input.parse()?;
        let gt = input.parse()?;
        let content;
        let brace = braced!(content in input);
        let args = content.parse_terminated(VerifyAccountArg::parse)?;
        Ok(Self {
            vis,
            mod_token,
            mod_ident,
            lt,
            account_info_gen,
            gt,
            brace,
            args,
        })
    }
}
impl ToTokens for VerifyAccountArgs {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.vis.to_tokens(tokens);
        self.mod_token.to_tokens(tokens);
        self.mod_ident.to_tokens(tokens);
        self.brace.surround(tokens, |tokens| {
            self.args.to_tokens(tokens);
        });
    }
}

mod kw {
    use super::custom_keyword;

    custom_keyword!(from);
    custom_keyword!(validate);
    custom_keyword!(multi);
    custom_keyword!(single);
    custom_keyword!(all_any);
    custom_keyword!(all_any_range);
}

pub struct VerifyAccountArg {
    type_generics: Generics,
    ty: Type,
    where_clause: Option<WhereClause>,
    brace: token::Brace,
    from: TypeList<kw::from>,
    validate: TypeList<kw::validate>,
    multi: TypeList<kw::multi>,
    single: TypeList<kw::single>,
}
impl VerifyAccountArg {
    pub fn into_token_stream(
        self,
        crate_name: &TokenStream,
        account_info_gen: &Ident,
    ) -> TokenStream {
        let mut generics = self.type_generics;
        if let Some(where_clause) = self.where_clause {
            generics
                .make_where_clause()
                .predicates
                .extend(where_clause.predicates.into_iter());
        }
        let ty = self.ty;
        let from = self.from.into_token_stream(
            "From",
            &ty,
            &generics,
            &quote! { #crate_name::account_argument::FromAccounts },
            account_info_gen,
        );
        let validate = self.validate.into_token_stream(
            "Validate",
            &ty,
            &generics,
            &quote! { #crate_name::account_argument::ValidateArgument },
            account_info_gen,
        );
        let multi = self.multi.into_token_stream(
            "Multi",
            &ty,
            &generics,
            &quote! { #crate_name::account_argument::MultiIndexable },
            account_info_gen,
        );
        let single = self.single.into_token_stream(
            "Single",
            &ty,
            &generics,
            &quote! { #crate_name::account_argument::SingleIndexable },
            account_info_gen,
        );

        let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
        quote! {
            #[automatically_derived]
            #[allow(clippy::type_repetition_in_bounds)]
            trait AccountArgumentTest #impl_gen: #crate_name::account_argument::AccountArgument<#account_info_gen> #where_clause {}
            #[automatically_derived]
            #[allow(clippy::type_repetition_in_bounds)]
            impl #impl_gen AccountArgumentTest #ty_gen for #ty #where_clause {}

            #from
            #validate
            #multi
            #single
        }
    }
}
impl Parse for VerifyAccountArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        // println!("input: {}", input);
        let type_generics = input.parse()?;
        // println!("type_generics: {}", input);
        let ty: Type = input.parse()?;
        let where_clause = input.parse()?;
        // println!("where_clause: {}", input);
        let content;
        let brace = braced!(content in input);
        let mut from: Option<TypeList<kw::from>> = None;
        let mut validate: Option<TypeList<kw::validate>> = None;
        let mut multi: Option<TypeList<kw::multi>> = None;
        let mut single: Option<TypeList<kw::single>> = None;
        while !content.is_empty() {
            let lookahead = content.lookahead1();
            if lookahead.peek(kw::from) {
                if let Some(from) = from.replace(content.parse()?) {
                    abort!(from, "Multiple `from` args")
                }
            } else if lookahead.peek(kw::validate) {
                if let Some(validate) = validate.replace(content.parse()?) {
                    abort!(validate, "Multiple `validate` args")
                }
            } else if lookahead.peek(kw::multi) {
                if let Some(multi) = multi.replace(content.parse()?) {
                    abort!(multi, "Multiple `multi` args")
                }
            } else if lookahead.peek(kw::single) {
                if let Some(single) = single.replace(content.parse()?) {
                    abort!(single, "Multiple `single` args")
                }
            } else {
                return Err(lookahead.error());
            }
        }
        Ok(Self {
            from: from.unwrap_or_else(|| abort!(ty, "Missing `from` arg")),
            validate: validate.unwrap_or_else(|| abort!(ty, "Missing `validate` arg")),
            multi: multi.unwrap_or_else(|| abort!(ty, "Missing `multi` arg")),
            single: single.unwrap_or_else(|| abort!(ty, "Missing `single` arg")),
            type_generics,
            ty,
            where_clause,
            brace,
        })
    }
}
impl ToTokens for VerifyAccountArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.type_generics.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        self.where_clause.to_tokens(tokens);
        self.brace.surround(tokens, |tokens| {
            self.from.to_tokens(tokens);
            self.validate.to_tokens(tokens);
            self.multi.to_tokens(tokens);
            self.single.to_tokens(tokens);
        });
    }
}

pub struct TypeList<T> {
    name_token: T,
    colon: Token![:],
    bracket: token::Bracket,
    types: Punctuated<TypeListItem, Token![;]>,
    semicolon: Token![;],
}
impl<T> TypeList<T> {
    fn into_token_stream(
        self,
        trait_prefix: &str,
        ty: &Type,
        generics: &Generics,
        impl_type: &TokenStream,
        account_info_gen: &Ident,
    ) -> TokenStream
    where
        T: ToTokens,
    {
        let types: Vec<_> = self
            .types
            .into_iter()
            .enumerate()
            .map(|(index, item)| {
                item.into_token_stream(
                    &format_ident!("{}{}", trait_prefix, index),
                    ty,
                    generics,
                    impl_type,
                    account_info_gen,
                )
            })
            .collect();
        quote! {
            #(#types)*
        }
    }
}
impl<T> Parse for TypeList<T>
where
    T: Parse,
{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name_token = input.parse()?;
        let colon = input.parse()?;
        let content;
        let bracket = bracketed!(content in input);
        let types = content.parse_terminated(TypeListItem::parse)?;
        let semicolon = input.parse()?;
        Ok(Self {
            name_token,
            colon,
            bracket,
            types,
            semicolon,
        })
    }
}
impl<T> ToTokens for TypeList<T>
where
    T: ToTokens,
{
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.name_token.to_tokens(tokens);
        self.colon.to_tokens(tokens);
        self.bracket.surround(tokens, |tokens| {
            self.types.to_tokens(tokens);
        });
        self.semicolon.to_tokens(tokens);
    }
}

pub struct TypeListItem {
    attributes: Vec<Attribute>,
    generics: Generics,
    ty: Type,
    where_clause: Option<WhereClause>,
}
impl TypeListItem {
    fn into_token_stream(
        self,
        trait_ident: &Ident,
        ty: &Type,
        generics: &Generics,
        impl_type: &TokenStream,
        account_info_gen: &Ident,
    ) -> TokenStream {
        let mut generics = generics.clone();
        generics.params.extend(self.generics.params.into_iter());
        let mut sorting = generics.params.into_iter().collect::<Vec<_>>();
        sorting.sort_unstable_by(|param1, param2| {
            let param_to_val = |param: &GenericParam| match param {
                GenericParam::Type(_) => 2,
                GenericParam::Lifetime(_) => 1,
                GenericParam::Const(_) => 3,
            };
            let param1 = param_to_val(param1);
            let param2 = param_to_val(param2);
            param1.cmp(&param2)
        });
        generics.params = sorting.into_iter().collect();
        if let Some(where_clause) = self.where_clause {
            generics
                .make_where_clause()
                .predicates
                .extend(where_clause.predicates.into_iter());
        }
        let self_ty = self.ty;
        let (impl_gen, ty_gen, where_clause) = generics.split_for_impl();
        quote! {
            #[automatically_derived]
            #[allow(clippy::type_repetition_in_bounds)]
            trait #trait_ident #impl_gen: #impl_type<#account_info_gen, #self_ty> #where_clause {}
            #[automatically_derived]
            #[allow(clippy::type_repetition_in_bounds)]
            impl #impl_gen #trait_ident #ty_gen for #ty #where_clause {}
        }
    }
}
impl Parse for TypeListItem {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let generics = input.parse()?;
        let ty = input.parse()?;
        let where_clause = input.parse()?;
        Ok(Self {
            attributes,
            generics,
            ty,
            where_clause,
        })
    }
}
impl ToTokens for TypeListItem {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.attributes
            .iter()
            .for_each(|attr| attr.to_tokens(tokens));
        self.generics.to_tokens(tokens);
        self.ty.to_tokens(tokens);
        self.where_clause.to_tokens(tokens);
    }
}
