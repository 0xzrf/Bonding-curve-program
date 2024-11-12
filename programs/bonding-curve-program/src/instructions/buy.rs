use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{self, Mint, Token, TokenAccount, Transfer as TokenTransfer},
};

use crate::{constants::*, states::LiquidityPool};
use std::f64::consts::E;

#[derive(Accounts)]
pub struct Buy<'info> {
    #[account(mut)]
    pub buyer: Signer<'info>,

    #[account(
        mut,
        seeds = [
            b"liquidity_pool",
            liquidity_pool.mint_a.key().as_ref(),
            liquidity_pool.mint_b.key().as_ref()
        ],
        bump = liquidity_pool.bump,
        has_one = mint_a,
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
    #[account(
            init_if_needed,
            payer = buyer,
            associated_token::mint = mint_a,
            associated_token::authority = buyer
        )]
    pub buyer_account_mint: Box<Account<'info, TokenAccount>>,

    #[account(
            mut,
            seeds = [b"reserve", liquidity_pool.key().as_ref()],
            bump
        )]
    pub reserve: SystemAccount<'info>,
    pub system_program: Program<'info, System>,
    pub token_program: Program<'info, Token>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

impl<'info> Buy<'info> {
    pub fn buy_linear(&mut self, amount: u64, bumps: &BuyBumps, swap_a: bool) -> Result<()> {
        let price_per_token = if swap_a {
            (self.mint_a.supply / 1000) + 10 // This is the linear bonding curve method
        } else {
            (self.mint_b.supply / 1000) + 10
        };

        let total_price = amount * price_per_token;

        let transfer_accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.reserve.to_account_info(),
        };

        let cpi_context = CpiContext::new(self.system_program.to_account_info(), transfer_accounts);

        transfer(cpi_context, total_price)?;

        let authority_bump = bumps.pool_authority;
        let authority_seed = &[
            &self.mint_a.key().to_bytes(),
            &self.mint_b.key().to_bytes(),
            AUTHORITY_SEED,
            &[authority_bump],
        ];

        let signer_seeds = &[&authority_seed[..]];

        if swap_a {
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TokenTransfer {
                        from: self.pool_account_a.to_account_info(),
                        to: self.buyer_account_mint.to_account_info(),
                        authority: self.pool_authority.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount,
            )?;
        } else {
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TokenTransfer {
                        from: self.pool_account_b.to_account_info(),
                        to: self.buyer_account_mint.to_account_info(),
                        authority: self.pool_authority.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount,
            )?;
        }
        Ok(())
    }

    pub fn buy_exponent(&mut self, amount: u64, bumps: &BuyBumps, swap_a: bool) -> Result<()> {
        let price_per_token = if swap_a {
            10.00 * E.powf(1 as f64 * self.mint_a.supply as f64)
        } else {
            10.00 * E.powf(1 as f64 * self.mint_a.supply as f64)
        };

        let total_price = amount as f64 * price_per_token;

        let transfer_accounts = Transfer {
            from: self.buyer.to_account_info(),
            to: self.reserve.to_account_info(),
        };

        let cpi_context = CpiContext::new(self.system_program.to_account_info(), transfer_accounts);

        transfer(cpi_context, total_price as u64)?;

        let authority_bump = bumps.pool_authority;
        let authority_seed = &[
            &self.mint_a.key().to_bytes(),
            &self.mint_b.key().to_bytes(),
            AUTHORITY_SEED,
            &[authority_bump],
        ];

        let signer_seeds = &[&authority_seed[..]];

        if swap_a {
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TokenTransfer {
                        from: self.pool_account_a.to_account_info(),
                        to: self.buyer_account_mint.to_account_info(),
                        authority: self.pool_authority.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount,
            )?;
        } else {
            token::transfer(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    TokenTransfer {
                        from: self.pool_account_b.to_account_info(),
                        to: self.buyer_account_mint.to_account_info(),
                        authority: self.pool_authority.to_account_info(),
                    },
                    signer_seeds,
                ),
                amount,
            )?;
        }
        Ok(())
    }

}
