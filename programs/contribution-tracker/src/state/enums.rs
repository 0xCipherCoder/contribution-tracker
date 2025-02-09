use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ContributionType {
    BugFix,
    FeatureDevelopment,
    CodeOptimization, 
    BugReport,
    TestContribution,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum Severity {
    Minor,
    Medium,
    Major,
    Critical,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum ContributionStatus {
    Pending,
    Approved,
    Rejected,
}