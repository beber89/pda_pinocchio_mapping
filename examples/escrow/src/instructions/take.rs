use pinocchio::{account_info::AccountInfo, msg, ProgramResult};

use crate::state::Share;
use pda_pinocchio_mapping::mapping;

pub fn process_take_instruction(accounts: &[AccountInfo], data: &[u8]) -> ProgramResult {
    msg!("Processing Take instruction");

    let [taker, maker, escrow_account, shares_account, _system_program, _rent_sysvar @ ..] =
        accounts
    else {
        return Err(pinocchio::program_error::ProgramError::NotEnoughAccountKeys);
    };

    let shares_bump = data[0];

    // No need for bump, therefore arguments start at 0
    let amount_to_receive = unsafe { *(data.as_ptr().add(1) as *const u64) };
    let amount_to_give = unsafe { *(data.as_ptr().add(8 + 1) as *const u64) };
    let shares_state = Share {
        maker: *maker.key(),
        taker: *taker.key(),
        amount: amount_to_give.to_le_bytes(),
        bump: shares_bump,
    };

    let shares = mapping!(b"shares", taker);
    shares.set(maker.key(), shares_state, shares_account)?;

    {
        pinocchio_system::instructions::Transfer {
            from: &taker,
            to: &maker,
            lamports: amount_to_receive,
        }
        .invoke()?;
    }
    Ok(())
}
