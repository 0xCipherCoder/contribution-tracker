use anchor_lang::prelude::*;
use super::enums::*;

#[account]
#[derive(InitSpace)]
pub struct Contribution {
    pub contributor: Pubkey,
    pub distribution_period: u64,
    pub contribution_type: ContributionType,
    pub severity: Severity,
    pub points: u8,
    pub timestamp: i64,
    #[max_len(100)]
    pub description: String,
    pub status: ContributionStatus,
    pub bump: u8,
}