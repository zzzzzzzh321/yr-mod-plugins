//! Immutable match-wide rules, queried from frozen configuration at admission.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchRulesContext {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurvivalRule {
    Retail,
    /// Tracked contributions, the full BaseUnit list in short games and
    /// owned Foot presence in normal games. Counts and lifecycle are host-owned.
    TrackedPresence,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchRulesPolicy {
    pub survival: SurvivalRule,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<EffectRuntimePolicy>,
}

/// Frozen selectors for existing shared numeric and crate owners. The
/// component chooses them once; instance presence never selects a rule mode.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EffectRuntimePolicy {
    pub apply_rof: bool,
    pub allow_aircraft_speed_crates: bool,
    pub allow_unarmed_firepower_crates: bool,
    pub cloak_crate_uses_cloakability: bool,
    #[serde(default)]
    pub cloak: CloakRuntimePolicy,
}

/// Parameters of the shared Foot cloak producer. The component selects the
/// existing predicates and their ordering; CloakRuntime keeps the only clock,
/// stage and sound state. All-false retains the retail producer.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct CloakRuntimePolicy {
    pub use_motion_query: bool,
    pub motion_blocks_raw_ability: bool,
    pub offline_blocks_ability: bool,
    pub blockers_apply_to_rank: bool,
    pub check_type_permission: bool,
    pub deny_parachuted: bool,
    pub check_recloak_before_weapon: bool,
    pub allow_active_ability_above_ground: bool,
}
