use proc_macro2::{Span, TokenStream};
use proc_macro_error::{abort, abort_call_site};
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{Data, DeriveInput, Expr, Fields, Generics, Ident, LitStr, Type, Variant};

use easy_proc::{find_attr, ArgumentList};

use crate::get_crate_name;
use crate::log_level::LogLevel;

#[derive(ArgumentList)]
pub struct InstructionListAttribute {
    #[argument(default = syn::parse_str("u64").unwrap())]
    discriminant_type: Type,
    #[argument(default)]
    log_level: LogLevel,
    #[argument(presence)]
    no_processor: bool,
    account_list: Type,
}
impl InstructionListAttribute {
    const IDENT: &'static str = "instruction_list";
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
            .map(InstructionListAttribute::parse_arguments)
            .unwrap_or_else(|| abort!(derive_input.ident, "Missing `instruction_list` attribute"));

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
        let (impl_generics, ty_generics, where_clause) = self.generics.split_for_impl();

        let discriminant_type = self.attribute.discriminant_type;
        let log_level = self.attribute.log_level;
        let account_list = self.attribute.account_list;

        let (variant_ident, variant_instruction_type, variant_discriminant, variant_processors) = {
            let mut variant_idents = Vec::with_capacity(self.variants.len());
            let mut variant_instruction_type = Vec::with_capacity(self.variants.len());
            let mut variant_discriminant = Vec::with_capacity(self.variants.len());
            let mut variant_processors = Vec::with_capacity(self.variants.len());
            for variant in self.variants {
                let instruction_type = &variant.attribute.instruction_type;
                variant_processors.push(
                    variant
                        .attribute
                        .processor
                        .unwrap_or_else(|| instruction_type.clone()),
                );
                variant_idents.push(variant.ident);
                variant_instruction_type.push(variant.attribute.instruction_type);
                variant_discriminant.push(
                    variant
                        .discriminant
                        .map(|expr| quote! { #expr })
                        .unwrap_or_else(|| {
                            variant_discriminant
                                .last()
                                .cloned()
                                .map(|last| quote! { (#last) + 1 })
                                .unwrap_or_else(|| quote! { 0 })
                        }),
                );
            }
            (
                variant_idents,
                variant_instruction_type,
                variant_discriminant,
                variant_processors,
            )
        };

        let processor = if self.attribute.no_processor {
            quote! {}
        } else {
            let instruction_prints = variant_ident.iter().map(|ident| {
                log_level.if_level(LogLevel::Info, |_| {
                    let message = LitStr::new(&format!("Instruction: {}", ident), ident.span());
                    quote! {
                        #crate_name::msg!(#message);
                    }
                })
            });

            quote! {
                #[automatically_derived]
                impl #impl_generics #crate_name::instruction_list::InstructionListProcessor<#ident> for #ident #ty_generics #where_clause{
                    fn process_instruction(
                        program_id: &'static #crate_name::Pubkey,
                        accounts: &mut impl #crate_name::account_argument::AccountInfoIterator,
                        mut data: &[u8],
                    ) -> #crate_name::CruiserResult<()>{
                        let discriminant = <<Self as #crate_name::instruction_list::InstructionList>::DiscriminantCompressed as #crate_name::borsh::BorshDeserialize>::deserialize(&mut data)?;
                        let discriminant = <<Self as #crate_name::instruction_list::InstructionList>::DiscriminantCompressed as #crate_name::compressed_numbers::CompressedNumber>::into_number(discriminant);
                        if false{
                            ::std::unreachable!();
                        }
                        #(else if discriminant == #variant_discriminant{
                            #instruction_prints
                            let mut data = <<#variant_instruction_type as #crate_name::instruction::Instruction>::Data as #crate_name::borsh::BorshDeserialize>::deserialize(&mut data)?;
                            let (from_data, validate_data, instruction_data) = <#variant_instruction_type as #crate_name::instruction::Instruction>::data_to_instruction_arg(data)?;
                            let mut accounts = <<#variant_instruction_type as #crate_name::instruction::Instruction>::Accounts as #crate_name::account_argument::FromAccounts<_>>::from_accounts(program_id, accounts, from_data)?;
                            #crate_name::account_argument::ValidateArgument::validate(&mut accounts, program_id, validate_data)?;
                            <#variant_processors as #crate_name::instruction::InstructionProcessor<#variant_instruction_type>>::process(
                                program_id,
                                instruction_data,
                                &mut accounts,
                            )?;
                            <<#variant_instruction_type as #crate_name::instruction::Instruction>::Accounts as #crate_name::account_argument::AccountArgument>::write_back(accounts, program_id)?;
                            Ok(())
                        })* else{
                            todo!();
                        }
                    }
                }
            }
        };

        quote! {
            #[automatically_derived]
            impl #impl_generics InstructionList for #ident #ty_generics #where_clause{
                type DiscriminantCompressed = #discriminant_type;
                type AccountList = #account_list;

                fn discriminant(self) -> ::std::num::NonZeroU64{
                    match self{
                        #(Self::#variant_ident => #crate_name::util::ToNonZero::to_non_zero(#variant_discriminant),)*
                    }
                }

                fn from_discriminant(discriminant: ::std::num::NonZeroU64) -> Option<Self>{
                    let discriminant = discriminant.get();
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

            #processor
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
    processor: Option<Type>,
}
impl InstructionListVariantAttribute {
    const IDENT: &'static str = "instruction";
}
