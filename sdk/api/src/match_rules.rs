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
    /// Selects the existing Script68 handler even when no current type names
    /// a conversion target. None keeps older-component/profile behavior.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script_type_conversion: Option<bool>,
    /// 0 means unrestricted; other AI Side tokens select token-1. The host
    /// still owns Side identity and AI trigger evaluation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub extended_ai_side_tokens: Option<bool>,
    /// Selects the existing registered-Side UI dispatch (Scenario.Side source,
    /// sidebar routes and EVA cache semantics). Resource names still come
    /// from the actual Side declaration; absence keeps the host's legacy mode.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registered_side_ui: Option<bool>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn old_match_rules_omit_conversion_and_explicit_off_round_trips() {
        let old = r#"{"survival":"retail"}"#;
        let mut policy: MatchRulesPolicy = serde_json::from_str(old).unwrap();
        assert_eq!(policy.script_type_conversion, None);
        assert_eq!(policy.registered_side_ui, None);
        assert_eq!(serde_json::to_string(&policy).unwrap(), old);
        policy.script_type_conversion = Some(false);
        policy.registered_side_ui = Some(false);
        let bytes = serde_json::to_vec(&policy).unwrap();
        assert_eq!(
            serde_json::from_slice::<MatchRulesPolicy>(&bytes).unwrap(),
            policy
        );
        assert!(
            std::str::from_utf8(&bytes)
                .unwrap()
                .contains("\"script_type_conversion\":false")
        );
    }
}
