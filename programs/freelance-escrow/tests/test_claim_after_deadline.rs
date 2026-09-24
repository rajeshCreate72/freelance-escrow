use anchor_lang::{
    AccountDeserialize, InstructionData, ToAccountMetas, Space,
    solana_program::{instruction::Instruction},
    prelude::Clock,
};
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use freelance_escrow::state::EscrowState;
mod common;
use common::*;

#[test]
fn test_claim_after_deadline() {
    let mut setup = setup_funded_and_accepted_escrow(9_999_999_999, 100_000, 1000);

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

    
    let claim_ix =  Instruction::new_with_bytes(
        freelance_escrow::id(),
        &freelance_escrow::instruction::ClaimAfterDeadline {}.data(),
        freelance_escrow::accounts::ClaimAfterDeadline {
            freelancer: setup.freelancer.pubkey(),
            escrow: setup.escrow,
        }
        .to_account_metas(None)
    );
    
    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[claim_ix.clone()], Some(&setup.freelancer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.freelancer]).unwrap();
    let res = setup.svm.send_transaction(tx);
    assert!(res.is_err(), "Should not allow claiming before review window + grace period elapse");

    // Warp clock forward, past completed_at + review_window + grace_period
    let mut clock: Clock = setup.svm.get_sysvar();
    clock.unix_timestamp += 200_000;
    setup.svm.set_sysvar(&clock);

    // To avoid error while restarting the new blockhash for claim
    setup.svm.expire_blockhash();
    
    let freelancer_before = setup.svm.get_account(&setup.freelancer.pubkey()).unwrap().lamports;
    
    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[claim_ix], Some(&setup.freelancer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.freelancer]).unwrap();
    
    let res = setup.svm.send_transaction(tx);
    
    assert!(res.is_ok(), "Claim failed {:?}", res);
    
    let escrow_account = setup.svm.get_account(&setup.escrow).unwrap();
    let freelancer_after = setup.svm.get_account(&setup.freelancer.pubkey()).unwrap().lamports;

    let mut data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowState::try_deserialize(&mut data).unwrap();
    let rent = setup.svm.minimum_balance_for_rent_exemption(8 + EscrowState::INIT_SPACE);

    assert_eq!(freelancer_after - freelancer_before, 1_000_000_000 - 5000);
    assert_eq!(escrow_account.lamports, rent);
    assert_eq!(escrow_state.status, freelance_escrow::state::Status::Released);
}