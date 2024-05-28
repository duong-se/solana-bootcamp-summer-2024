use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount, transfer},
};
use constants::{REWARD_VAULT_SEED, STAKE_INFO_SEED};
use errors::AppError;
use state::StakeInfo;

mod constants;
mod errors;
mod state;

declare_id!("Drzahf6sg5fttp1HHRNrCnrGzYTNnpjAzsF5vU5RXgxJ");

#[program]
pub mod stake_program {
    use anchor_spl::token::Transfer;

    use super::*;

    pub fn initialize(_ctx: Context<Initialize>) -> Result<()> {
        Ok(())
    }

    pub fn stake(ctx: Context<Stake>, amount: u64) -> Result<()> {
        let stake_info = &mut ctx.accounts.stake_info;

        if stake_info.is_staked {
            return Err(AppError::IsStaked.into());
        }

        if amount == 0 {
            return Err(AppError::NoToken.into());
        }

        let clock = Clock::get()?;
        stake_info.staker = ctx.accounts.staker.key();
        stake_info.mint = ctx.accounts.mint.key();
        stake_info.stake_at = clock.slot;
        stake_info.is_staked = true;
        stake_info.amount = amount;

        transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer{
                    from: ctx.accounts.staker_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.staker.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn unstake(ctx: Context<Stake>) -> Result<()> {
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

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct Stake<'info> {
    #[account(mut)]
    pub staker: Signer<'info>,

    pub mint: Account<'info, Mint>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = staker,
    )]
    pub staker_token_account: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = staker,
        seeds = [STAKE_INFO_SEED, staker.key().as_ref()],
        bump,
        space = 8 + StakeInfo::INIT_SPACE,
    )]
    pub stake_info: Account<'info, StakeInfo>,

    #[account(
        init_if_needed,
        payer = staker,
        associated_token::mint = mint,
        associated_token::authority = stake_info,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
