use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct ContributionTracker {
    pub admin: Pubkey,
    pub current_period: u64,
    pub period_duration: i64,
    pub total_points_all_time: u64,
    pub minimum_points_threshold: u32,
    pub reserve_pool_amount: u64,
    pub tokens_per_period: u64,
    pub bump: u8,
}