use anchor_lang::prelude::*;

use crate::state::{EscrowState, Status};
use crate::errors::EscrowError;

#[derive(Accounts)]
pub struct CompleteJob<'info> {
    pub freelancer: Signer<'info>,
    #[account(
        mut,
        has_one = freelancer,
        constraint = escrow.status == Status::Accepted,
        constraint = Clock::get()?.unix_timestamp <= escrow.deadline + escrow.grace_period @ EscrowError::TooLateToSetToComplete

    )]
    pub escrow: Account<'info, EscrowState>
}

pub fn handle_complete_job(ctx: Context<CompleteJob>) -> Result<()> {
    ctx.accounts.escrow.completed_at = Clock::get()?.unix_timestamp;
    ctx.accounts.escrow.status = Status::Completed;
    Ok(())
}