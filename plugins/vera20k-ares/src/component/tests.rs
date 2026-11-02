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
    assert!(
        evaluate(&request, &FrozenHost)
            .unwrap()
            .type_resource
            .is_none()
    );
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
            .is_none()
    );
}
