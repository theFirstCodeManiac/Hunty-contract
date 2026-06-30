use soroban_sdk::{contracttype, Address, Vec};

pub use reward_interface::{
    resolve_tier_amount, tiers_are_strictly_ascending, RewardConfig, TierError,
    TimeBasedRewardTier,
};

/// Outcome of a manually resolved distribution.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ResolutionStatus {
    Completed,
    Refunded,
}

/// Status of a reward distribution for a specific hunt and player.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionStatus {
    /// Whether any reward has been distributed.
    pub distributed: bool,
    /// XLM amount distributed (0 if none).
    pub xlm_amount: i128,
    /// NFT ID if an NFT was minted.
    pub nft_id: Option<u64>,
}

/// Internal record stored for each distribution.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DistributionRecord {
    pub xlm_amount: i128,
    pub nft_id: Option<u64>,
}

/// Configuration for a reward pool, set at creation time.
///
/// `time_based_tiers` is an optional list of (max_elapsed_seconds, xlm_amount)
/// pairs that define a conditional reward schedule based on how quickly a
/// player completes a hunt. When the list is empty, time-based conditional
/// rewards are disabled and the rest of the system behaves exactly as
/// before this feature was added. When the list is non-empty it must be
/// sorted in strictly ascending order of `max_completion_secs` (validated
/// in `set_pool_tiers`). The list can be updated after pool creation via
/// `set_pool_tiers` and queried via `get_pool_config`.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPoolConfig {
    /// Address of the hunt creator who owns this pool.
    /// Only the creator is authorized to fund it.
    pub creator: Address,
    /// Minimum XLM amount per distribution. 0 means no minimum enforced.
    pub min_distribution_amount: i128,
    /// Optional time-based reward tiers. When empty, the per-winner amount
    /// is computed from `xlm_pool / max_winners` as before. When populated,
    /// the appropriate tier's `xlm_amount` is selected at distribution time
    /// based on the player's (completion_time - registration_time) elapsed.
    pub time_based_tiers: Vec<TimeBasedRewardTier>,
}

/// Full status of a reward pool, returned by get_reward_pool().
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardPoolStatus {
    /// Current available balance for distributions.
    pub balance: i128,
    /// Cumulative total deposited into this pool across all fund calls.
    pub total_deposited: i128,
    /// Cumulative total distributed from this pool.
    pub total_distributed: i128,
    /// Pool creator / only authorized funder.
    pub creator: Address,
    /// Minimum XLM per distribution (0 = no minimum).
    pub min_distribution_amount: i128,
}

/// Result of a pool validation check, returned by validate_pool().
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ValidationResult {
    /// Whether the pool has sufficient funds for the required amount
    /// and the required amount meets the pool's minimum distribution size.
    pub is_valid: bool,
    /// Current pool balance at time of check.
    pub balance: i128,
    /// Required amount that was checked against.
    pub required: i128,
}
