use anchor_lang::prelude::*;
use anchor_spl::token::{TokenAccount, Mint, Token};
use constants::REWARD_VAULT_SEED;

mod constants;

declare_id!("Drzahf6sg5fttp1HHRNrCnrGzYTNnpjAzsF5vU5RXgxJ");

#[program]
pub mod stake_program {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = admin,
        seeds = [REWARD_VAULT_SEED],
        bump,
        token::mint = mint,
        token::authority = reward_vault,
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info,Token>,
    pub system_program: Program<'info, System>
}
