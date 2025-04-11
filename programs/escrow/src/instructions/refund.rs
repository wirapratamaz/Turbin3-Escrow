use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token::{close_account, CloseAccount}, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}};

use crate::state::{EscrowState, ErrorCode};


#[derive(Accounts)]
pub struct Refund<'info> {
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub maker: Signer<'info>,

    #[account(mut, 
        associated_token::mint=mint_a, 
        associated_token::authority=maker
    )]
    pub maker_mint_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut,
        associated_token::mint=mint_a, 
        associated_token::authority=escrow, 
        constraint = vault.owner == escrow.key() @ ErrorCode::InvalidVault
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(mut, 
        close=maker,
        seeds=[b"escrow", escrow.maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump=escrow.bump)]
    pub escrow: Box<Account<'info, EscrowState>>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
} 

impl<'info> Refund<'info>  {
    pub fn withdraw(&mut self)-> Result<()> {
        let amount = self.vault.amount;
        
        let cpi_accounts = TransferChecked {
            from: self.vault.to_account_info(),
            mint: self.mint_a.to_account_info(),
            to: self.maker_mint_a_ata.to_account_info(),
            authority: self.escrow.to_account_info()
        };

        let maker_key = self.maker.key();
        require!(self.escrow.maker == maker_key, ErrorCode::InvalidMaker);

        let seeds_bytes = self.escrow.seed.to_le_bytes();
        let escrow_signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(),
            seeds_bytes.as_ref(),
            &[self.escrow.bump]
        ]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            escrow_signer_seeds
        );

        transfer_checked(cpi_ctx, amount, self.mint_a.decimals)?;

        Ok(())
    }

    pub fn close_vault(&mut self)-> Result<()> {
        let cpi_accounts = CloseAccount {
            authority: self.escrow.to_account_info(),
            account: self.vault.to_account_info(),
            destination: self.maker.to_account_info()
        };

        let maker_key = self.escrow.maker.key();
        let seeds_bytes = self.escrow.seed.to_le_bytes();
        let escrow_signer_seeds: &[&[&[u8]]] = &[&[
            b"escrow",
            maker_key.as_ref(),
            seeds_bytes.as_ref(),
            &[self.escrow.bump]
        ]];

        let cpi_ctx = CpiContext::new_with_signer(
            self.token_program.to_account_info(),
            cpi_accounts,
            escrow_signer_seeds
        );

        close_account(cpi_ctx)?;

        Ok(())
    }
} 