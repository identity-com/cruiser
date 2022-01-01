#![warn(unused_import_braces, unused_imports)]

extern crate proc_macro;
use easy_proc_common::find_attrs;
use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use syn::parse::ParseStream;
use syn::punctuated::Punctuated;
use syn::{
    parse_macro_input, Data, DeriveInput, Expr, Field, Fields, GenericArgument, Ident, LitStr,
    PathArguments, PathSegment, Token, Type, TypePath,
};

#[proc_macro_error]
#[proc_macro_derive(ArgEnum, attributes(arg_enum))]
pub fn arg_enum_derive(ts: TokenStream) -> TokenStream {
    let arg_enum_ident = Ident::new("arg_enum", Span::call_site());
    let derive = parse_macro_input!(ts as DeriveInput);

    let generator_crate = crate_name("easy_proc").expect("Could not find `easy_proc`");
    let crate_name = match generator_crate {
        FoundCrate::Itself => quote! { ::easy_proc },
        FoundCrate::Name(name) => {
            let ident = Ident::new(&name, Span::call_site());
            quote! { ::#ident }
        }
    };

    let ident = derive.ident;
    let (impl_gen, ty_gen, where_clause) = derive.generics.split_for_impl();
    let data_struct = match derive.data {
        Data::Struct(data) => data,
        Data::Enum(data) => abort!(data.enum_token, "#[derive(ArgEnum)] only supports structs"),
        Data::Union(data) => abort!(data.union_token, "#[derive(ArgEnum)] only supports structs"),
    };
    let fields = match data_struct.fields {
        Fields::Named(named) => named.named,
        Fields::Unnamed(_) => abort!(ident, "Unnamed fields are not supported"),
        Fields::Unit => Punctuated::new(),
    };
    let field_names: Vec<Ident> = fields
        .iter()
        .map(|field| field.ident.as_ref().unwrap().clone())
        .collect();
    let field_strs = field_names
        .iter()
        .map(|name: &Ident| LitStr::new(name.to_string().as_str(), name.span()));
    let field_variants: Vec<_> = fields
        .into_iter()
        .map(|field| ArgEnumVariant::from_field(field, &arg_enum_ident))
        .collect();
    let inits = field_variants
        .iter()
        .zip(field_names.iter())
        .map(|(variant, variable_name)| variant.to_init(variable_name));
    let input_ident = Ident::new("__input", Span::call_site());
    let ident_ident = Ident::new("__ident", Span::call_site());
    let attr_ident = Ident::new("__attr", Span::call_site());
    let reads = field_variants
        .iter()
        .zip(field_names.iter())
        .map(|(variant, variable_name)| {
            variant.to_read(&input_ident, variable_name, &ident_ident, &crate_name)
        });
    let verifies = field_variants
        .iter()
        .zip(field_names.iter())
        .map(|(variant, variable_name)| variant.to_verify(variable_name, &attr_ident, &crate_name));

    (quote! {
        impl #impl_gen #crate_name::Arguments for #ident #ty_gen #where_clause{
            fn parse_arguments(#attr_ident: &::syn::Attribute) -> Self{
                #(#inits)*
                if let ::std::result::Result::Err(__error) = ::syn::Attribute::parse_args_with(#attr_ident, |#input_ident: ::syn::parse::ParseStream|{
                    'MainLoop: loop{
                        if #input_ident.is_empty(){
                            break 'MainLoop;
                        }
                        let #ident_ident: ::syn::Ident = #input_ident.parse()?;
                        let __ident_str = #ident_ident.to_string();
                        let __ident_str = __ident_str.as_str();
                        if false{}
                        #(
                            else if __ident_str == #field_strs {
                                #reads
                            }
                        )*
                        else {
                            #crate_name::proc_macro_error::abort!(#ident_ident, "Unknown argument `{}`", #ident_ident);
                        }

                        if #input_ident.peek(::syn::Token![,]) {
                            #input_ident.parse::<::syn::Token![,]>()?;
                        } else if !#input_ident.is_empty() {
                            #crate_name::proc_macro_error::abort!(
                                #input_ident.span(),
                                "Error parsing arguments, expected `,` or end of arguments"
                            )
                        }
                    }
                    Ok(())
                }){
                    #crate_name::proc_macro_error::abort_call_site!(
                        "Error parsing: `{}`", __error
                    )
                }
                Self{
                    #(#verifies)*
                }
            }
        }
    })
    .into()
}

enum ArgEnumVariant {
    Required(Type),
    Optional(Type),
    Many(Type),
    Custom(Type),
    OptionalCustom(Type),
    CustomMany(Type),
    Default(Type, Box<Expr>),
    Presence,
}
impl ArgEnumVariant {
    const PRESENCE_IDENT: &'static str = "presence";
    const RAW_TYPE_IDENT: &'static str = "raw_type";
    const CUSTOM_IDENT: &'static str = "custom";
    const DEFAULT_IDENT: &'static str = "default";

    fn from_field(field: Field, arg_enum_ident: &Ident) -> Self {
        let mut presence = None;
        let mut raw_type = None;
        let mut custom = None;
        let mut default = None;
        for attr in find_attrs(field.attrs, arg_enum_ident) {
            if let Err(error) = attr.parse_args_with(|input: ParseStream| {
                loop {
                    if input.is_empty() {
                        break;
                    }
                    let ident: Ident = input.parse()?;
                    let ident_str = ident.to_string();
                    let ident_str = ident_str.as_str();
                    if ident_str == Self::PRESENCE_IDENT {
                        if presence.is_some() {
                            abort!(ident, "Multiple `{}` arguments", Self::PRESENCE_IDENT);
                        }
                        presence = Some(ident);
                    } else if ident_str == Self::RAW_TYPE_IDENT {
                        if raw_type.is_some() {
                            abort!(ident, "Multiple `{}` arguments", Self::RAW_TYPE_IDENT);
                        }
                        raw_type = Some(ident);
                    } else if ident_str == Self::CUSTOM_IDENT {
                        if custom.is_some() {
                            abort!(ident, "Multiple `{}` arguments", Self::CUSTOM_IDENT);
                        }
                        custom = Some(ident);
                    } else if ident_str == Self::DEFAULT_IDENT {
                        if default.is_some() {
                            abort!(ident, "Multiple `{}` arguments", Self::DEFAULT_IDENT);
                        }
                        default = Some((
                            ident,
                            input.parse::<Token![=]>()?,
                            Box::new(input.parse::<Expr>()?),
                        ));
                    } else {
                        abort!(ident, "Unknown argument `{}`", ident);
                    }

                    if input.peek(Token![,]) {
                        input.parse::<Token![,]>()?;
                    } else if !input.is_empty() {
                        abort!(
                            input.span(),
                            "Error parsing arguments, expected `,` or end of arguments"
                        )
                    }
                }
                Ok(())
            }) {
                abort!(attr.tokens, "Error encountered parsing args: {}", error);
            }
        }
        if presence.is_some() as u8
            + (raw_type.is_some() || default.is_some()) as u8
            + custom.is_some() as u8
            > 1
        {
            abort!(field.ident.unwrap(), "Field has incompatible arguments");
        }
        if let Some(presence) = presence {
            if !is_bool(&field.ty) {
                abort!(presence, "Presence type must be `bool`")
            } else {
                Self::Presence
            }
        } else if let Some((_, _, default_expr)) = default {
            Self::Default(field.ty, default_expr)
        } else if raw_type.is_some() {
            Self::Required(field.ty)
        } else {
            match (custom.is_some(), is_option(&field.ty), is_vec(&field.ty)) {
                (false, None, None) => Self::Required(field.ty),
                (true, None, None) => Self::Custom(field.ty),
                (false, Some(ty), None) => Self::Optional(ty.clone()),
                (true, Some(ty), None) => Self::OptionalCustom(ty.clone()),
                (false, None, Some(ty)) => Self::Many(ty.clone()),
                (true, None, Some(ty)) => Self::CustomMany(ty.clone()),
                (_, Some(_), Some(_)) => unreachable!(),
            }
        }
    }

    fn to_init(&self, variable_ident: &Ident) -> proc_macro2::TokenStream {
        match self {
            Self::Required(ty)
            | Self::Optional(ty)
            | Self::Custom(ty)
            | Self::OptionalCustom(ty)
            | Self::Default(ty, _) => quote! {
                let mut #variable_ident: ::std::option::Option<#ty> = None;
            },
            Self::Many(ty) | Self::CustomMany(ty) => quote! {
                let mut #variable_ident: ::std::vec::Vec<#ty> = ::std::vec::Vec::new();
            },
            Self::Presence => quote! {
                let mut #variable_ident: bool = false;
            },
        }
    }

    fn to_read(
        &self,
        input_ident: &Ident,
        variable_ident: &Ident,
        ident_ident: &Ident,
        crate_name: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        match self {
            Self::Required(ty) | Self::Optional(ty) | Self::Default(ty, _) => {
                let error_string = LitStr::new(
                    &format!("Duplicate `{}` argument", variable_ident),
                    Span::call_site(),
                );
                quote! {
                    if ::std::option::Option::is_some(&#variable_ident) {
                        #crate_name::proc_macro_error::abort!(#ident_ident, #error_string);
                    }
                    <::syn::Token![=] as ::syn::parse::Parse>::parse(#input_ident)?;
                    #variable_ident = ::std::option::Option::Some(<#ty as ::syn::parse::Parse>::parse(#input_ident)?);
                }
            }
            Self::Many(ty) => quote! {
                <::syn::Token![=] as ::syn::parse::Parse>::parse(#input_ident)?;
                ::std::vec::Vec::push(&mut #variable_ident, <#ty as ::syn::parse::Parse>::parse(#input_ident)?);
            },
            Self::Custom(ty) | Self::OptionalCustom(ty) => {
                let error_string = LitStr::new(
                    &format!("Duplicate `{}` argument", variable_ident),
                    Span::call_site(),
                );
                quote! {
                    if ::std::option::Option::is_some(&#variable_ident) {
                        #crate_name::proc_macro_error::abort!(#ident_ident, #error_string);
                    }
                    #variable_ident = ::std::option::Option::Some(<#ty as ::syn::parse::Parse>::parse(#input_ident)?);
                }
            }
            Self::CustomMany(ty) => quote! {
                ::std::vec::Vec::push(&mut #variable_ident, <#ty as ::syn::parse::Parse>::parse(#input_ident)?);
            },
            Self::Presence => {
                let error_string = LitStr::new(
                    &format!("Duplicate `{}` argument", variable_ident),
                    Span::call_site(),
                );
                quote! {
                    if #variable_ident {
                        #crate_name::proc_macro_error::abort!(#ident_ident, #error_string);
                    }
                    #variable_ident = true;
                }
            }
        }
    }

    fn to_verify(
        &self,
        variable_ident: &Ident,
        attr_ident: &Ident,
        crate_name: &proc_macro2::TokenStream,
    ) -> proc_macro2::TokenStream {
        match self {
            Self::Required(_) | Self::Custom(_) => {
                let error_msg = LitStr::new(
                    &format!("Missing `{}` argument", variable_ident),
                    Span::call_site(),
                );
                quote! {
                    #variable_ident: match #variable_ident{
                        ::std::option::Option::Some(val) => val,
                        ::std::option::Option::None => #crate_name::proc_macro_error::abort!(#attr_ident, #error_msg),
                    },
                }
            }
            Self::Optional(_)
            | Self::OptionalCustom(_)
            | Self::Many(_)
            | Self::CustomMany(_)
            | Self::Presence => quote! {
                #variable_ident,
            },
            Self::Default(_, default) => quote! {
                #variable_ident: match #variable_ident{
                    ::std::option::Option::Some(val) => val,
                    ::std::option::Option::None => #default,
                },
            },
        }
    }
}

fn is_option(ty: &Type) -> Option<&Type> {
    is_type(ty, "Option")
}

fn is_vec(ty: &Type) -> Option<&Type> {
    is_type(ty, "Vec")
}

fn is_type<'a>(ty: &'a Type, name: &str) -> Option<&'a Type> {
    match ty {
        Type::Path(TypePath {
            qself: Option::None,
            path,
        }) if path
            .segments
            .first()
            .map(|segment| segment.ident.to_string().as_str() == name)
            .unwrap_or(false) =>
        {
            if let Some(PathSegment {
                arguments: PathArguments::AngleBracketed(args),
                ..
            }) = path.segments.last()
            {
                if let Some(GenericArgument::Type(ty)) = args.args.first() {
                    return Some(ty);
                }
            }
        }
        _ => {}
    }
    None
}

fn is_bool(ty: &Type) -> bool {
    matches!(ty, Type::Path(
        TypePath {
            qself: Option::None,
            path,
        }
    ) if path.is_ident(&Ident::new("bool", Span::call_site())))
}
