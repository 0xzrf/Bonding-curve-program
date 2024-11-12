use crate::states::LiquidityPool;
use anchor_lang::prelude::*;
use anchor_spl::{associated_token::AssociatedToken, token::{Mint, Token, TokenAccount}};

#[derive(Accounts)]
pub struct CreatePool<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: read only authority
    #[account(
        seeds = [
            mint_a.key().as_ref(),
            mint_b.key().as_ref(),
            b"pool_authority"
        ],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,
    #[account(
            init,
            seeds = [b"liquidity_pool", mint_a.key().as_ref(), mint_b.key().as_ref()],
            bump,
            payer = payer,
            space = LiquidityPool::INIT_SPACE + 8
        )]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    #[account(
        init, 
        seeds = [b"liquidity_token",mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump,
        mint::decimals = 6,
        mint::authority = pool_authority,
        payer = payer
    )]
    pub mint_liquidity: Box<Account<'info, Mint>>,
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(
        init, 
        payer = payer, 
        associated_token::mint = mint_a,
        associated_token::authority = pool_authority
    )]
    pub pool_account_a: Box<Account<'info, TokenAccount>>,

    #[account(
        init, 
        payer = payer, 
        associated_token::mint = mint_b,
        associated_token::authority = pool_authority
    )]
    pub pool_account_b: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>
}

pub fn create_pool(ctx: Context<CreatePool>)-> Result<()> {

    ctx.accounts.liquidity_pool.set_inner(LiquidityPool {
        mint_a: ctx.accounts.mint_a.key(),
        mint_b: ctx.accounts.mint_b.key(),
        bump: ctx.bumps.liquidity_pool,
        delta: 0
    });

    Ok(())
}

