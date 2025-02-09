use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{self, Token, TokenAccount, Mint, Transfer};
use crate::state::*;
use crate::error::ContributionError;
use crate::utils::*;
use crate::constants::*;

#[derive(Accounts)]
pub struct ClaimRewardsAccountConstraints<'info> {
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
            period_to_claim.period_number.to_le_bytes().as_ref()
        ],
        bump = period_to_claim.bump,
        constraint = period_to_claim.is_finalized @ ContributionError::PeriodNotEnded
    )]
    pub period_to_claim: Account<'info, DistributionPeriod>,

    #[account(
        mut,
        seeds = [
            CONTRIBUTOR_SEED,
            contributor.key().as_ref()
        ],
        bump = contributor_account.bump,
        constraint = contributor_account.authority == contributor.key() @ ContributionError::InvalidAuthority
    )]
    pub contributor_account: Account<'info, Contributor>,

    #[account(mut)]
    pub reward_vault: Account<'info, TokenAccount>,

    #[account(
        init_if_needed,
        payer = contributor,
        associated_token::mint = reward_mint,
        associated_token::authority = contributor
    )]
    pub contributor_token_account: Account<'info, TokenAccount>,

    pub reward_mint: Account<'info, Mint>,
    
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub associated_token_program: Program<'info, AssociatedToken>,
}

pub fn claim_rewards(
    context: Context<ClaimRewardsAccountConstraints>
) -> Result<()> {
    let period = &context.accounts.period_to_claim;
    let contributor_account = &mut context.accounts.contributor_account;

    // Validate not already claimed
    require!(
        contributor_account.last_claimed_period < period.period_number,
        ContributionError::AlreadyClaimed
    );

    // Calculate reward amount
    let reward_amount = PointCalculator::calculate_token_distribution(
        contributor_account.current_period_points,
        period.total_points,
        period.tokens_allocated,
    )?;

    if reward_amount > 0 {
        // Transfer rewards
        let transfer_ix = Transfer {
            from: context.accounts.reward_vault.to_account_info(),
            to: context.accounts.contributor_token_account.to_account_info(),
            authority: context.accounts.contribution_tracker.to_account_info(),
        };

        let seeds = &[
            CONTRIBUTION_TRACKER_SEED,
            &[context.accounts.contribution_tracker.bump],
        ];

        token::transfer(
            CpiContext::new_with_signer(
                context.accounts.token_program.to_account_info(),
                transfer_ix,
                &[&seeds[..]]
            ),
            reward_amount,
        )?;

        // Update contributor state
        contributor_account.last_claimed_period = period.period_number;
        contributor_account.current_period_points = 0;

        // Update period state
        let period = &mut context.accounts.period_to_claim;
        period.tokens_distributed = period.tokens_distributed
            .checked_add(reward_amount)
            .ok_or(ContributionError::DistributionCalculationError)?;
    }

    Ok(())
}