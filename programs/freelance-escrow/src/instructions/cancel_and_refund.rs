use anchor_lang::prelude::*;

use crate::state::{EscrowState, Status};
use crate::errors::EscrowError;


#[derive(Accounts)]
pub struct CancelAndRefund<'info> {
    #[account(mut)]
    client: Signer<'info>,
    #[account(
        mut,
        has_one = client,
        constraint = escrow.status == Status::Funded || escrow.status == Status::Accepted
    )]
    escrow: Account<'info, EscrowState>,
}

pub fn handle_cancel_and_refund(ctx: Context<CancelAndRefund>) -> Result<()> {
    let escrow = &ctx.accounts.escrow;
    let current_time = Clock::get()?.unix_timestamp;
    
    require!(current_time >= escrow.deadline, EscrowError::TooEarlyToCancel);

    let amount = ctx.accounts.escrow.amount;
    **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.client.to_account_info().try_borrow_mut_lamports()? += amount;

    ctx.accounts.escrow.status = Status::Cancelled;


    Ok(())
}