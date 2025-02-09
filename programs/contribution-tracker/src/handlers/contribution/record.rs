use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::ContributionError;
use crate::utils::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct RecordContributionAccountConstraints<'info> {
    #[account(mut)]
    pub contributor: Signer<'info>,

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
            contribution_tracker.current_period.to_le_bytes().as_ref()
        ],
        bump = current_period.bump,
        constraint = !current_period.is_finalized @ ContributionError::PeriodAlreadyFinalized
    )]
    pub current_period: Account<'info, DistributionPeriod>,

    #[account(
        init_if_needed,
        payer = contributor,
        space = ANCHOR_DISCRIMINATOR + Contributor::INIT_SPACE,
        seeds = [
            CONTRIBUTOR_SEED,
            contributor.key().as_ref()
        ],
        bump
    )]
    pub contributor_account: Account<'info, Contributor>,

    #[account(
        init,
        payer = contributor,
        space = ANCHOR_DISCRIMINATOR + Contribution::INIT_SPACE,
        seeds = [
            CONTRIBUTION_SEED,
            contributor.key().as_ref(),
            contribution_tracker.current_period.to_le_bytes().as_ref(),
            contributor_account.contributions.len().to_le_bytes().as_ref()
        ],
        bump
    )]
    pub contribution: Account<'info, Contribution>,

    pub system_program: Program<'info, System>,
}

pub fn record_contribution(
    context: Context<RecordContributionAccountConstraints>,
    contribution_type: ContributionType,
    severity: Severity,
    description: String,
) -> Result<()> {
    // Validate inputs
    ValidationUtils::validate_description(&description)?;
    
    // Validate contributor capacity
    ContributionValidator::validate_contributor_capacity(
        &context.accounts.contributor_account
    )?;

    let current_time = TimeUtils::get_current_time()?;

    // Calculate points based on contribution type and severity
    let points = PointCalculator::calculate_points(contribution_type, severity)?;

    // Initialize contribution record
    let contribution = &mut context.accounts.contribution;
    contribution.contributor = context.accounts.contributor.key();
    contribution.distribution_period = context.accounts.contribution_tracker.current_period;
    contribution.contribution_type = contribution_type;
    contribution.severity = severity;
    contribution.points = points;
    contribution.timestamp = current_time;
    contribution.description = description;
    contribution.status = ContributionStatus::Pending;
    contribution.bump = context.bumps.contribution;

    // Update contributor account
    let contributor_account = &mut context.accounts.contributor_account;
    contributor_account.authority = context.accounts.contributor.key();
    contributor_account.contributions.push(context.accounts.contribution.key());
    
    if contributor_account.bump == 0 {
        contributor_account.bump = context.bumps.contributor_account;
    }

    Ok(())
}