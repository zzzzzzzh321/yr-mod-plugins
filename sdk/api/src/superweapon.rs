//! Configuration-time SuperWeapon visits. Native reference allocation and
//! constructor order stay with the original Rules registry owner.
use serde::{Deserialize, Serialize};

/// Source facts from the existing Rules registry. The host retains lookup
/// prefixes and Country aliases privately at each physical reader; callers
/// cannot substitute a final catalog for an earlier lookup.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponRegistryView {
    pub superweapons: Vec<String>,
    pub reads: Vec<SuperWeaponReadPoint>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponReadPoint {
    pub site: SuperWeaponReadSite,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SuperWeaponReadSite {
    Type {
        ordinal: u32,
        /// None records an actual traversal visit without a Rules body.
        section: Option<String>,
    },
    Building {
        object: crate::ProductionTypeRef,
        section: String,
    },
}

/// A scoped reader request. `read` indexes the current physical pass, never a
/// caller-supplied registry size. Retained gameplay values belong to the guest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperReadValue<T> {
    pub read: u32,
    pub current: T,
}

/// Facts at one original traversal visit, not a completed Rules pass or a
/// projected final INI. Absent sections still reach initialization once.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigureSuperReadContext {
    pub read: u32,
    pub ordinal: u32,
    pub stored_id: String,
    pub body_read: bool,
    /// Current retained Type read with the native 0x28-byte capacity.
    /// No body means no read, rather than reusing a later projected Type.
    pub effective_type: Option<String>,
}

impl ConfigureSuperReadContext {
    pub fn validate(&self) -> Result<(), String> {
        let valid = |value: &str| {
            !value.is_empty() && value.len() <= 256 && !value.chars().any(char::is_control)
        };
        if !valid(&self.stored_id)
            || self
                .effective_type
                .as_deref()
                .is_some_and(|value| !valid(value))
            || !self.body_read && self.effective_type.is_some()
        {
            return Err("invalid SuperWeapon read point".into());
        }
        Ok(())
    }
}

/// The request supplies neither an arbitrary key nor a destination family.
/// None on Plan retains the old profile; Retain explicitly skips this read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuperReferenceRead {
    Retain,
    /// Execute the one existing SW.Warhead reader on the current physical
    /// section. Its missing/empty/null and existing-only lookup semantics remain
    /// host-owned; this is not a direct assignment of an arbitrary name.
    WarheadFromCurrentSection,
    /// Select the fixed Nuclear reference reader at this physical visit.
    /// Initialization creates the native default payload before its optional
    /// lookup-only override; neither names nor destination ordinals are supplied.
    NuclearFromCurrentSection {
        initialize: bool,
    },
}

/// One immutable batch after configuration. No per-frame plugin calls or
/// guest-owned charge, money, ticket, delivery queue or launch history.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponResourceContext {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SuperWeaponSemantics {
    pub additional_providers: bool,
    pub extended_availability: bool,
    pub definition_building_animation: bool,
    pub grant_charge_history: bool,
    pub paid_launches: bool,
    pub custom_payloads: bool,
    pub launch_presentation: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponAvailabilityPolicy {
    pub allow_player: bool,
    pub allow_ai: bool,
    pub required_houses: u32,
    pub forbidden_houses: u32,
    pub shots: i32,
    /// Building ordinals resolved at the original physical reader. Native
    /// AuxBuilding remains an independent host-owned requirement.
    pub auxiliary: Vec<u32>,
    pub negative: Vec<u32>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponChargePolicy {
    pub initial_ready: bool,
    pub virtual_charge: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponPresentationPolicy {
    pub animation: crate::NullableResourceName,
    pub height: i32,
    pub visibility: u8,
    pub sound: crate::NullableResourceName,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponTargetingPolicy {
    pub manual_target: u8,
    pub manual_house: u8,
    pub automatic_target: u8,
    pub automatic_house: u8,
    pub offensive_ai: bool,
    pub fire_into_shroud: bool,
    /// Native binary64 bits; NaN is intentionally representable because the
    /// original range reader distinguishes invalid input from absent (-1).
    pub range_minimum: u64,
    pub range_maximum: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuperWeaponDeliveryOwner {
    Invoker,
    Neutral,
    Special,
    Civilian,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum SuperWeaponPayloadPolicy {
    Retail,
    GenericWarhead {
        /// The binder reads this declaration's original retained WH slot.
        /// There is no second name lookup or allocation request here.
        damage: i32,
        targeting: SuperWeaponTargetingPolicy,
    },
    UnitDelivery {
        types: Vec<crate::ProductionTypeRef>,
        deferment: i32,
        owner: SuperWeaponDeliveryOwner,
        base_normal: bool,
        action: String,
        activation_sound: crate::NullableResourceName,
    },
    /// Immutable Nuclear handler settings. The host binds retained payload and
    /// warhead slots from the same original physical reader, never final names.
    Nuclear {
        /// Negative values retain the selected payload Weapon's damage.
        damage: i32,
        silo_launch: bool,
        take_off: crate::NullableResourceName,
        psi_warning: crate::NullableResourceName,
        activation_sound: crate::NullableResourceName,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponDeclaration {
    pub ordinal: u32,
    pub stored_id: String,
    pub availability: SuperWeaponAvailabilityPolicy,
    pub charge: SuperWeaponChargePolicy,
    pub constraints: u16,
    pub use_ai_targeting: bool,
    pub money_amount: i32,
    pub insufficient_funds_eva: crate::NullableResourceName,
    pub presentation: SuperWeaponPresentationPolicy,
    pub payload: SuperWeaponPayloadPolicy,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponProviderDeclaration {
    pub object: crate::ProductionTypeRef,
    /// Original SuperWeapon registry indices, with order and duplicates kept.
    pub additional: Vec<u32>,
}

/// None on Plan retains the previous profile. A present default policy selects
/// stock semantics, including in a legacy host; named resources are admitted
/// against the exact process-selected catalogs before any effective mutation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponResourcePolicy {
    pub semantics: SuperWeaponSemantics,
    pub sound_source_hash: Option<u64>,
    pub eva_source_hash: Option<u64>,
    pub definitions: Vec<SuperWeaponDeclaration>,
    pub providers: Vec<SuperWeaponProviderDeclaration>,
}

impl SuperWeaponResourcePolicy {
    pub fn validate(&self) -> Result<(), String> {
        use crate::{NullableResourceName, ProductionObjectKind};
        use std::collections::BTreeSet;
        const LIMIT: usize = 65_536;
        let valid_name = |name: &str| {
            !name.is_empty() && name.len() <= 256 && !name.chars().any(char::is_control)
        };
        let valid_reference =
            |value: &NullableResourceName, source: Option<u64>, audio: bool| match value {
                NullableResourceName::Named(name) => {
                    valid_name(name) && (!audio || source.is_some())
                }
                _ => true,
            };
        if self.definitions.len() > LIMIT || self.providers.len() > LIMIT {
            return Err("SuperWeapon declaration limit exceeded".into());
        }
        let mut ordinals = BTreeSet::new();
        for definition in &self.definitions {
            if !ordinals.insert(definition.ordinal)
                || !valid_name(&definition.stored_id)
                || definition.availability.auxiliary.len() > LIMIT
                || definition.availability.negative.len() > LIMIT
                || definition.constraints & !0x1ff != 0
                || definition.presentation.visibility & !7 != 0
                || !valid_reference(&definition.presentation.animation, None, false)
                || !valid_reference(&definition.presentation.sound, self.sound_source_hash, true)
                || !valid_reference(
                    &definition.insufficient_funds_eva,
                    self.eva_source_hash,
                    true,
                )
            {
                return Err("invalid SuperWeapon declaration".into());
            }
            match &definition.payload {
                SuperWeaponPayloadPolicy::Retail => {}
                SuperWeaponPayloadPolicy::Nuclear {
                    take_off,
                    psi_warning,
                    activation_sound,
                    ..
                } => {
                    if !valid_reference(take_off, None, false)
                        || !valid_reference(psi_warning, None, false)
                        || !valid_reference(activation_sound, self.sound_source_hash, true)
                    {
                        return Err("invalid Nuclear resource declaration".into());
                    }
                }
                SuperWeaponPayloadPolicy::GenericWarhead { targeting, .. } => {
                    if targeting.manual_house & !7 != 0 || targeting.automatic_house & !7 != 0 {
                        return Err("invalid SuperWeapon house mask".into());
                    }
                }
                SuperWeaponPayloadPolicy::UnitDelivery {
                    types,
                    deferment,
                    action,
                    activation_sound,
                    ..
                } => {
                    if types.len() > LIMIT
                        || *deferment < 0
                        || !valid_name(action)
                        || !valid_reference(activation_sound, self.sound_source_hash, true)
                    {
                        return Err("invalid SuperWeapon delivery declaration".into());
                    }
                }
            }
        }
        let mut buildings = BTreeSet::new();
        for provider in &self.providers {
            if provider.object.category != ProductionObjectKind::Building
                || !buildings.insert(provider.object)
                || provider.additional.len() > LIMIT
            {
                return Err("invalid SuperWeapon provider declaration".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{API_VERSION, Context, Plan, Request};

    #[test]
    fn superweapon_resource_selection_is_pure_and_rejects_cross_domain_authority() {
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::SuperWeaponResource(SuperWeaponResourceContext {}),
            config: vec![1],
            state: vec![],
        };
        request.validate().unwrap();
        let mut plan = Plan::unchanged(&request);
        // An explicit stock selection is meaningful, unlike absent policy.
        plan.superweapon_resource = Some(SuperWeaponResourcePolicy::default());
        plan.validate_for(&request).unwrap();
        plan.config = Some(vec![2]);
        assert!(plan.validate_for(&request).is_err());
        plan.config = None;
        plan.super_reference_read = Some(SuperReferenceRead::Retain);
        assert!(plan.validate_for(&request).is_err());
        plan.super_reference_read = None;
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        request.context = Context::ConfigureCountryRead;
        assert!(plan.validate_for(&request).is_err());
        request.context = Context::SuperWeaponResource(SuperWeaponResourceContext {});
        request.state.push(1);
        assert!(request.validate().is_err());
    }

    #[test]
    fn super_reference_reads_require_the_reached_body_and_configuration_authority() {
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::ConfigureSuperRead(ConfigureSuperReadContext {
                read: 0,
                ordinal: 0,
                stored_id: "PayloadSpecial".into(),
                body_read: false,
                effective_type: None,
            }),
            config: vec![1],
            state: vec![],
        };
        request.validate().unwrap();
        let mut plan = Plan::unchanged(&request);
        plan.config = Some(vec![2]);
        plan.super_reference_read = Some(SuperReferenceRead::Retain);
        plan.validate_for(&request).unwrap();
        plan.super_reference_read = Some(SuperReferenceRead::WarheadFromCurrentSection);
        assert!(plan.validate_for(&request).is_err());
        let Context::ConfigureSuperRead(context) = &mut request.context else {
            unreachable!()
        };
        context.body_read = true;
        context.effective_type = Some("GenericWarhead".into());
        request.validate().unwrap();
        plan.validate_for(&request).unwrap();
        assert_eq!(
            serde_json::from_slice::<Plan>(&serde_json::to_vec(&plan).unwrap()).unwrap(),
            plan
        );
        let mut wrong = request.clone();
        wrong.context = Context::ConfigureCountryRead;
        assert!(plan.validate_for(&wrong).is_err());
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        request.state.push(1);
        assert!(request.validate().is_err());
    }
}
