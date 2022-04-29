use crate::{get_crate_name, NAME_NONCE};
use heck::ToPascalCase;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use std::sync::atomic::Ordering;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, Expr, Lifetime, Token, Type};

pub struct GetProperties {
    value: Expr,
    ty: Type,
    properties: Punctuated<Property, Token![,]>,
    mutable: bool,
}

impl GetProperties {
    pub fn set_mutable(mut self, mutable: bool) -> Self {
        self.mutable = mutable;
        self
    }
}

impl Parse for GetProperties {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let value = input.parse()?;
        input.parse::<Token![,]>()?;
        let ty = input.parse()?;
        let content;
        braced!(content in input);
        let properties = content.parse_terminated(Property::parse)?;
        Ok(Self {
            value,
            ty,
            properties,
            mutable: false,
        })
    }
}

impl GetProperties {
    pub fn into_token_stream(self) -> TokenStream {
        let GetProperties {
            value,
            ty,
            properties,
            mutable,
        } = self;
        let crate_name = get_crate_name();
        let property_count = properties.len();
        let properties_pascal = properties
            .iter()
            .map(|p| format_ident!("{}", p.ident.to_string().as_str().to_pascal_case()))
            .collect::<Vec<_>>();
        let args = properties
            .iter()
            .map(|p| {
                p.arg
                    .clone()
                    .map_or_else(|| syn::parse_str("()").unwrap(), |val| val.1)
            })
            .collect::<Vec<_>>();
        let indexes = 0..property_count;

        let nonce = NAME_NONCE.fetch_add(1, Ordering::SeqCst);
        let a_lifetime = Lifetime::new(&format!("'a{}", nonce), Span::call_site());
        let b_lifetime = Lifetime::new(&format!("'b{}", nonce), Span::call_site());
        let a_ident = format_ident!("__A{}", nonce);

        let mut_token = if mutable {
            quote! { mut }
        } else {
            quote! {}
        };
        let access = if mutable {
            quote! { AccessMut }
        } else {
            quote! { Access }
        };
        let deref = if mutable {
            quote! { DerefMut }
        } else {
            quote! { Deref }
        };
        let raw_data = if mutable {
            quote! { InPlaceRawDataAccessMut::get_raw_data_mut }
        } else {
            quote! { InPlaceRawDataAccess::get_raw_data }
        };
        let read_write = if mutable {
            quote! { InPlaceWrite<_>>::write_with_arg }
        } else {
            quote! { InPlaceRead<_>>::read_with_arg }
        };
        let extra_wheres = if mutable {
            quote! { + #crate_name::util::MappableRefMut + #crate_name::util::TryMappableRefMut }
        } else {
            quote! {}
        };

        quote! {{
            fn get_properties<#a_lifetime, #b_lifetime, #a_ident>(
                value: &#b_lifetime #mut_token <#ty as #crate_name::in_place::InPlace>::#access<#a_lifetime, #a_ident>
            ) -> #crate_name::CruiserResult<(#(
                <<<#ty as #crate_name::in_place::InPlace>::#access<#a_lifetime, #a_ident> as #crate_name::in_place::InPlaceProperty<
                    { #crate_name::in_place::InPlacePropertiesList::index(<#ty as #crate_name::in_place::InPlaceProperties>::Properties::#properties_pascal) },
                >>::Property as #crate_name::in_place::InPlace>::#access<#b_lifetime, &#b_lifetime #mut_token [u8]>
            ),*)>
            where
                #ty: #crate_name::in_place::InPlaceProperties,
                #a_ident: ::std::ops::#deref<Target = [u8]>
                        + #a_lifetime
                        + #crate_name::util::MappableRef
                        + #crate_name::util::TryMappableRef
                        #extra_wheres,
            {
                const OFFSETS: [(usize, Option<usize>); #property_count] = #crate_name::in_place::calc_property_offsets([
                    #(<#ty as #crate_name::in_place::InPlaceProperties>::Properties::#properties_pascal),*
                ]);

                let mut data = #crate_name::in_place::#raw_data(value);
                Ok((
                    #({
                        #crate_name::util::Advance::try_advance(&mut data, OFFSETS[#indexes].0)?;
                        <<
                            <#ty as #crate_name::in_place::InPlace>::#access<#a_lifetime, #a_ident> as #crate_name::in_place::InPlaceProperty<{ #crate_name::in_place::InPlacePropertiesList::index(<#ty as #crate_name::in_place::InPlaceProperties>::Properties::#properties_pascal) }>
                        >::Property as #crate_name::in_place::#read_write(
                            match OFFSETS[#indexes].1{
                                Some(size) => #crate_name::util::Advance::try_advance(&mut data, size)?,
                                None => {
                                    let data_len = data.len();
                                    #crate_name::util::Advance::try_advance(&mut data, data_len)?
                                },
                            },
                            #args,
                        )?
                    }),*
                ))
            }
            get_properties(#value)
        }}
    }
}

struct Property {
    ident: syn::Ident,
    arg: Option<(Token![:], Expr)>,
}

impl Parse for Property {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let arg = if input.peek(Token![:]) {
            let colon: Token![:] = input.parse()?;
            let arg = input.parse()?;
            Some((colon, arg))
        } else {
            None
        };
        Ok(Self { ident, arg })
    }
}
