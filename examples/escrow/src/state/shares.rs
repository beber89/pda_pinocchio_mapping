use bytemuck::{Pod, Zeroable};
use pda_pinocchio_mapping::Bumpy;
use pda_pinocchio_mapping::Mapping;
use pinocchio::{
    account_info::AccountInfo,
    instruction::{Seed, Signer},
    program_error::ProgramError,
    pubkey::Pubkey,
    sysvars::{rent::Rent, Sysvar},
    ProgramResult,
};
use pinocchio_pubkey;
use pinocchio_system::instructions::CreateAccount;

pub trait Sizeable: Pod {
    const LEN: usize;
}
impl<T: Pod> Sizeable for T {
    const LEN: usize = core::mem::size_of::<T>();
}

impl Bumpy for Share {
    fn bump(&self) -> u8 {
        self.bump
    }
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Zeroable, Pod)]
pub struct Share {
    // Trader
    pub maker: [u8; 32],
    // Follow
    pub taker: [u8; 32],
    pub amount: [u8; 8],
    pub bump: u8,
}

impl Share {
    // TODO: Make as a macro
    /**
     * Create account
     *
     */
    pub fn mapping_set<T: Sizeable + Bumpy>(
        key: &Pubkey,
        value: T,
        account: &AccountInfo,
        payer: &AccountInfo,
    ) -> ProgramResult {
        let seed = [b"shares", key.as_slice(), &[value.bump()]];

        let account_pda = pinocchio_pubkey::derive_address(&seed, None, &crate::ID);
        assert_eq!(account_pda, *account.key(), "Mapping: Accounts Mismatching");
        let bump = [value.bump().to_le()];
        let seed = [
            pinocchio::instruction::Seed::from(b"shares"),
            pinocchio::instruction::Seed::from(key.as_slice()),
            pinocchio::instruction::Seed::from(&bump),
        ];
        let seeds = pinocchio::instruction::Signer::from(&seed);

        if account.owner() != &crate::ID {
            // Account does not exist
            CreateAccount {
                from: payer,
                to: account,
                lamports: Rent::get()?.minimum_balance(T::LEN),
                space: T::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[seeds.clone()])?;

            let mut data = account.try_borrow_mut_data()?;
            if data.len() != T::LEN {
                return Err(ProgramError::InvalidAccountData);
            }

            if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            let t_ref: &mut T = bytemuck::from_bytes_mut(&mut data);
            *t_ref = value;
            Ok(())
        } else {
            // Account already exists
            let mut data = account.try_borrow_mut_data()?;
            if data.len() != T::LEN {
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

    pub fn mapping_create<T: Sizeable + Bumpy>(
        key: &Pubkey,
        value: T,
        account: &AccountInfo,
        payer: &AccountInfo,
    ) -> ProgramResult {
        let seed = [b"shares", key.as_slice(), &[value.bump()]];

        let account_pda = pinocchio_pubkey::derive_address(&seed, None, &crate::ID);
        assert_eq!(account_pda, *account.key(), "Mapping: Accounts Mismatching");
        let bump = [value.bump().to_le()];
        let seed = [
            pinocchio::instruction::Seed::from(b"shares"),
            pinocchio::instruction::Seed::from(key.as_slice()),
            pinocchio::instruction::Seed::from(&bump),
        ];
        let seeds = pinocchio::instruction::Signer::from(&seed);

        if account.owner() != &crate::ID {
            CreateAccount {
                from: payer,
                to: account,
                lamports: Rent::get()?.minimum_balance(T::LEN),
                space: T::LEN as u64,
                owner: &crate::ID,
            }
            .invoke_signed(&[seeds.clone()])?;

            let mut data = account.try_borrow_mut_data()?;
            if data.len() != T::LEN {
                return Err(ProgramError::InvalidAccountData);
            }

            if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            let t_ref: &mut T = bytemuck::from_bytes_mut(&mut data);
            *t_ref = value;
            Ok(())
        } else {
            return Err(pinocchio::program_error::ProgramError::AccountAlreadyInitialized);
        }
    }

    // FIXME: Needs to be fixed (returns invalid AccountData)
    pub fn mapping_get<T: Sizeable>(
        key: &Pubkey,
        account: &AccountInfo,
        bump: u8,
    ) -> Result<T, ProgramError> {
        let seed = [b"shares", key.as_slice(), &[bump]];

        let account_pda = pinocchio_pubkey::derive_address(&seed, None, &crate::ID);
        assert_eq!(account_pda, *account.key(), "Mapping: Accounts Mismatching");

        if account.owner() != &crate::ID {
            // Account does not exist
            Err(ProgramError::IllegalOwner)
        } else {
            // Account exists
            let data = account.try_borrow_data()?;
            // FIXME: Throws the error here
            if data.len() != T::LEN {
                return Err(ProgramError::InvalidAccountData);
            }

            if (data.as_ptr() as usize) % core::mem::align_of::<Self>() != 0 {
                return Err(ProgramError::InvalidAccountData);
            }
            let t_ref: &T = bytemuck::from_bytes(&data);
            Ok(*t_ref)
        }
    }
}
