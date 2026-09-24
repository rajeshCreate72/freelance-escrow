use anchor_lang::{
    AccountDeserialize, InstructionData, ToAccountMetas,
    solana_program::{instruction::Instruction, system_program},
};
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;
use freelance_escrow::state::EscrowState;
mod common;
use common::*;

#[test]
fn test_fund_escrow() {
    let mut setup = setup_and_initialize_escrow(9_999_999_999, 100_000, 0);

    // build and send FundEscrow instruction here
    let instruction = Instruction::new_with_bytes(
        freelance_escrow::id(), 
        &freelance_escrow::instruction::FundEscrow {}.data(), 
        freelance_escrow::accounts::FundEscrow {
            client: setup.client.pubkey(),
            escrow: setup.escrow,
            system_program: system_program::ID,
        }
        .to_account_metas(None)
    );

    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&setup.client.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.client]).unwrap();

    let res = setup.svm.send_transaction(tx);
    assert!(res.is_ok(), "Fund failed: {:?}", res);

    let escrow_account = setup.svm.get_account(&setup.escrow).unwrap();
    let mut data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowState::try_deserialize(&mut data).unwrap();

    assert_eq!(escrow_state.status, freelance_escrow::state::Status::Funded);

}