//! Immutable resource decisions queried once per admitted type or weapon.
//! The engine's existing resource owners retain all mutable counts and clocks.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponResourceContext {
    pub weapon_name: String,
}

/// Immutable facts from the bound native weapon. Kept in a separate optional
/// hook so older guests still receive their exact WeaponResource wire shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponFiringResourceContext {
    pub weapon_name: String,
    pub animation_count: u32,
    pub is_sonic: bool,
    pub use_fire_particles: bool,
}

/// Stateless inputs for the existing shot-damage and muzzle-animation owners.
/// The component chooses the number of facing bins; the host retains the
/// animation references, firing coordinates, damage stages and shot lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponFiringPolicy {
    pub apply_damage: bool,
    /// Zero for an empty list, otherwise a power of two within that list.
    /// 65,536 selects the full-word, unwrapped art-offset mode; the host
    /// bounds-checks its resulting index rather than wrapping it to a bin.
    pub muzzle_facings: u32,
}

impl WeaponFiringPolicy {
    pub fn validate_for(self, context: &WeaponFiringResourceContext) -> Result<(), String> {
        if context.animation_count == 0 {
            if self.muzzle_facings != 0 {
                return Err("empty muzzle animation list has no facing bins".into());
            }
        } else if !self.muzzle_facings.is_power_of_two()
            || self.muzzle_facings > context.animation_count
            || self.muzzle_facings > 65_536
        {
            return Err(
                "muzzle facing bins exceed the bound animation list or direction word".into(),
            );
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeResourceContext {
    pub type_name: String,
    pub user: u8,
}

pub const WEAPON_USER_UNIT: u8 = 1;
pub const WEAPON_USER_INFANTRY: u8 = 2;
pub const WEAPON_USER_BUILDING: u8 = 4;
pub const WEAPON_USER_AIRCRAFT: u8 = 8;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WeaponResourcePolicy {
    pub cost: i32,
    pub reload_after_use: bool,
    pub users: u8,
    /// Use the ordinary weapon rearm calculation for a loaded building,
    /// instead of the retail one-frame magazine shortcut. Missing on older
    /// guests means retail cadence; counts/timers still belong to the host.
    #[serde(default, skip_serializing_if = "is_false")]
    pub use_weapon_rof: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

impl WeaponResourcePolicy {
    pub fn applies_to(self, user: u8) -> bool {
        self.users & user != 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReloadPolicy {
    pub normal_amount: i32,
    pub empty_amount: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DepletedWeaponPolicy {
    pub weapon_index: i32,
    pub threshold: i32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpotlightAttachment {
    #[default]
    Body,
    Turret,
    Barrel,
}

/// Immutable inputs for the existing host searchlight owner. Turning off type
/// rules keeps the original building behavior; None on the containing resource
/// preserves the legacy component/profile fallback.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpotlightPolicy {
    pub has_spotlight: Option<bool>,
    pub use_type_rules: bool,
    pub start_height: i32,
    pub distance: i32,
    pub attached_to: SpotlightAttachment,
    pub glow_flags: u8,
}

impl Default for SpotlightPolicy {
    fn default() -> Self {
        Self {
            has_spotlight: None,
            use_type_rules: false,
            start_height: 430,
            distance: 1024,
            attached_to: SpotlightAttachment::Body,
            glow_flags: 0,
        }
    }
}

/// Frozen inputs for the existing same-object conversion owner. A target may
/// name any registered Techno class; the host's runtime admission decides
/// whether the source can convert to it. Secondary ROT keeps its signed input
/// and is applied only during conversion by the existing Facing owner.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeConversionPolicy {
    pub script_target: Option<crate::ProductionTypeRef>,
    pub secondary_rot: Option<i32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypeResourcePolicy {
    pub users: u8,
    pub reload: Option<ReloadPolicy>,
    pub depleted_weapon: Option<DepletedWeaponPolicy>,
    /// Local input/cursor admission only. Simulation orders, AI, cargo and
    /// destruction keep their existing owners and are not blocked by this.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub input: Option<InputPolicy>,
    /// Optional survival contribution. None uses the canonical type default
    /// under an explicitly selected tracked-presence match rule; false is an
    /// explicit exclusion. A match-rules hook is required to own this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub keep_alive: Option<bool>,
    /// Type capability, independent of temporary/permanent cloak grants.
    /// Some(true) explicitly restores retail permission on a disabled module.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cloak_allowed: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spotlight: Option<SpotlightPolicy>,
    /// None preserves older-component/profile behavior. Some with an empty
    /// target and rate explicitly removes the corresponding legacy overrides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub conversion: Option<TypeConversionPolicy>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct InputPolicy {
    pub deny_manual_attack: bool,
    pub deny_manual_unload: bool,
    pub hide_transport_entry: bool,
    pub deny_self_guard_area: bool,
}

impl TypeResourcePolicy {
    pub const fn empty(users: u8) -> Self {
        Self {
            users,
            reload: None,
            depleted_weapon: None,
            input: None,
            keep_alive: None,
            cloak_allowed: None,
            spotlight: None,
            conversion: None,
        }
    }

    pub fn applies_to(self, user: u8) -> bool {
        self.users & user != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn firing_queries_cannot_select_unbound_bins_or_mutate_entity_state() {
        use crate::{API_VERSION, Context, Plan, Request};
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::WeaponFiringResource(WeaponFiringResourceContext {
                weapon_name: "Cannon".into(),
                animation_count: 17,
                is_sonic: true,
                use_fire_particles: false,
            }),
            config: vec![],
            state: vec![],
        };
        request.validate().unwrap();
        let mut plan = Plan::unchanged(&request);
        assert!(
            !serde_json::to_string(&plan)
                .unwrap()
                .contains("weapon_firing_resource")
        );
        plan.weapon_firing_resource = Some(WeaponFiringPolicy {
            apply_damage: true,
            muzzle_facings: 16,
        });
        plan.validate_for(&request).unwrap();
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        plan.weapon_firing_resource.as_mut().unwrap().muzzle_facings = 32;
        assert!(plan.validate_for(&request).is_err());
        plan.weapon_firing_resource.as_mut().unwrap().muzzle_facings = 16;
        request.context = Context::WeaponResource(WeaponResourceContext {
            weapon_name: "Cannon".into(),
        });
        assert!(plan.validate_for(&request).is_err());
    }

    #[test]
    fn old_type_policy_omits_input_and_new_input_round_trips() {
        let old = r#"{"users":1,"reload":null,"depleted_weapon":null}"#;
        let mut policy: TypeResourcePolicy = serde_json::from_str(old).unwrap();
        assert_eq!(policy.input, None);
        assert_eq!(policy.keep_alive, None);
        assert_eq!(policy.cloak_allowed, None);
        assert_eq!(policy.spotlight, None);
        assert_eq!(policy.conversion, None);
        assert_eq!(serde_json::to_string(&policy).unwrap(), old);
        policy.input = Some(InputPolicy {
            deny_manual_attack: true,
            ..Default::default()
        });
        assert_eq!(
            serde_json::from_slice::<TypeResourcePolicy>(&serde_json::to_vec(&policy).unwrap())
                .unwrap(),
            policy
        );
        policy.keep_alive = Some(false);
        policy.cloak_allowed = Some(false);
        let bytes = serde_json::to_vec(&policy).unwrap();
        assert!(
            std::str::from_utf8(&bytes)
                .unwrap()
                .contains("\"keep_alive\":false")
        );
        assert!(
            std::str::from_utf8(&bytes)
                .unwrap()
                .contains("\"cloak_allowed\":false")
        );
        assert_eq!(
            serde_json::from_slice::<TypeResourcePolicy>(&bytes)
                .unwrap()
                .keep_alive,
            Some(false)
        );
        policy.conversion = Some(TypeConversionPolicy {
            script_target: Some(crate::ProductionTypeRef {
                category: crate::ProductionObjectKind::Building,
                ordinal: 3,
            }),
            secondary_rot: Some(-255),
        });
        assert_eq!(
            serde_json::from_slice::<TypeResourcePolicy>(&serde_json::to_vec(&policy).unwrap())
                .unwrap(),
            policy
        );
    }

    #[test]
    fn old_weapon_policy_keeps_retail_cadence_and_new_policy_round_trips() {
        let old = r#"{"cost":2,"reload_after_use":true,"users":7}"#;
        let mut policy: WeaponResourcePolicy = serde_json::from_str(old).unwrap();
        assert!(!policy.use_weapon_rof);
        assert_eq!(serde_json::to_string(&policy).unwrap(), old);
        policy.use_weapon_rof = true;
        let bytes = serde_json::to_vec(&policy).unwrap();
        assert_eq!(
            serde_json::from_slice::<WeaponResourcePolicy>(&bytes).unwrap(),
            policy
        );
    }
}
