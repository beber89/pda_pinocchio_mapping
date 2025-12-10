#![cfg_attr(not(test), no_std)]
use pinocchio::{
    account_info::AccountInfo, entrypoint, nostd_panic_handler, pubkey::Pubkey, ProgramResult,
};

use crate::instructions::EscrowInstrctions;

#[cfg(test)]
extern crate std;

extern crate alloc;
pub use alloc::vec::Vec;

// Use the no_std panic handler.
#[cfg(target_os = "solana")]
nostd_panic_handler!();

#[cfg(test)]
mod tests;

mod instructions;
mod state;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pinocchio_pubkey::declare_id!("EBhru2Pe8LgDarZW128w9jMsJoVTukXZHY6Jf5ZmRqVi");

pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    assert_eq!(program_id, &ID);

    let (discriminator, data) = instruction_data
        .split_first()
        .ok_or(pinocchio::program_error::ProgramError::InvalidInstructionData)?;

    match EscrowInstrctions::try_from(discriminator)? {
        EscrowInstrctions::Make => instructions::process_make_instruction(accounts, data)?,
        EscrowInstrctions::Take => instructions::process_take_instruction(accounts, data)?,
        _ => return Err(pinocchio::program_error::ProgramError::InvalidInstructionData),
    }
    Ok(())
}
