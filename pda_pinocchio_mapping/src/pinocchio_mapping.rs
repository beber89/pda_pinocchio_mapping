use bytemuck::Pod;
use pinocchio::pubkey::Pubkey;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_pubkey::derive_address;
use pinocchio_system::instructions::CreateAccount;

/**
 */

pub trait Bumpy {
    ///
    fn bump(&self) -> u8;
}

/**
 *
 */
pub struct Mapping<'a> {
    ///
    pub program_id: &'a Pubkey,
    ///
    pub name: &'static [u8],
    ///
    pub payer: &'a AccountInfo,
}

impl<'a> Mapping<'a> {
    ///
    pub fn new(program_id: &'a Pubkey, name: &'static [u8], payer: &'a AccountInfo) -> Self {
        Self {
            program_id,
            name,
            payer,
        }
    }

    /** Writes a value into the PDA account associated with `(name, key)`.
     *
     * This method derives the PDA using:
     *   - the mapping's static `name`,
     *   - the provided `key`,
     *   - the bump extracted from `value`.
     *
     * Behavior:
     * - If the PDA account does not exist but deriveable by `program_id`,
     *   it is created with the required space and rent-exempt balance,
     *   then initialized with `value`.
     *
     * - If the PDA account already exists and is owned by `program_id`,
     *   its contents are overwritten with `value`.
     *
     * Safety & validation:
     * - Ensures the passed `account` matches the derived PDA.
     * - Ensures the account's data length matches `T::LEN`.
     * - Ensures the memory alignment is valid for bytemuck casting.
     *
     * Requirements:
     * - `T` must implement `Sizeable` (exposes `LEN`) and `Bumpy` (exposes `bump()`).
     * - `value` must be a POD type compatible with bytemuck.
     *
     * Returns:
     * - `ProgramResult::Ok(())` on success.
     * - `ProgramError` if account mismatch, invalid data, or system-instruction
     *   failures occur.
     */
    pub fn set<T: Pod + Bumpy>(
        self,
        key: &Pubkey,
        value: T,
        account: &AccountInfo,
    ) -> ProgramResult {
        let size_T = core::mem::size_of::<T>();
        let seed = [self.name.as_ref(), key.as_slice(), &[value.bump()]];

        let account_pda = derive_address(&seed, None, self.program_id);
        assert_eq!(account_pda, *account.key(), "Mapping: Accounts Mismatching");
        let bump = [value.bump().to_le()];
        let seed = [
            Seed::from(self.name.as_ref()),
            Seed::from(key.as_slice()),
            Seed::from(&bump),
        ];
        let seeds = Signer::from(&seed);

        if account.owner() != self.program_id {
            CreateAccount {
                from: self.payer,
                to: account,
                lamports: Rent::get()?.minimum_balance(size_T),
                space: size_T as u64,
                owner: self.program_id,
            }
            .invoke_signed(&[seeds.clone()])?;

            let mut data = account.try_borrow_mut_data()?;
            if data.len() != size_T {
                return Err(ProgramError::InvalidAccountData);
            }

            if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            let t_ref: &mut T = bytemuck::from_bytes_mut(&mut data);
            *t_ref = value;
            Ok(())
        } else {
            // Account already exists - overwrite
            let mut data = account.try_borrow_mut_data()?;
            if data.len() != size_T {
                return Err(ProgramError::InvalidAccountData);
            }

            if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            let t_ref: &mut T = bytemuck::from_bytes_mut(&mut data);
            *t_ref = value;
            Ok(())
        }
    }

    /**
     * Overwrites the value stored in the PDA account associated with `(name, key)`.
     *
     * This method derives the PDA using:
     *   - the mapping's static `name`,
     *   - the provided `key`,
     *   - the bump extracted from `value`.
     *
     * Behavior:
     * - Verifies that the passed `account` matches the derived PDA.
     * - Fails if the PDA account is not initialized or deriveable by `program_id`.
     * - If the account exists and is valid, its contents are overwritten with `value`.
     *
     * Safety & validation:
     * - Ensures the account's data length matches `T::LEN`.
     * - Ensures proper memory alignment for bytemuck casting.
     * - Casts raw account data into `T` and assigns the new `value`.
     *
     * Requirements:
     * - `T` must implement `Sizeable` (defines `LEN`) and `Bumpy` (defines `bump()`).
     * - The account must already be created
     *   and owned by `program_id`.
     *
     * Returns:
     * - `ProgramResult::Ok(())` if the value is successfully written.
     * - `ProgramError::UninitializedAccount` if the PDA does not exist or is not owned.
     * - `ProgramError::InvalidAccountData` if the stored data layout does not match `T`.
     */
    pub fn update<T: Pod + Bumpy>(
        self,
        key: &Pubkey,
        value: T,
        account: &AccountInfo,
    ) -> ProgramResult {
        let size_T = core::mem::size_of::<T>();
        let seed = [self.name.as_ref(), key.as_slice(), &[value.bump()]];

        let account_pda = derive_address(&seed, None, self.program_id);
        assert_eq!(account_pda, *account.key(), "Mapping: Accounts Mismatching");

        if account.owner() != self.program_id {
            Err(ProgramError::UninitializedAccount)
        } else {
            // Account already exists - overwrite
            let mut data = account.try_borrow_mut_data()?;
            if data.len() != size_T {
                return Err(ProgramError::InvalidAccountData);
            }

            if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            let t_ref: &mut T = bytemuck::from_bytes_mut(&mut data);
            *t_ref = value;
            Ok(())
        }
    }

    /**
     * Creates and initializes the PDA account associated with `(name, key)`.
     *
     * This method derives the PDA using:
     *   - the mapping's static `name`,
     *   - the provided `key`,
     *   - the bump extracted from `value`.
     *
     * Behavior:
     * - Verifies that the passed `account` matches the derived PDA.
     * - After creation, the account's data buffer is initialized with `value`.
     * - If the account already exists, the operation
     *   fails, as `create()` is intended for first-time initialization only.
     *
     * Safety & validation:
     * - Ensures account data length matches `T::LEN`.
     * - Ensures proper alignment for bytemuck casting into `T`.
     * - Writes the provided `value` directly into account data.
     *
     * Requirements:
     * - `T` must implement `Sizeable` (defines `LEN`) and `Bumpy` (defines `bump()`).
     * - The PDA must not already be initialized and deriveable by `program_id`.
     *
     * Returns:
     * - `ProgramResult::Ok(())` if the PDA is successfully created and initialized.
     * - `ProgramError::InvalidAccountData` for size or alignment mismatches.
     * - `ProgramError::IllegalOwner` if the PDA already exists and is owned.
     * - Propagated errors from system account creation or rent retrieval.
     */

    pub fn create<T: Pod + Bumpy>(
        self,
        key: &Pubkey,
        value: T,
        account: &AccountInfo,
    ) -> ProgramResult {
        let size_T = core::mem::size_of::<T>();
        let seed = [self.name.as_ref(), key.as_slice(), &[value.bump()]];

        let account_pda = derive_address(&seed, None, self.program_id);
        assert_eq!(account_pda, *account.key(), "Mapping: Accounts Mismatching");
        let bump = [value.bump().to_le()];
        let seed = [
            Seed::from(self.name.as_ref()),
            Seed::from(key.as_slice()),
            Seed::from(&bump),
        ];
        let seeds = Signer::from(&seed);

        if account.owner() != self.program_id {
            CreateAccount {
                from: self.payer,
                to: account,
                lamports: Rent::get()?.minimum_balance(size_T),
                space: size_T as u64,
                owner: self.program_id,
            }
            .invoke_signed(&[seeds.clone()])?;

            let mut data = account.try_borrow_mut_data()?;
            if data.len() != size_T {
                return Err(ProgramError::InvalidAccountData);
            }

            if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            let t_ref: &mut T = bytemuck::from_bytes_mut(&mut data);
            *t_ref = value;
            Ok(())
        } else {
            return Err(pinocchio::program_error::ProgramError::IllegalOwner);
        }
    }
}
