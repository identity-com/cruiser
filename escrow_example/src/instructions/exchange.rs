use crate::{EscrowAccount, EscrowAccounts, EscrowPDASeeder};
use cruiser::account_argument::AccountArgument;
use cruiser::account_types::close_account::CloseAccount;
use cruiser::account_types::data_account::DataAccount;
use cruiser::account_types::seeds::{Find, Seeds};
use cruiser::borsh::{BorshDeserialize, BorshSerialize};
use cruiser::instruction::Instruction;
use cruiser::spl::token::{Owner, TokenAccount, TokenProgram};
use cruiser::{borsh, AccountInfo};

pub struct Exchange;
impl<AI> Instruction<AI> for Exchange {
    type Accounts = ExchangeAccounts<AI>;
    type Data = ExchangeData;
}

#[derive(AccountArgument)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
pub struct ExchangeAccounts<AI> {
    #[validate(signer)]
    taker: AI,
    #[validate(writable, data = Owner(self.taker.key()))]
    taker_send_token_account: TokenAccount<AI>,
    #[validate(writable)]
    taker_receive_token_account: TokenAccount<AI>,
    #[validate(writable, key = &self.escrow_account.temp_token_account)]
    temp_token_account: TokenAccount<AI>,
    #[validate(writable, key = &self.escrow_account.initializer)]
    initializer: AI,
    #[validate(writable, key = &self.escrow_account.initializer_token_to_receive)]
    initializer_token_account: TokenAccount<AI>,
    #[validate(writable)]
    escrow_account: CloseAccount<AI, DataAccount<AI, EscrowAccounts, EscrowAccount>>,
    token_program: TokenProgram<AI>,
    #[validate(data = (EscrowPDASeeder, Find))]
    pda_account: Seeds<AI, EscrowPDASeeder>,
}
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ExchangeData {
    amount: u64,
}

#[cfg(feature = "processor")]
mod processor {
    use super::*;
    use cruiser::account_argument::Single;
    use cruiser::instruction::InstructionProcessor;
    use cruiser::{msg, CPIChecked, CruiserResult, GenericError, Pubkey, ToSolanaAccountInfo};
    use std::iter::empty;

    impl<'a, AI> InstructionProcessor<AI, Exchange> for Exchange
    where
        AI: ToSolanaAccountInfo<'a>,
    {
        type FromAccountsData = ();
        type ValidateData = ();
        type InstructionData = <Self as Instruction<AI>>::Data;

        fn data_to_instruction_arg(
            data: <Self as Instruction<AI>>::Data,
        ) -> CruiserResult<(
            Self::FromAccountsData,
            Self::ValidateData,
            Self::InstructionData,
        )> {
            Ok(((), (), data))
        }

        fn process(
            _program_id: &Pubkey,
            data: <Self as Instruction<AI>>::Data,
            accounts: &mut <Self as Instruction<AI>>::Accounts,
        ) -> CruiserResult<()> {
            if data.amount != accounts.escrow_account.expected_amount {
                return Err(GenericError::Custom {
                    error: format!(
                        "Amount (`{}`) did not equal expected (`{}`)",
                        data.amount, accounts.escrow_account.expected_amount
                    ),
                }
                .into());
            }

            msg!("Calling the token program to transfer tokens to the escrow's initializer...");
            accounts.token_program.transfer(
                CPIChecked,
                &accounts.taker_send_token_account,
                &accounts.initializer_token_account,
                &accounts.taker,
                accounts.escrow_account.expected_amount,
                empty(),
            )?;

            let seeds = accounts.pda_account.take_seed_set().unwrap();
            msg!("Calling the token program to transfer tokens to the taker...");
            accounts.token_program.transfer(
                CPIChecked,
                &accounts.temp_token_account,
                &accounts.taker_receive_token_account,
                accounts.pda_account.info(),
                accounts.temp_token_account.amount,
                [&seeds],
            )?;

            msg!("Calling the token program to close pda's temp account...");
            accounts.token_program.close_account(
                CPIChecked,
                &accounts.temp_token_account,
                &accounts.initializer,
                accounts.pda_account.info(),
                [&seeds],
            )?;

            msg!("Closing the escrow account...");
            accounts
                .escrow_account
                .set_fundee(accounts.initializer.clone());
            Ok(())
        }
    }
}
