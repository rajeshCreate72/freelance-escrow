use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use crate::state::{EscrowState, Status};

#[derive(Accounts)]
pub struct FundEscrow<'info> {
    #[account(mut)]
    pub client: Signer<'info>,
    #[account(mut)]
    pub escrow: Account<'info, EscrowState>,
    pub system_program: Program<'info, System>
}

pub fn handle_fund_escrow(ctx: Context<FundEscrow>) -> Result<()> {
    let amount = ctx.accounts.escrow.amount;

    let cpi_accounts = Transfer {
        from: ctx.accounts.client.to_account_info(),
        to: ctx.accounts.escrow.to_account_info(),
    };
    let cpi_program = ctx.accounts.system_program.key();
    let cpi_ctx = CpiContext::new(cpi_program, cpi_accounts);

    transfer(cpi_ctx, amount)?;

    ctx.accounts.escrow.status = Status::Funded;

    Ok(())
}