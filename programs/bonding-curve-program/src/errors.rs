use anchor_lang::prelude::*;


#[error_code]
pub enum Errors {
    #[msg("Invalid deposit amount")]
    InvalidDepositAmount,
    #[msg("Output amount too small")]
    OutputTooSmall,
    #[msg("Invariant violated")]
    InvarianceViolated,
    #[msg("Invalid withdraw amount")]
    InvalidWithdrawAmount
}