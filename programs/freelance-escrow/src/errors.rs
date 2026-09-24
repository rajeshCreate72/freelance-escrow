use anchor_lang::prelude::*;

#[error_code]
pub enum EscrowError {
    #[msg("Cannot claim before the review window and grace period have elapsed")]
    TooEarlyToClaim,

    #[msg("Cannot be canceled before deadline")]
    TooEarlyToCancel,

    #[msg("Cannot set to complete when deadline passed.")]
    TooLateToSetToComplete
}