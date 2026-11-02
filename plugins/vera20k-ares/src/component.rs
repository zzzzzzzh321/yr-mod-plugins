//! Static composition, selected by INI once per match. All code is present in
//! one component; simulation only uses the frozen selection, never live INI.
use crate::{
    features::{
        defense::{armor, shield},
        houses::survival,
        units::{attach_effect, cloak, manual_control},
        weapons::{ammo_cost, empty_weapon, reload},
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
}

pub fn manifest() -> ModuleManifest {
    ModuleManifest {
        api_version: API_VERSION,
        id: "vera.reference-gameplay".into(),
        version: "0.9.0".into(),
        state_version: 1,
        config_version: 9,
        hooks: vec![
            HookKind::Configure,
            HookKind::Attach,
            HookKind::Damage,
            HookKind::Tick,
            HookKind::WeaponResource,
            HookKind::TypeResource,
            HookKind::MatchRules,
            HookKind::EffectResource,
            HookKind::Effects,
            HookKind::ArmorResource,
        ],
    }
}

fn decode_config(bytes: &[u8]) -> Result<Configuration, String> {
    if bytes.is_empty() {
        Ok(Configuration {
            features: Features::for_new_match(),
            ..Configuration::default()
        })
    } else {
        serde_json::from_slice(bytes).map_err(|e| format!("component configuration: {e}"))
    }
}

pub(crate) fn evaluate<H: Host>(request: &Request, host: &H) -> Result<Plan, String> {
    request.validate()?;
    let mut config = decode_config(&request.config)?;
    if let Context::Configure(context) = &request.context {
        let view = ConfigView::new(host);
        config.features.apply_pass(&view);
        // Every physical pass owns its registered-reference read point, even
        // while disabled. Finalize only constrains the retained directories.
        config.attach_effect.configure(context, &view);
        config.cloak.configure(context, &view);
        // Armor identity and inheritance also belong to each reached physical
        // read, including layers before the module is enabled.
        config.armor.configure(context, &view);
        if context.final_pass {
            config.features.validate()?;
            if config.features.ares_ammo {
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
        Context::WeaponResource(context) => {
            let mut plan = Plan::unchanged(request);
            if config.features.ares_ammo {
                plan.weapon_resource = Some(config.ammunition.policy(&context.weapon_name));
            }
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
mod tests;
