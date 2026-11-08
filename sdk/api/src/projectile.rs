//! Immutable projectile recipes. Live Bullet identities, motion, random stream
//! and detonation lifetime belong to the host's existing owners.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectileResourceContext {
    pub projectile_name: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectileResourcePolicy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub airburst: Option<AirburstPolicy>,
}

/// Inputs for the existing host fan. Raw binary64 retains native exceptional
/// values and reader rounding; no guest owns a second target or Bullet list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AirburstPolicy {
    pub splits: bool,
    pub spread_bits: u64,
    pub retarget_accuracy_bits: u64,
    pub retarget_self: bool,
    pub around_target: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{API_VERSION, Context, Plan, Request};

    #[test]
    fn projectile_queries_are_immutable_and_cannot_cross_resource_domains() {
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::ProjectileResource(ProjectileResourceContext {
                projectile_name: "SHOT".into(),
            }),
            config: vec![],
            state: vec![],
        };
        let mut plan = Plan::unchanged(&request);
        assert!(
            !serde_json::to_string(&plan)
                .unwrap()
                .contains("projectile_resource")
        );
        plan.projectile_resource = Some(ProjectileResourcePolicy {
            airburst: Some(AirburstPolicy {
                splits: true,
                spread_bits: f64::NAN.to_bits(),
                retarget_accuracy_bits: 0,
                retarget_self: true,
                around_target: false,
            }),
        });
        plan.validate_for(&request).unwrap();
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        request.context = Context::MatchRules(crate::MatchRulesContext {});
        assert!(plan.validate_for(&request).is_err());
    }
}
