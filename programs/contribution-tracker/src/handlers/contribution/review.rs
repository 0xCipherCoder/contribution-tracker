use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ContributionError;
use crate::constants::*;

#[derive(Accounts)]
pub struct ReviewContributionAccountConstraints<'info> {
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
            contribution.distribution_period.to_le_bytes().as_ref()
        ],
        bump = current_period.bump,
        constraint = !current_period.is_finalized @ ContributionError::PeriodAlreadyFinalized
    )]
    pub current_period: Account<'info, DistributionPeriod>,

    #[account(
        mut,
        seeds = [
            CONTRIBUTOR_SEED,
            contribution.contributor.as_ref()
        ],
        bump = contributor_account.bump
    )]
    pub contributor_account: Account<'info, Contributor>,

    #[account(
        mut,
        constraint = contribution.status == ContributionStatus::Pending @ ContributionError::ContributionAlreadyProcessed
    )]
    pub contribution: Account<'info, Contribution>,
}

pub fn review_contribution(
    context: Context<ReviewContributionAccountConstraints>,
    approve: bool,
) -> Result<()> {
    let contribution = &mut context.accounts.contribution;
    let current_period = &mut context.accounts.current_period;
    let contributor_account = &mut context.accounts.contributor_account;
    let tracker = &mut context.accounts.contribution_tracker;

    if approve {
        // Update contribution status
        contribution.status = ContributionStatus::Approved;

        // Update period points
        current_period.total_points = current_period.total_points
            .checked_add(contribution.points as u64)
            .ok_or(ContributionError::DistributionCalculationError)?;

        // Update contributor points
        contributor_account.total_points_all_time = contributor_account.total_points_all_time
            .checked_add(contribution.points as u64)
            .ok_or(ContributionError::DistributionCalculationError)?;

        contributor_account.current_period_points = contributor_account.current_period_points
            .checked_add(contribution.points as u64)
            .ok_or(ContributionError::DistributionCalculationError)?;

        // Update global tracker
        tracker.total_points_all_time = tracker.total_points_all_time
            .checked_add(contribution.points as u64)
            .ok_or(ContributionError::DistributionCalculationError)?;
    } else {
        contribution.status = ContributionStatus::Rejected;
    }

    Ok(())
}