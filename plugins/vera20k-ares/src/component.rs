//! Pure resource recipes precede INI loading. Gameplay composition is then
//! frozen per match; simulation never reads live INI.
use crate::{
    features::{
        buildings::foundation,
        defense::{armor, shield},
        factions,
        houses::survival,
        presentation::{cameos, promotion_sounds},
        production,
        projectiles::{airburst, ballistic_scatter},
        resources::{startup, loading_theme},
        superweapons,
        units::{attach_effect, cloak, emp, manual_control, spotlight, type_conversion},
        weapons::{ammo_cost, empty_weapon, firing, projectile_range, reload},
    },
    support::{ConfigView, Host},
};
use serde::{Deserialize, Serialize};
use vera_mod_api::{API_VERSION, Context, HookKind, ModuleManifest, Plan, Projection, Request};

/// The only authority for feature activation. Physical Rules passes retain
/// previous values through the native bool reader; the final pass freezes them.
#[derive(Default, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
struct Features {
    shield: bool,
    overload: bool,
    ares_ammo: bool,
    ares_reload: bool,
    ares_no_ammo_weapon: bool,
    ares_manual_control: bool,
    ares_keep_alive: bool,
    ares_attach_effect: bool,
    ares_armor: bool,
    ares_production: bool,
    ares_emp: bool,
    ares_spotlight: bool,
    ares_type_conversion: bool,
    ares_foundation: bool,
    ares_faction: bool,
}

impl Features {
    /// Installed Ares hooks run without a second INI opt-in (for example
    /// GetROF_BuildingHack10047A60 at6FCFA4 and NoManualFire1004A460 at700536).
    /// Keep serde's historical Default separate: old frozen configuration
    /// must never acquire newly enabled features merely by being decoded.
    fn for_new_match() -> Self {
        Self {
            ares_ammo: true,
            ares_reload: true,
            ares_no_ammo_weapon: true,
            ares_manual_control: true,
            ares_keep_alive: true,
            ares_attach_effect: true,
            ares_armor: true,
            ares_production: true,
            ares_emp: true,
            ares_spotlight: true,
            ares_type_conversion: true,
            ares_foundation: true,
            ares_faction: true,
            ..Self::default()
        }
    }

    fn apply_pass<H: Host>(&mut self, view: &ConfigView<'_, H>) {
        let section = view.section_or_empty("VeraFeatures");
        self.shield = section.read_bool("Shield", self.shield);
        self.overload = section.read_bool("Overload", self.overload);
        self.ares_ammo = section.read_bool("AresAmmo", self.ares_ammo);
        self.ares_reload = section.read_bool("AresReload", self.ares_reload);
        self.ares_no_ammo_weapon = section.read_bool("AresNoAmmoWeapon", self.ares_no_ammo_weapon);
        self.ares_manual_control = section.read_bool("AresManualControl", self.ares_manual_control);
        self.ares_keep_alive = section.read_bool("AresKeepAlive", self.ares_keep_alive);
        self.ares_attach_effect = section.read_bool("AresAttachEffect", self.ares_attach_effect);
        self.ares_armor = section.read_bool("AresArmor", self.ares_armor);
        self.ares_production = section.read_bool("AresProduction", self.ares_production);
        self.ares_emp = section.read_bool("AresEMP", self.ares_emp);
        self.ares_spotlight = section.read_bool("AresSpotlight", self.ares_spotlight);
        self.ares_type_conversion =
            section.read_bool("AresTypeConversion", self.ares_type_conversion);
        self.ares_foundation = section.read_bool("AresFoundation", self.ares_foundation);
        // AresFaction is read only at the actual Country pre-body frontier.
        // Replaying a completed pass must not reselect an earlier reader.
    }

    fn validate(&self) -> Result<(), String> {
        if self.overload && !self.shield {
            return Err("[VeraFeatures] Overload=yes requires Shield=yes".into());
        }
        Ok(())
    }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    #[serde(default)]
    features: Features,
    #[serde(flatten)]
    shield: shield::Configuration,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    weapon_instances: Option<vera_mod_api::WeaponRegistryView>,
    ammunition: ammo_cost::Ammunition,
    reload: reload::Configuration,
    empty_weapon: empty_weapon::Configuration,
    #[serde(default)]
    manual_control: manual_control::Configuration,
    #[serde(default)]
    survival: survival::Configuration,
    #[serde(default)]
    attach_effect: attach_effect::Configuration,
    #[serde(default)]
    cloak: cloak::Configuration,
    #[serde(default)]
    armor: armor::Configuration,
    #[serde(default)]
    production: production::Configuration,
    #[serde(default)]
    emp: emp::Configuration,
    #[serde(default)]
    spotlight: spotlight::Configuration,
    #[serde(default)]
    type_conversion: type_conversion::Configuration,
    #[serde(default)]
    foundation: foundation::Configuration,
    #[serde(default)]
    factions: factions::Configuration,
    #[serde(default)]
    cameos: cameos::Configuration,
    #[serde(default)]
    promotion_sounds: promotion_sounds::Configuration,
    // Historical nonempty configuration without this domain stays unowned.
    #[serde(default)]
    superweapons: Option<superweapons::Configuration>,
    // Old nonempty frozen configuration does not acquire new firing rules.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    firing: Option<firing::Configuration>,
    // Range ownership is separate from the older firing configuration cohort.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    projectile_range: Option<projectile_range::Configuration>,
    // Historical configuration keeps the earlier unsupported Airburst tail.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    airburst: Option<airburst::Configuration>,
    // Scatter ownership is independent of Airburst and all weapon feature flags.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    ballistic_scatter: Option<ballistic_scatter::Configuration>,
    // An older frozen cohort keeps its original fragment constructor semantics.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    shrapnel_weapon_assignment: Option<bool>,
}

impl Configuration {
    fn validate_resource_domains(&self) -> Result<(), String> {
        if let Some(registry) = &self.weapon_instances {
            registry.validate()?;
            let count = registry.weapons.len();
            self.ammunition.validate_instances(count)?;
            if let Some(firing) = &self.firing {
                firing.validate_instances(count)?;
            }
            if let Some(range) = &self.projectile_range {
                range.validate_instances(count)?;
            }
        }
        if self.projectile_range.is_some() && self.firing.is_none() {
            return Err("projectile range configuration requires the firing domain".into());
        }
        if self.airburst.is_some() && self.projectile_range.is_none() {
            return Err("airburst configuration requires the projectile range domain".into());
        }
        if self.shrapnel_weapon_assignment == Some(true) && self.projectile_range.is_none() {
            return Err("shrapnel weapon assignment requires the projectile range domain".into());
        }
        Ok(())
    }

    fn weapon_identity(&self, name: &str) -> Result<Option<vera_mod_api::WeaponTypeRef>, String> {
        let Some(registry) = self
            .weapon_instances
            .as_ref()
            .filter(|registry| !registry.weapons.is_empty())
        else {
            return Ok(None);
        };
        let mut matches = registry
            .weapons
            .iter()
            .filter(|weapon| weapon.name.eq_ignore_ascii_case(name));
        let first = matches.next().ok_or("unknown frozen WeaponType name")?;
        if matches.next().is_some() {
            return Err("legacy weapon query cannot select a colliding WeaponType instance".into());
        }
        Ok(Some(first.identity))
    }

    fn weapon_resource_policy(
        &self,
        name: &str,
        identity: Option<vera_mod_api::WeaponTypeRef>,
    ) -> Result<Option<vera_mod_api::WeaponResourcePolicy>, String> {
        if !self.features.ares_ammo {
            return Ok(None);
        }
        Ok(Some(match identity {
            Some(identity) => self.ammunition.instance_policy(identity)?,
            None => self.ammunition.policy(name),
        }))
    }

    fn weapon_firing_policy(
        &self,
        context: &vera_mod_api::WeaponFiringResourceContext,
        identity: Option<vera_mod_api::WeaponTypeRef>,
    ) -> Result<Option<vera_mod_api::WeaponFiringPolicy>, String> {
        self.firing
            .as_ref()
            .map(|firing| {
                let mut policy = match identity {
                    Some(identity) => firing.instance_policy(identity, context)?,
                    None => firing.policy(context),
                };
                policy.projectile_range = self
                    .projectile_range
                    .as_ref()
                    .map(|range| match identity {
                        Some(identity) => range.instance_range(identity),
                        None => Ok(range.initial_range(&context.weapon_name)),
                    })
                    .transpose()?;
                Ok(policy)
            })
            .transpose()
    }
}

pub fn manifest() -> ModuleManifest {
    ModuleManifest {
        api_version: API_VERSION,
        id: "vera.reference-gameplay".into(),
        version: "0.37.0".into(),
        state_version: 1,
        config_version: 26,
        hooks: vec![
            HookKind::Configure,
            HookKind::Attach,
            HookKind::Damage,
            HookKind::Tick,
            HookKind::WeaponResource,
            HookKind::WeaponFiringResource,
            HookKind::TypeResource,
            HookKind::MatchRules,
            HookKind::EffectResource,
            HookKind::Effects,
            HookKind::ArmorResource,
            HookKind::ProductionResource,
            HookKind::ConfigureBoundary,
            HookKind::EmpResource,
            HookKind::FoundationResource,
            HookKind::StartupResources,
            HookKind::EvaCatalog,
            HookKind::LongListReaders,
            HookKind::SpawnIdentity,
            HookKind::PrismSupportDefaults,
            HookKind::SoundSampleAdmission,
            HookKind::ConfigureSuperRead,
            HookKind::SuperWeaponResource,
            HookKind::ConfigureCountryRead,
            HookKind::FactionResource,
            HookKind::ConfigureFactionPass,
            HookKind::LoadingThemeResource,
            HookKind::CameoResource,
            HookKind::TypeSoundResource,
            HookKind::ProjectileResource,
            HookKind::WeaponInstanceResource,
            HookKind::MasterTypeLists,
        ],
    }
}

fn decode_config(bytes: &[u8]) -> Result<Configuration, String> {
    if bytes.is_empty() {
        Ok(Configuration {
            features: Features::for_new_match(),
            superweapons: Some(superweapons::Configuration::for_new_match()),
            firing: Some(firing::Configuration::default()),
            projectile_range: Some(projectile_range::Configuration::default()),
            airburst: Some(airburst::Configuration::default()),
            ballistic_scatter: Some(ballistic_scatter::Configuration::default()),
            shrapnel_weapon_assignment: Some(true),
            weapon_instances: Some(vera_mod_api::WeaponRegistryView::default()),
            ..Configuration::default()
        })
    } else {
        serde_json::from_slice(bytes).map_err(|e| format!("component configuration: {e}"))
    }
}

pub(crate) fn evaluate<H: Host>(request: &Request, host: &H) -> Result<Plan, String> {
    request.validate()?;
    if let Context::MasterTypeLists = &request.context {
        // Ares RulesClass::BeforeTypeData679A15 registers WeaponTypes with
        // the factory before any SuperWeapon or Weapon body is read. This
        // installed declaration cannot depend on later match configuration.
        let mut plan = Plan::unchanged(request);
        plan.master_type_lists = Some(vera_mod_api::MasterTypeListsPolicy { weapon_types: true });
        return Ok(plan);
    }
    if let Context::SoundSampleAdmission = &request.context {
        // Fixed SOUNDMD sample admission is selected before configuration.
        // The host keeps the actual source identity and sample catalog.
        let mut plan = Plan::unchanged(request);
        plan.sound_sample_admission = Some(true);
        return Ok(plan);
    }
    if let Context::PrismSupportDefaults = &request.context {
        // Select the installed Prism reader default before Rules loading;
        // this declaration does not consume match configuration or Host data.
        let mut plan = Plan::unchanged(request);
        plan.prism_support_defaults = Some(true);
        return Ok(plan);
    }
    if let Context::SpawnIdentity = &request.context {
        // External launch identity admission is an installed startup rule,
        // independent of the later Country/faction configuration selection.
        let mut plan = Plan::unchanged(request);
        plan.spawn_identity = Some(vera_mod_api::SpawnIdentityPolicy {
            extended_countries: true,
            campaign_side_byte: true,
        });
        return Ok(plan);
    }
    if let Context::LongListReaders = &request.context {
        // AR247 is an installed reader fix, independent of gameplay flags.
        // Select before decoding configuration or asking Host for any INI.
        let mut plan = Plan::unchanged(request);
        plan.long_list_readers = Some(vera_mod_api::LongListReadersPolicy::Extended2048);
        return Ok(plan);
    }
    if let Context::EvaCatalog = &request.context {
        let mut plan = Plan::unchanged(request);
        plan.eva_catalog = Some(vera_mod_api::EvaCatalogPolicy::RegisteredVoiceColumns);
        return Ok(plan);
    }
    if let Context::StartupResources(context) = &request.context {
        // Startup recipes are installation declarations, not match features.
        // They must not decode frozen configuration or ask Host for the INIs
        // whose source composition/catalogs this query is about to select.
        let mut plan = Plan::unchanged(request);
        plan.startup_resources = Some(startup::recipe(context));
        return Ok(plan);
    }
    let mut config = decode_config(&request.config)?;
    if !matches!(&request.context, Context::ConfigureBoundary(_)) {
        config.validate_resource_domains()?;
    }
    if let Context::ConfigureSuperRead(context) = &request.context {
        let view = ConfigView::new(host);
        let mut plan = Plan::unchanged(request);
        if let Some(superweapons) = config.superweapons.as_mut() {
            plan.super_reference_read = Some(superweapons.configure_read(context, &view)?);
            view.finish()?;
            plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
        }
        return Ok(plan);
    }
    if let Context::LoadingThemeResource(context) = &request.context {
        let view = ConfigView::native_only(host);
        let mut plan = Plan::unchanged(request);
        plan.loading_theme = Some(loading_theme::query(
            &view,
            context,
            config.features.ares_faction,
        ));
        view.finish()?;
        return Ok(plan);
    }
    if let Context::ConfigureFactionPass = &request.context {
        let view = ConfigView::new(host);
        config.factions.configure_physical(&view)?;
        view.finish()?;
        let mut plan = Plan::unchanged(request);
        plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
        return Ok(plan);
    }
    if let Context::ConfigureCountryRead = &request.context {
        let view = ConfigView::new(host);
        config.features.ares_faction = view
            .section_or_empty("VeraFeatures")
            .read_bool("AresFaction", config.features.ares_faction);
        view.finish()?;
        let mut plan = Plan::unchanged(request);
        plan.country_read = Some(if config.features.ares_faction {
            vera_mod_api::CountryReadPolicy::ExistingOnly2048
        } else {
            vera_mod_api::CountryReadPolicy::RetailAllocate128
        });
        plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
        return Ok(plan);
    }
    if let Context::ConfigureBoundary(context) = &request.context {
        // Fresh configuration boundaries precede feature validation. Saved
        // programs keep their exact frozen bytes and never replay this path.
        // ScenarioClear10004EF1 recreates RulesExt while retained Side/Country
        // objects still reach E_multi's second primary read. Their one-shot
        // parent history dies only with the actual TypeReset6686C0 family.
        // Temporarily retain cameo configuration until the common TypeReset.
        // Native ClearScene685659 -> Ares10004D90 also clears SWTypeExt at
        // 10004F25; this is not a claim that its old surfaces survive Clear.
        // The formal cameo binder consumes a completed scenario after reset.
        // Pending promotion declarations use the same completed-scenario
        // boundary. TypeReset clears them but retains the fixed sound source.
        let mut promotion_sounds = std::mem::take(&mut config.promotion_sounds);
        let (production, emp, factions, cameos) = match context.boundary {
            vera_mod_api::ConfigurationBoundary::ScenarioClear => (
                production::Configuration::default(),
                emp::Configuration::default(),
                std::mem::take(&mut config.factions),
                std::mem::take(&mut config.cameos),
            ),
            vera_mod_api::ConfigurationBoundary::TypeReset => {
                promotion_sounds.reset_types();
                let mut retained = std::mem::take(&mut config.production);
                retained.reset_types();
                let mut retained_emp = std::mem::take(&mut config.emp);
                retained_emp.reset_types();
                (
                    retained,
                    retained_emp,
                    factions::Configuration::default(),
                    cameos::Configuration::default(),
                )
            }
        };
        config = Configuration {
            features: Features::for_new_match(),
            production,
            emp,
            factions,
            cameos,
            promotion_sounds,
            // Native SWTypeExt state is recreated at these real boundaries.
            // Only subsequent ConfigureSuperRead visits initialize each Type.
            superweapons: Some(superweapons::Configuration::for_new_match()),
            firing: Some(firing::Configuration::default()),
            projectile_range: Some(projectile_range::Configuration::default()),
            airburst: Some(airburst::Configuration::default()),
            ballistic_scatter: Some(ballistic_scatter::Configuration::default()),
            shrapnel_weapon_assignment: Some(true),
            weapon_instances: Some(vera_mod_api::WeaponRegistryView::default()),
            ..Configuration::default()
        };
        let mut plan = Plan::unchanged(request);
        plan.state.clear();
        plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
        return Ok(plan);
    }
    if let Context::Configure(context) = &request.context {
        let view = ConfigView::new(host);
        config.features.apply_pass(&view);
        // Every physical pass owns its registered-reference read point, even
        // while disabled. Finalize only constrains the retained directories.
        config.attach_effect.configure(context, &view);
        config.cloak.configure(context, &view);
        config.type_conversion.configure(context, &view);
        config.foundation.configure(context, &view)?;
        config.factions.configure(context, &view)?;
        config.cameos.configure(context, &view)?;
        if let Some(superweapons) = config.superweapons.as_mut() {
            superweapons.configure(context, &view)?;
        }
        // Armor identity and inheritance also belong to each reached physical
        // read, including layers before the module is enabled.
        config.armor.configure(context, &view);
        if !context.final_pass {
            // Resource domains consume the same read-only physical input; do not
            // transfer/deserialize its full catalog twice in one Rules pass.
            let physical = view.rules_registry_view()?;
            config.production.configure(&view, &physical)?;
            config.emp.configure(&view, &physical);
            config.spotlight.configure(&view, &physical)?;
            config.promotion_sounds.configure(&view, &physical)?;
            if let Some(registry) = config.weapon_instances.as_mut() {
                *registry = view.weapon_registry_view()?;
                config.ammunition.configure_instances(&view, registry)?;
                if let Some(firing) = config.firing.as_mut() {
                    firing.configure_instances(&view, registry)?;
                }
                if let Some(range) = config.projectile_range.as_mut() {
                    range.configure_instances(&view, registry)?;
                }
            } else {
                // Historical nonempty configuration keeps its original cohort.
                if let Some(firing) = config.firing.as_mut() {
                    firing.configure(host)?;
                }
                if let Some(range) = config.projectile_range.as_mut() {
                    range.configure(host)?;
                }
            }
            if let Some(airburst) = config.airburst.as_mut() {
                airburst.configure(host)?;
            }
            if let Some(scatter) = config.ballistic_scatter.as_mut() {
                scatter.configure(host)?;
            }
        }
        if context.final_pass {
            config.spotlight.finalize(context);
            config.features.validate()?;
            if config.features.ares_ammo && config.weapon_instances.is_none() {
                config.ammunition.configure(host, &view);
            }
            if config.features.ares_reload {
                config.reload.configure(context, &view);
            }
            if config.features.ares_no_ammo_weapon {
                config.empty_weapon.configure(context, &view)?;
            }
            if config.features.ares_manual_control {
                config.manual_control.configure(context, &view);
            }
            if config.features.ares_keep_alive {
                config.survival.configure(context, &view);
            }
        }
        config.validate_resource_domains()?;
        // Always accumulate shield definitions: a later layer may enable them.
        config
            .shield
            .configure(context, &view, config.features.overload);
        view.finish()?;
        let mut plan = Plan::unchanged(request);
        plan.config = Some(serde_json::to_vec(&config).map_err(|e| e.to_string())?);
        return Ok(plan);
    }
    config.features.validate()?;
    match &request.context {
        Context::EvaCatalog
        | Context::LongListReaders
        | Context::SpawnIdentity
        | Context::PrismSupportDefaults
        | Context::SoundSampleAdmission
        | Context::ConfigureSuperRead(_)
        | Context::ConfigureCountryRead
        | Context::ConfigureFactionPass => {
            unreachable!("handled before feature validation")
        }
        Context::FactionResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.faction_resource =
                Some(config.factions.query(context, config.features.ares_faction));
            Ok(plan)
        }
        Context::SuperWeaponResource(_) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.superweapon_resource = config
                .superweapons
                .as_ref()
                .map(|value| value.query())
                .transpose()?;
            Ok(plan)
        }
        Context::ConfigureBoundary(_) => unreachable!("handled before feature validation"),
        Context::CameoResource(_) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.cameo_resource = Some(config.cameos.query());
            Ok(plan)
        }
        Context::TypeSoundResource(_) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.type_sound_resource = Some(config.promotion_sounds.query());
            Ok(plan)
        }
        Context::FoundationResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.foundation_resource = Some(
                config
                    .foundation
                    .query(context, config.features.ares_foundation),
            );
            Ok(plan)
        }
        Context::EmpResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.emp_resource = Some(config.emp.query(context, config.features.ares_emp));
            Ok(plan)
        }
        Context::ProductionResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            if config.features.ares_production {
                plan.production_resource = Some(config.production.query(context));
            }
            Ok(plan)
        }
        Context::ArmorResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            if config.features.ares_armor {
                plan.armor_resource = Some(config.armor.query(context));
            }
            Ok(plan)
        }
        Context::MatchRules(_) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.match_rules = Some(vera_mod_api::MatchRulesPolicy {
                survival: if config.features.ares_keep_alive {
                    vera_mod_api::SurvivalRule::TrackedPresence
                } else {
                    vera_mod_api::SurvivalRule::Retail
                },
                effects: Some(vera_mod_api::EffectRuntimePolicy {
                    apply_rof: config.features.ares_attach_effect,
                    allow_aircraft_speed_crates: config.features.ares_attach_effect,
                    allow_unarmed_firepower_crates: config.features.ares_attach_effect,
                    cloak_crate_uses_cloakability: config.features.ares_attach_effect,
                    cloak: vera_mod_api::CloakRuntimePolicy {
                        use_motion_query: config.features.ares_attach_effect,
                        motion_blocks_raw_ability: config.features.ares_attach_effect,
                        offline_blocks_ability: config.features.ares_attach_effect,
                        blockers_apply_to_rank: config.features.ares_attach_effect,
                        check_type_permission: config.features.ares_attach_effect,
                        deny_parachuted: config.features.ares_attach_effect,
                        check_recloak_before_weapon: config.features.ares_attach_effect,
                        allow_active_ability_above_ground: config.features.ares_attach_effect,
                    },
                }),
                script_type_conversion: Some(config.features.ares_type_conversion),
                extended_ai_side_tokens: Some(config.features.ares_faction),
                registered_side_ui: Some(config.features.ares_faction),
                // Installed presentation fixes do not require an INI opt-in;
                // retained ART requests and palette assets remain host-owned.
                building_presentation: Some(vera_mod_api::BuildingPresentationPolicy {
                    independent_palette_groups: true,
                    destroy_anim_house_fallback: true,
                }),
                // The host retains the Anim damage owner and ART reference;
                // these installed fixes do not depend on feature INI flags.
                anim_damage: Some(vera_mod_api::AnimDamagePolicy {
                    use_source_house: true,
                    prefer_art_warhead: true,
                }),
                // Ordinary damage shares the frozen WH relationship fields;
                // disabling the EMP status mechanism does not disable this gate.
                warhead_house_filter: Some(true),
                // An old frozen cohort keeps its original unsupported tail.
                airburst_tail: Some(if config.airburst.is_some() {
                    vera_mod_api::AirburstTailPolicy::AresReplacement
                } else {
                    vera_mod_api::AirburstTailPolicy::UnsupportedReplacement
                }),
                projectile_fuel: config.projectile_range.as_ref().map(|_| true),
                // BulletClass_Shrapnel_WeaponType1/2 (Ares1001ABF0)
                // installs the actual Weapon without a CreateBullet budget copy.
                shrapnel_weapon_assignment: config.shrapnel_weapon_assignment,
            });
            Ok(plan)
        }
        Context::EffectResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            if config.features.ares_attach_effect {
                plan.effect_resource = attach_effect::resource(context, &config.attach_effect);
            }
            Ok(plan)
        }
        Context::Effects(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.effects = Some(if config.features.ares_attach_effect {
                attach_effect::evaluate(context, &config.attach_effect)
            } else {
                vera_mod_api::EffectPlan::unchanged(context)
            });
            Ok(plan)
        }
        Context::ProjectileResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.projectile_resource = config
                .airburst
                .as_ref()
                .map(|airburst| airburst.query(&context.projectile_name));
            if let Some(scatter) = config.ballistic_scatter.as_ref() {
                plan.projectile_resource
                    .get_or_insert_with(Default::default)
                    .ballistic_scatter = Some(scatter.query(&context.projectile_name));
            }
            Ok(plan)
        }
        Context::WeaponInstanceResource(context) => {
            let mut plan = Plan::unchanged(request);
            if let Some(registry) = &config.weapon_instances {
                let registered = registry
                    .weapons
                    .get(context.identity.ordinal as usize)
                    .ok_or("unknown frozen WeaponType ordinal")?;
                if registered.identity != context.identity || registered.name != context.weapon_name
                {
                    return Err("WeaponType instance query differs from frozen identity".into());
                }
                let firing_context = vera_mod_api::WeaponFiringResourceContext {
                    weapon_name: context.weapon_name.clone(),
                    animation_count: context.animation_count,
                    is_sonic: context.is_sonic,
                    use_fire_particles: context.use_fire_particles,
                };
                plan.weapon_instance_resource = Some(vera_mod_api::WeaponInstanceResourcePolicy {
                    resource: config
                        .weapon_resource_policy(&context.weapon_name, Some(context.identity))?,
                    firing: config.weapon_firing_policy(&firing_context, Some(context.identity))?,
                });
            }
            Ok(plan)
        }
        Context::WeaponFiringResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.weapon_firing_resource = config
                .weapon_firing_policy(context, config.weapon_identity(&context.weapon_name)?)?;
            Ok(plan)
        }
        Context::WeaponResource(context) => {
            let mut plan = Plan::unchanged(request);
            plan.weapon_resource = config.weapon_resource_policy(
                &context.weapon_name,
                config.weapon_identity(&context.weapon_name)?,
            )?;
            Ok(plan)
        }
        Context::TypeResource(context) => {
            let mut plan = Plan::unchanged(request);
            let reload = (config.features.ares_reload
                && context.user != vera_mod_api::WEAPON_USER_AIRCRAFT)
                .then(|| config.reload.policy(&context.type_name));
            let depleted_weapon = config
                .features
                .ares_no_ammo_weapon
                .then(|| config.empty_weapon.policy(&context.type_name))
                .flatten();
            let input = config.features.ares_manual_control.then(|| {
                config
                    .manual_control
                    .policy(&context.type_name, context.user)
            });
            let keep_alive = config
                .features
                .ares_keep_alive
                .then(|| config.survival.contribution(&context.type_name))
                .flatten();
            // Own the permission even when this module is disabled, so a save
            // cannot fall through to the current built-in INI's restriction.
            let cloak_allowed = !config.features.ares_attach_effect
                || config.cloak.allowed(&context.type_name, context.user);
            plan.type_resource = Some(vera_mod_api::TypeResourcePolicy {
                users: context.user,
                reload,
                depleted_weapon,
                input,
                keep_alive,
                cloak_allowed: Some(cloak_allowed),
                spotlight: Some(config.spotlight.query(
                    &context.type_name,
                    context.user,
                    config.features.ares_spotlight,
                )),
                conversion: Some(config.type_conversion.query(
                    &context.type_name,
                    context.user,
                    config.features.ares_type_conversion,
                )),
            });
            Ok(plan)
        }
        _ if config.features.shield => {
            shield::evaluate(request, &config.shield, host, config.features.overload)
        }
        _ => {
            let mut plan = Plan::unchanged(request);
            plan.state.clear();
            plan.projection = Some(Projection::default());
            Ok(plan)
        }
    }
}

#[cfg(test)]
pub(crate) fn freeze_shield(shield: shield::Configuration, overload: bool) -> Vec<u8> {
    serde_json::to_vec(&Configuration {
        features: Features {
            shield: true,
            overload,
            ..Default::default()
        },
        shield,
        ..Default::default()
    })
    .unwrap()
}

#[cfg(test)]
mod airburst_tests;
#[cfg(test)]
mod ballistic_scatter_tests;
#[cfg(test)]
mod cameo_tests;
#[cfg(test)]
mod emp_tests;
#[cfg(test)]
mod firing_tests;
#[cfg(test)]
mod foundation_tests;
#[cfg(test)]
mod production_tests;
#[cfg(test)]
mod projectile_range_tests;
#[cfg(test)]
mod promotion_sound_tests;
#[cfg(test)]
mod spotlight_tests;
#[cfg(test)]
mod startup_tests;
#[cfg(test)]
mod tests;
#[cfg(test)]
mod type_conversion_tests;

#[cfg(test)]
mod weapon_instance_tests;
