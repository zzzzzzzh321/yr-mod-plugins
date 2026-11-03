//! Composition checks use frozen config; host access here would be a regression.
use super::*;
use vera_mod_api::{DamageKernelInput, TypeResourceContext};

pub(super) struct FrozenHost;
impl Host for FrozenHost {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        panic!("live INI read")
    }
    fn config_read(&self, _: &str, _: &str, _: &str, _: &[u8]) -> Result<Vec<u8>, String> {
        panic!("live INI read")
    }
    fn registry(&self, _: &str) -> Vec<String> {
        panic!("live registry read")
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("unexpected damage calculation")
    }
}

#[test]
fn new_match_defaults_do_not_reinterpret_historical_frozen_configuration() {
    let query = |config, context| {
        let request = Request {
            api_version: API_VERSION,
            context,
            config,
            state: vec![],
        };
        let plan = evaluate(&request, &FrozenHost).unwrap();
        plan.validate_for(&request).unwrap();
        plan
    };
    let weapon = Context::WeaponResource(vera_mod_api::WeaponResourceContext {
        weapon_name: "ORDINARY".into(),
    });
    let policy = query(vec![], weapon.clone())
        .weapon_resource
        .expect("a newly installed Ares component owns ordinary weapon costs and cadence");
    assert_eq!(policy.cost, 1);
    assert!(policy.use_weapon_rof);
    let object = Context::TypeResource(TypeResourceContext {
        type_name: "ORDINARY".into(),
        user: vera_mod_api::WEAPON_USER_BUILDING,
    });
    let policy = query(vec![], object.clone()).type_resource.unwrap();
    assert_eq!(
        policy.reload,
        Some(vera_mod_api::ReloadPolicy {
            normal_amount: 1,
            empty_amount: 1
        })
    );
    assert_eq!(policy.input, Some(vera_mod_api::InputPolicy::default()));
    assert!(policy.depleted_weapon.is_none());
    assert_eq!(
        policy.conversion,
        Some(vera_mod_api::TypeConversionPolicy::default())
    );
    assert_eq!(
        policy.spotlight,
        Some(vera_mod_api::SpotlightPolicy {
            has_spotlight: Some(false),
            use_type_rules: true,
            ..Default::default()
        })
    );
    let armor = Context::ArmorResource(vera_mod_api::ArmorResourceContext::Catalog);
    assert_eq!(
        query(vec![], armor.clone()).armor_resource,
        Some(vera_mod_api::ArmorResourcePolicy::Catalog {
            names: vera_mod_api::BASE_ARMOR_NAMES
                .into_iter()
                .map(str::to_owned)
                .collect(),
        })
    );
    let production = Context::ProductionResource(vera_mod_api::ProductionResourceContext::Catalog);
    assert!(
        query(vec![], production.clone())
            .production_resource
            .is_some()
    );
    let emp = Context::EmpResource(vera_mod_api::EmpResourceContext::Rules);
    assert_eq!(
        query(vec![], emp.clone()).emp_resource,
        Some(vera_mod_api::EmpResourcePolicy::Rules {
            enabled: true,
            ai_recover_mission: None,
            reactivation_requires_powered_center: true,
        })
    );
    let fresh_match = query(
        vec![],
        Context::MatchRules(vera_mod_api::MatchRulesContext {}),
    )
    .match_rules
    .unwrap();
    assert!(fresh_match.effects.unwrap().apply_rof);
    assert_eq!(fresh_match.script_type_conversion, Some(true));
    // Historical nonempty configuration is not a new match, including when
    // an older producer omitted the feature selection field entirely.
    let mut historical = serde_json::to_value(Configuration::default()).unwrap();
    historical.as_object_mut().unwrap().remove("features");
    historical.as_object_mut().unwrap().remove("spotlight");
    historical
        .as_object_mut()
        .unwrap()
        .remove("type_conversion");
    let bytes = serde_json::to_vec(&historical).unwrap();
    let old_match = query(
        bytes.clone(),
        Context::MatchRules(vera_mod_api::MatchRulesContext {}),
    )
    .match_rules
    .unwrap();
    assert_eq!(
        old_match.effects,
        Some(vera_mod_api::EffectRuntimePolicy::default())
    );
    assert_eq!(old_match.script_type_conversion, Some(false));
    assert!(query(bytes.clone(), weapon).weapon_resource.is_none());
    assert!(query(bytes.clone(), armor).armor_resource.is_none());
    assert!(
        query(bytes.clone(), production)
            .production_resource
            .is_none()
    );
    assert_eq!(
        query(bytes.clone(), emp).emp_resource,
        Some(vera_mod_api::EmpResourcePolicy::Rules {
            enabled: false,
            ai_recover_mission: None,
            reactivation_requires_powered_center: false,
        })
    );
    assert_eq!(
        query(
            bytes.clone(),
            Context::EmpResource(vera_mod_api::EmpResourceContext::Type {
                user: vera_mod_api::WEAPON_USER_UNIT,
                name: "ORDINARY".into(),
            })
        )
        .emp_resource,
        Some(vera_mod_api::EmpResourcePolicy::Type {
            policy: vera_mod_api::EmpTypePolicy::default()
        })
    );
    let old_type = query(bytes, object).type_resource.unwrap();
    assert!(old_type.reload.is_none() && old_type.depleted_weapon.is_none());
    assert!(old_type.input.is_none() && old_type.keep_alive.is_none());
    assert_eq!(old_type.cloak_allowed, Some(true));
    assert_eq!(
        old_type.conversion,
        Some(vera_mod_api::TypeConversionPolicy::default())
    );
    assert_eq!(
        old_type.spotlight,
        Some(vera_mod_api::SpotlightPolicy::default())
    );
    let fresh = decode_config(&[]).unwrap();
    assert!(!fresh.features.shield && !fresh.features.overload);
}

#[test]
fn ammunition_uses_frozen_configuration_and_excludes_aircraft() {
    let mut config = serde_json::to_value(Configuration::default()).unwrap();
    config["features"]["ares_ammo"] = serde_json::json!(true);
    config["ammunition"] = serde_json::json!({"costs":{"PAIR":2,"FREE":0,"NEGATIVE":-2}});
    let frozen = serde_json::to_vec(&config).unwrap();
    for (name, cost) in [("PAIR", 2), ("FREE", 0), ("NEGATIVE", -2), ("ordinary", 1)] {
        let request = Request {
            api_version: API_VERSION,
            context: Context::WeaponResource(vera_mod_api::WeaponResourceContext {
                weapon_name: name.into(),
            }),
            config: frozen.clone(),
            state: vec![],
        };
        // HostFixture panics on any configuration read: restored policies
        // must use the frozen bytes, never the current work's INI.
        let plan = evaluate(&request, &FrozenHost).unwrap();
        plan.validate_for(&request).unwrap();
        let policy = plan.weapon_resource.unwrap();
        assert_eq!((policy.cost, policy.reload_after_use), (cost, cost > 0));
        assert!(policy.applies_to(vera_mod_api::WEAPON_USER_UNIT));
        assert!(!policy.applies_to(vera_mod_api::WEAPON_USER_AIRCRAFT));
        assert_eq!(
            serde_json::to_value(policy).unwrap()["use_weapon_rof"],
            true,
            "the Ares ammo chain must bypass the stock loaded-building shortcut"
        );
        assert!(plan.state.is_empty() && plan.projection.is_none());
    }
}

#[test]
fn type_policies_are_frozen_independent_and_exclude_aircraft_reload_only() {
    let mut config = serde_json::to_value(Configuration::default()).unwrap();
    config["features"]["ares_reload"] = serde_json::json!(true);
    config["features"]["ares_no_ammo_weapon"] = serde_json::json!(true);
    config["reload"] = serde_json::json!({"amounts":{"TANK":{"normal_amount":2,"empty_amount":3}}});
    config["empty_weapon"] =
        serde_json::json!({"weapons":{"TANK":{"weapon_index":1,"threshold":1}}});
    let mut request = Request {
        api_version: API_VERSION,
        context: Context::TypeResource(TypeResourceContext {
            type_name: "TANK".into(),
            user: vera_mod_api::WEAPON_USER_UNIT,
        }),
        config: serde_json::to_vec(&config).unwrap(),
        state: vec![],
    };
    let plan = evaluate(&request, &FrozenHost).unwrap();
    plan.validate_for(&request).unwrap();
    let policy = plan.type_resource.unwrap();
    assert_eq!(policy.reload.unwrap().empty_amount, 3);
    assert_eq!(policy.depleted_weapon.unwrap().threshold, 1);
    request.context = Context::TypeResource(TypeResourceContext {
        type_name: "TANK".into(),
        user: vera_mod_api::WEAPON_USER_AIRCRAFT,
    });
    let plane = evaluate(&request, &FrozenHost).unwrap();
    plane.validate_for(&request).unwrap();
    assert!(plane.type_resource.unwrap().reload.is_none());
    assert!(plane.type_resource.unwrap().depleted_weapon.is_some());
    config["features"]["ares_reload"] = serde_json::json!(false);
    config["features"]["ares_no_ammo_weapon"] = serde_json::json!(false);
    request.config = serde_json::to_vec(&config).unwrap();
    let disabled = evaluate(&request, &FrozenHost)
        .unwrap()
        .type_resource
        .unwrap();
    assert!(disabled.reload.is_none() && disabled.depleted_weapon.is_none());
    assert_eq!(disabled.cloak_allowed, Some(true));
}

#[test]
fn manual_input_is_frozen_class_scoped_and_independently_disabled() {
    use vera_mod_api::{
        InputPolicy, WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING, WEAPON_USER_INFANTRY,
        WEAPON_USER_UNIT,
    };
    let mut config = serde_json::to_value(Configuration::default()).unwrap();
    config["features"]["ares_manual_control"] = serde_json::json!(true);
    config["manual_control"] = serde_json::json!({"policies":{"TEST":{
        "deny_manual_attack":true,"deny_manual_unload":true,
        "hide_transport_entry":true,"deny_self_guard_area":true
    }}});
    let mut request = Request {
        api_version: API_VERSION,
        context: Context::TypeResource(TypeResourceContext {
            type_name: "TEST".into(),
            user: WEAPON_USER_UNIT,
        }),
        config: serde_json::to_vec(&config).unwrap(),
        state: vec![],
    };
    for user in [
        WEAPON_USER_UNIT,
        WEAPON_USER_INFANTRY,
        WEAPON_USER_BUILDING,
        WEAPON_USER_AIRCRAFT,
    ] {
        request.context = Context::TypeResource(TypeResourceContext {
            type_name: "TEST".into(),
            user,
        });
        let plan = evaluate(&request, &FrozenHost).unwrap();
        plan.validate_for(&request).unwrap();
        let policy = plan.type_resource.unwrap();
        assert_eq!(
            policy.input,
            Some(InputPolicy {
                deny_manual_attack: true,
                deny_manual_unload: matches!(user, WEAPON_USER_UNIT | WEAPON_USER_AIRCRAFT),
                hide_transport_entry: true,
                deny_self_guard_area: user == WEAPON_USER_INFANTRY,
            })
        );
        assert!(policy.reload.is_none() && policy.depleted_weapon.is_none());
    }
    request.context = Context::TypeResource(TypeResourceContext {
        type_name: "UNCONFIGURED".into(),
        user: WEAPON_USER_UNIT,
    });
    assert_eq!(
        evaluate(&request, &FrozenHost)
            .unwrap()
            .type_resource
            .unwrap()
            .input,
        Some(InputPolicy::default()),
        "enabled feature owns false decisions on restore too"
    );
    config["features"]["ares_manual_control"] = serde_json::json!(false);
    request.config = serde_json::to_vec(&config).unwrap();
    assert!(
        evaluate(&request, &FrozenHost)
            .unwrap()
            .type_resource
            .and_then(|policy| policy.input)
            .is_none()
    );
}

/// These are decoded host responses, not another INI parser. Exercise the
/// normal Configure composition and ConfigView's native-reader delegation.
struct SlotConfigHost(i32);
impl Host for SlotConfigHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        matches!(
            (section, key),
            ("VeraFeatures", "AresNoAmmoWeapon") | ("TANK", "NoAmmoWeapon" | "NoAmmoAmount")
        )
        .then(|| "host-owned fixture".into())
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        _: &[u8],
    ) -> Result<Vec<u8>, String> {
        match (section, key, reader) {
            ("VeraFeatures", "AresNoAmmoWeapon", "bool") => Ok(b"true".to_vec()),
            ("TANK", "NoAmmoWeapon", "ares-int") => {
                serde_json::to_vec(&self.0).map_err(|e| e.to_string())
            }
            ("TANK", "NoAmmoAmount", "ares-int") => Ok(b"3".to_vec()),
            _ => panic!("unexpected configuration reader: {section}/{key}/{reader}"),
        }
    }
    fn registry(&self, _: &str) -> Vec<String> {
        vec![]
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("no damage calculation during configure")
    }
}

fn configure_no_ammo_weapon(config: Vec<u8>, index: i32) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(vera_mod_api::ConfigureContext {
            pass: 0,
            source: "slot-fixture".into(),
            final_pass: true,
            object_types: vec!["TANK".into()],
            warheads: vec![],
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &SlotConfigHost(index)).expect("native i32 slot is admitted");
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

#[test]
fn no_ammo_weapon_config_preserves_higher_and_signed_boundary_indices() {
    use vera_mod_api::{WEAPON_USER_AIRCRAFT, WEAPON_USER_INFANTRY, WEAPON_USER_UNIT};
    // Weapon3, the last array slot and the signed upper boundary exercise
    // configuration, frozen queries and class routing without a cross-product.
    for (index, user) in [
        (2, WEAPON_USER_UNIT),
        (17, WEAPON_USER_INFANTRY),
        (i32::MAX, WEAPON_USER_AIRCRAFT),
    ] {
        let request = Request {
            api_version: API_VERSION,
            context: Context::TypeResource(TypeResourceContext {
                type_name: "TANK".into(),
                user,
            }),
            config: configure_no_ammo_weapon(vec![], index),
            state: vec![],
        };
        let plan = evaluate(&request, &FrozenHost).unwrap();
        plan.validate_for(&request).unwrap();
        let policy = plan.type_resource.unwrap();
        assert_eq!(
            policy.depleted_weapon,
            Some(vera_mod_api::DepletedWeaponPolicy {
                weapon_index: index,
                threshold: 3
            })
        );
        assert!(policy.applies_to(user));
    }
}

#[test]
fn negative_no_ammo_weapon_reconfiguration_removes_the_previous_override() {
    // Configure is reused with retained frozen configuration. A negative
    // index disables the earlier override instead of preserving a stale slot.
    let earlier = configure_no_ammo_weapon(vec![], 2);
    for index in [-2, i32::MIN] {
        let request = Request {
            api_version: API_VERSION,
            context: Context::TypeResource(TypeResourceContext {
                type_name: "TANK".into(),
                user: vera_mod_api::WEAPON_USER_UNIT,
            }),
            config: configure_no_ammo_weapon(earlier.clone(), index),
            state: vec![],
        };
        let plan = evaluate(&request, &FrozenHost).unwrap();
        plan.validate_for(&request).unwrap();
        assert!(
            plan.type_resource
                .and_then(|policy| policy.depleted_weapon)
                .is_none(),
            "negative index {index} disables the override"
        );
    }
}

/// Already-decoded native reader responses; parsing is owned by the host.
struct SurvivalConfigHost {
    disabled: bool,
}
impl Host for SurvivalConfigHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        (matches!((section, key), ("EXCLUDED" | "RETAINED", "KeepAlive"))
            || self.disabled && (section, key) == ("VeraFeatures", "AresKeepAlive"))
            .then(|| "host-owned fixture".into())
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        _: &[u8],
    ) -> Result<Vec<u8>, String> {
        match (section, key, reader) {
            ("VeraFeatures", "AresKeepAlive", "bool") if self.disabled => Ok(b"false".to_vec()),
            ("EXCLUDED", "KeepAlive", "ares-nullable-bool") => Ok(b"false".to_vec()),
            ("RETAINED", "KeepAlive", "ares-nullable-bool") => Ok(b"true".to_vec()),
            _ => panic!("unexpected configuration reader: {section}/{key}/{reader}"),
        }
    }
    fn registry(&self, _: &str) -> Vec<String> {
        vec![]
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("no damage calculation during configure")
    }
}

fn configure_survival(config: Vec<u8>, disabled: bool) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(vera_mod_api::ConfigureContext {
            pass: 0,
            source: "survival-fixture".into(),
            final_pass: true,
            object_types: vec!["UNSET".into(), "EXCLUDED".into(), "RETAINED".into()],
            warheads: vec![],
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &SurvivalConfigHost { disabled }).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

pub(super) fn frozen_query(config: &[u8], context: Context) -> Plan {
    let request = Request {
        api_version: API_VERSION,
        context,
        config: config.to_vec(),
        state: vec![],
    };
    let plan = evaluate(&request, &FrozenHost).unwrap();
    plan.validate_for(&request).unwrap();
    plan
}

#[test]
fn keep_alive_configuration_freezes_unset_and_explicit_contributions() {
    // No VeraFeatures opt-in: a newly installed component owns the match rule.
    let config = configure_survival(vec![], false);
    let plan = frozen_query(
        &config,
        Context::MatchRules(vera_mod_api::MatchRulesContext {}),
    );
    assert_eq!(
        plan.match_rules.unwrap().survival,
        vera_mod_api::SurvivalRule::TrackedPresence
    );
    for (name, expected) in [
        ("UNSET", None),
        ("EXCLUDED", Some(false)),
        ("RETAINED", Some(true)),
    ] {
        let plan = frozen_query(
            &config,
            Context::TypeResource(TypeResourceContext {
                type_name: name.into(),
                user: vera_mod_api::WEAPON_USER_UNIT,
            }),
        );
        assert_eq!(
            plan.type_resource.and_then(|policy| policy.keep_alive),
            expected,
            "{name}"
        );
    }
}

#[test]
fn keep_alive_explicit_off_freezes_retail_without_the_previous_override() {
    let enabled = configure_survival(vec![], false);
    let disabled = configure_survival(enabled, true);
    let plan = frozen_query(
        &disabled,
        Context::MatchRules(vera_mod_api::MatchRulesContext {}),
    );
    assert_eq!(
        plan.match_rules.unwrap().survival,
        vera_mod_api::SurvivalRule::Retail
    );
    let plan = frozen_query(
        &disabled,
        Context::TypeResource(TypeResourceContext {
            type_name: "RETAINED".into(),
            user: vera_mod_api::WEAPON_USER_UNIT,
        }),
    );
    assert_eq!(
        plan.type_resource.and_then(|policy| policy.keep_alive),
        None
    );
}

/// Decoded Host responses for one physical definition pass and later feature
/// selections. Reader/parser and registry timing integration remain host tests.
struct CompositeConfigHost {
    selection: Option<bool>,
    definitions: bool,
}
impl Host for CompositeConfigHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        ((section, key) == ("VeraFeatures", "Shield")
            || (section, key) == ("TANK", "ShieldType")
            || (section, key) == ("TANK", "Cloakable.Allowed")
            || self.selection.is_some() && (section, key) == ("VeraFeatures", "AresAttachEffect")
            || self.definitions
                && matches!(
                    (section, key),
                    ("ShieldTypes", "0")
                        | ("SDKShield", "Strength")
                        | (
                            "HIT",
                            "AttachEffect.Duration" | "AttachEffect.FirepowerMultiplier"
                        )
                ))
        .then(|| "host-owned fixture".into())
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "rules-registry-view" {
            return serde_json::to_vec(&vera_mod_api::ProductionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        match (section, key, reader) {
            ("VeraFeatures", "Shield", "bool") => Ok(b"true".to_vec()),
            ("VeraFeatures", "AresAttachEffect", "bool") => {
                serde_json::to_vec(&self.selection.unwrap()).map_err(|e| e.to_string())
            }
            ("ShieldTypes", "0", "name:2048") | ("TANK", "ShieldType", "name:128") => {
                Ok(br#""SDKShield""#.to_vec())
            }
            ("SDKShield", "Strength", "int") => Ok(b"100".to_vec()),
            ("HIT", "AttachEffect.Duration", "ares-int") => Ok(b"5".to_vec()),
            ("HIT", "AttachEffect.FirepowerMultiplier", "ares-double-bits") => {
                serde_json::to_vec(&0.5f64.to_bits()).map_err(|e| e.to_string())
            }
            ("TANK", "Cloakable.Allowed", "ares-bool") => {
                // The first vehicle read yields false. Later invalid native
                // reads retain each category's own previously decoded value;
                // the final disabling layer explicitly restores true.
                Ok(match (self.definitions, self.selection) {
                    (true, _) => b"false".to_vec(),
                    (false, Some(false)) => b"true".to_vec(),
                    _ => current.to_vec(),
                })
            }
            _ if reader.starts_with("string:") => Ok(current.to_vec()),
            _ => panic!("unexpected configuration reader: {section}/{key}/{reader}"),
        }
    }
    fn registry(&self, name: &str) -> Vec<String> {
        match name {
            "$vehicles" => vec!["TANK".into(), "ORDINARY".into()],
            "$infantry" if !self.definitions => vec!["TANK".into()],
            "$warheads" if self.definitions => vec!["HIT".into(), "ORDINARY".into()],
            // Enumerating whole actor/warhead sections would allocate all
            // unrelated retail key strings in the guest again. The existing
            // readers have exact-key presence probes for this configuration.
            "TANK" | "HIT" | "ORDINARY" => panic!("unexpected whole-section key import"),
            "ShieldTypes" if self.definitions => vec!["0".into()],
            "SDKShield" if self.definitions => vec!["Strength".into()],
            _ => vec![],
        }
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("no damage calculation in this composition check")
    }
}

#[test]
fn effects_shield_and_cloak_keep_separate_state_and_frozen_layered_selection() {
    use vera_mod_api::{
        AttachContext, EffectContext, EffectControl, EffectEvent, EffectPlan,
        EffectResourceContext, EffectResourceKind, EntityContext, MatchRulesContext, TickContext,
        WEAPON_USER_UNIT,
    };
    let configure = |config, pass, final_pass, selection, definitions| {
        let request = Request {
            api_version: API_VERSION,
            context: Context::Configure(vera_mod_api::ConfigureContext {
                pass,
                source: "composition-fixture".into(),
                final_pass,
                object_types: vec!["TANK".into(), "ORDINARY".into()],
                warheads: vec!["HIT".into(), "ORDINARY".into()],
            }),
            config,
            state: vec![],
        };
        let plan = evaluate(
            &request,
            &CompositeConfigHost {
                selection,
                definitions,
            },
        )
        .unwrap();
        plan.validate_for(&request).unwrap();
        plan.config.unwrap()
    };
    // Authored definitions survive the disabled physical pass; a later layer
    // enables them without re-reading the old pass through a projected INI.
    let first = configure(vec![], 0, false, Some(false), true);
    let enabled_pass = configure(first, 1, false, Some(true), false);
    let enabled = configure(enabled_pass.clone(), 2, true, None, false);
    let frozen: serde_json::Value = serde_json::from_slice(&enabled).unwrap();
    assert_eq!(frozen["attach_effect"]["objects"], serde_json::json!({}));
    assert_eq!(
        frozen["attach_effect"]["warheads"]
            .as_object()
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        frozen["cloak"]["permissions"],
        serde_json::json!({"1":{"TANK":false}})
    );
    let resource = EffectResourceContext {
        kind: EffectResourceKind::Warhead,
        name: "HIT".into(),
    };
    let policy = frozen_query(&enabled, Context::EffectResource(resource.clone()));
    assert_eq!(
        policy.effect_resource.unwrap().firepower_multiplier_bits,
        0.5f64.to_bits()
    );
    assert!(policy.state.is_empty() && policy.projection.is_none());
    let vehicle = frozen_query(
        &enabled,
        Context::TypeResource(TypeResourceContext {
            type_name: "TANK".into(),
            user: WEAPON_USER_UNIT,
        }),
    )
    .type_resource
    .unwrap();
    let infantry = frozen_query(
        &enabled,
        Context::TypeResource(TypeResourceContext {
            type_name: "TANK".into(),
            user: vera_mod_api::WEAPON_USER_INFANTRY,
        }),
    )
    .type_resource
    .unwrap();
    assert_eq!(vehicle.cloak_allowed, Some(false));
    assert_eq!(infantry.cloak_allowed, Some(true));
    let runtime = frozen_query(&enabled, Context::MatchRules(MatchRulesContext {}))
        .match_rules
        .unwrap()
        .effects
        .unwrap();
    assert!(runtime.cloak.check_type_permission && runtime.cloak.allow_active_ability_above_ground);
    assert!(
        frozen_query(&enabled, Context::MatchRules(MatchRulesContext {}))
            .match_rules
            .unwrap()
            .effects
            .unwrap()
            .allow_unarmed_firepower_crates
    );

    let entity = EntityContext {
        id: 1,
        type_name: "TANK".into(),
        owner: "Multi1".into(),
        armor: "heavy".into(),
        health: 100,
        max_health: 100,
        alive: true,
        in_limbo: false,
        immobilized: false,
        temporal: false,
        being_warped_out: false,
        powered: true,
        chrono_warp_latch: false,
        ai_frozen: false,
        deactivated: false,
        immune: false,
        type_immune: false,
        invulnerable: false,
    };
    let mut shield_request = Request {
        api_version: API_VERSION,
        context: Context::Attach(AttachContext {
            frame: 0,
            entity: entity.clone(),
        }),
        config: enabled.clone(),
        state: vec![],
    };
    let shield_plan = evaluate(&shield_request, &FrozenHost).unwrap();
    shield_plan.validate_for(&shield_request).unwrap();
    assert!(!shield_plan.state.is_empty() && shield_plan.effects.is_none());
    let effect_context = EffectContext {
        frame: 0,
        entity: entity.clone(),
        user: WEAPON_USER_UNIT,
        inside_transport: false,
        cloaked_or_cloaking: false,
        entries: vec![],
        control: EffectControl::Waiting(i32::MAX),
        animations_hidden: false,
        event: EffectEvent::Apply {
            resource: resource.clone(),
            source_owner: Some("Multi2".into()),
            source_is_owner: false,
            source_is_ally: false,
            affects_allies: false,
            verses_bits: 1.0f64.to_bits(),
        },
    };
    let effects = frozen_query(&enabled, Context::Effects(effect_context.clone()));
    assert!(effects.state.is_empty() && effects.projection.is_none());
    assert!(matches!(
        &effects.effects.unwrap().actions[0],
        vera_mod_api::EffectAction::Append { remaining: 5, .. }
    ));
    shield_request.state = shield_plan.state;
    shield_request.context = Context::Tick(TickContext { frame: 0, entity });
    let shield_tick = evaluate(&shield_request, &FrozenHost).unwrap();
    shield_tick.validate_for(&shield_request).unwrap();
    assert!(!shield_tick.state.is_empty() && shield_tick.effects.is_none());

    let disabled_pass = configure(enabled_pass, 2, false, Some(false), false);
    let disabled = configure(disabled_pass, 3, true, None, false);
    let frozen: serde_json::Value = serde_json::from_slice(&disabled).unwrap();
    assert_eq!(
        frozen["cloak"]["permissions"],
        serde_json::json!({}),
        "explicit true removes the sparse restriction"
    );
    assert_eq!(
        frozen_query(
            &disabled,
            Context::TypeResource(TypeResourceContext {
                type_name: "TANK".into(),
                user: WEAPON_USER_UNIT,
            })
        )
        .type_resource
        .unwrap()
        .cloak_allowed,
        Some(true),
        "disabled mode owns true instead of falling back to current built-in INI"
    );
    assert_eq!(
        frozen_query(&disabled, Context::MatchRules(MatchRulesContext {}))
            .match_rules
            .unwrap()
            .effects,
        Some(vera_mod_api::EffectRuntimePolicy::default())
    );
    assert!(
        frozen_query(&disabled, Context::EffectResource(resource))
            .effect_resource
            .is_none()
    );
    assert_eq!(
        frozen_query(&disabled, Context::Effects(effect_context.clone())).effects,
        Some(EffectPlan::unchanged(&effect_context))
    );
}

/// Decoded results from physical native readers. Numeric/parser equivalence
/// is exercised by the host's real-parser tests, not reimplemented here.
struct ArmorConfigHost(u8);

impl ArmorConfigHost {
    fn value(&self, section: &str, key: &str) -> Option<serde_json::Value> {
        use serde_json::json;
        match (self.0, section, key) {
            (0 | 4, "VeraFeatures", "AresArmor") => Some(json!(false)),
            (2, "VeraFeatures", "AresArmor") => Some(json!(true)),
            (0, "TARGET", "Armor") => Some(json!("future")),
            (1, "TARGET", "Armor") => Some(json!("future")),
            (0, "TREE", "Armor") => Some(json!("heavy")),
            (1, "ArmorTypes", "paper") => Some(json!("later")),
            (1, "ArmorTypes", "later") => Some(json!("steel")),
            (1, "ArmorTypes", "future") => Some(json!("later")),
            (2, "ArmorTypes", "later") => Some(json!("100%")),
            (0, "HIT", "Verses") => Some(json!(["50%"])),
            (1, "HIT", "Versus.steel") => Some(json!("25%")),
            (1, "HIT", "Versus.paper.ForceFire") => Some(json!(true)),
            (2, "HIT", "Versus.steel") => Some(json!("50%")),
            (3, "TARGET", "Armor") => Some(json!("future")),
            (3, "COLLISION", "Armor") => Some(json!("ABCDEFGHIJKLMNOPQRSTUVWXYZ12345")),
            (3, "HIT", "Versus.steel.ForceFire") => Some(json!("0%")),
            _ => None,
        }
    }
}

impl Host for ArmorConfigHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        self.value(section, key)
            .map(|_| "decoded host fixture".into())
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "rules-registry-view" {
            return serde_json::to_vec(&vera_mod_api::ProductionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if reader == "verse-token" {
            assert!(section.is_empty() && key.is_empty());
            let token: String = serde_json::from_slice(current).unwrap();
            let policy = match token.as_str() {
                "50%" => vera_mod_api::ArmorVersePolicy {
                    multiplier_bits: 0.5f64.to_bits(),
                    ..Default::default()
                },
                "25%" => vera_mod_api::ArmorVersePolicy {
                    multiplier_bits: 0.25f64.to_bits(),
                    ..Default::default()
                },
                "100%" => vera_mod_api::ArmorVersePolicy::default(),
                "steel" | "later" | "0%" => vera_mod_api::ArmorVersePolicy {
                    multiplier_bits: 0.0f64.to_bits(),
                    force_fire: false,
                    retaliate: false,
                    passive_acquire: false,
                },
                _ => panic!("unexpected verse token: {token}"),
            };
            return serde_json::to_vec(&policy).map_err(|e| e.to_string());
        }
        if key == "Versus.steel.ForceFire" && reader == "bool" {
            return Ok(b"false".to_vec());
        }
        let expected = match key {
            "AresArmor" | "Versus.paper.ForceFire" => "bool",
            "Verses" => "comma-list:2048",
            "paper" | "later" | "future" => "name:64",
            "Armor" | "Versus.steel" | "Versus.steel.ForceFire" => "name:32",
            _ => panic!("unexpected armor reader: {section}/{key}/{reader}"),
        };
        assert_eq!(reader, expected);
        serde_json::to_vec(&self.value(section, key).unwrap()).map_err(|e| e.to_string())
    }
    fn registry(&self, name: &str) -> Vec<String> {
        let names: &[&str] = match (self.0, name) {
            (0..=3, "$armor-before-type-data") => &["reached"],
            (1, "ArmorTypes") => &["paper", "later", "future"],
            (2, "ArmorTypes") => &["later"],
            (3, "ArmorTypes") => &[
                "abcdefghijklmnopqrstuvwxyz12345A",
                "abcdefghijklmnopqrstuvwxyz12345B",
                "steel.ForceFire",
            ],
            (0, "$armor-units") => &["TARGET"],
            (3, "$armor-units") => &["TARGET", "COLLISION"],
            (1, "$armor-infantry") => &["TARGET"],
            (0 | 1, "$armor-terrain") => &["TREE"],
            (0..=3, "$armor-warheads") => &["HIT"],
            (0, "HIT") => &["Verses"],
            (1, "HIT") => &["Versus.steel", "Versus.paper.ForceFire"],
            (2, "HIT") => &["Versus.steel"],
            (3, "HIT") => &["Versus.steel.ForceFire"],
            _ => &[],
        };
        names.iter().map(|name| (*name).into()).collect()
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("armor configuration cannot calculate damage")
    }
}

#[test]
fn armor_retains_physical_identity_and_inline_records_across_frozen_queries() {
    use vera_mod_api::{ArmorObjectKind, ArmorResourceContext, ArmorResourcePolicy};
    let configure = |config, pass, final_pass| {
        let request = Request {
            api_version: API_VERSION,
            context: Context::Configure(vera_mod_api::ConfigureContext {
                pass: u64::from(pass),
                source: "decoded-armor-fixture".into(),
                final_pass,
                object_types: vec!["TARGET".into()],
                warheads: vec!["HIT".into()],
            }),
            config,
            state: vec![],
        };
        let plan = evaluate(&request, &ArmorConfigHost(pass)).unwrap();
        plan.validate_for(&request).unwrap();
        plan.config.unwrap()
    };
    let mut config = vec![];
    // Definitions accumulate while explicitly off, then a later layer enables
    // the module. An absent later Armor does not retry its earlier unknown name.
    for pass in 0..=2 {
        config = configure(config, pass, false);
    }
    let frozen = configure(config.clone(), 5, true);
    let query = |bytes: &[u8], context| {
        let plan = frozen_query(bytes, Context::ArmorResource(context));
        assert!(plan.state.is_empty() && plan.projection.is_none());
        plan.armor_resource
    };
    let object = |category, name: &str| ArmorResourceContext::Object {
        category,
        name: name.into(),
    };
    assert_eq!(
        query(&frozen, object(ArmorObjectKind::Unit, "TARGET")),
        Some(ArmorResourcePolicy::Object { armor: 0 })
    );
    assert_eq!(
        query(&frozen, object(ArmorObjectKind::Infantry, "TARGET")),
        Some(ArmorResourcePolicy::Object { armor: 13 }),
        "same-named categories retain their distinct physical read points"
    );
    assert_eq!(
        query(&frozen, object(ArmorObjectKind::Terrain, "TREE")),
        Some(ArmorResourcePolicy::Object { armor: 5 })
    );
    assert_eq!(
        query(&frozen, object(ArmorObjectKind::Terrain, "UNREAD")),
        Some(ArmorResourcePolicy::Object { armor: 6 })
    );
    let Some(ArmorResourcePolicy::Catalog { names }) =
        query(&frozen, ArmorResourceContext::Catalog)
    else {
        panic!("missing frozen catalog")
    };
    assert_eq!(&names[11..], &["paper", "later", "future"]);
    let Some(ArmorResourcePolicy::Warhead {
        entries,
        use_medium_wood_target_mask,
    }) = query(
        &frozen,
        ArmorResourceContext::Warhead { name: "HIT".into() },
    )
    else {
        panic!("missing frozen warhead")
    };
    assert!(!use_medium_wood_target_mask);
    assert_eq!(entries[7].multiplier_bits, 0.5f64.to_bits());
    // Forward reference takes paper's own fallback; a ForceFire override is
    // an independent raw flag, not a guest interpretation of CanFire.
    assert_eq!(entries[11].multiplier_bits, 0.0f64.to_bits());
    assert!(entries[11].force_fire && !entries[11].retaliate);
    // Both backward inheritance and the following layer keep whole records;
    // changing steel/the catalog default does not update existing children.
    assert_eq!(entries[12].multiplier_bits, 0.25f64.to_bits());
    assert_eq!(entries[13], entries[12]);
    let rebound = configure(config.clone(), 3, false);
    assert_eq!(
        query(&rebound, object(ArmorObjectKind::Unit, "TARGET")),
        Some(ArmorResourcePolicy::Object { armor: 13 })
    );
    // Full32-byte raw keys stay unknown after their31-byte stores. Both are
    // allocated again on a later pass, while Find keeps the first stored ID.
    let repeated = configure(rebound, 3, false);
    let Some(ArmorResourcePolicy::Catalog { names }) =
        query(&repeated, ArmorResourceContext::Catalog)
    else {
        panic!("missing collision catalog")
    };
    assert_eq!(
        &names[14..],
        &[
            "abcdefghijklmnopqrstuvwxyz12345",
            "abcdefghijklmnopqrstuvwxyz12345",
            "steel.ForceFire",
            "abcdefghijklmnopqrstuvwxyz12345",
            "abcdefghijklmnopqrstuvwxyz12345",
        ]
    );
    assert_eq!(
        query(&repeated, object(ArmorObjectKind::Unit, "COLLISION")),
        Some(ArmorResourcePolicy::Object { armor: 14 })
    );
    let Some(ArmorResourcePolicy::Warhead { entries, .. }) = query(
        &repeated,
        ArmorResourceContext::Warhead { name: "HIT".into() },
    ) else {
        panic!("missing collision warhead")
    };
    // The same exact key is both steel's boolean flag and the verse for the
    // separately registered armor named steel.ForceFire; neither read is lost.
    assert!(!entries[7].force_fire);
    assert_eq!(entries[16].multiplier_bits, 0.0f64.to_bits());
    let disabled = configure(config, 4, false);
    assert!(query(&disabled, ArmorResourceContext::Catalog).is_none());
}
