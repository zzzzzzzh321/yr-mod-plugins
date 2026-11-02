//! Messages shared by the engine host and external gameplay components.
//! All persistent component data is returned to the host as bounded bytes.

mod armor;
mod configuration;
mod effects;
mod emp;
mod match_rules;
pub mod native_string;
mod production;
pub mod relationship;
mod resource;
pub mod timer;
pub use armor::{
    ArmorObjectKind, ArmorResourceContext, ArmorResourcePolicy, ArmorVersePolicy, BASE_ARMOR_NAMES,
};
pub use configuration::{ConfigurationBoundary, ConfigureBoundaryContext};
pub use effects::{
    EffectAction, EffectContext, EffectControl, EffectDescriptor, EffectEvent, EffectInstanceView,
    EffectPlan, EffectResourceContext, EffectResourceKind,
};
pub use emp::{
    EmpResourceContext, EmpResourcePolicy, EmpTypePolicy, EmpWarheadPolicy, NullableResourceName,
};
pub use match_rules::{
    CloakRuntimePolicy, EffectRuntimePolicy, MatchRulesContext, MatchRulesPolicy, SurvivalRule,
};
pub use production::{
    BASE_PRODUCTION_GROUPS, ProductionBuildLimitRule, ProductionGroup, ProductionObjectKind,
    ProductionObjectPolicy, ProductionPrerequisiteRule, ProductionReadPoint, ProductionReadSite,
    ProductionRegisteredType, ProductionRegistryPrefix, ProductionRegistryView,
    ProductionRequirement, ProductionResourceContext, ProductionResourcePolicy,
    ProductionSemantics, ProductionTypeRef,
};

pub use resource::{
    DepletedWeaponPolicy, InputPolicy, ReloadPolicy, TypeResourceContext, TypeResourcePolicy,
    WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY, WEAPON_USER_UNIT,
    WeaponResourceContext, WeaponResourcePolicy,
};

use serde::{Deserialize, Serialize};

pub const API_VERSION: u32 = 1;
pub const MAX_STATE_BYTES: usize = 16 * 1024;
pub const MAX_CONFIG_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_MESSAGE_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_OPERATIONS: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModuleManifest {
    pub api_version: u32,
    pub id: String,
    pub version: String,
    pub state_version: u32,
    pub config_version: u32,
    pub hooks: Vec<HookKind>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HookKind {
    Configure,
    Attach,
    Damage,
    Tick,
    WeaponResource,
    TypeResource,
    MatchRules,
    EffectResource,
    Effects,
    ArmorResource,
    ProductionResource,
    ConfigureBoundary,
    EmpResource,
}

impl HookKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Configure => "configure",
            Self::Attach => "attach",
            Self::Damage => "damage",
            Self::Tick => "tick",
            Self::WeaponResource => "weapon_resource",
            Self::TypeResource => "type_resource",
            Self::MatchRules => "match_rules",
            Self::EffectResource => "effect_resource",
            Self::Effects => "effects",
            Self::ArmorResource => "armor_resource",
            Self::ProductionResource => "production_resource",
            Self::ConfigureBoundary => "configure_boundary",
            Self::EmpResource => "emp_resource",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EntityContext {
    pub id: u64,
    pub type_name: String,
    pub owner: String,
    pub armor: String,
    pub health: i32,
    pub max_health: i32,
    pub alive: bool,
    pub in_limbo: bool,
    pub immobilized: bool,
    pub temporal: bool,
    pub being_warped_out: bool,
    pub powered: bool,
    pub chrono_warp_latch: bool,
    pub ai_frozen: bool,
    pub deactivated: bool,
    pub immune: bool,
    pub type_immune: bool,
    pub invulnerable: bool,
}

pub type EntityView = EntityContext;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ConfigureContext {
    pub pass: u64,
    pub source: String,
    pub final_pass: bool,
    #[serde(default)]
    pub object_types: Vec<String>,
    #[serde(default)]
    pub warheads: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttachContext {
    pub frame: i32,
    pub entity: EntityContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArmorVerses {
    pub armor: String,
    pub multiplier_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageContext {
    pub frame: i32,
    pub entity: EntityContext,
    pub attacker: Option<u64>,
    pub source_owner: Option<String>,
    pub source_is_ally: bool,
    pub source_is_owner: bool,
    pub target_is_neutral: bool,
    pub attacker_same_type_owner: bool,
    pub warhead_name: String,
    pub warhead_temporal: bool,
    pub warhead_psychedelic: bool,
    pub incoming: i32,
    pub distance_leptons: i32,
    pub ignore_defenses: bool,
    pub no_damage: bool,
    pub max_damage: i32,
    /// Native binary64 representation, not a JSON floating-point value.
    pub armor_multiplier_bits: u64,
    pub verses: Vec<ArmorVerses>,
    pub cell_spread_bits: u64,
    pub percent_at_max_bits: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TickContext {
    pub frame: i32,
    pub entity: EntityContext,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "data", rename_all = "snake_case")]
pub enum Context {
    Configure(ConfigureContext),
    Attach(AttachContext),
    Damage(DamageContext),
    Tick(TickContext),
    WeaponResource(WeaponResourceContext),
    TypeResource(TypeResourceContext),
    MatchRules(MatchRulesContext),
    EffectResource(EffectResourceContext),
    Effects(EffectContext),
    ArmorResource(ArmorResourceContext),
    ProductionResource(ProductionResourceContext),
    ConfigureBoundary(ConfigureBoundaryContext),
    EmpResource(EmpResourceContext),
}

impl Context {
    pub const fn kind(&self) -> HookKind {
        match self {
            Self::Configure(_) => HookKind::Configure,
            Self::Attach(_) => HookKind::Attach,
            Self::Damage(_) => HookKind::Damage,
            Self::Tick(_) => HookKind::Tick,
            Self::WeaponResource(_) => HookKind::WeaponResource,
            Self::TypeResource(_) => HookKind::TypeResource,
            Self::MatchRules(_) => HookKind::MatchRules,
            Self::EffectResource(_) => HookKind::EffectResource,
            Self::Effects(_) => HookKind::Effects,
            Self::ArmorResource(_) => HookKind::ArmorResource,
            Self::ProductionResource(_) => HookKind::ProductionResource,
            Self::ConfigureBoundary(_) => HookKind::ConfigureBoundary,
            Self::EmpResource(_) => HookKind::EmpResource,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub api_version: u32,
    pub context: Context,
    pub config: Vec<u8>,
    pub state: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Operation {
    AdjustHealth { entity: u64, delta: i32 },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Plan {
    pub api_version: u32,
    pub config: Option<Vec<u8>>,
    pub state: Vec<u8>,
    pub damage: Option<i32>,
    pub operations: Vec<Operation>,
    pub projection: Option<Projection>,
    /// Additive SDK v1 hook; omitted on the wire for older four-hook guests.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub weapon_resource: Option<WeaponResourcePolicy>,
    /// Optional immutable type policy; older guests omit this field.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub type_resource: Option<TypeResourcePolicy>,
    /// Optional API v1 capability, declared by the match_rules manifest hook.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub match_rules: Option<MatchRulesPolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_resource: Option<EffectDescriptor>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effects: Option<EffectPlan>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub armor_resource: Option<ArmorResourcePolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub production_resource: Option<ProductionResourcePolicy>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emp_resource: Option<EmpResourcePolicy>,
}

impl Plan {
    pub fn unchanged(request: &Request) -> Self {
        Self {
            api_version: API_VERSION,
            config: None,
            state: request.state.clone(),
            damage: None,
            operations: Vec::new(),
            projection: None,
            weapon_resource: None,
            type_resource: None,
            match_rules: None,
            effect_resource: None,
            effects: None,
            armor_resource: None,
            production_resource: None,
            emp_resource: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Projection {
    pub meters: Vec<Meter>,
    pub tint: Option<Tint>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Meter {
    pub id: String,
    pub current: i32,
    pub maximum: i32,
    pub visible: bool,
    pub active: bool,
    pub color_rgb: [u8; 3],
    pub pip_frame: u32,
    pub building_pip_frame: u32,
    pub building_empty_frame: u32,
    pub bracket_delta: i32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Tint {
    pub color_rgb: [u8; 3],
    pub intensity: i32,
    /// Generic relationship visibility: owner=1, allies=2, enemies=4, neutral=8.
    pub visible_to_houses: u8,
}

impl ModuleManifest {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_version != API_VERSION {
            return Err("unsupported Mod API version".into());
        }
        if self.id.is_empty()
            || self.id.len() > 64
            || !self
                .id
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b"._-".contains(&b))
        {
            return Err("invalid module identity".into());
        }
        if self.version.is_empty()
            || self.version.len() > 64
            || self.version.chars().any(char::is_control)
            || self.state_version == 0
            || self.config_version == 0
        {
            return Err("invalid module schema version".into());
        }
        let hooks: std::collections::BTreeSet<_> = self.hooks.iter().collect();
        if hooks.is_empty() || hooks.len() != self.hooks.len() {
            return Err("invalid module hook list".into());
        }
        if self.hooks.contains(&HookKind::EffectResource) != self.hooks.contains(&HookKind::Effects)
        {
            return Err("effect resources and their runtime must be declared together".into());
        }
        Ok(())
    }
}

impl Request {
    pub fn validate(&self) -> Result<(), String> {
        if self.api_version != API_VERSION {
            return Err("unsupported request API version".into());
        }
        if self.state.len() > MAX_STATE_BYTES || self.config.len() > MAX_CONFIG_BYTES {
            return Err("component input state/config exceeds its limit".into());
        }
        let user = match &self.context {
            Context::TypeResource(context) => Some(context.user),
            Context::Effects(context) => Some(context.user),
            Context::EmpResource(EmpResourceContext::Type { user, .. }) => Some(*user),
            _ => None,
        };
        if let Some(user) = user
            && !matches!(
                user,
                WEAPON_USER_UNIT
                    | WEAPON_USER_INFANTRY
                    | WEAPON_USER_BUILDING
                    | WEAPON_USER_AIRCRAFT
            )
        {
            return Err("invalid resource user category".into());
        }
        Ok(())
    }
    pub fn entity_id(&self) -> Option<u64> {
        match &self.context {
            Context::Configure(_) => None,
            Context::Attach(c) => Some(c.entity.id),
            Context::Damage(c) => Some(c.entity.id),
            Context::Tick(c) => Some(c.entity.id),
            Context::WeaponResource(_) => None,
            Context::TypeResource(_) => None,
            Context::MatchRules(_) => None,
            Context::EffectResource(_) => None,
            Context::Effects(context) => Some(context.entity.id),
            Context::ArmorResource(_) => None,
            Context::ProductionResource(_)
            | Context::ConfigureBoundary(_)
            | Context::EmpResource(_) => None,
        }
    }
}

impl Plan {
    /// Structural capability checks precede every caller's atomic commit.
    /// Semantic legality, lifecycle routing and final health clamping stay host-owned.
    pub fn validate_for(&self, request: &Request) -> Result<(), String> {
        if self.api_version != API_VERSION {
            return Err("unsupported plan API version".into());
        }
        if self.state.len() > MAX_STATE_BYTES
            || self
                .config
                .as_ref()
                .is_some_and(|c| c.len() > MAX_CONFIG_BYTES)
        {
            return Err("component output state/config exceeds its limit".into());
        }
        if self.config.is_some()
            && !matches!(
                request.context.kind(),
                HookKind::Configure | HookKind::ConfigureBoundary
            )
        {
            return Err("only configuration hooks may replace module configuration".into());
        }
        if self.damage.is_some() && request.context.kind() != HookKind::Damage {
            return Err("only damage may replace receiver damage".into());
        }
        if let Some(policy) = self.weapon_resource {
            let supported_users = WEAPON_USER_UNIT | WEAPON_USER_INFANTRY | WEAPON_USER_BUILDING;
            if request.context.kind() != HookKind::WeaponResource
                || policy.users == 0
                || policy.users & !supported_users != 0
            {
                return Err("invalid weapon resource policy or hook".into());
            }
        }
        if let Some(policy) = self.type_resource {
            let supported_users = WEAPON_USER_UNIT
                | WEAPON_USER_INFANTRY
                | WEAPON_USER_BUILDING
                | WEAPON_USER_AIRCRAFT;
            if request.context.kind() != HookKind::TypeResource
                || policy.users == 0
                || policy.users & !supported_users != 0
                || policy.reload.is_some() && policy.users & WEAPON_USER_AIRCRAFT != 0
                || policy.reload.is_none()
                    && policy.depleted_weapon.is_none()
                    && policy.input.is_none()
                    && policy.keep_alive.is_none()
                    && policy.cloak_allowed.is_none()
                || policy.depleted_weapon.is_some_and(|p| p.weapon_index < 0)
            {
                return Err("invalid type resource policy or hook".into());
            }
        }
        if matches!(
            request.context.kind(),
            HookKind::WeaponResource
                | HookKind::TypeResource
                | HookKind::MatchRules
                | HookKind::EffectResource
                | HookKind::ArmorResource
                | HookKind::ProductionResource
                | HookKind::ConfigureBoundary
                | HookKind::EmpResource
        ) && (!self.state.is_empty()
            || self.projection.is_some()
            || !self.operations.is_empty()
            || self.effects.is_some())
        {
            return Err("resource queries cannot write entity state or presentation".into());
        }
        if self.match_rules.is_some() && request.context.kind() != HookKind::MatchRules {
            return Err("only match_rules may select match-wide rules".into());
        }
        if self.effect_resource.is_some() && request.context.kind() != HookKind::EffectResource {
            return Err("only effect_resource may define an effect resource".into());
        }
        if let Some(policy) = &self.armor_resource {
            let Context::ArmorResource(context) = &request.context else {
                return Err("only armor_resource may define an armor resource".into());
            };
            policy.validate_for(context)?;
        }
        if let Some(policy) = &self.production_resource {
            let Context::ProductionResource(context) = &request.context else {
                return Err("only production_resource may define production resources".into());
            };
            policy.validate_for(context)?;
        }
        if let Some(policy) = &self.emp_resource {
            let Context::EmpResource(context) = &request.context else {
                return Err("only emp_resource may define EMP resources".into());
            };
            policy.validate_for(context)?;
        }
        if let Some(plan) = &self.effects {
            let Context::Effects(context) = &request.context else {
                return Err("only effects may mutate effect instances".into());
            };
            plan.validate(context)?;
        }
        if request.context.kind() == HookKind::Effects
            && (!self.state.is_empty() || self.projection.is_some() || !self.operations.is_empty())
        {
            return Err(
                "effect runtime uses the host instance store, not opaque entity state".into(),
            );
        }
        if self.operations.len() > MAX_OPERATIONS {
            return Err("component operation count exceeds its limit".into());
        }
        for op in &self.operations {
            match op {
                Operation::AdjustHealth { entity, .. } if Some(*entity) == request.entity_id() => {}
                _ => return Err("component operation targets another entity".into()),
            }
        }
        if let Some(projection) = &self.projection {
            if projection.meters.len() > 4 {
                return Err("component meter count exceeds its limit".into());
            }
            let mut ids = std::collections::BTreeSet::new();
            for meter in &projection.meters {
                if meter.id.is_empty()
                    || meter.id.len() > 64
                    || meter.id.chars().any(char::is_control)
                    || !ids.insert(&meter.id)
                    || meter.maximum < 0
                    || meter.visible && meter.maximum == 0
                {
                    return Err("invalid component meter".into());
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DamageKernelInput {
    pub damage: i32,
    pub cell_spread_bits: u64,
    pub percent_at_max_bits: u64,
    pub multiplier_bits: u64,
    pub distance_leptons: i32,
    pub no_damage: bool,
    pub max_damage: i32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn emp_resources_preserve_nullable_names_and_reject_simulation_writes() {
        let inherit = serde_json::to_vec(&NullableResourceName::Inherit).unwrap();
        let clear = serde_json::to_vec(&NullableResourceName::Clear).unwrap();
        assert_ne!(inherit, clear);
        assert_eq!(
            serde_json::from_slice::<NullableResourceName>(&clear).unwrap(),
            NullableResourceName::Clear
        );
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::EmpResource(EmpResourceContext::Type {
                user: WEAPON_USER_UNIT,
                name: "TANK".into(),
            }),
            config: vec![],
            state: vec![],
        };
        request.validate().unwrap();
        let mut plan = Plan::unchanged(&request);
        plan.emp_resource = Some(EmpResourcePolicy::Type {
            policy: EmpTypePolicy::default(),
        });
        plan.validate_for(&request).unwrap();
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        request.context = Context::EmpResource(EmpResourceContext::Warhead { name: "EMP".into() });
        assert!(
            plan.validate_for(&request).is_err(),
            "type policy cannot answer a warhead query"
        );
        request.context = Context::ConfigureBoundary(ConfigureBoundaryContext {
            boundary: ConfigurationBoundary::TypeReset,
        });
        assert!(
            plan.validate_for(&request).is_err(),
            "a configuration boundary cannot emit resources"
        );
        request.context = Context::EmpResource(EmpResourceContext::Type {
            user: 0,
            name: "TANK".into(),
        });
        assert!(request.validate().is_err());
    }

    #[test]
    fn production_resources_and_configuration_boundaries_reject_cross_domain_writes() {
        let mut groups = BASE_PRODUCTION_GROUPS
            .into_iter()
            .map(|name| ProductionGroup {
                name: name.into(),
                members: vec![],
                alternates: vec![],
            })
            .collect::<Vec<_>>();
        // Native source bytes are widened to characters; UTF-8 byte length
        // must not reject a valid31-source-byte retained identity.
        groups.push(ProductionGroup {
            name: "\u{e9}".repeat(31),
            members: vec![],
            alternates: vec![],
        });
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::ProductionResource(ProductionResourceContext::Catalog),
            config: vec![],
            state: vec![],
        };
        let mut plan = Plan::unchanged(&request);
        plan.production_resource = Some(ProductionResourcePolicy::Catalog {
            groups,
            semantics: ProductionSemantics::default(),
            ai_bypass_build_limit: [false; 3],
        });
        plan.validate_for(&request).unwrap();
        let Some(ProductionResourcePolicy::Catalog { groups, .. }) = &mut plan.production_resource
        else {
            unreachable!()
        };
        groups[0].members.push(ProductionRequirement::Group(99));
        assert!(plan.validate_for(&request).is_err());
        request.context = Context::ProductionResource(ProductionResourceContext::Object {
            category: ProductionObjectKind::Unit,
            name: "TANK".into(),
        });
        assert!(
            plan.validate_for(&request).is_err(),
            "Catalog cannot answer an Object query"
        );
        plan.production_resource = Some(ProductionResourcePolicy::Object {
            policy: ProductionObjectPolicy::default(),
        });
        plan.validate_for(&request).unwrap();
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        plan.production_resource = None;
        request.context = Context::ConfigureBoundary(ConfigureBoundaryContext {
            boundary: ConfigurationBoundary::TypeReset,
        });
        plan.config = Some(vec![]);
        plan.validate_for(&request).unwrap();
        plan.state.push(1);
        assert!(
            plan.validate_for(&request).is_err(),
            "configuration boundaries cannot write actor state"
        );
    }

    #[test]
    fn type_resource_queries_limit_aircraft_reload_and_reject_state_writes() {
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::TypeResource(TypeResourceContext {
                type_name: "TANK".into(),
                user: WEAPON_USER_UNIT,
            }),
            config: vec![],
            state: vec![],
        };
        let mut plan = Plan::unchanged(&request);
        plan.type_resource = Some(TypeResourcePolicy {
            users: WEAPON_USER_UNIT,
            reload: Some(ReloadPolicy {
                normal_amount: -2,
                empty_amount: 0,
            }),
            depleted_weapon: Some(DepletedWeaponPolicy {
                weapon_index: 1,
                threshold: 2,
            }),
            input: None,
            keep_alive: None,
            cloak_allowed: None,
        });
        plan.validate_for(&request).unwrap();
        plan.type_resource.as_mut().unwrap().users = WEAPON_USER_AIRCRAFT;
        assert!(
            plan.validate_for(&request).is_err(),
            "automatic type reload excludes aircraft"
        );
        plan.type_resource.as_mut().unwrap().reload = None;
        plan.type_resource.as_mut().unwrap().depleted_weapon = None;
        plan.type_resource.as_mut().unwrap().input = Some(InputPolicy {
            deny_manual_attack: true,
            ..Default::default()
        });
        request.context = Context::TypeResource(TypeResourceContext {
            type_name: "PLANE".into(),
            user: WEAPON_USER_AIRCRAFT,
        });
        plan.validate_for(&request).unwrap();
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        request.context = Context::WeaponResource(WeaponResourceContext {
            weapon_name: "GUN".into(),
        });
        assert!(plan.validate_for(&request).is_err());
    }

    #[test]
    fn resource_queries_reject_entity_writes_and_foreign_hook_policies() {
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::WeaponResource(WeaponResourceContext {
                weapon_name: "GUN".into(),
            }),
            config: vec![],
            state: vec![],
        };
        let mut plan = Plan::unchanged(&request);
        plan.weapon_resource = Some(WeaponResourcePolicy {
            cost: 2,
            reload_after_use: true,
            users: WEAPON_USER_UNIT,
            use_weapon_rof: false,
        });
        plan.validate_for(&request).unwrap();
        plan.weapon_resource.as_mut().unwrap().users = WEAPON_USER_AIRCRAFT;
        assert!(plan.validate_for(&request).is_err());
        plan.weapon_resource.as_mut().unwrap().users = WEAPON_USER_UNIT;
        plan.state.push(1);
        assert!(plan.validate_for(&request).is_err());
        plan.state.clear();
        request.context = Context::Configure(ConfigureContext {
            pass: 0,
            source: "root".into(),
            final_pass: true,
            object_types: vec![],
            warheads: vec![],
        });
        assert!(plan.validate_for(&request).is_err());
    }
}
