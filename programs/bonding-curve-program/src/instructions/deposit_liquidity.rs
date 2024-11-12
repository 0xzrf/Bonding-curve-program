use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, MintTo, Token, TokenAccount, Transfer},
};
use fixed::types::I64F64;
use half::f16;
use crate::{constants::*, states::*, constants::MINIMUM_LIQUIDITY, errors::Errors};

pub fn deposit_liquidity(
    ctx: Context<DepositLiquidity>,
    amount_a: u64,
    amount_b: u64,
) -> Result<()> {
    // This checks if the user has less amount of tokens than he has specified, returs all the amount that he has in the account
    let amount_a = if ctx.accounts.depositor_mint_a.amount < amount_a {
        ctx.accounts.depositor_mint_a.amount
    } else {
        amount_a
    };
    let  amount_b = if ctx.accounts.depositor_mint_b.amount < amount_b {
        ctx.accounts.depositor_mint_b.amount
    } else {
        amount_b
    };

    let pool_a = &ctx.accounts.pool_account_a;
    let pool_b = &ctx.accounts.pool_account_b;
    // Checks if there already exists liquidity in the pool or this is the first deposit in the pool
    let pool_creation = pool_a.amount == 0 && pool_b.amount == 0;
    let pool_a_initial = pool_a.amount;
    let pool_b_initial = pool_b.amount;
    //Assign amount_a and amount_b based on the constant of the pool so that the proportion of the pool remains the same and isn't disturbed due to
    // the additional deposit
    let (amount_a, amount_b) = if pool_creation {
        // Add the liquidity in the amount that has been specified
        (amount_a, amount_b)
    } else {
        let ratio = I64F64::from_num(pool_a.amount)
            .checked_mul(I64F64::from_num(pool_b.amount))
            .unwrap();

        if pool_a.amount > pool_b.amount {
            (
                I64F64::from_num(amount_b)
                    .checked_mul(ratio)
                    .unwrap()
                    .to_num::<u64>(),
                amount_b,
            )
        } else {
            (
                amount_a,
                I64F64::from_num(amount_a)
                    .checked_div(ratio)
                    .unwrap()
                    .to_num::<u64>(),
            )
        }
    };

    // Define delta after deposit
    
    // Defining liquidity which is defined as square root of the constant of the pool
    let mut liquidity = I64F64::from_num(amount_a)
    .checked_mul(I64F64::from_num(amount_b))
    .unwrap()
    .sqrt()
    .to_num::<u64>();

if pool_creation {
    if liquidity < MINIMUM_LIQUIDITY {
        return err!(Errors::InvalidDepositAmount);
    }
    
    liquidity -= MINIMUM_LIQUIDITY;
}

// Send the respective tokens to the appropriate pool accounts
token::transfer(
    CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        Transfer {
            from: ctx.accounts.depositor_mint_a.to_account_info(),
            to: ctx.accounts.pool_account_a.to_account_info(),
            authority: ctx.accounts.depositor.to_account_info(),
        },
    ),
    amount_a,
)?;

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.depositor_mint_b.to_account_info(),
                to: ctx.accounts.pool_account_b.to_account_info(),
                authority: ctx.accounts.depositor.to_account_info(),
            },
        ),
        amount_b,
    )?;
    
    // mint liquidity to the user
    let authority_bump = ctx.bumps.pool_authority;
    let authority_seeds = &[
        &ctx.accounts.mint_a.key().to_bytes(),
        &ctx.accounts.mint_b.key().to_bytes(),
        AUTHORITY_SEED, 
        &[authority_bump]
        ];
        let signer_seeds = &[&authority_seeds[..]];
        
        
        token::mint_to(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                MintTo {
                    authority: ctx.accounts.pool_authority.to_account_info(),
                    mint: ctx.accounts.mint_liquidity.to_account_info(),
                    to: ctx.accounts.depositor_account_liquidity.to_account_info(),
                }, 
                signer_seeds
            ),
            liquidity
        )?;

        if !pool_creation {
            ctx.accounts.pool_account_a.reload()?;
            ctx.accounts.pool_account_b.reload()?;
    
    
            let delta: f16 = f16::from_f32(
                (pool_a_initial as f32 / pool_b_initial as f32) // Tokens before deposit 
                / (ctx.accounts.pool_account_a.amount as f32 / ctx.accounts.pool_account_b.amount as f32) // Amount of tokens after the deposit
             );
            let delta_bits = delta.to_bits();
            ctx.accounts.liquidity_pool.delta = delta_bits;
        }
        
        Ok(())
    }
    
#[derive(Accounts)]
pub struct DepositLiquidity<'info> {
    #[account(
        mut,
        seeds = [
            b"liquidity_pool",
            liquidity_pool.mint_a.key().as_ref(),
            liquidity_pool.mint_b.key().as_ref()
        ],
        bump = liquidity_pool.bump,
        has_one = mint_a,
        has_one = mint_b
    )]
    pub liquidity_pool: Box<Account<'info, LiquidityPool>>,
    pub mint_a: Box<Account<'info, Mint>>,
    pub mint_b: Box<Account<'info, Mint>>,
    /// CHECK: Read only authority
    #[account(
        seeds = [
            liquidity_pool.mint_a.key().as_ref(),
            liquidity_pool.mint_b.key().as_ref(),
            b"pool_authority"
        ],
        bump
    )]
    pub pool_authority: AccountInfo<'info>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = pool_authority
    )]
    pub pool_account_a: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = pool_authority
    )]
    pub pool_account_b: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub depositor: Signer<'info>,
    #[account( 
        mut,
        seeds = [b"liquidity_token",mint_a.key().as_ref(), mint_b.key().as_ref()],
        bump,
    )]
    pub mint_liquidity: Box<Account<'info, Mint>>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint_liquidity,
        associated_token::authority = depositor
    )]
    pub depositor_account_liquidity: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_a,
        associated_token::authority = depositor
    )]
    pub depositor_mint_a: Box<Account<'info, TokenAccount>>,
    #[account(
        mut,
        associated_token::mint = mint_b,
        associated_token::authority = depositor
    )]
    pub depositor_mint_b: Box<Account<'info, TokenAccount>>,
    #[account(mut)]
    pub payer: Signer<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
