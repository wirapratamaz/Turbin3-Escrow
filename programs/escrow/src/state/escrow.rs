use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct EscrowState {
    pub seed: u64,
    pub maker: Pubkey,
    pub mint_a: Pubkey,
    pub mint_b: Pubkey,
    pub receive_amount: u64,
    pub bump: u8,
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Vault Account")]
    InvalidVault,
    #[msg("Invalid Maker Account")]
    InvalidMaker,
} 