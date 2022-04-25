use crate::get_crate_name;
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{braced, Expr, Token, Type};

pub struct GetProperties {
    value: Expr,
    ty: Type,
    properties: Punctuated<Property, Token![,]>,
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
        })
    }
}
impl GetProperties {
    pub fn into_token_stream(self) -> TokenStream {
        let GetProperties {
            value,
            ty,
            properties,
            ..
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

        quote! {{
            fn get_properties<A>(
                value: &mut <#ty as #crate_name::in_place::InPlace>::Access<A>
            ) -> #crate_name::CruiserResult<(#(
                <<<#ty as #crate_name::in_place::InPlace>::Access<A> as #crate_name::in_place::InPlaceProperty<
                    { #crate_name::in_place::InPlacePropertiesList::index(<#ty as #crate_name::in_place::InPlaceProperties>::Properties::#properties_pascal) },
                >>::Property as #crate_name::in_place::InPlace>::Access<&'_ mut [u8]>
            ),*)>
            where
                A: ::std::ops::DerefMut<Target = [u8]>
            {
                const OFFSETS: [(usize, usize); #property_count] = calc_property_offsets([
                    #(<#ty as #crate_name::in_place::InPlaceProperties>::Properties::#properties_pascal),*
                ]);

                let mut data = #crate_name::in_place::InPlaceGetData::get_raw_data_mut(value);
                Ok((
                    #({
                        #crate_name::util::Advance::try_advance(&mut data, OFFSETS[#indexes].0)?;
                        <<
                            <#ty as InPlace>::Access<A> as #crate_name::in_place::InPlaceProperty<{ #crate_name::in_place::InPlacePropertiesList::index(<#ty as #crate_name::in_place::InPlaceProperties>::Properties::#properties_pascal) }>
                        >::Property as #crate_name::in_place::InPlaceWrite<_>>::write_with_arg(
                            match OFFSET[#indexes].1{
                                Some(size) => #crate_name::util::Advance::try_advance(&mut data, OFFSETS[#indexes].1)?,
                                None => data,
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
