use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{transfer, Mint, Token, TokenAccount},
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
                Transfer {
                    from: ctx.accounts.staker_token_account.to_account_info(),
                    to: ctx.accounts.vault_token_account.to_account_info(),
                    authority: ctx.accounts.staker.to_account_info(),
                },
            ),
            amount,
        )?;

        Ok(())
    }

    pub fn unstake(ctx: Context<UnStake>, amount: u64) -> Result<()> {
        const DENUMERATOR: u64 = 100_000;
        const NUMERATOR: u64 = 1_000; // aka 1%
        let stake_info = &ctx.accounts.stake_info;
        let stake_key = ctx.accounts.staker.key();
        let mint_key = ctx.accounts.mint.key();
        if !stake_info.is_staked {
            return Err(AppError::NotStaked.into());
        }

        if amount > stake_info.amount {
            return Err(AppError::OverStakeBalance.into());
        }

        let clock = Clock::get()?;
        let slot_passed = clock.slot - stake_info.stake_at;
        // transfer stake amount to staker token amount
        let stake_info_signer_seeds: &[&[&[u8]]] = &[&[
            STAKE_INFO_SEED,
            stake_key.as_ref(),
            mint_key.as_ref(),
            &[ctx.bumps.stake_info],
        ]];
        transfer(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: ctx.accounts.vault_token_account.to_account_info(),
                    to: ctx.accounts.staker_token_account.to_account_info(),
                    authority: ctx.accounts.stake_info.to_account_info(),
                },
                stake_info_signer_seeds,
            ),
            stake_info.amount,
        )?;

        let reward_by_amount = stake_info
            .amount
            .checked_mul(NUMERATOR)
            .and_then(|res| res.checked_div(DENUMERATOR))
            .unwrap();

        // transfer reward from reward vault to staker token amount
        let reward = slot_passed.checked_mul(reward_by_amount).unwrap(); // Handling potential overflow
        msg!("reward: {}", reward);

        let reward_vault_signer_seeds: &[&[&[u8]]] = &[&[
            REWARD_VAULT_SEED,
            mint_key.as_ref(),
            &[ctx.bumps.reward_vault],
        ]];
        let stake_info = &mut ctx.accounts.stake_info;
        if reward > 0 {
            transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
                    Transfer {
                        from: ctx.accounts.reward_vault.to_account_info(),
                        to: ctx.accounts.staker_token_account.to_account_info(),
                        authority: ctx.accounts.reward_vault.to_account_info(),
                    },
                    reward_vault_signer_seeds,
                ),
                reward,
            )?;
            // Update stake info
            stake_info.amount -= amount;
            stake_info.stake_at = clock.slot;
        }

        if stake_info.amount == 0 {
            stake_info.is_staked = false;

            // close staker vault token account
            anchor_spl::token::close_account(CpiContext::new_with_signer(
                ctx.accounts.token_program.to_account_info(),
                anchor_spl::token::CloseAccount {
                    account: ctx.accounts.vault_token_account.to_account_info(),
                    destination: ctx.accounts.staker.to_account_info(),
                    authority: ctx.accounts.stake_info.to_account_info(),
                },
                stake_info_signer_seeds,
            ))?;
        }
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
        seeds = [REWARD_VAULT_SEED, mint.key().as_ref()],
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
        seeds = [STAKE_INFO_SEED, staker.key().as_ref(), mint.key().as_ref()],
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

#[derive(Accounts)]
pub struct UnStake<'info> {
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
        mut,
        seeds = [STAKE_INFO_SEED, staker.key().as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub stake_info: Account<'info, StakeInfo>,

    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = stake_info,
    )]
    pub vault_token_account: Account<'info, TokenAccount>,

    #[account(
        mut,
        seeds = [REWARD_VAULT_SEED, mint.key().as_ref()],
        bump,
        token::mint = mint,
        token::authority = reward_vault,
    )]
    pub reward_vault: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}
