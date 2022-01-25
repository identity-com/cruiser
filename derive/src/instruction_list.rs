use easy_proc::{find_attr, ArgumentList};
use proc_macro2::{Span, TokenStream};
use proc_macro_crate::{crate_name, FoundCrate};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use syn::{
    parenthesized, token, Data, DeriveInput, Expr, Fields, Generics, Ident, LitStr, Token, Type,
    Variant, Visibility,
};

#[derive(ArgumentList)]
pub struct InstructionListAttribute {
    build_enum_ident: Option<Ident>,
}
impl InstructionListAttribute {
    const IDENT: &'static str = "instruction_list";
}

pub struct InstructionListDerive {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    attribute: Option<InstructionListAttribute>,
    variants: Vec<InstructionListVariant>,
}
impl Parse for InstructionListDerive {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let instruction_list_ident = Ident::new(InstructionListAttribute::IDENT, Span::call_site());
        let variant_attr_ident =
            Ident::new(InstructionListVariantAttribute::IDENT, Span::call_site());
        let derive_input: DeriveInput = input.parse()?;

        let instruction_list_attribute = find_attr(derive_input.attrs, &instruction_list_ident)
            .as_ref()
            .map(InstructionListAttribute::parse_arguments);

        let variants = match derive_input.data {
            Data::Struct(_) | Data::Union(_) => {
                abort_call_site!("derive `InstructionList` supports only enums")
            }
            Data::Enum(enum_data) => enum_data.variants,
        };

        let variants = variants
            .into_iter()
            .map(|variant| InstructionListVariant::from_variant(variant, &variant_attr_ident))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self {
            vis: derive_input.vis,
            ident: derive_input.ident,
            generics: derive_input.generics,
            attribute: instruction_list_attribute,
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

        let enum_ident = self
            .attribute
            .and_then(|a| a.build_enum_ident)
            .unwrap_or_else(|| {
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
                variant_discriminant.push(
                    variant
                        .discriminant
                        .map(|expr| quote! { #expr })
                        .unwrap_or_else(|| {
                            let last = variant_discriminant
                                .last()
                                .cloned()
                                .unwrap_or_else(|| quote! { 0 });
                            quote! {
                                #last + 1
                            }
                        }),
                );
            }
            (
                variant_ident,
                variant_instruction_type,
                variant_discriminant,
            )
        };

        let instruction_prints = variant_ident
            .iter()
            .map(|ident| LitStr::new(&format!("Instruction: {}", ident), ident.span()));

        quote! {
            #[automatically_derived]
            impl #impl_generics InstructionList for #ident #ty_generics #where_clause{
                type BuildEnum = #enum_ident;

                fn build_instruction(
                    program_id: #crate_name::Pubkey,
                    build_enum: Self::BuildEnum,
                ) -> #crate_name::GeneratorResult<#crate_name::SolanaInstruction>{
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

struct InstructionListVariant {
    ident: Ident,
    discriminant: Option<Expr>,
    attribute: InstructionListVariantAttribute,
}
impl InstructionListVariant {
    fn from_variant(value: Variant, attr_ident: &Ident) -> Result<Self, syn::Error> {
        match &value.fields {
            Fields::Unit => {}
            _ => abort!(
                value,
                "derive `InstructionList` only supports unit enum values"
            ),
        }

        let attribute = InstructionListVariantAttribute::parse_arguments(
            find_attr(value.attrs.iter(), attr_ident).unwrap_or_else(|| {
                abort!(value, "Variant missing `{}` attribute", attr_ident);
            }),
        );

        Ok(Self {
            ident: value.ident,
            discriminant: value.discriminant.map(|val| val.1),
            attribute,
        })
    }
}

#[derive(ArgumentList)]
struct InstructionListVariantAttribute {
    instruction_type: Type,
}
impl InstructionListVariantAttribute {
    const IDENT: &'static str = "instruction";
}
