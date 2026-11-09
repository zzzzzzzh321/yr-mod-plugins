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

/// Common immutable targeting, independent of the selected effect handler.
/// Configuration owns physical-read retention; the host owns target search,
/// launch-site observations and all live state. No per-frame callback is added.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponCommonTargetingPolicy {
    /// Original AITargeting names in reader order (0..21).
    pub mode: u8,
    /// Effective preference: none=0, offensive=1, defensive=2.
    pub preference: u8,
    pub auto_fire: bool,
    pub manual_fire: bool,
    pub fire_into_shroud: bool,
    pub manual_target: u8,
    pub manual_house: u8,
    pub automatic_target: u8,
    pub automatic_house: u8,
    /// Retained native binary64 bits, including nonfinite authored values.
    /// A malformed physical read retains its previous value. The original
    /// prepared range gate and per-site predicate interpret these separately.
    pub range_minimum: u64,
    pub range_maximum: u64,
}

impl SuperWeaponCommonTargetingPolicy {
    pub fn validate(&self) -> Result<(), String> {
        if self.mode >= 21 || self.preference >= 3 {
            return Err("invalid common SuperWeapon targeting mode or preference".into());
        }
        if self.manual_house & !7 != 0 || self.automatic_house & !7 != 0 {
            return Err("invalid common SuperWeapon targeting house mask".into());
        }
        Ok(())
    }

    /// The old Generic ABI is a compatibility projection, not another source
    /// of effective ranges, masks or AI selection when common data is present.
    fn matches_legacy(&self, legacy: &SuperWeaponTargetingPolicy) -> bool {
        self.manual_target == legacy.manual_target
            && self.manual_house == legacy.manual_house
            && self.automatic_target == legacy.automatic_target
            && self.automatic_house == legacy.automatic_house
            && (self.mode == 8) == legacy.offensive_ai
            && self.fire_into_shroud == legacy.fire_into_shroud
            && self.range_minimum == legacy.range_minimum
            && self.range_maximum == legacy.range_maximum
    }
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

/// Exact common TechnoType pointers resolved at their physical Super read.
/// Keep order and duplicates: a stored name does not identify an allocation.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponInfluencePolicy {
    pub designators: Vec<crate::ProductionTypeRef>,
    pub any_designator: bool,
    pub inhibitors: Vec<crate::ProductionTypeRef>,
    pub any_inhibitor: bool,
}

/// Nullable per-Type ranges in cells. None uses the current Type's raw Sight;
/// zero and negative values are explicit overrides, not default sentinels.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TechnoInfluenceDeclaration {
    pub object: crate::ProductionTypeRef,
    pub designator_range: Option<i32>,
    pub inhibitor_range: Option<i32>,
}

/// Resolved fixed-EVAMD references. Clear is explicit silence; Inherit is not
/// permitted here. InsufficientFunds retains its existing declaration field.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SuperWeaponEvaPolicy {
    pub ready: crate::NullableResourceName,
    pub activated: crate::NullableResourceName,
    pub detected: crate::NullableResourceName,
    pub impatient: crate::NullableResourceName,
    pub select_target: crate::NullableResourceName,
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
    /// Historical absence preserves the host's previous EVA producers. A
    /// present policy retains silence and first-initialization independently.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eva: Option<SuperWeaponEvaPolicy>,
    pub presentation: SuperWeaponPresentationPolicy,
    pub payload: SuperWeaponPayloadPolicy,
    /// None preserves the historical payload-specific targeting ABI. A present
    /// declaration selects the common host target/AI owners for this definition.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub common_targeting: Option<SuperWeaponCommonTargetingPolicy>,
    /// None keeps historical external no-influence behavior. A declaration
    /// requires the explicit techno_influence domain on the resource batch.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub influence: Option<SuperWeaponInfluencePolicy>,
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
    /// None is historical domain absence; Some(empty) selects all-unset
    /// nullable ranges. Listed exact native slots override that unset base.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub techno_influence: Option<Vec<TechnoInfluenceDeclaration>>,
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
        if self.definitions.len() > LIMIT
            || self.providers.len() > LIMIT
            || self
                .techno_influence
                .as_ref()
                .is_some_and(|types| types.len() > LIMIT)
        {
            return Err("SuperWeapon declaration limit exceeded".into());
        }
        let mut influence_types = BTreeSet::new();
        for declaration in self.techno_influence.iter().flatten() {
            if !influence_types.insert(declaration.object) {
                return Err("duplicate Techno influence declaration".into());
            }
        }
        let mut ordinals = BTreeSet::new();
        for definition in &self.definitions {
            if let Some(eva) = &definition.eva {
                for reference in [
                    &eva.ready,
                    &eva.activated,
                    &eva.detected,
                    &eva.impatient,
                    &eva.select_target,
                ] {
                    if matches!(reference, NullableResourceName::Inherit)
                        || !valid_reference(reference, self.eva_source_hash, true)
                    {
                        return Err("invalid resolved SuperWeapon EVA reference".into());
                    }
                }
            }
            if let Some(targeting) = &definition.common_targeting {
                targeting.validate()?;
            }
            if let Some(influence) = &definition.influence {
                if self.techno_influence.is_none()
                    || influence.designators.len() > LIMIT
                    || influence.inhibitors.len() > LIMIT
                {
                    return Err("invalid SuperWeapon influence domain".into());
                }
            }
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
                    if definition
                        .common_targeting
                        .as_ref()
                        .is_some_and(|common| !common.matches_legacy(targeting))
                    {
                        return Err(
                            "conflicting common and historical SuperWeapon targeting".into()
                        );
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

    fn definition() -> SuperWeaponDeclaration {
        SuperWeaponDeclaration {
            ordinal: 0,
            stored_id: "Atomic".into(),
            availability: SuperWeaponAvailabilityPolicy {
                allow_player: true,
                allow_ai: true,
                required_houses: u32::MAX,
                forbidden_houses: 0,
                shots: -1,
                auxiliary: vec![],
                negative: vec![],
            },
            charge: SuperWeaponChargePolicy::default(),
            constraints: 4,
            use_ai_targeting: false,
            money_amount: 0,
            insufficient_funds_eva: crate::NullableResourceName::Inherit,
            eva: None,
            presentation: SuperWeaponPresentationPolicy {
                animation: crate::NullableResourceName::Inherit,
                height: 0,
                visibility: 7,
                sound: crate::NullableResourceName::Inherit,
            },
            payload: SuperWeaponPayloadPolicy::Retail,
            common_targeting: None,
            influence: None,
        }
    }

    #[test]
    fn influence_preserves_old_absence_and_checks_exact_domain_members() {
        let mut policy = SuperWeaponResourcePolicy {
            definitions: vec![definition()],
            ..SuperWeaponResourcePolicy::default()
        };
        let old = serde_json::to_value(&policy).unwrap();
        assert!(old.get("techno_influence").is_none());
        assert!(old["definitions"][0].get("influence").is_none());
        let restored: SuperWeaponResourcePolicy = serde_json::from_value(old).unwrap();
        assert!(restored.techno_influence.is_none());
        assert!(restored.definitions[0].influence.is_none());
        restored.validate().unwrap();

        let unit = crate::ProductionTypeRef {
            category: crate::ProductionObjectKind::Unit,
            ordinal: 0,
        };
        policy.definitions[0].influence = Some(SuperWeaponInfluencePolicy {
            designators: vec![unit, unit],
            ..SuperWeaponInfluencePolicy::default()
        });
        assert!(
            policy.validate().is_err(),
            "new lists require an explicit range domain"
        );
        policy.techno_influence = Some(vec![]);
        policy.validate().unwrap();
        let range = TechnoInfluenceDeclaration {
            object: unit,
            designator_range: Some(0),
            inhibitor_range: Some(-1),
        };
        policy.techno_influence = Some(vec![range.clone(), range]);
        assert!(
            policy.validate().is_err(),
            "final ranges have one owner per native slot"
        );
        policy.techno_influence.as_mut().unwrap()[1].object.category =
            crate::ProductionObjectKind::Infantry;
        policy.validate().unwrap();
        let bytes = serde_json::to_vec(&policy).unwrap();
        assert_eq!(
            serde_json::from_slice::<SuperWeaponResourcePolicy>(&bytes).unwrap(),
            policy
        );
    }

    #[test]
    fn common_targeting_keeps_old_absence_and_rejects_conflicting_authority() {
        let mut policy = SuperWeaponResourcePolicy::default();
        policy.definitions.push(definition());
        let old = serde_json::to_value(&policy).unwrap();
        assert!(old["definitions"][0].get("common_targeting").is_none());
        let restored: SuperWeaponResourcePolicy = serde_json::from_value(old).unwrap();
        assert!(restored.definitions[0].common_targeting.is_none());
        restored.validate().unwrap();

        let common = SuperWeaponCommonTargetingPolicy {
            mode: 8,
            preference: 0,
            auto_fire: false,
            manual_fire: true,
            fire_into_shroud: true,
            manual_target: 0,
            manual_house: 0,
            automatic_target: 0x38,
            automatic_house: 4,
            range_minimum: f64::NEG_INFINITY.to_bits(),
            range_maximum: f64::INFINITY.to_bits(),
        };
        policy.definitions[0].payload = SuperWeaponPayloadPolicy::GenericWarhead {
            damage: 1,
            targeting: SuperWeaponTargetingPolicy {
                manual_target: 0,
                manual_house: 0,
                automatic_target: 0x38,
                automatic_house: 4,
                offensive_ai: true,
                fire_into_shroud: true,
                range_minimum: common.range_minimum,
                range_maximum: common.range_maximum,
            },
        };
        policy.definitions[0].common_targeting = Some(common.clone());
        policy.validate().unwrap();
        let current = policy.definitions[0].common_targeting.as_mut().unwrap();
        current.mode = 21;
        assert!(policy.validate().is_err());
        policy.definitions[0].common_targeting = Some(common.clone());
        policy.definitions[0]
            .common_targeting
            .as_mut()
            .unwrap()
            .preference = 3;
        assert!(policy.validate().is_err());
        policy.definitions[0].common_targeting = Some(common);
        policy.definitions[0]
            .common_targeting
            .as_mut()
            .unwrap()
            .range_maximum = 5.0_f64.to_bits();
        assert!(policy.validate().unwrap_err().contains("conflicting"));
    }

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
