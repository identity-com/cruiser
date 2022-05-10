//! An account that represents a program written with cruiser and therefore easily callable.

use crate::account_argument::{AccountArgument, MultiIndexable, SingleIndexable};
use crate::account_types::PhantomAccount;
use crate::cpi::{CPIClientDynamic, CPIClientStatic, CPIMethod, InstructionAndAccounts};
use crate::instruction::ReturnValue;
use crate::instruction_list::InstructionListItem;
use crate::pda_seeds::PDASeedSet;
use crate::program::{CruiserProgram, Program, ProgramKey};
use crate::util::{get_return_data_buffered, MaybeOwned};
use crate::{AccountInfo, CruiserResult, ToSolanaAccountInfo};
use array_init::array_init;
use cruiser::instruction::Instruction;
use solana_program::pubkey::Pubkey;

// verify_account_arg_impl! {
//     mod cruiser_program_account_check<AI>{
//         <AI, P> CruiserProgramAccount<AI, P> where AI: AccountInfo, P: CruiserProgram{
//             from: [()];
//             validate: [()];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// A cruiser program that can be called with its client functions
#[derive(AccountArgument, Debug, Clone)]
#[account_argument(account_info = AI, generics = [where AI: AccountInfo])]
pub struct CruiserProgramAccount<AI, P>(#[validate(key = &P::KEY)] pub AI, PhantomAccount<AI, P>)
where
    P: CruiserProgram;
impl<'a, AI, P> CruiserProgramAccount<AI, P>
where
    AI: ToSolanaAccountInfo<'a>,
    P: CruiserProgram,
{
    /// Calls one of this program's functions that has statically sized account length
    pub fn invoke<'b, 'c: 'b, 'd, I, const N: usize>(
        &'d self,
        cpi: impl CPIMethod,
        instruction: I,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> CruiserResult<<I::Instruction as Instruction<AI>>::ReturnType>
    where
        P::InstructionList: InstructionListItem<I::Instruction>,
        I: CPIClientStatic<'d, N, InstructionList = P::InstructionList, AccountInfo = AI>,
    {
        let InstructionAndAccounts {
            instruction,
            accounts,
        } = instruction.instruction(&self.0);
        PDASeedSet::invoke_signed_multiple(
            cpi,
            &instruction,
            &array_init::<_, _, N>(|index| accounts[index].as_ref()),
            seeds,
        )?;
        Self::ret()
    }

    /// Calls one of this program's functions that has dynamically sized account length
    pub fn invoke_variable_sized<'b, 'c: 'b, 'd, I>(
        &'d self,
        cpi: impl CPIMethod,
        instruction: I,
        seeds: impl IntoIterator<Item = &'b PDASeedSet<'c>>,
    ) -> CruiserResult<<I::Instruction as Instruction<AI>>::ReturnType>
    where
        P::InstructionList: InstructionListItem<I::Instruction>,
        I: CPIClientDynamic<'d, InstructionList = P::InstructionList, AccountInfo = AI>,
    {
        let InstructionAndAccounts {
            instruction,
            accounts,
        } = instruction.instruction(&self.0);
        PDASeedSet::invoke_signed_variable_size_multiple(
            cpi,
            &instruction,
            accounts.iter().map(MaybeOwned::as_ref),
            seeds,
        )?;
        Self::ret()
    }

    fn ret<R: ReturnValue>() -> CruiserResult<R> {
        let max_size = R::max_size();
        if max_size > 0 {
            let mut buffer = vec![0; max_size];
            let mut return_program = Pubkey::new_from_array([0; 32]);
            let size = get_return_data_buffered(&mut buffer, &mut return_program)?;
            if return_program == Self::KEY {
                R::from_returned(Some(&mut buffer[0..size]), Some(&return_program))
            } else {
                #[allow(unused_qualifications)]
                R::from_returned(Option::<&mut [u8]>::None, Some(&return_program))
            }
        } else {
            #[allow(unused_qualifications)]
            R::from_returned(Option::<&mut [u8]>::None, None)
        }
    }
}
impl<AI, P> ProgramKey for CruiserProgramAccount<AI, P>
where
    P: CruiserProgram,
{
    const KEY: Pubkey = P::KEY;
}

impl<AI, P> Program for CruiserProgramAccount<AI, P>
where
    AI: AccountInfo,
    P: CruiserProgram,
{
}
impl<AI, P, I> MultiIndexable<I> for CruiserProgramAccount<AI, P>
where
    AI: AccountInfo + MultiIndexable<I>,
    P: CruiserProgram,
{
    fn index_is_signer(&self, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_signer(indexer)
    }

    fn index_is_writable(&self, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_writable(indexer)
    }

    fn index_is_owner(&self, owner: &Pubkey, indexer: I) -> CruiserResult<bool> {
        self.0.index_is_owner(owner, indexer)
    }
}
impl<AI, P, I> SingleIndexable<I> for CruiserProgramAccount<AI, P>
where
    AI: AccountInfo + SingleIndexable<I>,
    P: CruiserProgram,
{
    fn index_info(&self, indexer: I) -> CruiserResult<&AI> {
        self.0.index_info(indexer)
    }
}
