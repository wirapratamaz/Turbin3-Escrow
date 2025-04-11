use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token_interface::{Mint, TokenAccount, TokenInterface, transfer_checked, TransferChecked}};

use crate::state::EscrowState;


#[derive(Accounts)]
#[instruction(seeds: u64)]
pub struct Make<'info> {
    #[account(mut)]
    pub maker: Signer<'info>,
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,
    #[account(mut, 
        associated_token::mint=mint_a, 
        associated_token::authority=maker
    )]
    pub maker_mint_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(init, 
        payer=maker, 
        space=EscrowState::INIT_SPACE+8, 
        seeds=[b"escrow", maker.key().as_ref(), seeds.to_le_bytes().as_ref()], 
        bump)]
    pub escrow: Box<Account<'info, EscrowState>>,

    #[account(init, 
        associated_token::mint=mint_a, 
        associated_token::authority=escrow, 
        payer=maker)]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}

impl<'info> Make<'info> {
    pub fn init_escrow_state(&mut self, seeds: u64, receive_amount: u64, bumps: MakeBumps) -> Result<()> {
        self.escrow.set_inner(EscrowState { 
            seed: seeds, 
            maker: self.maker.key(), 
            mint_a: self.mint_a.key(), 
            mint_b: self.mint_b.key(), 
            receive_amount, 
            bump: bumps.escrow
         });
        Ok(())
    }

    pub fn deposit(&mut self, amount: u64) -> Result<()> {
        let cpi_accounts = TransferChecked {
            from: self.maker_mint_a_ata.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.vault.to_account_info(),
            authority: self.maker.to_account_info()
        };

        let cpi_ctx = CpiContext::new(
            self.token_program.to_account_info(), 
            cpi_accounts
        );

        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)?;

        Ok(())
    }
} 