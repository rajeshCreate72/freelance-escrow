use anchor_lang::prelude::*;
pub mod state;
pub mod instructions;
pub use instructions::*;
pub mod errors;

declare_id!("HfXLi5zKpUUUuNfRiX5mC7YWURacwGFabfY27sd89DMq");

#[program]
pub mod freelance_escrow {
    use super::*;

    pub fn initialize_escrow (
        ctx: Context<InitializeEscrow>,
        job_id: u64,
        freelancer: Pubkey,
        amount: u64,
        deadline: i64,
        grace_period: i64,
        review_window: i64
    ) -> Result<()> {
        handle_initialize_escrow(ctx, job_id, freelancer, amount, deadline, grace_period, review_window)
    }

    pub fn fund_escrow(
        ctx: Context<FundEscrow>,
    ) -> Result<()> {
        handle_fund_escrow(ctx)
    }

    pub fn accept_job(ctx: Context<AcceptJob>) -> Result<()> {
        handle_accept_job(ctx)
    }

    pub fn complete_job(ctx: Context<CompleteJob>) -> Result<()> {
        handle_complete_job(ctx)
    }

    pub fn release_funds(ctx: Context<ReleaseFunds>) -> Result<()> {
        handle_release_funds(ctx)
    }

    pub fn claim_after_deadline(ctx:Context<ClaimAfterDeadline>) -> Result<()> {
        handle_claim_after_deadline(ctx)
    }

    pub fn cancel_and_refund(ctx: Context<CancelAndRefund>) -> Result<()> {
        handle_cancel_and_refund(ctx)
    }
}