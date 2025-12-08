use pinocchio::{account_info::AccountInfo, entrypoint, pubkey::Pubkey, ProgramResult};

use crate::instructions::EscrowInstrctions;

mod instructions;
mod state;
mod tests;

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
