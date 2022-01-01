use std::convert::{TryFrom, TryInto};
use std::intrinsics::abort;
use proc_macro2::Span;
use proc_macro_error::{abort, abort_call_site};
use syn::{Generics, Visibility, Ident, DeriveInput, Data, Type, Attribute, Token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;
use test::RunIgnored::No;

pub struct InstructionListProcessorDerive {
    vis: Visibility,
    ident: Ident,
    generics: Generics,
    attribute: InstructionListProcessorAttribute,
    variants: Vec<InstructionListProcessorVariant>,
}
impl Parse for InstructionListProcessorDerive{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let derive_input: DeriveInput = input.parse()?;

        let attribute = derive_input.attrs.try_into()?;

        let variants = match derive_input.data {
            Data::Struct(_) | Data::Union(_) => {
                abort_call_site!("derive `InstructionListProcessor` supports only enums");
            },
            Data::Enum(enum_data) => enum_data.variants,
        };

        let variants = variants.into_iter()
            .map(InstructionListProcessorVariant::try_from)
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

// fn process_instruction(
//     program_id:#crate_name::Pubkey,
//     accounts: &mut impl #crate_name::AccountInfoIterator,
//     mut data: &[u8],
// ) -> #crate_name::GeneratorResult<()>{
// let data = &mut data;
// #[deny(unreachable_patterns)]
// match *#crate_name::Take::take_single(data)?{
// #(
// #variant_discriminant => {
// #crate_name::msg!(#instruction_prints);
// let mut instruction_data = ::borsh::BorshDeserialize::deserialize(data)?;
// let instruction_arg = <#variant_instruction_type as #crate_name::Instruction>::data_to_instruction_arg(&mut instruction_data)?;
// let mut accounts = #crate_name::FromAccounts::<_>::from_accounts(program_id, accounts, instruction_arg)?;
// let system_program = <#variant_instruction_type as #crate_name::Instruction>::process(program_id, instruction_data, &mut accounts)?;
// #crate_name::AccountArgument::write_back(accounts, program_id, system_program.as_ref())
// }
// )*
// 255 => ::std::result::Result::Err(#crate_name::GeneratorError::UnsupportedInterface.into()),
// #[allow(unreachable_patterns)]
// x => ::std::result::Result::Err(#crate_name::GeneratorError::UnknownInstruction {
// instruction: x.to_string(),
// }.into()),
// }
// }

struct InstructionListProcessorAttribute{
    instruction_list: Ident,
}
impl InstructionListProcessorAttribute{
    const IDENT: &'static str = "instruction_list_processor";

    fn build(attrs: &Vec<Attribute>, ident: &Ident) -> syn::Result<Self>{

    }
}
impl TryFrom<&Vec<Attribute>> for InstructionListProcessorAttribute{
    type Error = syn::Error;

    fn try_from(value: &Vec<Attribute>) -> Result<Self, Self::Error> {
        let mut attribute = None;
        let self_ident = Ident::new(Self::IDENT, Span::call_site());
        for attr in value{
            if attr.path.is_ident(&self_ident) && attribute.replace(attr.clone()).is_some(){
                abort!(attr, "Duplicate `{}` attribute", Self::IDENT);
            }
        }
        match attribute {
            None => abort_call_site!("Missing `{}` attribute", Self::IDENT),
            Some(attribute) => {
                let args: InstructionListProcessorArgs = attribute.parse_args()?;
                let mut instruction_list = None;
                Ok()
            }
        }
    }
}
struct InstructionListProcessorArgs(Punctuated<InstructionListProcessorAttributeArg, Token![,]>);
impl Parse for InstructionListProcessorArgs{
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self(
            input.parse_terminated(InstructionListProcessorAttributeArg::parse)?
        ))
    }
}

enum InstructionListProcessorAttributeArg{
    InstructionList(Ident),
}
