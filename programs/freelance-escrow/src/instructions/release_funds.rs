use anchor_lang::prelude::*;
use crate::state::{EscrowState, Status};


#[derive(Accounts)]
pub struct ReleaseFunds<'info> {
    pub client: Signer<'info>,
    #[account(
        mut, 
        has_one = client,
        has_one = freelancer,
        seeds = [b"escrow", client.key().as_ref(), escrow.job_id.to_le_bytes().as_ref()],
        bump,
        constraint = escrow.status == Status::Completed  
    )]
    pub escrow: Account<'info, EscrowState>,
    #[account(mut)]
    pub freelancer: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
}

pub fn handle_release_funds(ctx: Context<ReleaseFunds>) -> Result<()> {
    let amount = ctx.accounts.escrow.amount;

    **ctx.accounts.escrow.to_account_info().try_borrow_mut_lamports()? -= amount;
    **ctx.accounts.freelancer.to_account_info().try_borrow_mut_lamports()? += amount;

    ctx.accounts.escrow.status = Status::Released;

    Ok(())
}
