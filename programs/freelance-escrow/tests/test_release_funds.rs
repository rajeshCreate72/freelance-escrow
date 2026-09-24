use anchor_lang::{
    AccountDeserialize, InstructionData, ToAccountMetas, Space,
    solana_program::{instruction::Instruction, system_program},
};
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use freelance_escrow::state::EscrowState;
mod common;
use common::*;

#[test]
fn test_release_funds() {
    let mut setup = setup_funded_and_accepted_escrow(1000, 0, 0);

    let instruction_complete_job = Instruction::new_with_bytes(
        freelance_escrow::id(), 
        &freelance_escrow::instruction::CompleteJob {}.data(), 
        freelance_escrow::accounts::CompleteJob {
            freelancer: setup.freelancer.pubkey(),
            escrow: setup.escrow
        }
        .to_account_metas(None)
    );

    let blockhash_complete_job = setup.svm.latest_blockhash();
    let msg_job = Message::new_with_blockhash(&[instruction_complete_job], Some(&setup.freelancer.pubkey()), &blockhash_complete_job);
    let tx_job = VersionedTransaction::try_new(VersionedMessage::Legacy(msg_job), &[&setup.freelancer]).unwrap();
    setup.svm.send_transaction(tx_job).unwrap();

    let freelance_lamports_before = setup.svm.get_account(&setup.freelancer.pubkey()).unwrap().lamports;

    let instruction = Instruction::new_with_bytes(
        freelance_escrow::id(),
        &freelance_escrow::instruction::ReleaseFunds {}.data(), 
        freelance_escrow::accounts::ReleaseFunds {
            client: setup.client.pubkey(),
            escrow: setup.escrow,
            freelancer: setup.freelancer.pubkey(),
            system_program: system_program::ID
        }
        .to_account_metas(None)
    );

    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&setup.client.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.client]).unwrap();

    let res = setup.svm.send_transaction(tx);
    assert!(res.is_ok(), "Funds cannot be relesed: {:?}", res);


    let escrow_account = setup.svm.get_account(&setup.escrow).unwrap();
    let freelance_lamports_after  = setup.svm.get_account(&setup.freelancer.pubkey()).unwrap().lamports;
    let rent = setup.svm.minimum_balance_for_rent_exemption(8 + EscrowState::INIT_SPACE);

    let mut data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowState::try_deserialize(&mut data).unwrap();

    assert_eq!(freelance_lamports_after - freelance_lamports_before, 1_000_000_000);
    assert_eq!(escrow_account.lamports, rent);
    assert_eq!(escrow_state.status, freelance_escrow::state::Status::Released);
}