use anchor_lang::prelude::*;

use crate::state::{EscrowState, Status};
use crate::errors::EscrowError;


#[derive(Accounts)]
pub struct ClaimAfterDeadline<'info> {
    freelancer: Signer<'info>,
    #[account(
        mut,
        has_one = freelancer,
        constraint = escrow.status == Status::Completed
    )]
    escrow: Account<'info, EscrowState>,
}

pub fn handle_claim_after_deadline(ctx: Context<ClaimAfterDeadline>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;
    let current_time = Clock::get()?.unix_timestamp;
    let claimable_at = escrow.completed_at + escrow.review_window + escrow.grace_period;

    require!(current_time >= claimable_at, EscrowError::TooEarlyToClaim);

    let amount = ctx.accounts.escrow.amount;
    **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.freelancer.to_account_info().try_borrow_mut_lamports()? += amount;

    ctx.accounts.escrow.status = Status::Released;

    Ok(())
}