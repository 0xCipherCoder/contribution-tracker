use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct DistributionPeriod {
    pub period_number: u64,
    pub start_time: i64,
    pub end_time: i64,
    pub total_points: u64,
    pub tokens_allocated: u64,
    pub tokens_distributed: u64,
    pub is_finalized: bool,
    pub bump: u8,
}