use crate::get_crate_name;
use easy_proc::{find_attr, ArgumentList};
use heck::ToPascalCase;
use proc_macro2::TokenStream;
use proc_macro_error::abort;
use quote::{format_ident, quote};
use syn::parse::Parse;
use syn::{
    Attribute, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, FieldsUnnamed, Ident,
    Type,
};

#[derive(ArgumentList, Default)]
pub struct InPlaceArgs {
    access_struct_name: Option<Ident>,
    properties_enum_name: Option<Ident>,
}
impl InPlaceArgs {
    const IDENT: &'static str = "in_place";
}

#[derive(ArgumentList, Default)]
pub struct InPlaceFieldArgs {
    #[argument(presence)]
    dynamic_size: bool,
}
impl InPlaceFieldArgs {
    const IDENT: &'static str = "in_place";
}

pub struct InPlaceDerive {
    tokens: TokenStream,
}

pub fn get_attr<'a, Attr: ArgumentList, I: IntoIterator<Item = &'a Attribute>>(
    attrs: I,
    attr_name: &'static str,
) -> Option<Attr> {
    find_attr(attrs, &format_ident!("{}", attr_name)).map(Attr::parse_arguments)
}

impl Parse for InPlaceDerive {
    #[allow(clippy::too_many_lines)]
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let crate_name = get_crate_name();
        let derive = input.parse::<DeriveInput>()?;

        let InPlaceArgs {
            access_struct_name,
            properties_enum_name,
        } = get_attr::<InPlaceArgs, _>(derive.attrs.iter(), InPlaceArgs::IDENT).unwrap_or_default();
        let access_struct_name =
            access_struct_name.unwrap_or_else(|| format_ident!("{}Access", derive.ident));
        let properties_enum_name =
            properties_enum_name.unwrap_or_else(|| format_ident!("{}Properties", derive.ident));
        let DeriveInput {
            vis,
            ident,
            generics,
            data,
            ..
        } = derive;
        let DataStruct { fields, .. } = match data {
            Data::Struct(data) => data,
            Data::Enum(_) | Data::Union(_) => abort!(
                ident.span(),
                "`#[derive(InPlace)]` can only be used on structs"
            ),
        };
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
        let create = fields.create(&crate_name);
        let read = fields.read(&access_struct_name);
        let write = fields.write(&access_struct_name);
        let enum_idents = fields.enum_idents();
        let offset = offsets(&crate_name, &enum_idents);
        let sizes = fields.sizes(&crate_name, &enum_idents);
        let property_impls = fields.property_impls(&crate_name, &access_struct_name);

        let tokens = quote! {
            impl #impl_generics const #crate_name::in_place::InPlace for #ident #ty_generics #where_clause {
                type Access<'__a, __A>
                where
                    Self: '__a,
                    __A: '__a + #crate_name::util::MappableRef + #crate_name::util::TryMappableRef
                = #access_struct_name<__A>;
            }
            impl #impl_generics const #crate_name::in_place::InPlaceProperties for #ident #ty_generics #where_clause {
                type Properties = #properties_enum_name;
            }
            impl #impl_generics #crate_name::in_place::InPlaceCreate for #ident #ty_generics #where_clause {
                 fn create_with_arg<__A: ::std::ops::DerefMut<Target = [u8]>>(mut __data: __A, _arg: ()) -> #crate_name::CruiserResult {
                    #create
                 }
            }
            impl #impl_generics #crate_name::in_place::InPlaceRead for #ident #ty_generics #where_clause {
                fn read_with_arg<'__a, __A>(__data: __A, _arg: ()) -> #crate_name::CruiserResult<<Self as #crate_name::in_place::InPlace>::Access<'__a, __A>>
                where
                    Self: '__a,
                    __A: '__a + ::std::ops::Deref<Target = [u8]> + #crate_name::util::MappableRef + #crate_name::util::TryMappableRef,
                {
                    #read
                }
            }
            impl #impl_generics #crate_name::in_place::InPlaceWrite for #ident #ty_generics #where_clause {
                fn write_with_arg<'__a, __A>(__data: __A, _arg: ()) -> #crate_name::CruiserResult<<Self as #crate_name::in_place::InPlace>::AccessMut<'__a, __A>>
                where
                    Self: '__a,
                    __A: '__a
                        + ::std::ops::DerefMut<Target = [u8]>
                        + #crate_name::util::MappableRef
                        + #crate_name::util::TryMappableRef
                        + #crate_name::util::MappableRefMut
                        + #crate_name::util::TryMappableRefMut,
                {
                    #write
                }
            }

            #vis struct #access_struct_name<__A>(__A);
            impl<__A> const #crate_name::in_place::InPlaceRawDataAccess for #access_struct_name<__A>
            where
                __A: ~const ::std::ops::Deref<Target = [u8]>,
            {
                fn get_raw_data(&self) -> &[u8] {
                    &*self.0
                }
            }
            impl<__A> const #crate_name::in_place::InPlaceRawDataAccessMut for #access_struct_name<__A>
            where
                __A: ~const ::std::ops::DerefMut<Target = [u8]>,
            {
                fn get_raw_data_mut(&mut self) -> &mut [u8] {
                    &mut *self.0
                }
            }

            #[derive(Copy, Clone, Debug, PartialEq, Eq)]
            #vis enum #properties_enum_name{
                #(#enum_idents,)*
            }
            impl const #crate_name::in_place::InPlacePropertiesList for #properties_enum_name {
                fn index(self) -> usize{
                    self as usize
                }

                fn offset(self) -> usize {
                    #offset
                }

                fn size(self) -> ::std::option::Option<usize> {
                    #sizes
                }
            }
            #property_impls
        };
        Ok(Self { tokens })
    }
}

trait InPlaceFields {
    fn create(&self, crate_name: &TokenStream) -> TokenStream;
    fn read(&self, access_struct_name: &Ident) -> TokenStream;
    fn write(&self, access_struct_name: &Ident) -> TokenStream;
    fn enum_idents(&self) -> Vec<Ident>;
    fn sizes<'a>(
        &self,
        crate_name: &TokenStream,
        enum_idents: impl IntoIterator<Item = &'a Ident>,
    ) -> TokenStream;
    fn property_impls(&self, crate_name: &TokenStream, access_struct_name: &Ident) -> TokenStream;
}

impl InPlaceFields for Fields {
    fn create(&self, crate_name: &TokenStream) -> TokenStream {
        match self {
            Fields::Named(fields) => fields.create(crate_name),
            Fields::Unnamed(fields) => fields.create(crate_name),
            Fields::Unit => quote! { ::std::result::Result::Ok(()) },
        }
    }

    fn read(&self, access_struct_name: &Ident) -> TokenStream {
        quote! { Ok(#access_struct_name(__data)) }
    }

    fn write(&self, access_struct_name: &Ident) -> TokenStream {
        quote! { Ok(#access_struct_name(__data)) }
    }

    fn enum_idents(&self) -> Vec<Ident> {
        match self {
            Fields::Named(fields) => fields.enum_idents(),
            Fields::Unnamed(fields) => fields.enum_idents(),
            Fields::Unit => vec![],
        }
    }

    fn sizes<'a>(
        &self,
        crate_name: &TokenStream,
        enum_idents: impl IntoIterator<Item = &'a Ident>,
    ) -> TokenStream {
        match self {
            Fields::Named(fields) => fields.sizes(crate_name, enum_idents),
            Fields::Unnamed(fields) => fields.sizes(crate_name, enum_idents),
            Fields::Unit => quote! { Some(0) },
        }
    }

    fn property_impls(&self, crate_name: &TokenStream, access_struct_name: &Ident) -> TokenStream {
        match self {
            Fields::Named(fields) => fields.property_impls(crate_name, access_struct_name),
            Fields::Unnamed(fields) => fields.property_impls(crate_name, access_struct_name),
            Fields::Unit => quote! {},
        }
    }
}

impl InPlaceFields for FieldsNamed {
    fn create(&self, crate_name: &TokenStream) -> TokenStream {
        create(self.named.iter(), crate_name)
    }

    fn read(&self, _access_struct_name: &Ident) -> TokenStream {
        unreachable!()
    }

    fn write(&self, _access_struct_name: &Ident) -> TokenStream {
        unreachable!()
    }

    fn enum_idents(&self) -> Vec<Ident> {
        self.named
            .iter()
            .map(|field| {
                format_ident!(
                    "{}",
                    field.ident.as_ref().unwrap().to_string().to_pascal_case()
                )
            })
            .collect()
    }

    fn sizes<'a>(
        &self,
        crate_name: &TokenStream,
        enum_idents: impl IntoIterator<Item = &'a Ident>,
    ) -> TokenStream {
        sizes(self.named.iter(), crate_name, enum_idents)
    }

    fn property_impls(&self, crate_name: &TokenStream, access_struct_name: &Ident) -> TokenStream {
        property_impls(self.named.iter(), crate_name, access_struct_name)
    }
}

impl InPlaceFields for FieldsUnnamed {
    fn create(&self, crate_name: &TokenStream) -> TokenStream {
        create(self.unnamed.iter(), crate_name)
    }

    fn read(&self, _access_struct_name: &Ident) -> TokenStream {
        unreachable!()
    }

    fn write(&self, _access_struct_name: &Ident) -> TokenStream {
        unreachable!()
    }

    fn enum_idents(&self) -> Vec<Ident> {
        (0..self.unnamed.len())
            .map(|i| format_ident!("Index{}", i))
            .collect()
    }

    fn sizes<'a>(
        &self,
        crate_name: &TokenStream,
        enum_idents: impl IntoIterator<Item = &'a Ident>,
    ) -> TokenStream {
        sizes(self.unnamed.iter(), crate_name, enum_idents)
    }

    fn property_impls(&self, crate_name: &TokenStream, access_struct_name: &Ident) -> TokenStream {
        property_impls(self.unnamed.iter(), crate_name, access_struct_name)
    }
}

fn create<'a>(iter: impl IntoIterator<Item = &'a Field>, crate_name: &TokenStream) -> TokenStream {
    let out = iter.into_iter().map(|field| {
        let Field { ty, .. } = field;
        let attr = get_attr::<InPlaceFieldArgs, _>(field.attrs.iter(), InPlaceFieldArgs::IDENT).unwrap_or_default();
        if attr.dynamic_size{
            quote! {
                <#ty as #crate_name::in_place::InPlaceCreate>::create_with_arg(__data, ())?;
            }
        } else {
            quote! {
                <#ty as #crate_name::in_place::InPlaceCreate>::create_with_arg(#crate_name::util::Advance::try_advance(&mut __data, <#ty as #crate_name::on_chain_size::OnChainSize>::ON_CHAIN_SIZE)?, ())?;
            }
        }

    });
    quote! {
        let mut __data = &mut *__data;
        #(#out)*
        ::std::result::Result::Ok(())
    }
}

fn offsets<'a, 'b>(
    crate_name: &TokenStream,
    enum_idents: impl IntoIterator<Item = &'b Ident>,
) -> TokenStream {
    let mut last_ident = None;
    let out = enum_idents.into_iter().map(|enum_ident| {
        let offset = last_ident.replace(enum_ident).map_or_else(
            || quote! { 0 },
            |enum_ident| {
                quote! {
                <Self as #crate_name::in_place::InPlacePropertiesList>::offset(Self::#enum_ident)
                    + match <Self as ::cruiser::in_place::InPlacePropertiesList>::size(Self::#enum_ident) {
                        ::std::option::Option::Some(size) => size,
                        ::std::option::Option::None => {
                            ::std::panic!("Middle element unsized!")
                        }
                    }
                }
            },
        );
        quote! {
            Self::#enum_ident => #offset,
        }
    });
    quote! {
        match self {
            #(#out)*
        }
    }
}

fn sizes<'a, 'b>(
    iter: impl IntoIterator<Item = &'a Field>,
    crate_name: &TokenStream,
    enum_idents: impl IntoIterator<Item = &'b Ident>,
) -> TokenStream {
    let out = iter.into_iter().zip(enum_idents).map(|(field, enum_ident): (&Field, &Ident)| {
        let attr = get_attr::<InPlaceFieldArgs, _>(field.attrs.iter(), InPlaceFieldArgs::IDENT).unwrap_or_default();
        if attr.dynamic_size{
            quote! {
                Self::#enum_ident => ::std::option::Option::None,
            }
        } else {
            let Field { ty, .. } = field;
            quote! {
                Self::#enum_ident => ::std::option::Option::Some(<#ty as #crate_name::on_chain_size::OnChainSize>::ON_CHAIN_SIZE),
            }
        }
    });
    quote! {
        match self {
            #(#out)*
        }
    }
}

fn property_impls<'a, 'b>(
    iter: impl IntoIterator<Item = &'a Field>,
    crate_name: &TokenStream,
    access_struct_name: &Ident,
) -> TokenStream {
    let out = iter
        .into_iter()
        .map(|f| &f.ty)
        .enumerate()
        .map(|(index, ty): (usize, &Type)| {
            quote! {
                impl<__A> const #crate_name::in_place::InPlaceProperty<#index> for #access_struct_name<__A> {
                    type Property = #ty;
                }
            }
        });
    quote! {
        #(#out)*
    }
}

impl InPlaceDerive {
    pub fn into_token_stream(self) -> TokenStream {
        self.tokens
    }
}
