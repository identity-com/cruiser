use std::ops::Deref;

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::on_chain_size::OnChainSize;
use crate::CruiserResult;
use cruiser::AccountInfo;
use solana_program::program_pack::Pack;
use solana_program::pubkey::Pubkey;

use crate::spl::token::TokenProgramAccount;

// verify_account_arg_impl! {
//     mod mint_account_check<AI>{
//         <AI> MintAccount<AI> where AI: AccountInfo{
//             from: [()];
//             validate: [()];
//             multi: [<I> I where TokenProgramAccount<AI>: MultiIndexable<AI, I>];
//             single: [<I> I where TokenProgramAccount<AI>: SingleIndexable<AI, I>];
//         }
//     }
// }

/// A Mint account owned by the token program
#[derive(Debug)]
pub struct MintAccount<AI> {
    data: spl_token::state::Mint,
    /// The account associated
    pub account: TokenProgramAccount<AI>,
}

impl const OnChainSize for spl_token::state::Mint {
    /// Pulled from packed source
    const ON_CHAIN_SIZE: usize = 82;
}

impl<AI> OnChainSize for MintAccount<AI> {
    const ON_CHAIN_SIZE: usize = spl_token::state::Mint::ON_CHAIN_SIZE;
}

impl<AI> Deref for MintAccount<AI>
where
    AI: AccountInfo,
{
    type Target = spl_token::state::Mint;

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl<AI> AccountArgument for MintAccount<AI>
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

impl<AI> FromAccounts for MintAccount<AI>
where
    AI: AccountInfo,
{
    fn from_accounts(
        program_id: &Pubkey,
        infos: &mut impl AccountInfoIterator<Item = AI>,
        arg: (),
    ) -> CruiserResult<Self> {
        let account: TokenProgramAccount<AI> = FromAccounts::from_accounts(program_id, infos, arg)?;
        let data = spl_token::state::Mint::unpack(&*account.0.data())?;
        Ok(Self { data, account })
    }

    fn accounts_usage_hint(arg: &()) -> (usize, Option<usize>) {
        TokenProgramAccount::<AI>::accounts_usage_hint(arg)
    }
}

impl<AI> ValidateArgument for MintAccount<AI>
where
    AI: AccountInfo,
{
    fn validate(&mut self, _program_id: &Pubkey, _arg: ()) -> CruiserResult<()> {
        Ok(())
    }
}

impl<AI, I> MultiIndexable<I> for MintAccount<AI>
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

impl<AI, I> SingleIndexable<I> for MintAccount<AI>
where
    AI: AccountInfo,
    TokenProgramAccount<AI>: SingleIndexable<I, AccountInfo = AI>,
{
    fn index_info(&self, indexer: I) -> CruiserResult<&AI> {
        self.account.index_info(indexer)
    }
}
