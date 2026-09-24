use anchor_lang::prelude::*;
use crate::state::EscrowState;
use crate::state::Status;

#[derive(Accounts)]
#[instruction(job_id: u64)]
pub struct InitializeEscrow<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(
        init, 
        payer=client, 
        space=8 + EscrowState::INIT_SPACE, 
        seeds=[b"escrow", client.key().as_ref(), job_id.to_le_bytes().as_ref()], 
        bump
    )]
    pub escrow: Account<'info, EscrowState>,
    pub system_program: Program<'info, System>
}

pub fn handle_initialize_escrow(
    ctx: Context<InitializeEscrow>, 
    job_id: u64, 
    freelancer: Pubkey, 
    amount: u64, 
    deadline: i64,
    grace_period: i64,
    review_window: i64
) -> Result<()> {
    let escrow = &mut ctx.accounts.escrow;

    escrow.client = ctx.accounts.client.key();
    escrow.freelancer = freelancer;
    escrow.job_id = job_id;
    escrow.amount = amount;
    escrow.deadline = deadline;
    escrow.grace_period = grace_period;
    escrow.review_window = review_window;
    escrow.status = Status::Pending;
    escrow.completed_at = 0;

    Ok(())
}