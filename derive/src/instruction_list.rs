use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site};
use quote::{quote, ToTokens};
use syn::parse::{Parse, ParseStream};
use syn::{
    bracketed, token, Data, DeriveInput, Expr, Fields, Generics, Ident, LitStr, Type, Variant,
    WhereClause,
};

use easy_proc::{find_attr, ArgumentList};

use crate::get_crate_name;
use crate::log_level::LogLevel;

#[derive(ArgumentList)]
struct InstructionListAttribute {
    #[argument(default = syn::parse_str("u64").unwrap())]
    discriminant_type: Type,
    #[argument(default)]
    log_level: LogLevel,
    #[argument(default = syn::parse_str("\"processor\"").unwrap())]
    processor_feature: LitStr,
    #[argument(presence)]
    no_processor: bool,
    account_info: AccountInfoArg,
    account_list: Type,
}
impl InstructionListAttribute {
    const IDENT: &'static str = "instruction_list";
}

struct AccountInfoArg {
    bracket: token::Bracket,
    generics: Generics,
    ty: Type,
    where_clause: Option<WhereClause>,
}
impl Parse for AccountInfoArg {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let bracket = bracketed!(content in input);
        Ok(Self {
            bracket,
            generics: content.parse()?,
            ty: content.parse()?,
            where_clause: content.parse()?,
        })
    }
}
impl ToTokens for AccountInfoArg {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        self.bracket.surround(tokens, |tokens| {
            self.generics.to_tokens(tokens);
            self.ty.to_tokens(tokens);
            self.where_clause.to_tokens(tokens);
        });
    }
}

pub struct InstructionListDerive {
    ident: Ident,
    generics: Generics,
    attribute: InstructionListAttribute,
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
            .map_or_else(
                || abort!(derive_input.ident, "Missing `instruction_list` attribute"),
                InstructionListAttribute::parse_arguments,
            );

        let variants = match derive_input.data {
            Data::Struct(_) | Data::Union(_) => {
                abort_call_site!("derive `InstructionList` supports only enums")
            }
            Data::Enum(enum_data) => enum_data.variants,
        };

        let variants: Vec<_> = variants
            .into_iter()
            .map(|variant| InstructionListVariant::from_variant(variant, &variant_attr_ident))
            .collect();

        if instruction_list_attribute.no_processor {
            for variant in &variants {
                if let Some(processor) = &variant.attribute.processor {
                    abort!(processor, "`no_processor` passed for instruction list");
                }
            }
        }

        Ok(Self {
            ident: derive_input.ident,
            generics: derive_input.generics,
            attribute: instruction_list_attribute,
            variants,
        })
    }
}
impl InstructionListDerive {
    pub fn into_token_stream(self) -> TokenStream {
        let crate_name = get_crate_name();

        let ident = self.ident;
        let mut generics = self.generics.clone();
        generics
            .params
            .extend(self.attribute.account_info.generics.params.into_iter());
        if let Some(where_clause) = self.attribute.account_info.generics.where_clause {
            generics
                .make_where_clause()
                .predicates
                .extend(where_clause.predicates.into_iter());
        }
        if let Some(where_clause) = self.attribute.account_info.where_clause {
            generics
                .make_where_clause()
                .predicates
                .extend(where_clause.predicates.into_iter());
        }
        let (impl_generics, _, where_clause) = generics.split_for_impl();
        let (main_impl_generics, ty_generics, main_where_clause) = self.generics.split_for_impl();
        let account_info_ty = self.attribute.account_info.ty;

        let discriminant_type = self.attribute.discriminant_type;
        let log_level = self.attribute.log_level;
        let account_list = self.attribute.account_list;

        let (variant_ident, variant_instruction_type, variant_discriminant, variant_processors) =
            Self::split_variants(self.variants);

        let processor = if self.attribute.no_processor {
            TokenStream::new()
        } else {
            let instruction_prints = variant_ident.iter().map(|ident| {
                log_level.if_level(LogLevel::Info, |_| {
                    let message = LitStr::new(&format!("Instruction: {}", ident), ident.span());
                    quote! {
                        #crate_name::msg!(#message);
                    }
                })
            });
            let processor_feature = self.attribute.processor_feature;

            quote! {
                #[cfg(feature = #processor_feature)]
                #[automatically_derived]
                impl #impl_generics #crate_name::instruction_list::InstructionListProcessor<#account_info_ty, #ident> for #ident #ty_generics #where_clause{
                    fn process_instruction(
                        program_id: &#crate_name::Pubkey,
                        accounts: &mut impl #crate_name::account_argument::AccountInfoIterator<Item = #account_info_ty>,
                        mut data: &[u8],
                    ) -> #crate_name::CruiserResult<()>{
                        let discriminant = <<Self as #crate_name::instruction_list::InstructionList>::DiscriminantCompressed as #crate_name::borsh::BorshDeserialize>::deserialize(&mut data)?;
                        let discriminant = <<Self as #crate_name::instruction_list::InstructionList>::DiscriminantCompressed as #crate_name::compressed_numbers::CompressedNumber>::into_number(discriminant);
                        if false{
                            ::std::unreachable!();
                        }
                        #(else if discriminant == #variant_discriminant{
                            #instruction_prints
                            #crate_name::util::process_instruction::<#account_info_ty, #variant_instruction_type, #variant_processors, _>(program_id, accounts, data)
                        })* else{
                            todo!();
                        }
                    }
                }
            }
        };

        let list_items = variant_instruction_type.iter().zip(variant_discriminant.iter()).map(|(instruction_type, discriminant)|{
            quote! {
                #[automatically_derived]
                unsafe impl #main_impl_generics #crate_name::instruction_list::InstructionListItem<#instruction_type> for #ident #ty_generics #main_where_clause{
                    fn discriminant() -> u64{
                        #discriminant
                    }
                }
            }
        }).collect::<Vec<_>>();

        quote! {
            #[automatically_derived]
            impl #main_impl_generics #crate_name::instruction_list::InstructionList for #ident #ty_generics #main_where_clause{
                type DiscriminantCompressed = #discriminant_type;
                type AccountList = #account_list;

                fn from_discriminant(discriminant: u64) -> Option<Self>{
                    if false{
                        ::std::unreachable!();
                    }
                    #(else if discriminant == #variant_discriminant{
                        Some(Self::#variant_ident)
                    })*
                    else{
                        None
                    }
                }
            }

            #(#list_items)*
            #processor
        }
    }

    fn split_variants(
        variants: Vec<InstructionListVariant>,
    ) -> (Vec<Ident>, Vec<Type>, Vec<TokenStream>, Vec<Type>) {
        let mut variant_idents = Vec::with_capacity(variants.len());
        let mut variant_instruction_type = Vec::with_capacity(variants.len());
        let mut variant_discriminant = Vec::with_capacity(variants.len());
        let mut variant_processors = Vec::with_capacity(variants.len());
        for variant in variants {
            let instruction_type = &variant.attribute.instruction_type;
            variant_processors.push(
                variant
                    .attribute
                    .processor
                    .unwrap_or_else(|| instruction_type.clone()),
            );
            variant_idents.push(variant.ident);
            variant_instruction_type.push(variant.attribute.instruction_type);
            variant_discriminant.push(variant.discriminant.map_or_else(
                || {
                    variant_discriminant
                        .last()
                        .cloned()
                        .map_or_else(|| quote! { 0 }, |last| quote! { (#last) + 1 })
                },
                |expr| quote! { #expr },
            ));
        }
        (
            variant_idents,
            variant_instruction_type,
            variant_discriminant,
            variant_processors,
        )
    }
}

struct InstructionListVariant {
    ident: Ident,
    discriminant: Option<Expr>,
    attribute: InstructionListVariantAttribute,
}
impl InstructionListVariant {
    fn from_variant(value: Variant, attr_ident: &Ident) -> Self {
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

        Self {
            ident: value.ident,
            discriminant: value.discriminant.map(|val| val.1),
            attribute,
        }
    }
}

#[derive(ArgumentList)]
struct InstructionListVariantAttribute {
    instruction_type: Type,
    processor: Option<Type>,
}
impl InstructionListVariantAttribute {
    const IDENT: &'static str = "instruction";
}
