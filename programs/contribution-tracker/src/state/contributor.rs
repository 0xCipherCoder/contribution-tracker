use anchor_lang::prelude::*;

#[account]
#[derive(InitSpace)]
pub struct Contributor {
    pub authority: Pubkey,
    pub total_points_all_time: u64,
    pub current_period_points: u64,
    pub last_claimed_period: u64,
    #[max_len(100)]
    pub contributions: Vec<Pubkey>,
    pub bump: u8,
}