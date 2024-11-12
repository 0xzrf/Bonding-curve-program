use anchor_lang::prelude::*;

declare_id!("5UveSxoLCEgMnHfgKr35DN5CTzWCfm96DvCAUPMkcHnk");

#[program]
pub mod bonding_curve_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
