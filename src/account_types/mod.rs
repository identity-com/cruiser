//! Standard account types. These are all optional, you can build your own if you don't like something in one of them.

pub mod close_account;
pub mod discriminant_account;
#[cfg(all(feature = "in_place", feature = "nightly"))]
pub mod in_place_account;
pub mod init_account;
pub mod init_or_zeroed_account;
pub mod program_account;
pub mod rent_exempt;
pub mod rest;
pub mod seeds;
pub mod sys_var;
pub mod system_program;
pub mod zeroed_account;
