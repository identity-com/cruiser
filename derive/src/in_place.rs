use crate::account_argument::NamedTupple;
use crate::get_crate_name;
use easy_proc::{find_attr, ArgumentList};
use proc_macro2::{Span, TokenStream};
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::{Data, Fields, FieldsNamed, FieldsUnnamed, GenericParam, Ident, Lifetime, LifetimeDef};

#[derive(ArgumentList)]
pub struct InPlaceArgs {
    #[argument(default)]
    data: NamedTupple,
    struct_name: Option<Ident>,
}
impl InPlaceArgs {
    const NAME: &'static str = "in_place";
}

pub struct InPlaceDerive {
    tokens: TokenStream,
}
impl Parse for InPlaceDerive {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut derive_input = input.parse::<syn::DeriveInput>()?;

        let crate_name = get_crate_name();
        let ident = derive_input.ident;
        let args = InPlaceArgs::parse_arguments(
            find_attr(&derive_input.attrs, &format_ident!("{}", InPlaceArgs::NAME)).unwrap(),
        );
        let gen_struct = |fields: &Fields, unit_ender: &TokenStream| match fields {
            Fields::Named(FieldsNamed { named, .. }) => {
                let fields = named.iter().map(|f| {
                    let name = f.ident.as_ref().unwrap();
                    let ty = &f.ty;
                    quote! {
                        #name: <#ty as #crate_name::in_place::InPlace<'__a>>::Access,
                    }
                });
                quote! {
                    #(#fields,)*
                }
            }
            Fields::Unnamed(FieldsUnnamed { unnamed, .. }) => {
                let fields = unnamed.iter().map(|f| {
                    let ty = &f.ty;
                    quote! {
                        <#ty as #crate_name::in_place::InPlace<'__a>>::Access,
                    }
                });
                quote! {
                    #(#fields,)*
                }
            }
            Fields::Unit => unit_ender.clone(),
        };
        let data = match &derive_input.data {
            Data::Struct(struct_data) => gen_struct(&struct_data.fields, &quote! {;}),
            Data::Enum(enum_data) => {
                let mut variant_name = Vec::with_capacity(enum_data.variants.len());
                let mut variant_types = Vec::with_capacity(enum_data.variants.len());
                for variant in &enum_data.variants {
                    variant_name.push(&variant.ident);
                    variant_types.push(gen_struct(&variant.fields, &quote! {,}));
                }
                quote! {
                    #(
                        #variant_name #variant_types
                    )*
                }
            }
            Data::Union(union_data) => abort!(
                union_data.union_token,
                "Unions are not supported in in_place"
            ),
        };
        let struct_name = args
            .struct_name
            .unwrap_or_else(|| format_ident!("{}Access", ident));
        derive_input.generics.params.insert(
            0,
            GenericParam::Lifetime(LifetimeDef::new(Lifetime::new("'__a", Span::call_site()))),
        );

        let (impl_generics, ty_generics, where_clause) = derive_input.generics.split_for_impl();
        Ok(Self {
            tokens: quote! {
                pub struct #struct_name<'__a> {
                    #data
                }
                impl #impl_generics #crate_name::in_place::InPlace<'__a, > for #ident #ty_generics #where_clause {
                    type Access = #struct_name<'__a>;

                    fn read_with_arg(data: &mut &'a mut [u8], arg: A) -> CruiserResult<Self::Access>{

                    }
                }
            },
        })
    }
}
impl InPlaceDerive {
    pub fn into_token_stream(self) -> proc_macro2::TokenStream {
        self.tokens
    }
}
