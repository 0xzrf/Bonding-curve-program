use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub bump: u8,
    pub delta: u16
}

#[account]
#[derive(InitSpace)]
pub struct LiquidityProvider {
    
}