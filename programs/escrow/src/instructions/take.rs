use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken, token::{close_account, CloseAccount}, token_interface::{transfer_checked, Mint, TokenAccount, TokenInterface, TransferChecked}};

use crate::state::{EscrowState, ErrorCode};


#[derive(Accounts)]
pub struct Take<'info> {
    #[account(mut)]
    pub taker: Signer<'info>,
    
    pub mint_a: Box<InterfaceAccount<'info, Mint>>,
    pub mint_b: Box<InterfaceAccount<'info, Mint>>,

    #[account(mut)]
    pub maker: SystemAccount<'info>,

    #[account(mut, 
        associated_token::mint=mint_b, 
        associated_token::authority=taker
    )]
    pub taker_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,
    
    #[account(mut, 
        close=taker,
        seeds=[b"escrow", escrow.maker.key().as_ref(), escrow.seed.to_le_bytes().as_ref()],
        bump=escrow.bump,
        has_one=mint_b,
        has_one=mint_a,
        has_one=maker,
    )]
    pub escrow: Box<Account<'info, EscrowState>>,

    #[account(mut,
        associated_token::mint=mint_a, 
        associated_token::authority=escrow, 
        constraint = vault.owner == escrow.key() @ ErrorCode::InvalidVault
    )]
    pub vault: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(init_if_needed, 
        associated_token::mint=mint_a, 
        associated_token::authority=taker,
        payer=taker,
    )]
    pub taker_mint_a_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    #[account(init_if_needed, 
        associated_token::mint=mint_b, 
        associated_token::authority=maker,
        payer=taker,
    )]
    pub maker_mint_b_ata: Box<InterfaceAccount<'info, TokenAccount>>,

    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Interface<'info, TokenInterface>
}

impl<'info> Take<'info> {
    pub fn transfer(&mut self) -> Result<()> {
        // Transfer token B from taker to maker
        {
            let cpi_accounts = TransferChecked {
                from: self.taker_mint_b_ata.to_account_info(),
                mint: self.mint_b.to_account_info(),
                to: self.maker_mint_b_ata.to_account_info(),
                authority: self.taker.to_account_info()
            };
            
            let cpi_ctx = CpiContext::new(
                self.token_program.to_account_info(),
                cpi_accounts
            );
            
            transfer_checked(cpi_ctx, self.escrow.receive_amount, self.mint_b.decimals)?;
        }
        
        // Transfer token A from vault to taker
        {
            let cpi_accounts = TransferChecked {
                from: self.vault.to_account_info(),
                mint: self.mint_a.to_account_info(),
                to: self.taker_mint_a_ata.to_account_info(),
                authority: self.escrow.to_account_info()
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
            
            transfer_checked(cpi_ctx, self.vault.amount, self.mint_a.decimals)?;
        }
        
        Ok(())
    }

    pub fn close_vault(&mut self) -> Result<()> {
        let cpi_accounts = CloseAccount {
            authority: self.escrow.to_account_info(),
            account: self.vault.to_account_info(),
            destination: self.taker.to_account_info()
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