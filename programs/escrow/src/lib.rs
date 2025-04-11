use anchor_lang::prelude::*;

pub mod instructions;
pub mod state;

use crate::instructions::*;

declare_id!("7RCce6afhK2Y5oXoX4ob8beY1MpNzy6BDZJ6Sm1JB2wi");

#[program]
pub mod escrow {
    use super::*;

    pub fn make(ctx: Context<Make>, seeds: u64, receive_amount: u64, deposit_amount: u64) -> Result<()> {
        ctx.accounts.init_escrow_state(seeds, receive_amount, ctx.bumps)?;
        ctx.accounts.deposit(deposit_amount)?;
        Ok(())
    }

    pub fn take(ctx: Context<Take>) -> Result<()> {
        ctx.accounts.transfer()?;
        ctx.accounts.close_vault()?;
        Ok(())
    }
    pub fn refund(ctx: Context<Refund>)-> Result<()>  {
        ctx.accounts.withdraw()?;
        ctx.accounts.close_vault()?;
        Ok(())
    }
}
