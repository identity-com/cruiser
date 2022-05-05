//! General imports for `cruiser`.
pub use crate::{
    account_argument::{
        AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, Single,
        SingleIndexable, ToSolanaAccountMeta, ValidateArgument,
    },
    account_list::{AccountList, AccountListItem},
    account_types::{
        close_account::CloseAccount,
        cruiser_program_account::CruiserProgramAccount,
        data_account::DataAccount,
        discriminant_account::DiscriminantAccount,
        init_account::{InitAccount, InitArgs},
        init_or_zeroed_account::InitOrZeroedAccount,
        rent_exempt::RentExempt,
        rest::Rest,
        seeds::{BumpSeed, FindBump, Seeds},
        sys_var::SysVar,
        system_program::{CreateAccount, SystemProgram},
        zeroed_account::ZeroedAccount,
    },
    borsh::{self, BorshDeserialize, BorshSerialize},
    compressed_numbers::CompressedNumber,
    error::{CruiserError, Error},
    impls::option::{IfSome, IfSomeArg, OptionMatch},
    instruction::{Instruction, InstructionProcessor, ReturnValue},
    instruction_list::{
        InstructionList, InstructionListCPI, InstructionListCPIDynamic, InstructionListCPIStatic,
        InstructionListItem, InstructionListProcessor,
    },
    on_chain_size::{OnChainSize, OnChainSizeWithArg},
    pda_seeds::{PDAGenerator, PDASeed, PDASeedSet, PDASeeder},
    program::{CruiserProgram, Program, ProgramKey},
    util::{
        Advance, AdvanceArray, MappableRef, MappableRefMut, MaybeOwned, TryMappableRef,
        TryMappableRefMut,
    },
    AccountInfo, CPIChecked, CPIMethod, CPIUnchecked, CruiserAccountInfo, CruiserResult,
    GenericError, Pubkey, SolanaAccountMeta, SolanaInstruction, ToSolanaAccountInfo, UnixTimestamp,
};
pub use solana_program::{rent::Rent, sysvar::Sysvar};
pub use std::num::{
    NonZeroI128, NonZeroI16, NonZeroI32, NonZeroI64, NonZeroI8, NonZeroIsize, NonZeroU128,
    NonZeroU16, NonZeroU32, NonZeroU64, NonZeroU8, NonZeroUsize,
};
pub use std::ops::{Deref, DerefMut};

#[cfg(feature = "client")]
pub use crate::{
    client::{system_program, HashedSigner},
    solana_sdk::signature::{Keypair, Signer},
};

#[cfg(feature = "in_place")]
pub use crate::{
    account_types::in_place_account::{CreateInPlace, InPlaceAccount, NoOwnerInPlace},
    in_place::{
        get_properties, get_properties_mut, GetNum, InPlace, InPlaceCreate, InPlaceRead,
        InPlaceUnit, InPlaceUnitCreate, InPlaceUnitRead, InPlaceUnitWrite, InPlaceWrite, SetNum,
    },
};

#[cfg(feature = "spl-token")]
pub use crate::spl::token::{
    MintAccount, TokenAccount, TokenAccountMint, TokenAccountOwner, TokenAccountOwnerAndMint,
    TokenProgram,
};

#[cfg(all(feature = "spl-token", feature = "client"))]
pub use crate::client::token;

#[cfg(feature = "small_vec")]
pub use crate::types::small_vec::{Vec16, Vec8};
