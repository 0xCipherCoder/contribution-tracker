use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};

pub mod state;
pub mod error;
pub mod utils;
pub mod handlers;
pub mod constants;

use handlers::*;
use state::*;

declare_id!("87vicYP81ZyDmJU28bi2MkJyJMhU1wJWNmBjt41K9yvw");

#[program]
pub mod contribution_tracker {
    use super::*;

    // Admin Instructions
    pub fn initialize_tracker(
        context: Context<InitializeTrackerAccountConstraints>,
        period_duration: i64,
        minimum_points_threshold: u32,
        tokens_per_period: u64,
    ) -> Result<()> {
        handlers::admin::initialize::initialize_tracker(
            context,
            period_duration,
            minimum_points_threshold,
            tokens_per_period,
        )
    }

    pub fn start_new_period(
        context: Context<StartNewPeriodAccountConstraints>
    ) -> Result<()> {
        handlers::admin::manage_period::start_new_period(context)
    }

    // Contribution Instructions
    pub fn record_contribution(
        context: Context<RecordContributionAccountConstraints>,
        contribution_type: ContributionType,
        severity: Severity,
        description: String,
    ) -> Result<()> {
        handlers::contribution::record::record_contribution(
            context,
            contribution_type,
            severity,
            description,
        )
    }

    pub fn review_contribution(
        context: Context<ReviewContributionAccountConstraints>,
        approve: bool,
    ) -> Result<()> {
        handlers::contribution::review::review_contribution(
            context,
            approve,
        )
    }

    // Distribution Instructions
    pub fn process_period_distribution(
        context: Context<ProcessPeriodDistributionAccountConstraints>
    ) -> Result<()> {
        handlers::distribution::process_period::process_period_distribution(context)
    }

    pub fn claim_rewards(
        context: Context<ClaimRewardsAccountConstraints>
    ) -> Result<()> {
        handlers::distribution::claim::claim_rewards(context)
    }
}