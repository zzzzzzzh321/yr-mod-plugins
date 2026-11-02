//! Focused behavior checks; the engine integration tests supply production INI
//! readers and the real warhead kernel to the built WASM component.
use super::config;
use crate::{
    component::{evaluate, freeze_shield},
    support::Host,
};
use vera_mod_api::{API_VERSION, Context, Plan, Request};
use vera_mod_api::{
    ArmorVerses, AttachContext, DamageContext, DamageKernelInput, EntityContext, TickContext,
};

struct HostFixture;
impl Host for HostFixture {
    fn config(&self, _: &str, _: &str) -> Option<String> {
        panic!("unexpected configuration read")
    }
    fn config_read(&self, _: &str, _: &str, _: &str, _: &[u8]) -> Result<Vec<u8>, String> {
        panic!("unexpected configuration read")
    }
    fn registry(&self, _: &str) -> Vec<String> {
        panic!("unexpected registry read")
    }
    fn damage_kernel(&self, input: &DamageKernelInput) -> Result<i32, String> {
        // Zero-distance, 100%-Verses fixture; core's native kernel is covered
        // independently and is deliberately not reimplemented in this test.
        assert_eq!(input.distance_leptons, 0);
        assert_eq!(input.multiplier_bits, 1.0f64.to_bits());
        Ok(input.damage)
    }
}

fn entity() -> EntityContext {
    EntityContext {
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
    }
}

fn attached(local: [i32; 3], global: [i32; 3], overload: bool) -> Request {
    let config = config::fixture_configuration(local, global);
    let mut request = Request {
        api_version: API_VERSION,
        context: Context::Attach(AttachContext {
            frame: 0,
            entity: entity(),
        }),
        config: freeze_shield(config, overload),
        state: vec![],
    };
    request.state = evaluate(&request, &HostFixture).unwrap().state;
    request
}

fn damage(request: &mut Request, frame: i32, amount: i32) {
    request.context = Context::Damage(DamageContext {
        frame,
        entity: entity(),
        attacker: None,
        source_owner: None,
        source_is_ally: false,
        source_is_owner: false,
        target_is_neutral: false,
        attacker_same_type_owner: false,
        warhead_name: "WH".into(),
        warhead_temporal: false,
        warhead_psychedelic: false,
        incoming: amount,
        distance_leptons: 0,
        ignore_defenses: false,
        no_damage: false,
        max_damage: i32::MAX,
        armor_multiplier_bits: 1.0f64.to_bits(),
        verses: vec![ArmorVerses {
            armor: "none".into(),
            multiplier_bits: 1.0f64.to_bits(),
        }],
        cell_spread_bits: 0f64.to_bits(),
        percent_at_max_bits: 1f64.to_bits(),
    });
}

fn tick_plan(request: &mut Request, frame: i32, entity: EntityContext) -> Plan {
    request.context = Context::Tick(TickContext { frame, entity });
    evaluate(request, &HostFixture).unwrap()
}

#[test]
fn damage_is_a_repeatable_plan_and_recovery_survives_state_serialization() {
    let mut request = attached([-1; 3], [-1; 3], false);
    damage(&mut request, 10, 20);
    let snapshot = request.state.clone();
    let first = evaluate(&request, &HostFixture).unwrap();
    let preview = evaluate(&request, &HostFixture).unwrap();
    assert_eq!(first, preview);
    assert_eq!(request.state, snapshot);
    assert_eq!(first.damage, Some(0));
    assert_eq!(first.projection.as_ref().unwrap().meters[0].current, 80);
    request.state = first.state;
    request.context = Context::Tick(TickContext {
        frame: 99,
        entity: entity(),
    });
    let before = evaluate(&request, &HostFixture).unwrap();
    assert_eq!(before.projection.as_ref().unwrap().meters[0].current, 80);
    request.context = Context::Tick(TickContext {
        frame: 100,
        entity: entity(),
    });
    assert_eq!(
        evaluate(&request, &HostFixture)
            .unwrap()
            .projection
            .unwrap()
            .meters[0]
            .current,
        100
    );
}

#[test]
fn pips_choose_local_then_global_then_native_fallback() {
    let local = attached([20, 21, 22], [10, 11, 12], false);
    let meter = evaluate(&local, &HostFixture)
        .unwrap()
        .projection
        .unwrap()
        .meters
        .remove(0);
    assert_eq!((meter.pip_frame, meter.building_pip_frame), (20, 30));
    let global = attached([-1; 3], [10, 11, 12], false);
    let meter = evaluate(&global, &HostFixture)
        .unwrap()
        .projection
        .unwrap()
        .meters
        .remove(0);
    assert_eq!((meter.pip_frame, meter.building_pip_frame), (10, 40));
    let bare = attached([-1; 3], [-1; 3], false);
    let meter = evaluate(&bare, &HostFixture)
        .unwrap()
        .projection
        .unwrap()
        .meters
        .remove(0);
    assert_eq!((meter.pip_frame, meter.building_pip_frame), (16, 5));
}

#[test]
fn chronosphere_skips_absorption_and_recovery_without_temporal_timer_pause() {
    let mut request = attached([-1; 3], [-1; 3], false);
    damage(&mut request, 0, 20);
    if let Context::Damage(context) = &mut request.context {
        context.entity.being_warped_out = true;
        assert!(!context.entity.temporal);
    }
    let during_warp = evaluate(&request, &HostFixture).unwrap();
    assert_eq!(during_warp.damage, Some(20));
    assert_eq!(during_warp.projection.unwrap().meters[0].current, 100);

    damage(&mut request, 0, 20);
    request.state = evaluate(&request, &HostFixture).unwrap().state;
    let after_hit = request.state.clone();
    let mut chrono = entity();
    chrono.chrono_warp_latch = true;
    chrono.being_warped_out = true;
    request.state = tick_plan(&mut request, 30, chrono.clone()).state;
    request.state = tick_plan(&mut request, 90, chrono).state;
    assert_eq!(
        tick_plan(&mut request, 90, entity())
            .projection
            .unwrap()
            .meters[0]
            .current,
        100,
        "Chronosphere kept the original frame-90 deadline"
    );

    request.state = after_hit;
    let mut temporal = entity();
    temporal.temporal = true;
    temporal.being_warped_out = true;
    request.state = tick_plan(&mut request, 30, temporal.clone()).state;
    request.state = tick_plan(&mut request, 300, temporal).state;
    request.state = tick_plan(&mut request, 300, entity()).state;
    assert_eq!(
        tick_plan(&mut request, 359, entity())
            .projection
            .unwrap()
            .meters[0]
            .current,
        80,
        "Temporal retained sixty paused recovery frames"
    );
    assert_eq!(
        tick_plan(&mut request, 360, entity())
            .projection
            .unwrap()
            .meters[0]
            .current,
        100
    );
}

#[test]
fn guest_house_policy_preserves_neutral_owner_and_null_source_order() {
    let mut request = attached([-1; 3], [-1; 3], false);
    let mut config: super::Configuration = serde_json::from_slice(&request.config).unwrap();
    config.warheads.insert(
        "WH".into(),
        config::ShieldWarheadRules {
            affects_allies: false,
            affects_owner: Some(true),
            affects_neutral: false,
            ..Default::default()
        },
    );
    request.config = freeze_shield(config, false);
    damage(&mut request, 0, 20);
    if let Context::Damage(context) = &mut request.context {
        context.source_owner = Some("Multi1".into());
        context.source_is_owner = true;
        context.source_is_ally = true;
        context.target_is_neutral = true;
    }
    assert_eq!(
        evaluate(&request, &HostFixture)
            .unwrap()
            .projection
            .unwrap()
            .meters[0]
            .current,
        100,
        "the neutral gate precedes an explicit owner opt-in"
    );
    if let Context::Damage(context) = &mut request.context {
        context.target_is_neutral = false;
    }
    assert_eq!(
        evaluate(&request, &HostFixture)
            .unwrap()
            .projection
            .unwrap()
            .meters[0]
            .current,
        80,
        "an explicit owner opt-in overrides AffectsAllies=no"
    );
    if let Context::Damage(context) = &mut request.context {
        context.target_is_neutral = true;
        context.source_owner = None;
    }
    assert_eq!(
        evaluate(&request, &HostFixture)
            .unwrap()
            .projection
            .unwrap()
            .meters[0]
            .current,
        80,
        "a null source bypasses all house gates"
    );
}

fn tick_and_commit_points(request: &mut Request, frame: i32) -> i32 {
    let plan = tick_plan(request, frame, entity());
    let points = plan.projection.as_ref().unwrap().meters[0].current;
    request.state = plan.state;
    points
}

#[test]
fn overload_commits_second_hit_and_keeps_recovery_paused_until_it_ends() {
    let mut request = attached([-1; 3], [-1; 3], true);
    damage(&mut request, 10, 60);
    let first = evaluate(&request, &HostFixture).unwrap();
    assert_eq!(first.damage, Some(0));
    assert!(!first.projection.as_ref().unwrap().meters[0].active);
    assert!(first.operations.is_empty());
    request.state = first.state;
    damage(&mut request, 11, 10);
    let second = evaluate(&request, &HostFixture).unwrap();
    assert_eq!(second.damage, Some(10));
    request.state = second.state;

    let cooled = tick_plan(&mut request, 85, entity());
    assert!(cooled.projection.as_ref().unwrap().meters[0].active);
    assert_eq!(cooled.projection.as_ref().unwrap().meters[0].current, 40);
    request.state = cooled.state;
    assert_eq!(tick_and_commit_points(&mut request, 101), 40);
    assert_eq!(tick_and_commit_points(&mut request, 174), 40);
    assert_eq!(tick_and_commit_points(&mut request, 175), 65);
}

#[test]
fn overload_pauses_combat_delays_for_both_healing_and_respawn() {
    for (hit, before_recovery, recovered) in [(60, 40, 65), (100, 0, 50)] {
        let mut request = attached([-1; 3], [-1; 3], true);
        request.config = freeze_shield(config::fixture_with_combat_delay(12), true);
        damage(&mut request, 10, hit);
        request.state = evaluate(&request, &HostFixture).unwrap().state;
        damage(&mut request, 11, 10);
        let second = evaluate(&request, &HostFixture).unwrap();
        assert_eq!(second.damage, Some(10));
        request.state = second.state;

        assert_eq!(tick_and_commit_points(&mut request, 85), before_recovery);
        assert_eq!(tick_and_commit_points(&mut request, 96), before_recovery);
        // The twelve-frame delay finishes at 97, then starts the 90-frame rate.
        assert_eq!(tick_and_commit_points(&mut request, 97), before_recovery);
        assert_eq!(tick_and_commit_points(&mut request, 175), before_recovery);
        assert_eq!(tick_and_commit_points(&mut request, 186), before_recovery);
        assert_eq!(tick_and_commit_points(&mut request, 187), recovered);
    }
}

#[test]
fn disabled_features_do_not_create_shields_or_weapon_policies() {
    let frozen = freeze_shield(config::fixture_configuration([-1; 3], [-1; 3]), false);
    let mut config = serde_json::from_slice::<serde_json::Value>(&frozen).unwrap();
    config["features"] = serde_json::json!({"shield":false,"overload":false,"ares_ammo":false});
    config["ammunition"] = serde_json::json!({"costs":{"PAIR":2}});
    let mut request = Request {
        api_version: API_VERSION,
        context: Context::Attach(AttachContext {
            frame: 0,
            entity: entity(),
        }),
        config: serde_json::to_vec(&config).unwrap(),
        state: Vec::new(),
    };
    let plan = evaluate(&request, &HostFixture).unwrap();
    assert!(
        plan.state.is_empty(),
        "disabled shield must not subscribe an object"
    );
    assert!(plan.projection.unwrap().meters.is_empty());
    request.context = Context::WeaponResource(vera_mod_api::WeaponResourceContext {
        weapon_name: "PAIR".into(),
    });
    assert!(
        evaluate(&request, &HostFixture)
            .unwrap()
            .weapon_resource
            .is_none()
    );
}
