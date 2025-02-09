use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use crate::state::*;
use crate::error::ContributionError;
use crate::utils::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct ProcessPeriodDistributionAccountConstraints<'info> {
    #[account(
        constraint = admin.key() == contribution_tracker.admin @ ContributionError::UnauthorizedAdmin
    )]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [CONTRIBUTION_TRACKER_SEED],
        bump = contribution_tracker.bump
    )]
    pub contribution_tracker: Account<'info, ContributionTracker>,

    #[account(
        mut,
        seeds = [
            DISTRIBUTION_PERIOD_SEED,
            period_to_process.period_number.to_le_bytes().as_ref()
        ],
        bump = period_to_process.bump,
        constraint = !period_to_process.is_finalized @ ContributionError::PeriodAlreadyFinalized
    )]
    pub period_to_process: Account<'info, DistributionPeriod>,

    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub reserve_vault: Account<'info, TokenAccount>,

    pub reward_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
}

pub fn process_period_distribution(
    context: Context<ProcessPeriodDistributionAccountConstraints>
) -> Result<()> {
    let period = &mut context.accounts.period_to_process;
    let tracker = &mut context.accounts.contribution_tracker;
    
    // Validate period has ended
    let current_time = TimeUtils::get_current_time()?;
    require!(
        current_time >= period.end_time,
        ContributionError::PeriodNotEnded
    );

    // Check if minimum points threshold is met
    let tokens_to_distribute = if period.total_points >= tracker.minimum_points_threshold as u64 {
        period.tokens_allocated
    } else {
        // Only distribute 50% if threshold not met
        period.tokens_allocated
            .checked_div(2)
            .ok_or(ContributionError::DistributionCalculationError)?
    };

    // Calculate amount to move to reserve
    let reserve_amount = period.tokens_allocated
        .checked_sub(tokens_to_distribute)
        .ok_or(ContributionError::DistributionCalculationError)?;

    if reserve_amount > 0 {
        // Transfer to reserve vault
        let transfer_to_reserve_ix = Transfer {
            from: context.accounts.reward_vault.to_account_info(),
            to: context.accounts.reserve_vault.to_account_info(),
            authority: context.accounts.admin.to_account_info(),
        };

        token::transfer(
            CpiContext::new(
                context.accounts.token_program.to_account_info(),
                transfer_to_reserve_ix,
            ),
            reserve_amount,
        )?;

        // Update reserve pool amount
        tracker.reserve_pool_amount = tracker.reserve_pool_amount
            .checked_add(reserve_amount)
            .ok_or(ContributionError::DistributionCalculationError)?;
    }

    // Update period state
    period.tokens_allocated = tokens_to_distribute;
    period.is_finalized = true;

    Ok(())
}