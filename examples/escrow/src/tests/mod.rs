#[cfg(test)]
mod tests {
    use crate::state::Share;
    use std::path::PathBuf;

    use litesvm::LiteSVM;
    use solana_instruction::{AccountMeta, Instruction};
    use solana_keypair::Keypair;
    use solana_message::Message;
    use solana_native_token::LAMPORTS_PER_SOL;
    use solana_program::{msg, rent::Rent, sysvar::SysvarId};
    use solana_pubkey::Pubkey;
    use solana_signer::Signer;
    use solana_transaction::Transaction;

    const INITIAL_BALANCE: u64 = 10 * LAMPORTS_PER_SOL;

    fn program_id() -> Pubkey {
        Pubkey::from(crate::ID)
    }

    fn setup() -> (LiteSVM, Keypair, Keypair) {
        let mut svm = LiteSVM::new();
        let payer = Keypair::new(); // also maker
        let taker = Keypair::new();

        svm.airdrop(&payer.pubkey(), INITIAL_BALANCE)
            .expect("Airdrop failed");
        svm.airdrop(&taker.pubkey(), INITIAL_BALANCE)
            .expect("Airdrop failed");

        // Load program SO file
        let so_path = PathBuf::from("../../target/deploy/escrow.so");
        let program_data = std::fs::read(so_path).expect("Failed to read program SO file");

        svm.add_program(program_id(), &program_data);

        (svm, payer, taker)
    }

    #[test]
    pub fn test_make_instruction() {
        let (mut svm, payer, _) = setup();

        let program_id = program_id();

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &program_id,
        );

        let system_program = solana_sdk_ids::system_program::ID;

        let amount_to_receive: u64 = 2_000_000_000; // 2 SOL with 9 decimal places
        let amount_to_give: u64 = 1_000_000_000; // 1 SOL with 9 decimal places
        let bump: u8 = escrow.1;

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![0u8], // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
    }

    #[test]
    pub fn test_take_instruction() {
        let (mut svm, payer, taker) = setup();

        let program_id = program_id();

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let shares = Pubkey::find_program_address(
            &[b"shares".as_ref(), payer.pubkey().as_ref()],
            &program_id,
        );
        msg!("shares PDA: {}\n", shares.0);

        // Derive the PDA for the escrow account using the maker's public key and a seed value
        let escrow = Pubkey::find_program_address(
            &[b"escrow".as_ref(), payer.pubkey().as_ref()],
            &program_id,
        );
        msg!("Escrow PDA: {}\n", escrow.0);

        let system_program = solana_sdk_ids::system_program::ID;

        let amount_to_receive: u64 = 2_000_000_000; // 2 SOL with 9 decimal places
        let amount_to_give: u64 = 1_000_000_000; // 1 SOL with 9 decimal places
        let bump: u8 = escrow.1;
        let shares_bump: u8 = shares.1;
        msg!("Bump: {}", bump);

        // Create the "Make" instruction to deposit tokens into the escrow
        let make_data = [
            vec![0u8], // Discriminator for "Make" instruction
            bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(),
            amount_to_give.to_le_bytes().to_vec(),
        ]
        .concat();
        let make_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(payer.pubkey(), true),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: make_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[make_ix], Some(&payer.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&payer], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        let maker_initial_balance = svm
            .get_account(&payer.pubkey())
            .expect("Could not retrieve Account")
            .lamports;

        // Log transaction details
        msg!("\n\nMake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);

        // Take
        // Create the "Take" instruction to deposit tokens into the escrow
        let take_ix_data = [
            vec![1u8], // Discriminator for "Take" instruction
            shares_bump.to_le_bytes().to_vec(),
            amount_to_receive.to_le_bytes().to_vec(), // Amount received by maker from taker
            amount_to_give.to_le_bytes().to_vec(),    // Amount given by maker to taker
        ]
        .concat();
        let take_ix = Instruction {
            program_id: program_id,
            accounts: vec![
                AccountMeta::new(taker.pubkey(), true),
                AccountMeta::new(payer.pubkey(), false),
                AccountMeta::new(escrow.0, false),
                AccountMeta::new(shares.0, false),
                AccountMeta::new(system_program, false),
                AccountMeta::new(Rent::id(), false),
            ],
            data: take_ix_data,
        };

        // Create and send the transaction containing the "Make" instruction
        let message = Message::new(&[take_ix], Some(&taker.pubkey()));
        let recent_blockhash = svm.latest_blockhash();

        let transaction = Transaction::new(&[&taker], message, recent_blockhash);

        // Send the transaction and capture the result
        let tx = svm.send_transaction(transaction).unwrap();

        // Log transaction details
        msg!("\n\nTake transaction sucessfull");
        msg!("CUs Consumed: {}", tx.compute_units_consumed);

        // POSTCONDITIONS
        let maker_account = svm
            .get_account(&payer.pubkey())
            .expect("Could not retrieve account properly");
        assert_eq!(
            maker_account.lamports,
            maker_initial_balance + amount_to_receive
        );

        let taker_account = svm
            .get_account(&taker.pubkey())
            .expect("Could not retrieve account properly");
        // Taker pays amout_togive + CUs
        assert!(
            taker_account.lamports
                < INITIAL_BALANCE - amount_to_receive - tx.compute_units_consumed
        );

        let shares_bytes = svm
            .get_account(&shares.0)
            .expect("Could not retrieve account properly")
            .data;

        let shares_ref: &Share = bytemuck::from_bytes(&shares_bytes);
        msg!("amount is {}", u64::from_le_bytes(shares_ref.amount));
    }
}
