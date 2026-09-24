use anchor_lang::{
    InstructionData, ToAccountMetas,
    prelude::Pubkey,
    solana_program::{instruction::Instruction, system_program},
};
use litesvm::LiteSVM;
use solana_keypair::Keypair;
use solana_message::{Message, VersionedMessage};
use solana_signer::Signer;
use solana_transaction::versioned::VersionedTransaction;

pub struct TestSetup {
    pub svm: LiteSVM,
    pub client: Keypair,
    pub freelancer: Keypair,
    pub escrow: Pubkey,
    pub job_id: u64,
}

pub fn setup_and_initialize_escrow(deadline: i64, grace_period: i64, review_window: i64) -> TestSetup {
    let program_id = freelance_escrow::id();
    let client = Keypair::new();
    let freelancer = Keypair::new();

    let mut svm = LiteSVM::new();
    let bytes = include_bytes!(concat!(env!("CARGO_TARGET_TMPDIR"), "/../deploy/freelance_escrow.so"));
    svm.add_program(program_id, bytes).unwrap();
    svm.airdrop(&client.pubkey(), 2_000_000_000).unwrap();
    svm.airdrop(&freelancer.pubkey(), 2_000_000_000).unwrap();

    let job_id: u64 = 1;
    let (escrow, _bump) = Pubkey::find_program_address(
        &[b"escrow", client.pubkey().as_ref(), &job_id.to_le_bytes()],
        &program_id,
    );

    let instruction = Instruction::new_with_bytes(
        program_id,
        &freelance_escrow::instruction::InitializeEscrow {
            job_id,
            freelancer: freelancer.pubkey(),
            amount: 1_000_000_000,
            deadline,
            grace_period,
            review_window,
        }
        .data(),
        freelance_escrow::accounts::InitializeEscrow {
            client: client.pubkey(),
            escrow,
            system_program: system_program::ID,
        }
        .to_account_metas(None),
    );

    let blockhash = svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[instruction], Some(&client.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&client]).unwrap();
    let res = svm.send_transaction(tx);
    assert!(res.is_ok(), "Initialize failed: {:?}", res);

    TestSetup { svm, client, freelancer, escrow, job_id }
}

pub fn setup_funded_and_accepted_escrow(deadline: i64, grace_period: i64, review_window: i64) -> TestSetup {
    let mut setup = setup_and_initialize_escrow(deadline, grace_period, review_window);

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

    let accept_ix = Instruction::new_with_bytes(
        freelance_escrow::id(),
        &freelance_escrow::instruction::AcceptJob {}.data(),
        freelance_escrow::accounts::AcceptJob {
            freelancer: setup.freelancer.pubkey(),
            escrow: setup.escrow,
        }
        .to_account_metas(None),
    );
    let blockhash = setup.svm.latest_blockhash();
    let msg = Message::new_with_blockhash(&[accept_ix], Some(&setup.freelancer.pubkey()), &blockhash);
    let tx = VersionedTransaction::try_new(VersionedMessage::Legacy(msg), &[&setup.freelancer]).unwrap();
    setup.svm.send_transaction(tx).unwrap();

    setup
}