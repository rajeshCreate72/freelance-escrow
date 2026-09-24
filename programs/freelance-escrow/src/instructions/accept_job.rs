use anchor_lang::prelude::*;
use crate::state::{EscrowState, Status};
use crate::errors::EscrowError;

#[derive(Accounts)]
pub struct AcceptJob<'info> {
    pub freelancer: Signer<'info>,

    #[account(
        mut,
        has_one = freelancer,
        constraint = escrow.status == Status::Funded,
        constraint = Clock::get()?.unix_timestamp <= escrow.deadline + escrow.grace_period @ EscrowError::TooLateToSetToComplete
    )]
    pub escrow: Account<'info, EscrowState>
}

pub fn handle_accept_job(ctx: Context<AcceptJob>) -> Result<()> {
    ctx.accounts.escrow.status = Status::Accepted;

    Ok(())
}