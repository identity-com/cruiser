use std::ops::Deref;

use crate::account_argument::{
    AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable, ValidateArgument,
};
use crate::on_chain_size::OnChainSize;
use crate::{AccountInfo, CruiserResult, GenericError};
use cruiser::account_argument::AccountArgument;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use crate::spl::token::TokenProgramAccount;

// verify_account_arg_impl! {
//     mod token_account_check<AI>{
//         <AI> TokenAccount<AI> where AI: AccountInfo{
//             from: [()];
//             validate: [(); <'a> Owner<'a>];
//             multi: [<I> I where TokenProgramAccount<AI>: MultiIndexable<AI, I>];
//             single: [<I> I where TokenProgramAccount<AI>: SingleIndexable<AI, I>];
//         }
//     }
// }

/// A token account owned by the token program
#[derive(Debug, Clone)]
pub struct TokenAccount<AI> {
    data: spl_token::state::Account,
    /// The account associated
    pub account: TokenProgramAccount<AI>,
}

impl const OnChainSize for spl_token::state::Account {
    /// Pulled from packed source
    const ON_CHAIN_SIZE: usize = 165;
}

impl<AI> OnChainSize for TokenAccount<AI> {
    /// Pulled from packed source
    const ON_CHAIN_SIZE: usize = spl_token::state::Account::ON_CHAIN_SIZE;
}

impl<AI> Deref for TokenAccount<AI> {
    type Target = spl_token::state::Account;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<AI> AccountArgument for TokenAccount<AI>
where
    AI: AccountInfo,
{
    type AccountInfo = AI;

    fn write_back(self, program_id: &Pubkey) -> CruiserResult<()> {
        self.account.write_back(program_id)
    }

    fn add_keys(&self, add: impl FnMut(Pubkey) -> CruiserResult<()>) -> CruiserResult<()> {
        self.account.add_keys(add)
    }
}

impl<AI> FromAccounts for TokenAccount<AI>
where
    AI: AccountInfo,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = AI>,
        arg: (),
    ) -> CruiserResult<Self> {
        let account: TokenProgramAccount<AI> = FromAccounts::from_accounts(program_id, infos, arg)?;
        let data = spl_token::state::Account::unpack(&*account.data())?;
        Ok(Self { data, account })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        TokenProgramAccount::<AI>::accounts_usage_hint(arg)
    }
}

impl<AI> ValidateArgument for TokenAccount<AI>
where
    AI: AccountInfo,
{
    fn validate(&mut self, program_id: &Pubkey, arg: ()) -> CruiserResult<()> {
        self.account.validate(program_id, arg)?;
        Ok(())
    }
}

/// Validates that the given key is the owner of the [`TokenAccount`]
#[derive(Debug, Copy, Clone)]
pub struct TokenAccountOwner<'a>(pub &'a Pubkey);

impl<AI> ValidateArgument<TokenAccountOwner<'_>> for TokenAccount<AI>
where
    AI: AccountInfo,
{
    fn validate(&mut self, program_id: &Pubkey, arg: TokenAccountOwner) -> CruiserResult<()> {
        self.validate(program_id, ())?;
        if &self.data.owner == arg.0 {
            Ok(())
        } else {
            Err(GenericError::InvalidAccount {
                account: self.data.owner,
                expected: *arg.0,
            }
            .into())
        }
    }
}

/// Validates that the given key is the mint of the [`TokenAccount`]
#[derive(Debug, Copy, Clone)]
pub struct TokenAccountMint<'a>(pub &'a Pubkey);

impl<AI> ValidateArgument<TokenAccountMint<'_>> for TokenAccount<AI>
where
    AI: AccountInfo,
{
    fn validate(&mut self, program_id: &Pubkey, arg: TokenAccountMint) -> CruiserResult<()> {
        self.validate(program_id, ())?;
        if &self.data.mint == arg.0 {
            Ok(())
        } else {
            Err(GenericError::InvalidAccount {
                account: self.data.mint,
                expected: *arg.0,
            }
            .into())
        }
    }
}

/// Validates that the account has a given owner and mint
#[derive(Debug, Copy, Clone)]
pub struct TokenAccountOwnerAndMint<'a> {
    owner: &'a Pubkey,
    mint: &'a Pubkey,
}

impl<AI> ValidateArgument<TokenAccountOwnerAndMint<'_>> for TokenAccount<AI>
where
    AI: AccountInfo,
{
    fn validate(
        &mut self,
        program_id: &Pubkey,
        arg: TokenAccountOwnerAndMint,
    ) -> CruiserResult<()> {
        self.validate(program_id, ())?;
        if &self.data.owner != arg.owner {
            Err(GenericError::InvalidAccount {
                account: self.data.owner,
                expected: *arg.owner,
            }
            .into())
        } else if &self.data.mint != arg.mint {
            Err(GenericError::InvalidAccount {
                account: self.data.mint,
                expected: *arg.mint,
            }
            .into())
        } else {
            Ok(())
        }
    }
}

impl<AI, I> MultiIndexable<I> for TokenAccount<AI>
where
    AI: AccountInfo,
    TokenProgramAccount<AI>: MultiIndexable<I>,
{
    fn index_is_signer(&self, indexer: I) -> CruiserResult<bool> {
        self.account.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: I) -> CruiserResult<bool> {
        self.account.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: I) -> CruiserResult<bool> {
        self.account.index_is_owner(owner, indexer)
    }
}

impl<AI, I> SingleIndexable<I> for TokenAccount<AI>
where
    AI: AccountInfo,
    TokenProgramAccount<AI>: SingleIndexable<I, AccountInfo = AI>,
{
    fn index_info(&self, indexer: I) -> CruiserResult<&AI> {
        self.account.index_info(indexer)
    }
}
