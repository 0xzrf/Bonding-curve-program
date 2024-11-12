use anchor_lang::prelude::*;
use anchor_spl::{
    token::{
        self,
        Token,
        TokenAccount,
        Mint,
        Transfer
    },
    associated_token::AssociatedToken
};

use crate::{
    errors::Errors,
    states::*,
    constants::*
};
use fixed::types::I64F64;
use half::f16;

#[derive(Accounts)]
pub struct Swap<'info> {
    #[account(mut)]
    pub trader: Signer<'info>,
    #[account(
        mut,
        seeds = [
            b"liquidity_pool",
            liquidity_pool.mint_a.key().as_ref(),
            liquidity_pool.mint_b.key().as_ref(),
        ], 
        bump = liquidity_pool.bump,
        has_one = mint_a,
        has_one = mint_b,
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
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    #[account(
        mut, 
        associated_token::authority = pool_authority,
        associated_token::mint  = mint_a
    )]
    pub pool_account_a: Box<Account<'info, TokenAccount>>,
    #[account(
        mut, 
        associated_token::authority = pool_authority,
        associated_token::mint  = mint_b
    )]
    pub pool_account_b: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed, 
        payer = trader,
        associated_token::mint = mint_a,
        associated_token::authority = trader
    )]
    pub trader_ata_a: Box<Account<'info, TokenAccount>>,
    #[account(
        init_if_needed, 
        payer = trader,
        associated_token::mint = mint_b,
        associated_token::authority = trader
    )]
    pub trader_ata_b: Box<Account<'info, TokenAccount>>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>   
}


pub fn swap(ctx: Context<Swap>, swap_a: bool, input_amount: u64, min_amount_out: u64) -> Result<()> {
    // Protect from user taking in more out from the pool than they have
    let input = if swap_a && input_amount > ctx.accounts.trader_ata_a.amount {
        ctx.accounts.trader_ata_a.amount
    } else if !swap_a && input_amount > ctx.accounts.trader_ata_b.amount {
        ctx.accounts.trader_ata_b.amount
    } else {
        input_amount
    };
    
    let pool_a_initial = ctx.accounts.pool_account_a.amount;
    let pool_b_initial = ctx.accounts.pool_account_b.amount;

    let fee = calculate_fee(ctx.accounts.liquidity_pool.delta);

    let taxed_amount = (input * fee)/ 1000;
    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_account_b;
    
    let output = if swap_a {
        I64F64::from_num(taxed_amount)
        .checked_mul(I64F64::from_num(pool_b.amount))
        .unwrap()
        .checked_div(
            I64F64::from_num(pool_a.amount)
            .checked_add(I64F64::from_num(taxed_amount))
            .unwrap()
        )
        .unwrap()
    } else {
        I64F64::from_num(taxed_amount)
        .checked_mul(I64F64::from_num(pool_a.amount))
        .unwrap()
        .checked_div(
            I64F64::from_num(pool_b.amount)
            .checked_add(I64F64::from_num(taxed_amount))
            .unwrap()
        )
        .unwrap()
    }
    .to_num::<u64>();

if output < min_amount_out {
    return err!(Errors::OutputTooSmall)
}
let invariance = pool_a.amount * pool_b.amount;

let authority_bump = ctx.bumps.pool_authority;
let authority_seed = &[
    &ctx.accounts.mint_a.key().to_bytes(),
    &ctx.accounts.mint_b.key().to_bytes(),
    AUTHORITY_SEED,
    &[authority_bump]
    ];
    
    let signer_seeds = &[&authority_seed[..]];
        
        if swap_a {
            token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    authority: ctx.accounts.trader.to_account_info(),
                    from: ctx.accounts.trader_ata_a.to_account_info(),
                    to: ctx.accounts.pool_account_a.to_account_info()
                }
            ),
            input
        )?;
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    authority: ctx.accounts.pool_authority.to_account_info(),
                    from: ctx.accounts.pool_account_b.to_account_info(),
                    to: ctx.accounts.trader_ata_b.to_account_info()
                },
                signer_seeds
            ),
            output
        )?;
    } else {
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    authority: ctx.accounts.trader.to_account_info(),
                    from: ctx.accounts.trader_ata_b.to_account_info(),
                    to: ctx.accounts.pool_account_b.to_account_info()
                }
            ),
            input
        )?;
        token::transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    authority: ctx.accounts.pool_authority.to_account_info(),
                    from: ctx.accounts.pool_account_a.to_account_info(),
                    to: ctx.accounts.trader_ata_a.to_account_info()
                },
                signer_seeds
            ),      
            output
        )?;
    }
    msg!(
        "Traded {} tokens ({} after fees) for {}",
        input,
        taxed_amount,
        output
    );
    // Verify the invariant still holds
    // Reload accounts because of the CPIs
    // We tolerate if the new invariant is higher because it means a rounding error for LPs
    ctx.accounts.pool_account_a.reload()?;
    ctx.accounts.pool_account_b.reload()?;

    let delta: f16 = f16::from_f32(
        (pool_a_initial as f32 / pool_b_initial as f32) // Tokens before deposit 
        / (ctx.accounts.pool_account_a.amount as f32 / ctx.accounts.pool_account_b.amount as f32) // Amount of tokens after the deposit
     );
    let delta_bits = delta.to_bits();
    ctx.accounts.liquidity_pool.delta = delta_bits;

    if invariance > ctx.accounts.pool_account_a.amount * ctx.accounts.pool_account_a.amount {
        return err!(Errors::InvarianceViolated);
    }
    Ok(())
}



fn calculate_fee(delta: u16) -> u64 {
    let delta_formatted =  f16::from_bits(delta);

    if delta_formatted > f16::from_f32(-0.25) && delta_formatted < f16::from_f32(0.33) {
        return 3;
    } if delta_formatted > f16::from_f32(-0.25) && delta_formatted < f16::from_f32(0.33) {

    }

    2
}
