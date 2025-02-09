use anchor_lang::prelude::*;

#[error_code]
pub enum ContributionError {
    #[msg("The provided contribution type is invalid")]
    InvalidContributionType,

    #[msg("The provided severity level is invalid")]
    InvalidSeverityLevel,

    #[msg("Points must be between 1 and 10")]
    InvalidPointsRange,

    #[msg("Description length exceeds maximum allowed")]
    DescriptionTooLong,

    #[msg("The distribution period has not ended yet")]
    PeriodNotEnded,

    #[msg("The distribution period has already ended")]
    PeriodAlreadyEnded,

    #[msg("The distribution period has already been finalized")]
    PeriodAlreadyFinalized,

    #[msg("Cannot claim rewards for future periods")]
    FuturePeriodClaim,

    #[msg("Rewards for this period have already been claimed")]
    AlreadyClaimed,

    #[msg("Invalid period duration provided")]
    InvalidPeriodDuration,

    #[msg("Contribution has already been processed")]
    ContributionAlreadyProcessed,

    #[msg("Invalid authority provided")]
    InvalidAuthority,

    #[msg("Insufficient points for distribution")]
    InsufficientPoints,

    #[msg("Maximum contributions reached for this period")]
    MaxContributionsReached,

    #[msg("Contribution not found")]
    ContributionNotFound,

    #[msg("Invalid token allocation")]
    InvalidTokenAllocation,

    #[msg("Contributor account not initialized")]
    ContributorNotInitialized,

    #[msg("Only admin can perform this action")]
    UnauthorizedAdmin,

    #[msg("Period transition in progress")]
    PeriodTransitionInProgress,

    #[msg("Invalid timestamp provided")]
    InvalidTimestamp,

    #[msg("Contribution requires review")]
    ContributionRequiresReview,

    #[msg("Invalid contribution status transition")]
    InvalidStatusTransition,

    #[msg("Reserve pool calculation error")]
    ReservePoolError,

    #[msg("Distribution calculation error")]
    DistributionCalculationError,

    #[msg("Period does not exist")]
    PeriodDoesNotExist,

    #[msg("Contribution must be approved before points are awarded")]
    UnapprovedContribution,

    #[msg("Cannot modify finalized contribution")]
    CannotModifyFinalized,

    #[msg("Invalid points threshold")]
    InvalidPointsThreshold,

    #[msg("Token transfer failed")]
    TokenTransferFailed,
}