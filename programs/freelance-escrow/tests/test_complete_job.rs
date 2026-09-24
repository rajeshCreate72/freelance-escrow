use anchor_lang::{
    AccountDeserialize, InstructionData, ToAccountMetas,
    solana_program::{instruction::Instruction},
};
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use freelance_escrow::state::EscrowState;
mod common;
use common::*;

#[test]
fn test_complete_job() {
    let mut setup = setup_funded_and_accepted_escrow(1000, 0, 0);

    let instruction = Instruction::new_with_bytes(
        freelance_escrow::id(), 
        &freelance_escrow::instruction::CompleteJob {}.data(), 
        freelance_escrow::accounts::CompleteJob {
            freelancer: setup.freelancer.pubkey(),
            escrow: setup.escrow
        }
        .to_account_metas(None)
    );

    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&setup.freelancer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.freelancer]).unwrap();

    let res = setup.svm.send_transaction(tx);

    assert!(res.is_ok(), "Complete job instruction failed {:?}", res);

    let escrow_account = setup.svm.get_account(&setup.escrow).unwrap();
    let mut data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowState::try_deserialize(&mut data).unwrap();

    assert_eq!(escrow_state.status, freelance_escrow::state::Status::Completed);
}