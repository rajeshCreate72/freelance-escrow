use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, InitSpace, Clone, PartialEq, Debug)]
pub enum Status {
    Funded, 
    Accepted, 
    Released,
    Completed,
    Cancelled, 
    Pending  
}

#[account]
#[derive(InitSpace)]
pub struct EscrowState {
    pub client: Pubkey,
    pub freelancer: Pubkey,
    pub amount: u64,
    pub status: Status,
    pub deadline: i64,
    pub grace_period: i64,
    pub review_window: i64,
    pub completed_at: i64,
    pub job_id: u64
}