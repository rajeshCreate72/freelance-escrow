use anchor_lang::{
    AccountDeserialize, InstructionData, ToAccountMetas, Space,
    solana_program::{instruction::Instruction, system_program},
    prelude::Clock,
};
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use freelance_escrow::state::EscrowState;
mod common;
use common::*;

#[test]
fn test_cancel_and_refund_funded() {
    let mut setup = setup_and_initialize_escrow(9_999_999_999, 0, 0);

    // fund it only — no accept_job
    let fund_ix = Instruction::new_with_bytes(
        freelance_escrow::id(),
        &freelance_escrow::instruction::FundEscrow {}.data(),
        freelance_escrow::accounts::FundEscrow {
            client: setup.client.pubkey(),
            escrow: setup.escrow,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );
    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[fund_ix], Some(&setup.client.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.client]).unwrap();
    setup.svm.send_transaction(tx).unwrap();

    let mut clock: Clock = setup.svm.get_sysvar();
    clock.unix_timestamp += 10_000_000_000;
    setup.svm.set_sysvar(&clock);
    
    let client_before = setup.svm.get_account(&setup.client.pubkey()).unwrap().lamports;

    let cancel_ix = Instruction::new_with_bytes(
        freelance_escrow::id(),
        &freelance_escrow::instruction::CancelAndRefund {}.data(),
        freelance_escrow::accounts::CancelAndRefund {
            client: setup.client.pubkey(),
            escrow: setup.escrow,
        }
        .to_account_metas(None),
    );

    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[cancel_ix], Some(&setup.client.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.client]).unwrap();
    let res = setup.svm.send_transaction(tx);
    assert!(res.is_ok(), "Cancel from Funded should succeed: {:?}", res);

    let escrow_account = setup.svm.get_account(&setup.escrow).unwrap();
    let client_after = setup.svm.get_account(&setup.client.pubkey()).unwrap().lamports;
    let rent = setup.svm.minimum_balance_for_rent_exemption(8 + EscrowState::INIT_SPACE);

    let mut data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowState::try_deserialize(&mut data).unwrap();

    assert_eq!(client_after - client_before, 1_000_000_000 - 5000);
    assert_eq!(escrow_account.lamports, rent);
    assert_eq!(escrow_state.status, freelance_escrow::state::Status::Cancelled);
}

#[test]
fn test_cancel_and_refund_accepted() {
    let mut setup = setup_funded_and_accepted_escrow(9_999_999_999, 0, 0);

    let mut clock: Clock = setup.svm.get_sysvar();
    clock.unix_timestamp += 10_000_000_000;
    setup.svm.set_sysvar(&clock);
    
    let client_before = setup.svm.get_account(&setup.client.pubkey()).unwrap().lamports;

    let cancel_ix = Instruction::new_with_bytes(
        freelance_escrow::id(),
        &freelance_escrow::instruction::CancelAndRefund {}.data(),
        freelance_escrow::accounts::CancelAndRefund {
            client: setup.client.pubkey(),
            escrow: setup.escrow,
        }
        .to_account_metas(None),
    );

    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[cancel_ix], Some(&setup.client.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.client]).unwrap();
    let res = setup.svm.send_transaction(tx);
    assert!(res.is_ok(), "Cancel from Funded should succeed: {:?}", res);

    let escrow_account = setup.svm.get_account(&setup.escrow).unwrap();
    let client_after = setup.svm.get_account(&setup.client.pubkey()).unwrap().lamports;
    let rent = setup.svm.minimum_balance_for_rent_exemption(8 + EscrowState::INIT_SPACE);

    let mut data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowState::try_deserialize(&mut data).unwrap();

    assert_eq!(client_after - client_before, 1_000_000_000 - 5000);
    assert_eq!(escrow_account.lamports, rent);
    assert_eq!(escrow_state.status, freelance_escrow::state::Status::Cancelled);
}