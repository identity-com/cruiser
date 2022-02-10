use solana_program::system_program::ID as SYSTEM_PROGRAM_ID;

#[cfg(feature = "nightly")]
pub use in_place_account::*;
pub use init_account::*;
pub use init_or_zeroed_account::*;
pub use program_account::*;
pub use rest::*;
pub use seeds::*;
pub use sys_var::*;
pub use system_program::*;
pub use zeroed_account::*;

use crate::*;

#[cfg(feature = "nightly")]
mod in_place_account;
mod init_account;
mod init_or_zeroed_account;
mod program_account;
mod rest;
mod seeds;
mod sys_var;
mod system_program;
mod zeroed_account;
