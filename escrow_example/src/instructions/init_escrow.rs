use crate::{EscrowAccount, EscrowAccounts};
use cruiser::prelude::*;

pub struct InitEscrow;

impl<AI> Instruction<AI> for InitEscrow {
    type Accounts = InitEscrowAccounts<AI>;
    type Data = InitEscrowData;
    type ReturnType = ();
}

#[derive(AccountArgument)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
#[validate(generics = [< 'a > where AI: ToSolanaAccountInfo < 'a >])]
pub struct InitEscrowAccounts<AI> {
    #[validate(signer)]
    initializer: AI,
    #[validate(writable, data = TokenAccountOwner(self.initializer.key()))]
    temp_token_account: TokenAccount<AI>,
    initializer_token_account: TokenAccount<AI>,
    #[from(data = EscrowAccount::default())]
    #[validate(writable, data = (InitArgs{
        funder: Some(&self.initializer),
        funder_seeds: None,
        rent: None,
        space: InitStaticSized,
        system_program: self.system_program.as_ref(),
        account_seeds: None,
        cpi: CPIChecked,
    },))]
    escrow_account: RentExempt<InitOrZeroedAccount<AI, EscrowAccounts, EscrowAccount>>,
    token_program: TokenProgram<AI>,
    system_program: Option<SystemProgram<AI>>,
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
