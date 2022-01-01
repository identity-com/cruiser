use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use std::convert::{TryFrom, TryInto};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    Attribute, Data, DeriveInput, Expr, Fields, Generics, Ident, LitStr, Token, Type,
    Variant, Visibility,
};

pub struct InstructionListDerive {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    attribute: InstructionListAttribute,
    variants: Vec<InstructionListVariant>,
}
impl Parse for InstructionListDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derive_input: DeriveInput = input.parse()?;

        let attribute = derive_input.attrs.try_into()?;

        let variants = match derive_input.data {
            Data::Struct(_) | Data::Union(_) => {
                abort_call_site!("derive `InstructionList` supports only enums")
            }
            Data::Enum(enum_data) => enum_data.variants,
        };

        let variants = variants
            .into_iter()
            .map(InstructionListVariant::try_from)
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            vis: derive_input.vis,
            ident: derive_input.ident,
            generics: derive_input.generics,
            attribute,
            variants,
        })
    }
}
impl InstructionListDerive {
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

        let vis = self.vis;
        let ident = self.ident;
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let enum_ident = self.attribute.build_enum_ident.unwrap_or_else(|| {
            Ident::new(
                &("Build".to_string() + &ident.to_string()),
                Span::call_site(),
            )
        });

        let (variant_ident, variant_instruction_type, variant_discriminant) = {
            let mut variant_ident = Vec::with_capacity(self.variants.len());
            let mut variant_instruction_type = Vec::with_capacity(self.variants.len());
            let mut variant_discriminant = Vec::with_capacity(self.variants.len());
            for variant in self.variants {
                variant_ident.push(variant.ident);
                variant_instruction_type.push(variant.attribute.instruction_type);
                variant_discriminant.push(variant.discriminant.map(|expr| quote!{ #expr }).unwrap_or_else(||{
                    let last = variant_discriminant.last().cloned().unwrap_or_else(|| quote! { 0 });
                    quote! {
                        #last + 1
                    }
                }));
            }
            (
                variant_ident,
                variant_instruction_type,
                variant_discriminant,
            )
        };

        let instruction_prints = variant_ident
            .iter()
            .map(|ident| LitStr::new(&format!("Instruction: {}", ident.to_string()), ident.span()));

        quote! {
            #[automatically_derived]
            impl #impl_generics InstructionList for #ident #ty_generics #where_clause{
                type BuildEnum = #enum_ident;

                fn build_instruction(
                    program_id: #crate_name::Pubkey,
                    build_enum: Self::BuildEnum,
                ) -> GeneratorResult<#crate_name::SolanaInstruction>{
                    match build_enum{
                        #(
                            Self::BuildEnum::#variant_ident(build) => {
                                let (accounts, data_assoc) = <#variant_instruction_type as #crate_name::Instruction>::build_instruction(program_id, build)?;
                                let mut data = ::std::vec![#variant_discriminant];
                                ::borsh::BorshSerialize::serialize(&data_assoc, &mut data)?;
                                Ok(#crate_name::SolanaInstruction{ program_id, accounts, data })
                            },
                        )*
                    }
                }

                fn discriminant(self) -> u8{
                    match self{
                        #(Self::#variant_ident => #variant_discriminant,)*
                    }
                }
            }

            /// The build enum for [`#ident`]
            #[allow(missing_docs)]
            #[derive(Debug)]
            #vis enum #enum_ident #impl_generics #where_clause{
                #(
                    #variant_ident(<#variant_instruction_type as #crate_name::Instruction>::BuildArg),
                )*
            }
        }
    }
}

struct InstructionListAttribute {
    build_enum_ident: Option<Ident>,
}
impl InstructionListAttribute {
    const IDENT: &'static str = "instruction_list";
}
impl TryFrom<Vec<Attribute>> for InstructionListAttribute {
    type Error = syn::Error;

    fn try_from(value: Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut attribute = None;
        let self_ident = Ident::new(Self::IDENT, Span::call_site());
        for attr in value {
            if attr.path.is_ident(&self_ident) && attribute.replace(attr.clone()).is_some() {
                abort!(attr, "Duplicate `{}` attribute", Self::IDENT);
            }
        }
        match attribute {
            None => Ok(Self {
                build_enum_ident: None,
            }),
            Some(attribute) => {
                let args: InstructionListArgs = attribute.parse_args()?;
                let mut build_enum_ident = None;
                for arg in args.0 {
                    match arg {
                        InstructionListAttributeArg::BuildEnumIdent {
                            ident,
                            build_enum_ident: new_ident,
                        } => {
                            if build_enum_ident.replace(new_ident).is_some() {
                                abort!(
                                    ident,
                                    "duplicate `{}` argument for attribute `{}`",
                                    InstructionListAttributeArg::BUILD_ENUM_IDENT_IDENT,
                                    Self::IDENT
                                );
                            }
                        }
                    }
                }

                Ok(Self {
                    build_enum_ident,
                })
            }
        }
    }
}
struct InstructionListArgs(Punctuated<InstructionListAttributeArg, Token![,]>);
impl Parse for InstructionListArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input.parse_terminated(InstructionListAttributeArg::parse)?,
        ))
    }
}

enum InstructionListAttributeArg {
    BuildEnumIdent {
        ident: Ident,
        build_enum_ident: Ident,
    },
}
impl InstructionListAttributeArg {
    const BUILD_ENUM_IDENT_IDENT: &'static str = "build_enum";
}
impl Parse for InstructionListAttributeArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == Self::BUILD_ENUM_IDENT_IDENT {
            input.parse::<Token![=]>()?;
            Ok(Self::BuildEnumIdent {
                ident,
                build_enum_ident: input.parse()?,
            })
        } else {
            abort!(
                ident,
                "Unknown `{}` argument `{}`",
                InstructionListAttribute::IDENT,
                ident
            )
        }
    }
}

struct InstructionListVariant {
    ident: Ident,
    discriminant: Option<Expr>,
    attribute: InstructionListVariantAttribute,
}
impl TryFrom<Variant> for InstructionListVariant {
    type Error = syn::Error;

    fn try_from(value: Variant) -> Result<Self, Self::Error> {
        match &value.fields {
            Fields::Unit => {}
            _ => abort!(
                value,
                "derive `InstructionList` only supports unit enum values"
            ),
        }

        let attribute = (&value.ident, value.attrs).try_into()?;

        Ok(Self {
            ident: value.ident,
            discriminant: value.discriminant.map(|val| val.1),
            attribute,
        })
    }
}

struct InstructionListVariantAttribute {
    instruction_type: Type,
}
impl InstructionListVariantAttribute {
    pub const IDENT: &'static str = "instruction_list";
}
impl TryFrom<(&Ident, Vec<Attribute>)> for InstructionListVariantAttribute {
    type Error = syn::Error;

    fn try_from(value: (&Ident, Vec<Attribute>)) -> Result<Self, Self::Error> {
        let mut attribute = None;
        let self_ident = Ident::new(Self::IDENT, Span::call_site());
        for attr in value.1 {
            if attr.path.is_ident(&self_ident) && attribute.replace(attr.clone()).is_some() {
                abort!(attr, "Duplicate `{}` attribute", Self::IDENT);
            }
        }
        match attribute {
            None => abort!(value.0, "Variant missing `{}` attribute", Self::IDENT),
            Some(attribute) => {
                let args: InstructionListVariantArgs = attribute.parse_args()?;
                let mut instruction_type = None;
                for arg in args.0 {
                    match arg {
                        InstructionListVariantArg::Instruction { ident, ty } => {
                            if instruction_type.replace(ty).is_some() {
                                abort!(
                                    ident,
                                    "duplicate `{}` argument for attribute `{}`",
                                    InstructionListVariantArg::INSTRUCTION_IDENT,
                                    Self::IDENT
                                );
                            }
                        }
                    }
                }

                Ok(Self {
                    instruction_type: instruction_type.unwrap_or_else(|| {
                        abort!(
                            attribute,
                            "Attribute `{}` missing `{}` argument",
                            Self::IDENT,
                            InstructionListVariantArg::INSTRUCTION_IDENT
                        )
                    }),
                })
            }
        }
    }
}

struct InstructionListVariantArgs(Punctuated<InstructionListVariantArg, Token![,]>);
impl Parse for InstructionListVariantArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input.parse_terminated(InstructionListVariantArg::parse)?,
        ))
    }
}

enum InstructionListVariantArg {
    Instruction { ident: Ident, ty: Type },
}
impl InstructionListVariantArg {
    const INSTRUCTION_IDENT: &'static str = "instruction";
}
impl Parse for InstructionListVariantArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        if ident == Self::INSTRUCTION_IDENT {
            input.parse::<Token![=]>()?;
            Ok(Self::Instruction {
                ident,
                ty: input.parse()?,
            })
        } else {
            abort!(
                ident,
                "Unknown `{}` argument `{}`",
                InstructionListVariantAttribute::IDENT,
                ident
            )
        }
    }
}
