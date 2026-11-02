//! Frozen electromagnetic-pulse declarations. The host owns status counters,
//! offline transitions, mission scheduling, trigger dispatch and held animation
//! references; these queries never cache live simulation answers.
use serde::{Deserialize, Serialize};

/// A nullable registered resource pointer whose unset value inherits a host
/// default. Explicit Clear must survive JSON independently from Inherit.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    content = "name",
    rename_all = "snake_case",
    deny_unknown_fields
)]
pub enum NullableResourceName {
    #[default]
    Inherit,
    Clear,
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EmpResourceContext {
    Rules,
    Type { user: u8, name: String },
    Warhead { name: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmpTypePolicy {
    pub modifier_bits: u64,
    pub threshold: i32,
    pub immune: Option<bool>,
    pub veteran_immune: bool,
    pub elite_immune: bool,
    pub sparkles: NullableResourceName,
}

impl Default for EmpTypePolicy {
    fn default() -> Self {
        Self {
            modifier_bits: 1.0f64.to_bits(),
            threshold: -1,
            immune: None,
            veteran_immune: false,
            elite_immune: false,
            sparkles: NullableResourceName::Inherit,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EmpWarheadPolicy {
    pub duration: i32,
    pub cap: i32,
    pub sparkles: Option<String>,
    pub affects_enemies: bool,
    pub affects_owner: Option<bool>,
}

impl Default for EmpWarheadPolicy {
    fn default() -> Self {
        Self {
            duration: 0,
            cap: -1,
            sparkles: None,
            affects_enemies: true,
            affects_owner: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum EmpResourcePolicy {
    Rules {
        /// Select the existing bounded host EMP status mechanism.
        enabled: bool,
        /// Native mission discriminant. The host validates it with its single
        /// mission vocabulary before binding; no guest copy of that table.
        ai_recover_mission: Option<u16>,
        /// Reactivation checks the real ordered House power-center records.
        reactivation_requires_powered_center: bool,
    },
    Type {
        policy: EmpTypePolicy,
    },
    Warhead {
        policy: EmpWarheadPolicy,
    },
}

impl EmpResourcePolicy {
    pub(crate) fn validate_for(&self, context: &EmpResourceContext) -> Result<(), String> {
        if !matches!(
            (self, context),
            (Self::Rules { .. }, EmpResourceContext::Rules)
                | (Self::Type { .. }, EmpResourceContext::Type { .. })
                | (Self::Warhead { .. }, EmpResourceContext::Warhead { .. })
        ) {
            return Err("EMP policy does not match its resource query".into());
        }
        Ok(())
    }
}
