use anchor_lang::prelude::*;

use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Burn, Mint, TokenAccount, Transfer, Token}
};

use fixed::types::I64F64;
use half::f16;
use crate::{
    constants::*,
    states::*,
    errors::Errors
};


pub fn widthdraw_lp(ctx: Context<Withdraw>, amount: u64) -> Result<()> {

    require!(ctx.accounts.withdraw_liquidity.amount >= amount,Errors::InvalidWithdrawAmount); // Checks if the user is trying to call the function with liquidity amount more that they own
    let pool_a_initial = ctx.accounts.pool_account_a.amount;
    let pool_b_initial = ctx.accounts.pool_account_b.amount;

    let authroity_bumps = ctx.bumps.pool_authority;
    let authority_seeds = &[
        &ctx.accounts.mint_a.key().to_bytes(),
        &ctx.accounts.mint_b.key().to_bytes(),
        AUTHORITY_SEED,
        &[authroity_bumps]
        ];
    let signer_seeds = &[&authority_seeds[..]];
        
    

    let amount_a = I64F64::from_num(amount)
        .checked_mul(I64F64::from_num(ctx.accounts.pool_account_a.amount))
        .unwrap()
        .checked_div(I64F64::from_num(
            ctx.accounts.mint_liquidity.supply + MINIMUM_LIQUIDITY
        ))
        .unwrap()
        .floor()
        .to_num::<u64>();
      
    token::transfer(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.pool_account_a.to_account_info(),
                to: ctx.accounts.withdrawer_account_a.to_account_info(),
                authority: ctx.accounts.pool_authority.to_account_info(),
            },
            signer_seeds,
        ),
        amount_a,
    )?;
    
    let amount_b = I64F64::from_num(amount)
    .checked_mul(I64F64::from_num(ctx.accounts.pool_account_b.amount))
    .unwrap()
    .checked_div(I64F64::from_num(
        ctx.accounts.mint_liquidity.supply + MINIMUM_LIQUIDITY,
    ))
    .unwrap()
    .floor()
    .to_num::<u64>();
    token::transfer(
    CpiContext::new_with_signer(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.pool_account_b.to_account_info(),
            to: ctx.accounts.withdrawer_account_b.to_account_info(),
            authority: ctx.accounts.pool_authority.to_account_info(),
        },
        signer_seeds,
    ),
    amount_b,
)?;

    
    // Burn the liquidity tokens
    // It will fail if the amount is invalid
    token::burn(
    CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Burn {
            mint: ctx.accounts.mint_liquidity.to_account_info(),
            from: ctx.accounts.withdraw_liquidity.to_account_info(),
            authority: ctx.accounts.withdrawer.to_account_info(),
        },
    ),
    amount,
    )?;

    ctx.accounts.pool_account_a.reload()?;
    ctx.accounts.pool_account_b.reload()?;

    // stores the new delta of the pool
    let delta: f16 = f16::from_f32(
        (pool_a_initial as f32 / pool_b_initial as f32) // Tokens before deposit 
        / (ctx.accounts.pool_account_a.amount as f32 / ctx.accounts.pool_account_b.amount as f32) // Amount of tokens after the deposit
     );
    let delta_bits = delta.to_bits();
    ctx.accounts.liquidity_pool.delta = delta_bits;

    Ok(())

}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    pub withdrawer: Signer<'info>,
    #[account(
    mut,
        seeds = [
            b"liquidity_pool",
            liquidity_pool.mint_a.key().as_ref(),
            liquidity_pool.mint_b.key().as_ref(),
        ], 
        bump = liquidity_pool.bump
    )]
    pub liquidity_pool: Account<'info, LiquidityPool>,
    #[account( 
        mut,
        seeds = [b"liquidity_token",mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump,
    )]
    pub mint_liquidity: Box<Account<'info, Mint>>,
    /// CHECK: read only authority
    #[account(
    seeds = [
    mint_a.key().as_ref(),
    mint_b.key().as_ref(),
    AUTHORITY_SEED
    ],
    bump
     )]
    pub pool_authority: AccountInfo<'info>,
    #[account(mut)]
    pub mint_a: Account<'info, Mint>,
    #[account(mut)]
    pub mint_b: Account<'info, Mint>,
    #[account(
        mut, 
        associated_token::authority = pool_authority,
        associated_token::mint  = mint_a
    )]
    pub pool_account_a: Account<'info, TokenAccount>,
    #[account(
        mut, 
        associated_token::authority = pool_authority,
        associated_token::mint  = mint_b
    )]
    pub pool_account_b: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = withdrawer,
        associated_token::mint = mint_liquidity,
        associated_token::authority = withdrawer
    )]
    pub withdraw_liquidity: Box<Account<'info, TokenAccount>>,
    #[account(
        mut, 
        associated_token::authority = withdrawer,
        associated_token::mint  = mint_a
    )]
    pub withdrawer_account_a: Account<'info, TokenAccount>,
    #[account(
        mut, 
        associated_token::authority = withdrawer,
        associated_token::mint  = mint_b
    )]
    pub withdrawer_account_b: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>    
}
