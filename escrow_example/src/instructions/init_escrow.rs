use crate::{EscrowAccount, EscrowAccounts};
use cruiser::account_argument::AccountArgument;
use cruiser::account_types::init_account::InitArgs;
use cruiser::account_types::init_or_zeroed_account::InitOrZeroedAccount;
use cruiser::account_types::rent_exempt::RentExempt;
use cruiser::account_types::system_program::SystemProgram;
use cruiser::borsh::{BorshDeserialize, BorshSerialize};
use cruiser::instruction::Instruction;
use cruiser::on_chain_size::OnChainStaticSize;
use cruiser::spl::token::{Owner, TokenAccount, TokenProgram};
use cruiser::{borsh, AccountInfo, CPIChecked, ToSolanaAccountInfo};

pub struct InitEscrow;
impl<AI> Instruction<AI> for InitEscrow
where
    AI: AccountInfo,
{
    type Accounts = InitEscrowAccounts<AI>;
    type Data = InitEscrowData;
}
#[derive(AccountArgument)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
#[validate(generics = [<'a> where AI: ToSolanaAccountInfo<'a>])]
pub struct InitEscrowAccounts<AI> {
    #[validate(signer)]
    initializer: AI,
    #[validate(writable, data = Owner(self.initializer.key()))]
    temp_token_account: TokenAccount<AI>,
    initializer_token_account: TokenAccount<AI>,
    #[from(data = EscrowAccount::default())]
    #[validate(writable, data = (InitArgs{
        funder: &self.initializer,
        funder_seeds: None,
        rent: None,
        space: EscrowAccount::on_chain_static_size() + 8,
        system_program: &self.system_program,
        account_seeds: None,
        cpi: CPIChecked,
    },))]
    escrow_account: RentExempt<InitOrZeroedAccount<AI, EscrowAccounts, EscrowAccount>>,
    token_program: TokenProgram<AI>,
    system_program: SystemProgram<AI>,
}
#[derive(BorshSerialize, BorshDeserialize)]
pub struct InitEscrowData {
    amount: u64,
}
#[cfg(feature = "processor")]
mod processor {
    use super::*;
    use crate::EscrowPDASeeder;
    use cruiser::account_argument::Single;
    use cruiser::instruction::InstructionProcessor;
    use cruiser::pda_seeds::PDAGenerator;
    use cruiser::{msg, CruiserResult, Pubkey};
    use std::iter::empty;

    impl<'a, AI> InstructionProcessor<AI, InitEscrow> for InitEscrow
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        type FromAccountsData = ();
        type ValidateData = ();
        type InstructionData = <InitEscrow as Instruction<AI>>::Data;

        fn data_to_instruction_arg(
            data: <InitEscrow as Instruction<AI>>::Data,
        ) -> CruiserResult<(
            Self::FromAccountsData,
            Self::ValidateData,
            Self::InstructionData,
        )> {
            Ok(((), (), data))
        }

        fn process(
            program_id: &Pubkey,
            data: Self::InstructionData,
            accounts: &mut <Self as Instruction<AI>>::Accounts,
        ) -> CruiserResult<()> {
            let escrow_account = &mut accounts.escrow_account;
            escrow_account.initializer = *accounts.initializer.key();
            escrow_account.temp_token_account = *accounts.temp_token_account.info().key();
            escrow_account.initializer_token_to_receive =
                *accounts.initializer_token_account.info().key();
            escrow_account.expected_amount = data.amount;

            let (pda, _) = EscrowPDASeeder.find_address(program_id);

            msg!("Calling the token program to transfer token account ownership...");
            accounts.token_program.set_authority(
                CPIChecked,
                &accounts.temp_token_account,
                &pda,
                &accounts.initializer,
                empty(),
            )?;

            Ok(())
        }
    }
}
