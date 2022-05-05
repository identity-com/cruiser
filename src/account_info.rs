use std::cell::{Ref, RefCell, RefMut};
use std::mem::{size_of, transmute};
use std::ops::{Deref, DerefMut};
use std::ptr::addr_of;
use std::rc::Rc;
use std::slice::{from_raw_parts, from_raw_parts_mut};

use crate::account_argument::{
    AccountArgument, AccountInfoIterator, FromAccounts, MultiIndexable, SingleIndexable,
    ValidateArgument,
};
use crate::util::{MappableRef, MappableRefMut, TryMappableRef, TryMappableRefMut};
use crate::{CruiserResult, GenericError, SolanaAccountInfo};
use solana_program::clock::Epoch;
use solana_program::entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE};
use solana_program::msg;
use solana_program::program_error::ProgramError;
use solana_program::program_memory::sol_memset;
use solana_program::pubkey::Pubkey;
use solana_program::system_instruction::MAX_PERMITTED_DATA_LENGTH;

use crate::AllAny;

/// A trait representing accounts on Solana. Can take many different forms.
pub trait AccountInfo:
    Clone
    + AccountArgument<AccountInfo = Self>
    + FromAccounts
    + FromAccounts<Self>
    + FromAccounts<Option<Self>>
    + ValidateArgument
    + MultiIndexable
    + MultiIndexable<AllAny>
    + SingleIndexable
{
    /// The return of [`AccountInfo::lamports`]
    type Lamports<'a>: Deref<Target = u64>
    where
        Self: 'a;
    /// The return of [`AccountInfo::lamports_mut`]
    type LamportsMut<'a>: DerefMut<Target = u64>
    where
        Self: 'a;
    /// The return of [`AccountInfo::data`]
    type Data<'a>: Deref<Target = [u8]> + MappableRef + TryMappableRef
    where
        Self: 'a;
    /// The return of [`AccountInfo::data_mut`]
    type DataMut<'a>: DerefMut<Target = [u8]>
        + MappableRef
        + TryMappableRef
        + MappableRefMut
        + TryMappableRefMut
    where
        Self: 'a;
    /// The return of [`AccountInfo::owner`]
    type Owner<'a>: Deref<Target = Pubkey>
    where
        Self: 'a;

    /// Gets the key of the account
    #[must_use]
    fn key(&self) -> &Pubkey;
    /// Returns true if this account is a signer
    #[must_use]
    fn is_signer(&self) -> bool;
    /// Returns true if this account is writable
    #[must_use]
    fn is_writable(&self) -> bool;
    /// Returns a shared ref to the lamports of this account
    #[must_use]
    fn lamports(&self) -> Self::Lamports<'_>;
    /// Returns a mutable ref to the lamports of this account
    #[must_use]
    fn lamports_mut(&self) -> Self::LamportsMut<'_>;
    /// Returns a shared ref to the data of this account
    #[must_use]
    fn data(&self) -> Self::Data<'_>;
    /// Returns a mutable ref to the data of this account
    #[must_use]
    fn data_mut(&self) -> Self::DataMut<'_>;
    /// Reallocates the data of this account allowing for size change after initialization. Must be done by the owning program.
    /// Should use [`SafeRealloc`] whenever possible.
    ///
    /// # Safety
    /// Not the worst for safety but there are ways that this function can cause you to write over data you didn't intend to.
    /// [`SolanaAccountInfo`] has no way of tracking the original data size so doesn't track it and relies on transaction checking to keep the size increase bounded.
    /// This means that a case can arise where the data is increased by a huge number, seeing memory of other accounts and their meta-data, written to, and then the data size shrunk back down.
    /// This is an edge case but can happen if you shrink after growing an account.
    /// Use of this function in this way should be considered a security bug (unless you guarentee that [`MAX_PERMITTED_DATA_INCREASE`] is not surpassed).
    ///
    /// [`CruiserAccountInfo`] avoids this by tracking the original size and therefore implements [`SafeRealloc`].
    unsafe fn realloc_unsafe(&self, new_len: usize, zero_init: bool) -> CruiserResult;
    /// Returns a shared ref to the owner of this account
    #[must_use]
    fn owner(&self) -> Self::Owner<'_>;
    /// Unsafe access to changing the owner of this account. You should use [`SafeOwnerChange::owner_mut`] if possible.
    ///
    /// # Safety
    /// Solana's way of doing this for [`SolanaAccountInfo`] is to use [`write_volatile`](std::ptr::write_volatile) on a shared ref (see [`SolanaAccountInfo::assign`]).
    /// This is wildly wrong and can be eviscerated by the optimizer (Rust has even more rules to follow than C in unsafe code).
    /// The only way to prevent this is to set your opt level to 0, turn off LTO, and pray.
    /// Even then LLVM can make a silent change (not tied to a new rust version) that suddenly opens your program to attack.
    /// If this function is used from a [`SolanaAccountInfo`] it should be considered a security bug.
    ///
    /// [`CruiserAccountInfo`] avoids this by putting the owner in a [`RefCell`] allowing mutable access.
    /// It therefore implements [`SafeOwnerChange`] and should be used wherever owner change is needed.
    unsafe fn set_owner_unsafe(&self, new_owner: &Pubkey);
    /// This account's data contains a loaded program (and is now read-only)
    #[must_use]
    fn executable(&self) -> bool;
    /// The epoch at which this account will next owe rent
    #[must_use]
    fn rent_epoch(&self) -> Epoch;
}

/// Account info can safely assign the owner.
pub trait SafeOwnerChange: AccountInfo {
    /// The return value of [`SafeOwnerChange::owner_mut`]
    type OwnerMut<'a>: DerefMut<Target = Pubkey>
    where
        Self: 'a;
    /// Returns a mutable ref to the owner of this account
    fn owner_mut(&self) -> Self::OwnerMut<'_>;
}

/// Account info can safely realloc.
pub trait SafeRealloc: AccountInfo {
    /// Reallocates an account safely by checking data size.
    /// If this can be called in a cpi from the same program or earlier owning program of this account you should use [`SafeRealloc::realloc_cpi_safe`].
    fn realloc(&self, new_len: usize, zero_init: bool) -> CruiserResult;
    /// Reallocates an account safely by checking data size, only allows for 1/4 the increase of [`MAX_PERMITTED_DATA_INCREASE`].
    /// This limited growth means that a cpi call can never exceed [`MAX_PERMITTED_DATA_INCREASE`].
    fn realloc_cpi_safe(&self, new_len: usize, zero_init: bool) -> CruiserResult;
}

/// Account info can be turned into a [`SolanaAccountInfo`].
pub trait ToSolanaAccountInfo<'as_info>: AccountInfo {
    /// Turns this into a solana account info for interoperability and CPI.
    ///
    /// # Safety
    /// Only use this when the resulting account info will never be used after another use of self or any values stemming from self.
    unsafe fn to_solana_account_info(&self) -> SolanaAccountInfo<'as_info>;
}

// verify_account_arg_impl! {
//     mod account_info_check<CruiserAccountInfo>{
//         CruiserAccountInfo{
//             from: [()];
//             validate: [()];
//             multi: [(); AllAny];
//             single: [()];
//         }
//     }
// }

/// A custom version of Solana's [`AccountInfo`](solana_program::account_info::AccountInfo) that allows for owner changes.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CruiserAccountInfo {
    /// The public key of the account.
    pub key: &'static Pubkey,
    /// Whether the account is a signer of the transaction
    pub is_signer: bool,
    /// Whether the account is writable
    pub is_writable: bool,
    /// How many lamports the account has.
    ///
    /// # Change Limitations
    /// - Lamports must not have been created or destroyed by transaction's end
    /// - Lamports may only be subtracted from accounts owned by the subtracting program
    pub lamports: Rc<RefCell<&'static mut u64>>,
    /// The data the account stores. Public information, can be read by anyone on the network.
    /// Also stores the starting length so can error if changed too far.
    ///
    /// # Change Limitations
    /// - Data size may only be changed by the system program
    /// - Data size cannot be changed once set except by account wipe if no rent
    /// - Data can only be changed by the owning program
    /// - Data will be wiped if there is no rent
    pub data: Rc<RefCell<&'static mut [u8]>>,
    /// The original data size. Can  only see in it's own call meaning the parent CPI size won't be passed down.
    pub original_data_len: &'static usize,
    /// The owning program of the account, defaults to the system program for new accounts
    ///
    /// # Change Limitations
    /// - Owner can only be changed by the owning program
    /// - All data must be zeroed to be transferred
    pub owner: &'static RefCell<&'static mut Pubkey>,
    /// Whether or not the account is executable
    pub executable: bool,
    /// The next epoch this account owes rent. Can be rent free by giving two years of rent.
    pub rent_epoch: Epoch,
}
impl CruiserAccountInfo {
    unsafe fn read_value<T: Copy>(input: *mut u8, offset: &mut usize) -> &'static mut T {
        let out = &mut *input.add(*offset).cast::<T>();
        *offset += size_of::<T>();
        out
    }

    /// Deserializes the program input
    ///
    /// # Safety
    /// Must only be called on solana program input.
    pub unsafe fn deserialize(input: *mut u8) -> (&'static Pubkey, Vec<Self>, &'static [u8]) {
        let mut offset = 0;

        let num_accounts = *Self::read_value::<u64>(input, &mut offset) as usize;
        let mut accounts = Vec::with_capacity(num_accounts);
        for _ in 0..num_accounts {
            let dup_info = *Self::read_value::<u8>(input, &mut offset);
            if dup_info == u8::MAX {
                let is_signer = *Self::read_value::<u8>(input, &mut offset) != 0;
                let is_writable = *Self::read_value::<u8>(input, &mut offset) != 0;
                let executable = *Self::read_value::<u8>(input, &mut offset) != 0;
                //padding to u64
                offset += size_of::<u32>();
                // Safe because Pubkey is transparent to [u8; 32]
                let key = &*Self::read_value::<Pubkey>(input, &mut offset);
                let owner =
                    &*Box::leak(Box::new(RefCell::new(Self::read_value(input, &mut offset))));
                let lamports = Rc::new(RefCell::new(Self::read_value(input, &mut offset)));
                let data_len = *Self::read_value::<u64>(input, &mut offset) as usize;
                let data = Rc::new(RefCell::new(from_raw_parts_mut(
                    input.add(offset),
                    data_len,
                )));
                let original_data_len = &*Box::leak(Box::new(data_len));
                offset += data_len + MAX_PERMITTED_DATA_INCREASE;
                offset += (offset as *const u8).align_offset(BPF_ALIGN_OF_U128);

                let rent_epoch = *Self::read_value::<Epoch>(input, &mut offset);

                accounts.push(Self {
                    key,
                    is_signer,
                    is_writable,
                    lamports,
                    data,
                    original_data_len,
                    owner,
                    executable,
                    rent_epoch,
                });
            } else {
                offset += 7;

                accounts.push(accounts[dup_info as usize].clone());
            }
        }

        let instruction_data_len = *Self::read_value::<u64>(input, &mut offset) as usize;
        let instruction_data = from_raw_parts(input.add(offset), instruction_data_len);
        offset += instruction_data_len;

        let program_id = &*(input.add(offset).cast::<Pubkey>());
        (program_id, accounts, instruction_data)
    }

    /// Turns this into a normal [`solana_program::account_info::AccountInfo`] for usage with standard functions.
    ///
    /// # Safety
    /// The resulting account info has owner as a shared reference that can be modified.
    /// Only use this when the resulting account info will never be used after another use of self or any values stemming from self.
    #[must_use]
    pub unsafe fn to_solana_account_info<'a>(&self) -> SolanaAccountInfo<'a> {
        SolanaAccountInfo {
            key: self.key,
            is_signer: self.is_signer,
            is_writable: self.is_writable,
            lamports: transmute::<Rc<RefCell<&'static mut u64>>, Rc<RefCell<&'a mut u64>>>(
                self.lamports.clone(),
            ),
            data: transmute::<Rc<RefCell<&'static mut [u8]>>, Rc<RefCell<&'a mut [u8]>>>(
                self.data.clone(),
            ),
            #[allow(clippy::deref_addrof)]
            owner: &*(addr_of!(**self.owner.borrow())),
            executable: self.executable,
            rent_epoch: self.rent_epoch,
        }
    }

    unsafe fn realloc_unchecked(&self, new_len: usize, zero_init: bool) {
        // Copied from Solana's realloc code.
        let mut self_data = self.data.borrow_mut();
        let old_len = self_data.len();

        // This part specifically is okay because alignment is designed in
        #[allow(clippy::cast_ptr_alignment)]
        self_data
            .as_mut_ptr()
            .offset(-8)
            .cast::<u64>()
            .write(new_len as u64);

        // I did this part better
        self.data.as_ptr().cast::<usize>().offset(1).write(new_len);

        // No idea what sol_memset will silently break if I pass zero length so this check stays for now
        if zero_init && new_len > old_len {
            // Another function that is actually unsafe but isn't marked as so...
            sol_memset(*self_data, 0, new_len.saturating_sub(old_len));
        }
    }
}
impl AccountInfo for CruiserAccountInfo {
    type Lamports<'a> = Ref<'a, u64>;
    type LamportsMut<'a> = RefMut<'a, u64>;
    type Data<'a> = Ref<'a, [u8]>;
    type DataMut<'a> = RefMut<'a, [u8]>;
    type Owner<'a> = Ref<'a, Pubkey>;

    #[inline]
    fn key(&self) -> &Pubkey {
        self.key
    }

    #[inline]
    fn is_signer(&self) -> bool {
        self.is_signer
    }

    #[inline]
    fn is_writable(&self) -> bool {
        self.is_writable
    }

    #[inline]
    fn lamports(&self) -> Self::Lamports<'_> {
        Ref::map(self.lamports.borrow(), |val| &**val)
    }

    #[inline]
    fn lamports_mut(&self) -> Self::LamportsMut<'_> {
        RefMut::map(self.lamports.borrow_mut(), |val| *val)
    }

    #[inline]
    fn data(&self) -> Self::Data<'_> {
        Ref::map(self.data.borrow(), |val| &**val)
    }

    #[inline]
    fn data_mut(&self) -> Self::DataMut<'_> {
        RefMut::map(self.data.borrow_mut(), |val| *val)
    }

    #[inline]
    unsafe fn realloc_unsafe(&self, new_len: usize, zero_init: bool) -> CruiserResult {
        self.realloc(new_len, zero_init)
    }

    #[inline]
    fn owner(&self) -> Self::Owner<'_> {
        Ref::map(self.owner.borrow(), |owner| &**owner)
    }

    #[inline]
    unsafe fn set_owner_unsafe(&self, new_owner: &Pubkey) {
        **self.owner.borrow_mut() = *new_owner;
    }

    #[inline]
    fn executable(&self) -> bool {
        self.executable
    }

    #[inline]
    fn rent_epoch(&self) -> Epoch {
        self.rent_epoch
    }
}
impl SafeOwnerChange for CruiserAccountInfo {
    type OwnerMut<'a> = RefMut<'a, Pubkey>;

    fn owner_mut(&self) -> Self::OwnerMut<'_> {
        RefMut::map(self.owner.borrow_mut(), |val| *val)
    }
}
impl SafeRealloc for CruiserAccountInfo {
    fn realloc(&self, new_len: usize, zero_init: bool) -> CruiserResult {
        let max_new_len = self
            .original_data_len
            .checked_add(MAX_PERMITTED_DATA_INCREASE)
            .expect("Data is far too big")
            .min(MAX_PERMITTED_DATA_LENGTH as usize);
        if new_len > max_new_len {
            return Err(GenericError::TooLargeDataIncrease {
                original_len: *self.original_data_len,
                new_len,
                max_new_len,
            }
            .into());
        }

        // Safety: data length was checked
        unsafe {
            self.realloc_unchecked(new_len, zero_init);
        }
        Ok(())
    }

    fn realloc_cpi_safe(&self, new_len: usize, zero_init: bool) -> CruiserResult {
        let max_new_len = self
            .original_data_len
            .checked_add(MAX_PERMITTED_DATA_INCREASE / 4)
            .expect("Data is far too big")
            .min(MAX_PERMITTED_DATA_LENGTH as usize);
        if new_len > max_new_len {
            return Err(GenericError::TooLargeDataIncrease {
                original_len: *self.original_data_len,
                new_len,
                max_new_len,
            }
            .into());
        }

        // Safety: data length was checked
        unsafe {
            self.realloc_unchecked(new_len, zero_init);
        }
        Ok(())
    }
}
impl<'as_info> ToSolanaAccountInfo<'as_info> for CruiserAccountInfo {
    unsafe fn to_solana_account_info(&self) -> SolanaAccountInfo<'as_info> {
        self.to_solana_account_info()
    }
}
impl<'b> AccountInfo for SolanaAccountInfo<'b> {
    type Lamports<'a>
    where
        Self: 'a,
    = Ref<'a, u64>;
    type LamportsMut<'a>
    where
        Self: 'a,
    = RefMut<'a, u64>;
    type Data<'a>
    where
        Self: 'a,
    = Ref<'a, [u8]>;
    type DataMut<'a>
    where
        Self: 'a,
    = RefMut<'a, [u8]>;
    type Owner<'a>
    where
        Self: 'a,
    = &'a Pubkey;

    #[inline]
    fn key(&self) -> &Pubkey {
        self.key
    }

    #[inline]
    fn is_signer(&self) -> bool {
        self.is_signer
    }

    #[inline]
    fn is_writable(&self) -> bool {
        self.is_writable
    }

    #[inline]
    fn lamports(&self) -> Self::Lamports<'_> {
        Ref::map(self.lamports.borrow(), |val| &**val)
    }

    #[inline]
    fn lamports_mut(&self) -> Self::LamportsMut<'_> {
        RefMut::map(self.lamports.borrow_mut(), |val| *val)
    }

    #[inline]
    fn data(&self) -> Self::Data<'_> {
        Ref::map(self.data.borrow(), |data| &**data)
    }

    #[inline]
    fn data_mut(&self) -> Self::DataMut<'_> {
        RefMut::map(self.data.borrow_mut(), |data| *data)
    }

    #[inline]
    unsafe fn realloc_unsafe(&self, new_len: usize, zero_init: bool) -> CruiserResult {
        Ok(self.realloc(new_len, zero_init)?)
    }

    #[inline]
    fn owner(&self) -> Self::Owner<'_> {
        self.owner
    }

    #[inline]
    unsafe fn set_owner_unsafe(&self, new_owner: &Pubkey) {
        msg!("Called `SolanaAccountInfo::assign`! This should be considered a security bug");
        self.assign(new_owner);
    }

    #[inline]
    fn executable(&self) -> bool {
        self.executable
    }

    #[inline]
    fn rent_epoch(&self) -> Epoch {
        self.rent_epoch
    }
}
impl<'as_info> ToSolanaAccountInfo<'as_info> for SolanaAccountInfo<'as_info> {
    unsafe fn to_solana_account_info(&self) -> SolanaAccountInfo<'as_info> {
        self.clone()
    }
}
impl_account_info!(CruiserAccountInfo);
const _: fn() = || {
    // Only callable when `$type` implements all traits in `$($trait)+`.
    fn assert_impl_all<'as_infos, T: ?Sized + AccountInfo + ToSolanaAccountInfo<'as_infos>>() {}
    assert_impl_all::<CruiserAccountInfo>();
};
impl_account_info!(SolanaAccountInfo<'a>, <'a>);
const _: fn() = || {
    // Only callable when `$type` implements all traits in `$($trait)+`.
    fn assert_impl_all<'a, T: ?Sized + AccountInfo + ToSolanaAccountInfo<'a>>() {}
    assert_impl_all::<SolanaAccountInfo>();
};

#[cfg(test)]
pub mod account_info_test {
    use std::cell::RefCell;
    use std::rc::Rc;

    use rand::{thread_rng, Rng};
    use solana_program::entrypoint::{BPF_ALIGN_OF_U128, MAX_PERMITTED_DATA_INCREASE};

    use crate::account_argument::{MultiIndexable, Single};
    use crate::AllAny;
    use crate::{CruiserAccountInfo, Pubkey};

    fn add<const N: usize>(data: &mut Vec<u8>, add: [u8; N]) {
        for item in IntoIterator::into_iter(add) {
            data.push(item);
        }
    }
    fn pad(data: &mut Vec<u8>, add: usize) {
        for _ in 0..add {
            data.push(0);
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn add_account<const N: usize>(
        data: &mut Vec<u8>,
        is_signer: bool,
        is_writable: bool,
        is_executable: bool,
        key: Pubkey,
        owner: Pubkey,
        lamports: u64,
        account_data: [u8; N],
        rent_epoch: u64,
    ) {
        data.push(u8::MAX);
        data.push(u8::from(is_signer));
        data.push(u8::from(is_writable));
        data.push(u8::from(is_executable));
        add(data, 0u32.to_ne_bytes());
        add(data, key.to_bytes());
        add(data, owner.to_bytes());
        add(data, lamports.to_ne_bytes());
        add(data, (N as u64).to_ne_bytes());
        add(data, account_data);
        add(data, [0; MAX_PERMITTED_DATA_INCREASE]);
        let extra = (data.len() as *const u8).align_offset(BPF_ALIGN_OF_U128);
        pad(data, extra);
        add(data, rent_epoch.to_ne_bytes());
    }

    #[test]
    fn deserialization_test() {
        let key1 = Pubkey::new_unique();
        let owner1 = Pubkey::new_unique();
        let key2 = Pubkey::new_unique();
        let owner2 = Pubkey::new_unique();
        let program_id = Pubkey::new_unique();

        let mut data = Vec::new();
        add(&mut data, 3u64.to_ne_bytes());
        add_account(
            &mut data, true, true, false, key1, owner1, 100, [32; 10], 1828,
        );
        add_account(
            &mut data, false, false, true, key2, owner2, 100_000, [56; 1000], 567,
        );
        data.push(0);
        add(&mut data, [9; 7]);
        add(&mut data, 50u64.to_ne_bytes());
        add(&mut data, [224; 50]);
        add(&mut data, program_id.to_bytes());

        let (solana_program_id, solana_accounts, solana_instruction_data) =
            unsafe { crate::solana_program::entrypoint::deserialize(data.as_mut_ptr()) };
        assert_eq!(solana_program_id, &program_id);
        assert_eq!(solana_accounts.len(), 3);
        assert!(solana_accounts[0].is_signer);
        assert!(solana_accounts[0].is_writable);
        assert!(!solana_accounts[0].executable);
        assert_eq!(solana_accounts[0].key, &key1);
        assert_eq!(solana_accounts[0].owner, &owner1);
        assert_eq!(**solana_accounts[0].lamports.borrow(), 100);
        assert_eq!(solana_accounts[0].data.borrow().len(), 10);
        assert!(solana_accounts[0]
            .data
            .borrow()
            .iter()
            .all(|data| *data == 32));
        assert_eq!(solana_accounts[0].rent_epoch, 1828);
        assert!(!solana_accounts[1].is_signer);
        assert!(!solana_accounts[1].is_writable);
        assert!(solana_accounts[1].executable);
        assert_eq!(solana_accounts[1].key, &key2);
        assert_eq!(solana_accounts[1].owner, &owner2);
        assert_eq!(**solana_accounts[1].lamports.borrow(), 100_000);
        assert_eq!(solana_accounts[1].data.borrow().len(), 1000);
        assert!(solana_accounts[1]
            .data
            .borrow()
            .iter()
            .all(|data| *data == 56));
        assert_eq!(solana_accounts[1].rent_epoch, 567);
        assert!(solana_accounts[2].is_signer);
        assert!(solana_accounts[2].is_writable);
        assert!(!solana_accounts[2].executable);
        assert_eq!(solana_accounts[2].key, &key1);
        assert_eq!(solana_accounts[2].owner, &owner1);
        assert_eq!(**solana_accounts[2].lamports.borrow(), 100);
        assert_eq!(solana_accounts[2].data.borrow().len(), 10);
        assert!(solana_accounts[2]
            .data
            .borrow()
            .iter()
            .all(|data| *data == 32));
        assert_eq!(solana_accounts[2].rent_epoch, 1828);
        assert_eq!(solana_instruction_data.len(), 50);
        assert!(solana_instruction_data.iter().all(|data| *data == 224));

        let (generator_program_id, generator_accounts, generator_instruction_data) =
            unsafe { crate::CruiserAccountInfo::deserialize(data.as_mut_ptr()) };
        assert_eq!(generator_program_id, &program_id);
        assert_eq!(generator_accounts.len(), 3);
        assert!(generator_accounts[0].is_signer);
        assert!(generator_accounts[0].is_writable);
        assert!(!generator_accounts[0].executable);
        assert_eq!(generator_accounts[0].key, &key1);
        assert_eq!(**generator_accounts[0].owner.borrow(), owner1);
        assert_eq!(**generator_accounts[0].lamports.borrow(), 100);
        assert_eq!(generator_accounts[0].data.borrow().len(), 10);
        assert!(generator_accounts[0]
            .data
            .borrow()
            .iter()
            .all(|data| *data == 32));
        assert_eq!(generator_accounts[0].rent_epoch, 1828);
        assert!(!generator_accounts[1].is_signer);
        assert!(!generator_accounts[1].is_writable);
        assert!(generator_accounts[1].executable);
        assert_eq!(generator_accounts[1].key, &key2);
        assert_eq!(**generator_accounts[1].owner.borrow(), owner2);
        assert_eq!(**generator_accounts[1].lamports.borrow(), 100_000);
        assert_eq!(generator_accounts[1].data.borrow().len(), 1000);
        assert!(generator_accounts[1]
            .data
            .borrow()
            .iter()
            .all(|data| *data == 56));
        assert_eq!(generator_accounts[1].rent_epoch, 567);
        assert!(generator_accounts[2].is_signer);
        assert!(generator_accounts[2].is_writable);
        assert!(!generator_accounts[2].executable);
        assert_eq!(generator_accounts[2].key, &key1);
        assert_eq!(**generator_accounts[2].owner.borrow(), owner1);
        assert_eq!(**generator_accounts[2].lamports.borrow(), 100);
        assert_eq!(generator_accounts[2].data.borrow().len(), 10);
        assert!(generator_accounts[2]
            .data
            .borrow()
            .iter()
            .all(|data| *data == 32));
        assert_eq!(generator_accounts[2].rent_epoch, 1828);
        assert_eq!(generator_instruction_data.len(), 50);
        assert!(generator_instruction_data.iter().all(|data| *data == 224));

        assert_eq!(
            *solana_accounts[0].lamports.borrow() as *const u64,
            *generator_accounts[0].lamports.borrow() as *const u64
        );
        assert_eq!(
            *solana_accounts[1].lamports.borrow() as *const u64,
            *generator_accounts[1].lamports.borrow() as *const u64
        );
        assert_eq!(
            *solana_accounts[0].data.borrow() as *const [u8],
            *generator_accounts[0].data.borrow() as *const [u8]
        );
        assert_eq!(
            *solana_accounts[1].data.borrow() as *const [u8],
            *generator_accounts[1].data.borrow() as *const [u8]
        );
        assert_eq!(
            solana_accounts[0].owner as *const Pubkey,
            *generator_accounts[0].owner.borrow() as *const Pubkey
        );
        assert_eq!(
            solana_accounts[1].owner as *const Pubkey,
            *generator_accounts[1].owner.borrow() as *const Pubkey
        );
    }

    fn random_account_info(rng: &mut impl Rng) -> CruiserAccountInfo {
        let data_len: usize = rng.gen_range(16, 1024 + 1);
        let mut data = vec![0; data_len];
        for val in &mut data {
            *val = rng.gen();
        }
        CruiserAccountInfo {
            key: Box::leak(Box::new(Pubkey::new(&rng.gen::<[u8; 32]>()))),
            is_signer: rng.gen(),
            is_writable: rng.gen(),
            lamports: Rc::new(RefCell::new(Box::leak(Box::new(rng.gen())))),
            original_data_len: Box::leak(Box::new(data.len())),
            data: Rc::new(RefCell::new(Box::leak(data.into_boxed_slice()))),
            owner: Box::leak(Box::new(RefCell::new(Box::leak(Box::new(Pubkey::new(
                &rng.gen::<[u8; 32]>(),
            )))))),
            executable: rng.gen(),
            rent_epoch: rng.gen(),
        }
    }
    #[must_use]
    pub fn account_info_eq(first: &CruiserAccountInfo, second: &CruiserAccountInfo) -> bool {
        first.key == second.key
            && first.is_signer == second.is_signer
            && first.is_writable == second.is_writable
            && **first.lamports.borrow() == **second.lamports.borrow()
            && **first.data.borrow() == **second.data.borrow()
            && **first.owner.borrow() == **second.owner.borrow()
            && first.executable == second.executable
            && first.rent_epoch == second.rent_epoch
    }

    #[test]
    fn is_signer_test() {
        let mut rng = thread_rng();
        let mut account_info = random_account_info(&mut rng);
        assert_eq!(
            account_info.is_signer,
            account_info.index_is_signer(()).unwrap()
        );
        assert_eq!(
            account_info.is_signer,
            account_info.index_is_signer(AllAny::All).unwrap()
        );
        assert_eq!(
            account_info.is_signer,
            account_info.index_is_signer(AllAny::Any).unwrap()
        );
        assert_eq!(
            !account_info.is_signer,
            account_info.index_is_signer(AllAny::NotAll).unwrap()
        );
        assert_eq!(
            !account_info.is_signer,
            account_info.index_is_signer(AllAny::NotAny).unwrap()
        );
        account_info.is_signer = !account_info.is_signer;
        assert_eq!(
            account_info.is_signer,
            account_info.index_is_signer(()).unwrap()
        );
        assert_eq!(
            account_info.is_signer,
            account_info.index_is_signer(AllAny::All).unwrap()
        );
        assert_eq!(
            account_info.is_signer,
            account_info.index_is_signer(AllAny::Any).unwrap()
        );
        assert_eq!(
            !account_info.is_signer,
            account_info.index_is_signer(AllAny::NotAll).unwrap()
        );
        assert_eq!(
            !account_info.is_signer,
            account_info.index_is_signer(AllAny::NotAny).unwrap()
        );
    }

    #[test]
    fn is_writable_test() {
        let mut rng = thread_rng();
        let mut account_info = random_account_info(&mut rng);
        assert_eq!(
            account_info.is_writable,
            account_info.index_is_writable(()).unwrap()
        );
        assert_eq!(
            account_info.is_writable,
            account_info.index_is_writable(AllAny::All).unwrap()
        );
        assert_eq!(
            account_info.is_writable,
            account_info.index_is_writable(AllAny::Any).unwrap()
        );
        assert_eq!(
            !account_info.is_writable,
            account_info.index_is_writable(AllAny::NotAll).unwrap()
        );
        assert_eq!(
            !account_info.is_writable,
            account_info.index_is_writable(AllAny::NotAny).unwrap()
        );
        account_info.is_signer = !account_info.is_signer;
        assert_eq!(
            account_info.is_writable,
            account_info.index_is_writable(()).unwrap()
        );
        assert_eq!(
            account_info.is_writable,
            account_info.index_is_writable(AllAny::All).unwrap()
        );
        assert_eq!(
            account_info.is_writable,
            account_info.index_is_writable(AllAny::Any).unwrap()
        );
        assert_eq!(
            !account_info.is_writable,
            account_info.index_is_writable(AllAny::NotAll).unwrap()
        );
        assert_eq!(
            !account_info.is_writable,
            account_info.index_is_writable(AllAny::NotAny).unwrap()
        );
    }

    #[test]
    fn is_owner_test() {
        let mut rng = thread_rng();
        let account_info = random_account_info(&mut rng);
        assert!(account_info
            .index_is_owner(*account_info.owner.borrow(), ())
            .unwrap());
        assert!(account_info
            .index_is_owner(*account_info.owner.borrow(), AllAny::All)
            .unwrap());
        assert!(account_info
            .index_is_owner(*account_info.owner.borrow(), AllAny::Any)
            .unwrap());
        assert!(!account_info
            .index_is_owner(*account_info.owner.borrow(), AllAny::NotAll)
            .unwrap());
        assert!(!account_info
            .index_is_owner(*account_info.owner.borrow(), AllAny::NotAny)
            .unwrap());
        assert!(!account_info
            .index_is_owner(&Pubkey::new(&rng.gen::<[u8; 32]>()), ())
            .unwrap());
        assert!(!account_info
            .index_is_owner(&Pubkey::new(&rng.gen::<[u8; 32]>()), AllAny::All)
            .unwrap());
        assert!(!account_info
            .index_is_owner(&Pubkey::new(&rng.gen::<[u8; 32]>()), AllAny::Any)
            .unwrap());
        assert!(account_info
            .index_is_owner(&Pubkey::new(&rng.gen::<[u8; 32]>()), AllAny::NotAll)
            .unwrap());
        assert!(account_info
            .index_is_owner(&Pubkey::new(&rng.gen::<[u8; 32]>()), AllAny::NotAny)
            .unwrap());
    }

    #[test]
    fn get_inf0_test() {
        let mut rng = thread_rng();
        let account_info = random_account_info(&mut rng);
        assert_eq!(account_info.info(), &account_info);
    }
}
