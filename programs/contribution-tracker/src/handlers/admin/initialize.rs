use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ContributionError;
use crate::utils::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct InitializeTrackerAccountConstraints<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        payer = admin,
        space = ANCHOR_DISCRIMINATOR + ContributionTracker::INIT_SPACE,
        seeds = [CONTRIBUTION_TRACKER_SEED],
        bump
    )]
    pub contribution_tracker: Account<'info, ContributionTracker>,

    #[account(
        init,
        payer = admin,
        space = ANCHOR_DISCRIMINATOR + DistributionPeriod::INIT_SPACE,
        seeds = [
            DISTRIBUTION_PERIOD_SEED,
            0u64.to_le_bytes().as_ref()
        ],
        bump
    )]
    pub initial_period: Account<'info, DistributionPeriod>,

    pub system_program: Program<'info, System>,
}

pub fn initialize_tracker(
    context: Context<InitializeTrackerAccountConstraints>,
    period_duration: i64,
    minimum_points_threshold: u32,
    tokens_per_period: u64,
) -> Result<()> {
    // Validate inputs
    TimeUtils::validate_period_duration(period_duration)?;
    
    require!(
        minimum_points_threshold >= 100,
        ContributionError::InvalidPointsThreshold
    );

    require!(
        tokens_per_period > 0,
        ContributionError::InvalidTokenAllocation
    );

    let current_time = TimeUtils::get_current_time()?;

    // Initialize tracker state
    let tracker = &mut context.accounts.contribution_tracker;
    tracker.admin = context.accounts.admin.key();
    tracker.current_period = 0;
    tracker.period_duration = period_duration;
    tracker.total_points_all_time = 0;
    tracker.minimum_points_threshold = minimum_points_threshold;
    tracker.reserve_pool_amount = 0;
    tracker.tokens_per_period = tokens_per_period;
    tracker.bump = context.bumps.contribution_tracker;

    // Initialize first period

    
    let initial_period = &mut context.accounts.initial_period;
    initial_period.period_number = 0;
    initial_period.start_time = current_time;
    initial_period.end_time = current_time.checked_add(period_duration)
        .ok_or(ContributionError::InvalidTimestamp)?;
    initial_period.total_points = 0;
    initial_period.tokens_allocated = tokens_per_period;
    initial_period.tokens_distributed = 0;
    initial_period.is_finalized = false;
    initial_period.bump = context.bumps.initial_period;

    Ok(())
}