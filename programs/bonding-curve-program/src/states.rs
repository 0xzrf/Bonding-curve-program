use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct LiquidityPool {
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub bump: u8,
    pub reserve_bump: u8
}

#[account]
#[derive(InitSpace)]
pub struct LiquidityProvider {
    
}