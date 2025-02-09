use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ContributionError;
use crate::utils::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct StartNewPeriodAccountConstraints<'info> {
    #[account(
        mut,
        seeds = [CONTRIBUTION_TRACKER_SEED],
        bump = contribution_tracker.bump,
        has_one = admin @ ContributionError::UnauthorizedAdmin,
    )]
    pub contribution_tracker: Account<'info, ContributionTracker>,

    #[account(
        mut,
        seeds = [
            DISTRIBUTION_PERIOD_SEED,
            contribution_tracker.current_period.to_le_bytes().as_ref()
        ],
        bump = current_period.bump
    )]
    pub current_period: Account<'info, DistributionPeriod>,

    #[account(
        init,
        payer = admin,
        space = ANCHOR_DISCRIMINATOR + DistributionPeriod::INIT_SPACE,
        seeds = [
            DISTRIBUTION_PERIOD_SEED,
            (contribution_tracker.current_period + 1).to_le_bytes().as_ref()
        ],
        bump
    )]
    pub next_period: Account<'info, DistributionPeriod>,

    #[account(mut)]
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

pub fn start_new_period(context: Context<StartNewPeriodAccountConstraints>) -> Result<()> {
    let current_time = TimeUtils::get_current_time()?;
    
    // Validate current period is ended
    require!(
        current_time >= context.accounts.current_period.end_time,
        ContributionError::PeriodNotEnded
    );

    // Finalize current period if not already finalized
    if !context.accounts.current_period.is_finalized {
        context.accounts.current_period.is_finalized = true;
    }

    let tracker = &mut context.accounts.contribution_tracker;
    let next_period_number = tracker.current_period.checked_add(1)
        .ok_or(ContributionError::DistributionCalculationError)?;

    // Initialize next period
    let next_period = &mut context.accounts.next_period;
    next_period.period_number = next_period_number;
    next_period.start_time = current_time;
    next_period.end_time = current_time.checked_add(tracker.period_duration)
        .ok_or(ContributionError::InvalidTimestamp)?;
    next_period.total_points = 0;
    next_period.tokens_allocated = tracker.tokens_per_period;
    next_period.tokens_distributed = 0;
    next_period.is_finalized = false;
    next_period.bump = context.bumps.next_period;

    // Update tracker
    tracker.current_period = next_period_number;

    Ok(())
}