//! Composition checks use frozen config; host access here would be a regression.
use super::*;
use vera_mod_api::{DamageKernelInput, TypeResourceContext};

struct FrozenHost;
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
    assert!(
        query(
            vec![],
            Context::MatchRules(vera_mod_api::MatchRulesContext {})
        )
        .match_rules
        .unwrap()
        .effects
        .unwrap()
        .apply_rof
    );
    // Historical nonempty configuration is not a new match, including when
    // an older producer omitted the feature selection field entirely.
    let mut historical = serde_json::to_value(Configuration::default()).unwrap();
    historical.as_object_mut().unwrap().remove("features");
    let bytes = serde_json::to_vec(&historical).unwrap();
    assert_eq!(
        query(
            bytes.clone(),
            Context::MatchRules(vera_mod_api::MatchRulesContext {})
        )
        .match_rules
        .unwrap()
        .effects,
        Some(vera_mod_api::EffectRuntimePolicy::default())
    );
    assert!(query(bytes.clone(), weapon).weapon_resource.is_none());
    let old_type = query(bytes, object).type_resource.unwrap();
    assert!(old_type.reload.is_none() && old_type.depleted_weapon.is_none());
    assert!(old_type.input.is_none() && old_type.keep_alive.is_none());
    assert_eq!(old_type.cloak_allowed, Some(true));
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

fn frozen_query(config: &[u8], context: Context) -> Plan {
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
