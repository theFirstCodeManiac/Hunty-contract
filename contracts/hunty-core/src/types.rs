use soroban_sdk::{contracttype, Address, BytesN, Env, Map, String, Vec};

/// Semantic version (major.minor.patch). Compatible if major matches and self >= required.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn is_compatible_with(&self, required: &SemVer) -> bool {
        self.major == required.major
            && (self.minor > required.minor
                || (self.minor == required.minor && self.patch >= required.patch))
    }
}

/// Semantic version (major.minor.patch). Compatible if major matches and self >= required.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SemVer {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
}

impl SemVer {
    pub fn is_compatible_with(&self, required: &SemVer) -> bool {
        self.major == required.major
            && (self.minor > required.minor
                || (self.minor == required.minor && self.patch >= required.patch))
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum HuntStatus {
    Draft,
    Active,
    Completed,
    Cancelled,
    Paused,
    EmergencyStopped,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RewardConfig {
    pub xlm_pool: i128,
    pub nft_enabled: bool,
    pub nft_contract: Option<Address>,
    pub max_winners: u32,
    pub claimed_count: u32,
    pub nft_rarity: u32,
    pub nft_tier: u32,
}

pub type HuntRewardConfig = RewardConfig;

#[contracttype]
#[derive(Clone, Debug)]
pub struct Hunt {
    pub hunt_id: u64,
    pub creator: Address,
    pub title: String,
    pub description: String,
    pub status: HuntStatus,
    pub created_at: u64,
    pub activated_at: u64,
    pub end_time: u64,
    pub reward_config: RewardConfig,
    pub time_bonus_start_bps: Option<u32>,
    pub time_bonus_min_bps: Option<u32>,
    pub time_bonus_decay_secs: Option<u64>,
    pub total_clues: u32,
    pub required_clues: u32,
    pub completed_count: u32,
    pub max_submissions_per_minute: u32,
    pub max_attempts_per_clue: u32,
    pub start_multiplier_bps: u32,
}

/// Stored clue with SHA256 answer hash. The hash is never exposed via get_clue/list_clues or events.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Clue {
    pub clue_id: u32,
    pub question: String,
    pub answer_hashes: Vec<BytesN<32>>,
    pub points: u32,
    pub is_required: bool,
    pub difficulty: u32,
}

/// Clue info returned by get_clue/list_clues. Excludes answer hash.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClueInfo {
    pub clue_id: u32,
    pub question: String,
    pub points: u32,
    pub is_required: bool,
    pub difficulty: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntCancelledEvent {
    pub hunt_id: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntDeactivatedEvent {
    pub hunt_id: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct HuntActivatedEvent {
    pub hunt_id: u64,
    pub activated_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq, Default)]
pub struct Location {
    pub latitude: i64,  // Degrees * 1_000_000
    pub longitude: i64, // Degrees * 1_000_000
    pub radius: u32,
}


/// Internal compact storage representation of player progress.
/// Does not store `player` or `hunt_id` — those are already the storage key.
///
/// ## Compact encoding
/// - Timestamps are delta-encoded as `u32` offsets from the hunt's `activated_at`,
///   saving 4 bytes each vs full `u64` UNIX timestamps. The max delta (~136 years)
///   far exceeds any realistic hunt duration.
/// - Boolean fields (`is_completed`, `reward_claimed`) are packed into `flags`.
/// - `clue_attempts` values use `u32` (Soroban's smallest XDR integer).
#[contracttype]
#[derive(Clone, Debug)]
pub struct StoredPlayerProgress {
    pub completed_clues: Vec<u32>,
    pub total_score: u32,
    pub required_completed_count: u32,

    /// Seconds elapsed from hunt `activated_at` to player registration.
    /// Reconstruct absolute: `activated_at + started_at_delta`.
    pub started_at_delta: u32,

    /// Seconds elapsed from player registration to hunt completion, or 0 if not completed.
    /// Reconstruct absolute: `activated_at + started_at_delta + completed_at_delta`.
    pub completed_at_delta: u32,

    /// Bit flags for boolean fields to reduce storage footprint.
    /// BIT0 (1): is_completed
    /// BIT1 (2): reward_claimed
    /// BIT2–BIT7: reserved for future use
    pub flags: u8,
    pub started_at: u64,
    pub completed_at: u64,
    /// Packed boolean flags: bit 0 = is_completed, bit 1 = reward_claimed
    pub flags: u32,
    pub recent_submissions: Vec<u64>,
}



/// Public view of player progress, with `player` and `hunt_id` reconstructed from the key.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerProgress {
    pub player: Address,
    pub hunt_id: u64,
    pub completed_clues: Vec<u32>,
    pub completed_clue_index: Map<u32, bool>,
    pub total_score: u32,
    pub started_at: u64,
    pub completed_at: u64,
    pub is_completed: bool,
    pub reward_claimed: bool,
    pub recent_submissions: Vec<u64>,
}

impl PlayerProgress {
    pub fn new(env: &Env, player: Address, hunt_id: u64, current_time: u64) -> Self {
        Self {
            player,
            hunt_id,
            completed_clues: Vec::new(env),
            completed_clue_index: Map::new(env),
            total_score: 0,
            started_at: current_time,
            completed_at: 0,
            is_completed: false,
            reward_claimed: false,
            recent_submissions: Vec::new(env),
        }
    }

    /// Extract is_completed flag from packed flags byte
    fn flags_to_is_completed(flags: u32) -> bool {
        (flags & 0x01) != 0
    }

    /// Extract reward_claimed flag from packed flags byte
    fn flags_to_reward_claimed(flags: u32) -> bool {
        (flags & 0x02) != 0
    }

    /// Pack boolean flags into a single byte
    fn bools_to_flags(is_completed: bool, reward_claimed: bool) -> u32 {
        let mut flags = 0u32;
        if is_completed {
            flags |= 0x01;
        }
        if reward_claimed {
            flags |= 0x02;
        }
        flags
    }

    /// Convert to the compact form stored on-chain (drops redundant key fields).
    ///
    /// `activated_at` is the hunt's activation timestamp, used to delta-encode
    /// `started_at` and `completed_at` into compact `u32` offsets.
    pub fn to_stored(&self, activated_at: u64) -> StoredPlayerProgress {
        let mut flags: u8 = 0;
        if self.is_completed {
            flags |= 0b0000_0001;
        }
        if self.reward_claimed {
            flags |= 0b0000_0010;
        }

        // Delta-encode timestamps relative to hunt activation.
        let started_at_delta = self.started_at.saturating_sub(activated_at) as u32;
        let completed_at_delta = if self.completed_at == 0 {
            0u32
        } else {
            self.completed_at.saturating_sub(self.started_at) as u32
        };

        StoredPlayerProgress {
            completed_clues: self.completed_clues.clone(),
            total_score: self.total_score,
            required_completed_count: self.required_completed_count,
            started_at_delta,
            completed_at_delta,
            flags,
            started_at: self.started_at,
            completed_at: self.completed_at,
            flags: Self::bools_to_flags(self.is_completed, self.reward_claimed),
            recent_submissions: self.recent_submissions.clone(),
        }
    }


    /// Reconstruct from stored form plus the key fields.
    ///
    /// `activated_at` is the hunt's activation timestamp, used to reconstruct
    /// absolute timestamps from the stored deltas.
    pub fn from_stored(
        env: &Env,
        stored: StoredPlayerProgress,
        player: Address,
        hunt_id: u64,
        activated_at: u64,
    ) -> Self {
        let mut completed_clue_index = Map::new(env);
        for i in 0..stored.completed_clues.len() {
            let clue_id = stored.completed_clues.get(i).unwrap();
            completed_clue_index.set(clue_id, true);
        }

        // Reconstruct absolute timestamps from deltas.
        let started_at = activated_at + (stored.started_at_delta as u64);
        let completed_at = if stored.completed_at_delta == 0 {
            0u64
        } else {
            started_at + (stored.completed_at_delta as u64)
        };

    pub fn from_stored(stored: StoredPlayerProgress, player: Address, hunt_id: u64) -> Self {
        Self {
            player,
            hunt_id,
            completed_clues: stored.completed_clues,
            completed_clue_index,
            total_score: stored.total_score,
            required_completed_count: stored.required_completed_count,
            started_at,
            completed_at,
            is_completed: (stored.flags & 0b0000_0001) != 0,
            reward_claimed: (stored.flags & 0b0000_0010) != 0,
            clue_attempts: stored.clue_attempts,
            total_score: stored.total_score,
            started_at: stored.started_at,
            completed_at: stored.completed_at,
            is_completed: Self::flags_to_is_completed(stored.flags),
            reward_claimed: Self::flags_to_reward_claimed(stored.flags),
            recent_submissions: stored.recent_submissions,
        }
    }

    pub fn has_completed_clue(&self, clue_id: u32) -> bool {
        for i in 0..self.completed_clues.len() {
            if self.completed_clues.get(i).unwrap() == clue_id {
                return true;
            }
        }
        false
    }

    pub fn complete_clue(&mut self, _env: &Env, clue_id: u32, points: u32) -> Result<(), crate::errors::HuntErrorCode> {
        if !self.has_completed_clue(clue_id) {
            self.completed_clues.push_back(clue_id);
            self.total_score = self.total_score.checked_add(points)
                .ok_or(crate::errors::HuntErrorCode::ScoreOverflow)?;
        }
        Ok(())
    }

}

impl Hunt {
    pub fn is_active(&self, current_time: u64) -> bool {
        self.status == HuntStatus::Active && (self.end_time == 0 || current_time < self.end_time)
    }

    pub fn has_rewards_available(&self) -> bool {
        self.reward_config.claimed_count < self.reward_config.max_winners
    }
}

impl RewardConfig {
    pub fn new(
        _env: &Env,
        xlm_pool: i128,
        nft_enabled: bool,
        nft_contract: Option<Address>,
        max_winners: u32,
        nft_rarity: u32,
        nft_tier: u32,
    ) -> Self {
        Self {
            xlm_pool,
            nft_enabled,
            nft_contract,
            max_winners,
            claimed_count: 0,
            nft_rarity,
            nft_tier,
        }
    }

    pub fn reward_per_winner(&self) -> i128 {
        if self.max_winners == 0 {
            0
        } else {
            self.xlm_pool / (self.max_winners as i128)
        }
    }
}

// Events
#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntCreatedEvent {
    pub hunt_id: u64,
    pub creator: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorBlacklistedEvent {
    pub creator: Address,
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CreatorRemovedFromBlacklistEvent {
    pub creator: Address,
    pub admin: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntStatusChangedEvent {
    pub hunt_id: u64,
    pub old_status: HuntStatus,
    pub new_status: HuntStatus,
    pub changed_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ClueCompletedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub clue_id: u32,
    pub points_earned: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct HuntCompletedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub total_score: u32,
    pub completion_time: u64,
    pub completion_rank: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub xlm_amount: i128,
    pub nft_awarded: bool,
}

/// Emitted when a clue is added. Does not expose the answer hash.
#[contracttype]
#[derive(Clone, Debug)]
pub struct ClueAddedEvent {
    pub hunt_id: u64,
    pub clue_id: u32,
    pub creator: Address,
    pub question: String,
    pub points: u32,
    pub is_required: bool,
    /// Difficulty multiplier (1-10).
    pub difficulty: u32,
}

/// Emitted when a player registers for an active hunt.
#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerRegisteredEvent {
    pub hunt_id: u64,
    pub player: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerBannedEvent {
    pub hunt_id: u64,
    pub player: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct PlayerUnbannedEvent {
    pub hunt_id: u64,
    pub player: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct AnswerIncorrectEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub clue_id: u32,
    pub timestamp: u64,
}

/// Leaderboard entry for a single player in a hunt (read-only query result).
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardEntry {
    pub rank: u32,
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
}

/// Stored top-N leaderboard entry maintained incrementally on score changes.
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardIndexEntry {
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
}

/// Aggregate statistics for a hunt (read-only query result).
#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HuntStatistics {
    pub total_players: u32,
    pub completed_count: u32,
    pub completion_rate_percent: u32,
    pub total_score_sum: u64,
    pub average_score: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct ClueAliasesAddedEvent {
    pub hunt_id: u64,
    pub clue_id: u32,
    pub creator: Address,
    pub aliases_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardManagerSetEvent {
    pub old_address: Option<Address>,
    pub new_address: Address,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TimeBonusConfig {
    pub start_multiplier_bps: u32,
    pub min_multiplier_bps: u32,
    pub decay_duration_secs: u64,
}

impl TimeBonusConfig {
    pub fn is_valid(&self) -> bool {
        self.decay_duration_secs > 0
            && self.start_multiplier_bps >= self.min_multiplier_bps
            && self.min_multiplier_bps >= 10_000
    }

    pub fn multiplier_bps_at(&self, elapsed_secs: u64) -> u32 {
        if self.decay_duration_secs == 0 {
            return self.min_multiplier_bps;
        }

        if elapsed_secs >= self.decay_duration_secs {
            return self.min_multiplier_bps;
        }

        let start = self.start_multiplier_bps as u128;
        let min = self.min_multiplier_bps as u128;
        let span = start.saturating_sub(min);
        let decay = (span * elapsed_secs as u128) / self.decay_duration_secs as u128;
        (start.saturating_sub(decay)) as u32
    }
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RateLimitStatus {
    pub creations_today: u32,
    pub daily_limit: u32,
    pub cooldown_seconds: u64,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardRow {
    pub index: u32,
    pub player: Address,
    pub score: u32,
    pub completed_at: u64,
    pub is_completed: bool,
}

#[contracttype]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LeaderboardWindow {
    pub entries: Vec<LeaderboardRow>,
    pub next_index: u32,
    pub finished: bool,
    pub queried_at: u64,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimFailedEvent {
    pub hunt_id: u64,
    pub player: Address,
    pub error_code: u32,
}
