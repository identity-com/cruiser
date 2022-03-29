//! The escrow program from the paulx blog

use cruiser::account_argument::{AccountArgument, Single};
use cruiser::account_list::AccountList;
use cruiser::account_types::close_account::CloseAccount;
use cruiser::account_types::data_account::DataAccount;
use cruiser::account_types::init_account::InitArgs;
use cruiser::account_types::init_or_zeroed_account::InitOrZeroedAccount;
use cruiser::account_types::rent_exempt::RentExempt;
use cruiser::account_types::seeds::{Find, Seeds};
use cruiser::account_types::system_program::SystemProgram;
use cruiser::borsh::{BorshDeserialize, BorshSerialize};
use cruiser::instruction::{Instruction, InstructionProcessor};
use cruiser::instruction_list::InstructionList;
use cruiser::on_chain_size::{OnChainSize, OnChainStaticSize};
use cruiser::pda_seeds::{PDAGenerator, PDASeed, PDASeeder};
use cruiser::spl::token::{Owner, TokenAccount, TokenProgram};
use cruiser::CPIChecked;
use cruiser::{
    borsh, entrypoint_list, msg, AccountInfo, CruiserResult, GenericError, Pubkey,
    ToSolanaAccountInfo,
};
use std::iter::empty;

entrypoint_list!(EscrowInstructions, EscrowInstructions);

#[derive(InstructionList, Copy, Clone)]
#[instruction_list(account_list = EscrowAccounts, account_info = [<'a, AI> AI where AI: ToSolanaAccountInfo<'a>])]
pub enum EscrowInstructions {
    #[instruction(instruction_type = InitEscrow)]
    InitEscrow,
    #[instruction(instruction_type = InitEscrow)]
    Exchange,
}
pub mod client {
    use crate::{BorshSerialize, EscrowInstructions, Pubkey};
    use cruiser::account_argument::ToSolanaAccountMeta;
    use cruiser::instruction_list::{
        InstructionList, InstructionListClient, InstructionListClientStatic,
    };
    use cruiser::{CruiserResult, SolanaInstruction};

    pub struct InitEscrow<'a, AI> {
        accounts: [&'a AI; 6],
        data: Option<Vec<u8>>,
    }
    impl<'a, AI> InitEscrow<'a, AI> {
        pub fn new(
            initializer: &'a AI,
            temp_token_account: &'a AI,
            initializer_token_account: &'a AI,
            escrow_account: &'a AI,
            token_program: &'a AI,
            system_program: &'a AI,
            amount: u64,
        ) -> CruiserResult<Self> {
            let mut data = Vec::with_capacity(8 + 8);
            EscrowInstructions::InitEscrow
                .discriminant_compressed()
                .serialize(&mut data)?;
            amount.serialize(&mut data)?;
            Ok(Self {
                accounts: [
                    initializer,
                    temp_token_account,
                    initializer_token_account,
                    escrow_account,
                    token_program,
                    system_program,
                ],
                data: Some(data),
            })
        }
    }
    impl<'a, 'b, AI> InstructionListClient<EscrowInstructions> for InitEscrow<'a, AI>
    where
        AI: ToSolanaAccountMeta,
    {
        type AccountInfo = AI;

        fn instruction(&mut self, program_id: &Pubkey) -> SolanaInstruction {
            SolanaInstruction {
                program_id: *program_id,
                accounts: self
                    .accounts
                    .into_iter()
                    .map(|account| account.to_solana_account_meta())
                    .collect(),
                data: self.data.take().unwrap(),
            }
        }
    }
    impl<'a, AI> InstructionListClientStatic<EscrowInstructions, 7> for InitEscrow<'a, AI>
    where
        AI: ToSolanaAccountMeta,
    {
        fn to_accounts_static<'b>(&'b self, program_account: &'b AI) -> [&'b AI; 7] {
            // TODO: Replace this with a const push operation when willing to go to const generics
            [
                self.accounts[0],
                self.accounts[1],
                self.accounts[2],
                self.accounts[3],
                self.accounts[4],
                self.accounts[5],
                program_account,
            ]
        }
    }

    // pub struct
}

#[derive(AccountList)]
pub enum EscrowAccounts {
    EscrowAccount(EscrowAccount),
}

#[derive(BorshSerialize, BorshDeserialize, Default)]
pub struct EscrowAccount {
    pub initializer: Pubkey,
    pub temp_token_account: Pubkey,
    pub initializer_token_to_receive: Pubkey,
    pub expected_amount: u64,
}
impl OnChainSize<()> for EscrowAccount {
    fn on_chain_max_size(_arg: ()) -> usize {
        Pubkey::on_chain_static_size() * 3 + u64::on_chain_static_size()
    }
}

#[derive(Debug)]
struct EscrowPDASeeder;
impl PDASeeder for EscrowPDASeeder {
    fn seeds<'a>(&'a self) -> Box<dyn Iterator<Item = &'a dyn PDASeed> + 'a> {
        Box::new([&"escrow" as &dyn PDASeed].into_iter())
    }
}

pub struct InitEscrow;
impl<AI> Instruction<AI> for InitEscrow
where
    AI: AccountInfo,
{
    type Data = InitEscrowData;
    type Accounts = InitEscrowAccounts<AI>;
}
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
#[derive(BorshSerialize, BorshDeserialize)]
pub struct InitEscrowData {
    amount: u64,
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
        space: EscrowAccount::on_chain_static_size(),
        system_program: &self.system_program,
        account_seeds: None,
        cpi: CPIChecked,
    },))]
    escrow_account: RentExempt<InitOrZeroedAccount<AI, EscrowAccounts, EscrowAccount>>,
    token_program: TokenProgram<AI>,
    system_program: SystemProgram<AI>,
}

pub struct Exchange;
impl<AI> Instruction<AI> for Exchange
where
    AI: AccountInfo,
{
    type Data = ExchangeData;
    type Accounts = ExchangeAccounts<AI>;
}
impl<'a, AI> InstructionProcessor<AI, Exchange> for Exchange
where
    AI: ToSolanaAccountInfo<'a> + Clone,
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
#[derive(BorshSerialize, BorshDeserialize)]
pub struct ExchangeData {
    amount: u64,
}
#[derive(AccountArgument)]
#[account_argument(account_info = AI)]
pub struct ExchangeAccounts<AI>
where
    AI: AccountInfo,
{
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
