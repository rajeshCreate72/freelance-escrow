use anchor_lang::{ AccountDeserialize };
use solana_signer::Signer;
use freelance_escrow::state::EscrowState;
mod common;
use common::*;

#[test]
fn test_initialize_escrow() {
    let setup = setup_and_initialize_escrow(9_999_999_999, 100_000, 0);

    let escrow_account = setup.svm.get_account(&setup.escrow).unwrap();
    let mut data: &[u8] = &escrow_account.data;
    let escrow_state = EscrowState::try_deserialize(&mut data).unwrap();

    assert_eq!(escrow_state.client, setup.client.pubkey());
    assert_eq!(escrow_state.job_id, setup.job_id);
    assert_eq!(escrow_state.amount, 1_000_000_000);
}