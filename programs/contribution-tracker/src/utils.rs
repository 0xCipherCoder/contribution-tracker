use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use crate::constants::*;
use crate::state::*;
use crate::error::ContributionError;

pub struct ValidationUtils;

impl ValidationUtils {
    pub fn validate_points(points: u8) -> Result<()> {
        require!(
            points >= 1 && points <= 10,
            ContributionError::InvalidPointsRange
        );
        Ok(())
    }

    pub fn validate_description(description: &str) -> Result<()> {
        require!(
            description.len() <= 100,
            ContributionError::DescriptionTooLong
        );
        Ok(())
    }

    pub fn validate_admin(admin: &Pubkey, signer: &Pubkey) -> Result<()> {
        require!(
            admin == signer,
            ContributionError::UnauthorizedAdmin
        );
        Ok(())
    }

    pub fn validate_period_state(period: &DistributionPeriod) -> Result<()> {
        require!(
            !period.is_finalized,
            ContributionError::PeriodAlreadyFinalized
        );
        Ok(())
    }
}

pub struct TimeUtils;

impl TimeUtils {
    pub fn get_current_time() -> Result<i64> {
        let clock = Clock::get()?;
        Ok(clock.unix_timestamp)
    }

    pub fn validate_period_duration(duration: i64) -> Result<()> {
        // Minimum 1 day, maximum 30 days in seconds
        require!(
            duration >= 86400 && duration <= 2592000,
            ContributionError::InvalidPeriodDuration
        );
        Ok(())
    }

    pub fn is_period_ended(period: &DistributionPeriod) -> Result<bool> {
        let current_time = Self::get_current_time()?;
        Ok(current_time >= period.end_time)
    }
}

pub struct PointCalculator;

impl PointCalculator {
    pub fn calculate_points(
        contribution_type: ContributionType,
        severity: Severity,
    ) -> Result<u8> {
        let points = match (contribution_type, severity) {
            (ContributionType::BugFix, Severity::Minor) => 2,
            (ContributionType::BugFix, Severity::Medium) => 4,
            (ContributionType::BugFix, Severity::Major) => 7,
            (ContributionType::BugFix, Severity::Critical) => 10,
            
            (ContributionType::FeatureDevelopment, Severity::Minor) => 3,
            (ContributionType::FeatureDevelopment, Severity::Medium) => 5,
            (ContributionType::FeatureDevelopment, Severity::Major) => 8,
            (ContributionType::FeatureDevelopment, Severity::Critical) => 10,
            
            (ContributionType::CodeOptimization, Severity::Minor) => 2,
            (ContributionType::CodeOptimization, Severity::Medium) => 4,
            (ContributionType::CodeOptimization, Severity::Major) => 7,
            (ContributionType::CodeOptimization, Severity::Critical) => 8,
            
            (ContributionType::BugReport, Severity::Minor) => 1,
            (ContributionType::BugReport, Severity::Medium) => 3,
            (ContributionType::BugReport, Severity::Major) => 6,
            (ContributionType::BugReport, Severity::Critical) => 9,
            
            (ContributionType::TestContribution, Severity::Minor) => 2,
            (ContributionType::TestContribution, Severity::Medium) => 4,
            (ContributionType::TestContribution, Severity::Major) => 6,
            (ContributionType::TestContribution, Severity::Critical) => 7,
        };
        
        ValidationUtils::validate_points(points)?;
        Ok(points)
    }

    pub fn calculate_token_distribution(
        user_points: u64,
        total_points: u64,
        token_pool: u64,
    ) -> Result<u64> {
        require!(total_points > 0, ContributionError::InsufficientPoints);
        
        Ok((user_points as u128)
            .checked_mul(token_pool as u128)
            .ok_or(ContributionError::DistributionCalculationError)?
            .checked_div(total_points as u128)
            .ok_or(ContributionError::DistributionCalculationError)? as u64)
    }
}

pub struct PdaUtils;

impl PdaUtils {
    pub fn get_contribution_tracker_address(
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[CONTRIBUTION_TRACKER_SEED],
            program_id,
        )
    }

    pub fn get_distribution_period_address(
        period_number: u64,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                DISTRIBUTION_PERIOD_SEED,
                period_number.to_le_bytes().as_ref(),
            ],
            program_id,
        )
    }

    pub fn get_contributor_address(
        authority: &Pubkey,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                CONTRIBUTOR_SEED,
                authority.as_ref(),
            ],
            program_id,
        )
    }

    pub fn get_contribution_address(
        contributor: &Pubkey,
        period_number: u64,
        sequence: u64,
        program_id: &Pubkey,
    ) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[
                CONTRIBUTION_SEED,
                contributor.as_ref(),
                period_number.to_le_bytes().as_ref(),
                sequence.to_le_bytes().as_ref(),
            ],
            program_id,
        )
    }
}

pub struct ContributionValidator;

impl ContributionValidator {
    pub fn validate_contribution_status_transition(
        current_status: ContributionStatus,
        new_status: ContributionStatus,
    ) -> Result<()> {
        match (current_status, new_status) {
            (ContributionStatus::Pending, ContributionStatus::Approved) => Ok(()),
            (ContributionStatus::Pending, ContributionStatus::Rejected) => Ok(()),
            _ => Err(ContributionError::InvalidStatusTransition.into()),
        }
    }

    pub fn validate_contributor_capacity(
        contributor: &Contributor,
    ) -> Result<()> {
        require!(
            contributor.contributions.len() < 100,
            ContributionError::MaxContributionsReached
        );
        Ok(())
    }
}