//! Shield decisions and state, executed only by the external component.
use super::{
    Configuration,
    config::{ShieldType, ShieldWarheadRules},
};
use crate::support::Host;
use vera_mod_api::timer::{CdTimer, PAUSED_START_FRAME};
use vera_mod_api::{
    Context, DamageContext, DamageKernelInput, EntityContext, Meter, Plan, Projection, Request,
    Tint,
};

/// One `CDTimerClass` with the one bit [`CdTimer`] cannot carry: whether it was
/// ever started. A never-started timer is neither `Completed` nor `InProgress`;
/// a paused timer keeps its remaining frames and resumes.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
struct ShieldTimer {
    timer: CdTimer,
    started: bool,
}

impl ShieldTimer {
    fn start(&mut self, frame: i32, duration: i32) {
        self.timer.start(frame, duration);
        self.started = true;
    }

    fn stop(&mut self) {
        self.timer = CdTimer::default();
        self.started = false;
    }

    fn pause(&mut self, frame: i32) {
        if self.started {
            self.timer.pause(frame);
        }
    }

    fn resume_if_paused(&mut self, frame: i32) {
        if self.started && !timer_started(self.timer) {
            self.timer.resume(frame);
        }
    }

    fn has_started(self) -> bool {
        timer_started(self.timer)
    }

    fn completed(self, frame: i32) -> bool {
        self.has_started() && self.timer.expired(frame)
    }

    fn in_progress(self, frame: i32) -> bool {
        self.has_started() && !self.timer.expired(frame)
    }
}

/// The four `CDTimerClass` instances `ShieldClass` uses for recovery. The
/// timers are frame-anchored, so pausing stores the remaining frames and
/// resuming re-anchors, matching [`CdTimer`]'s native contract.
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
struct ShieldTimers {
    self_healing_combat_restart: ShieldTimer,
    self_healing: ShieldTimer,
    respawn_combat_restart: ShieldTimer,
    respawn: ShieldTimer,
}

impl ShieldTimers {
    fn pause_all(&mut self, frame: i32) {
        self.self_healing_combat_restart.pause(frame);
        self.self_healing.pause(frame);
        self.respawn_combat_restart.pause(frame);
        self.respawn.pause(frame);
    }

    fn resume_all_if_paused(&mut self, frame: i32) {
        self.self_healing_combat_restart.resume_if_paused(frame);
        self.self_healing.resume_if_paused(frame);
        self.respawn_combat_restart.resume_if_paused(frame);
        self.respawn.resume_if_paused(frame);
    }
}

/// One object's shield. `type_name` is the bound `[ShieldTypes]` entry; a
/// shield whose entry disappeared from the catalog keeps no behaviour.
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
struct ShieldState {
    type_name: String,
    hp: i32,
    online: bool,
    temporal: bool,
    self_healing_enabled: bool,
    /// `ShieldClass::LastBreakFrame`: retained state for the later
    /// `Shield.MinimumReplaceDelay`/idle-anim chain; no consumer in this one.
    last_break_frame: u32,
    timers: ShieldTimers,
    heat: i32,
    overload: ShieldTimer,
}

impl ShieldState {
    /// `ShieldClass`'s constructor: `SetHP(InitialStrength.Get(Strength))`,
    /// `Online=true`, no timer started.
    fn new(type_name: String, initial_strength: i32, strength: i32) -> Self {
        Self {
            type_name,
            hp: initial_strength.min(strength),
            online: true,
            temporal: false,
            self_healing_enabled: true,
            last_break_frame: 0,
            timers: ShieldTimers::default(),
            heat: 0,
            overload: ShieldTimer::default(),
        }
    }

    pub fn type_name(&self) -> &str {
        &self.type_name
    }

    pub fn is_active(&self, frame: i32) -> bool {
        self.hp > 0 && self.online && !self.overload.in_progress(frame)
    }

    /// `ShieldClass::IsBrokenAndNonRespawning`: no points and no recovery.
    pub fn broken_non_respawning(&self, ty: &ShieldType) -> bool {
        self.hp <= 0 && ty.respawn() == 0.0
    }
}

/// `CDTimerClass::HasStarted`.
fn timer_started(timer: CdTimer) -> bool {
    timer.start_frame() != PAUSED_START_FRAME
}

/// `ShieldClass::GetPercentageAmount`.
fn percentage_amount(status: f64, strength: i32) -> i32 {
    if status == 0.0 {
        return 0;
    }
    if (-1.0..=1.0).contains(&status) {
        (f64::from(strength) * status).round() as i32
    } else {
        status.trunc() as i32
    }
}

/// `GeneralUtils::SafeMultiply(int, double)`: truncate toward zero, clamp to
/// the signed 32-bit range.
fn safe_multiply(value: i32, multiplier: f64) -> i32 {
    let product = f64::from(value) * multiplier;
    if product > f64::from(i32::MAX) {
        i32::MAX
    } else if product < f64::from(i32::MIN) {
        i32::MIN
    } else {
        product as i32
    }
}

/// `Math::clamp(value, min, max)`: `value < min` first, then `value > max`.
/// Unlike `Ord::clamp`, a reversed min/max pair yields `min` instead of
/// panicking.
fn clamp_native(value: i32, min: i32, max: i32) -> i32 {
    if value < min {
        min
    } else if value > max {
        max
    } else {
        value
    }
}

pub(crate) fn evaluate<H: Host>(
    request: &Request,
    config: &Configuration,
    host: &H,
    overload_enabled: bool,
) -> Result<Plan, String> {
    let mut plan = Plan::unchanged(request);
    let state: Option<ShieldState> = if request.state.is_empty() {
        None
    } else {
        Some(serde_json::from_slice(&request.state).map_err(|e| format!("shield state: {e}"))?)
    };
    let (next, entity, frame) = match &request.context {
        Context::Attach(context) => {
            let next = config
                .objects
                .get(&context.entity.type_name.to_ascii_uppercase())
                .and_then(|name| config.catalog.entry(name))
                .filter(|ty| ty.strength() > 0)
                .map(|ty| {
                    ShieldState::new(ty.name().to_owned(), ty.initial_strength(), ty.strength())
                });
            (next, &context.entity, context.frame)
        }
        Context::Damage(context) => {
            let next = if let Some(state) = state {
                if let Some((next, remaining)) =
                    damage_stage(state, config, context, host, overload_enabled)?
                {
                    plan.damage = Some(remaining);
                    Some(next)
                } else {
                    serde_json::from_slice(&request.state).map_err(|e| e.to_string())?
                }
            } else {
                None
            };
            (next, &context.entity, context.frame)
        }
        Context::Tick(context) => {
            let next = state.and_then(|state| {
                tick(
                    state,
                    config,
                    &context.entity,
                    context.frame,
                    overload_enabled,
                )
            });
            (next, &context.entity, context.frame)
        }
        Context::Configure(_)
        | Context::ConfigureBoundary(_)
        | Context::ConfigureCountryRead
        | Context::ConfigureSuperRead(_)
        | Context::ConfigureFactionPass => {
            return Err("configuration must use the configuration hook".into());
        }
        Context::WeaponResource(_)
        | Context::TypeResource(_)
        | Context::MatchRules(_)
        | Context::ArmorResource(_)
        | Context::ProductionResource(_)
        | Context::EmpResource(_)
        | Context::FoundationResource(_)
        | Context::CameoResource(_)
        | Context::TypeSoundResource(_)
        | Context::SuperWeaponResource(_)
        | Context::StartupResources(_)
        | Context::EvaCatalog
        | Context::LongListReaders
        | Context::SpawnIdentity
        | Context::PrismSupportDefaults
        | Context::SoundSampleAdmission
        | Context::FactionResource(_)
        | Context::LoadingThemeResource(_)
        | Context::EffectResource(_)
        | Context::Effects(_) => {
            return Err("resource policy must use its own hook".into());
        }
    };
    plan.projection = Some(next.as_ref().map_or_else(Projection::default, |state| {
        projection(state, config, entity, frame)
    }));
    plan.state = next
        .map(|state| serde_json::to_vec(&state).map_err(|e| e.to_string()))
        .transpose()?
        .unwrap_or_default();
    Ok(plan)
}

fn shield_armor<'a>(ty: &'a ShieldType, target: &'a EntityContext) -> &'a str {
    if ty.inherit_armor_from_techno() {
        // Preserve the retained Phobos allow/disallow expression, including
        // its short circuit for an empty allow list.
        let allowed = ty.inherit_armor_allowed().is_empty()
            || ty
                .inherit_armor_allowed()
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&target.type_name))
                && !ty
                    .inherit_armor_disallowed()
                    .iter()
                    .any(|name| name.eq_ignore_ascii_case(&target.type_name));
        if allowed {
            return &target.armor;
        }
    }
    ty.armor()
}

fn verses(context: &DamageContext, armor: &str) -> f64 {
    // Unknown names resolve to the original None armor, the first registry item.
    context
        .verses
        .iter()
        .find(|entry| entry.armor.eq_ignore_ascii_case(armor))
        .or_else(|| context.verses.first())
        .map_or(1.0, |entry| f64::from_bits(entry.multiplier_bits))
}

fn penetrated(
    state: &ShieldState,
    ty: &ShieldType,
    rules: &ShieldWarheadRules,
    psychedelic: bool,
) -> bool {
    if !rules.penetration_allowed(state.type_name()) {
        return false;
    }
    if psychedelic {
        !ty.immune_to_berserk()
    } else {
        rules.penetrate
    }
}

/// Retained WarheadTypeExt::CanTargetHouse (Phobos Body.cpp:9..35):
/// a missing source passes; neutral, owner, ally and enemy gates run in order.
/// The host supplies relationship facts, never this compatibility decision.
fn can_target_house(rules: &ShieldWarheadRules, context: &DamageContext) -> bool {
    if context.source_owner.is_none() {
        return true;
    }
    if !rules.affects_neutral && context.target_is_neutral {
        return false;
    }
    if rules.affects_owner.unwrap_or(rules.affects_allies) && context.source_is_owner {
        return true;
    }
    if rules.affects_allies && context.source_is_ally {
        return !context.source_is_owner;
    }
    rules.affects_enemies && !context.source_is_ally
}

fn damage_stage<H: Host>(
    mut state: ShieldState,
    config: &Configuration,
    context: &DamageContext,
    host: &H,
    overload_enabled: bool,
) -> Result<Option<(ShieldState, i32)>, String> {
    if context.ignore_defenses {
        return Ok(None);
    }
    let Some(ty) = config.catalog.entry(state.type_name()) else {
        return Ok(None);
    };
    let fallback = ShieldWarheadRules::default();
    let warhead = config
        .warheads
        .get(&context.warhead_name.to_ascii_uppercase())
        .unwrap_or(&fallback);
    let frame = context.frame;
    let input = context.incoming;
    let active = state.is_active(frame);
    let before_hp = state.hp;
    // Native ShieldClass::ReceiveDamage, hook 0x701900+0x6. All side effects
    // remain in the returned state; preview and commit execute the same leaf.
    let shield_return = (|| -> Result<i32, String> {
        if !active
            || context.entity.being_warped_out
            || input == 0
            || context.entity.invulnerable
            || penetrated(&state, ty, warhead, context.warhead_psychedelic)
            || context.entity.immune
            || context.entity.type_immune && context.attacker_same_type_owner
        {
            return Ok(input);
        }
        let shield_armor = shield_armor(ty, &context.entity);
        let arms = verses(context, shield_armor);
        let mut armor_multiplier = 1.0;
        let (shield_damage, health_damage) =
            if can_target_house(warhead, context) && !context.warhead_temporal {
                let mut normalized = input;
                if input >= 0 && ty.apply_armor_mult(config.globals.apply_armor_mult()) {
                    armor_multiplier = f64::from_bits(context.armor_multiplier_bits);
                    normalized = if armor_multiplier == 0.0 {
                        0
                    } else {
                        ((f64::from(input) / armor_multiplier) as i32).max(0)
                    };
                }
                let calculated = host.damage_kernel(&DamageKernelInput {
                    damage: if input < 0 {
                        input.wrapping_neg()
                    } else {
                        normalized
                    },
                    cell_spread_bits: context.cell_spread_bits,
                    percent_at_max_bits: context.percent_at_max_bits,
                    multiplier_bits: arms.to_bits(),
                    distance_leptons: context.distance_leptons,
                    no_damage: context.no_damage,
                    max_damage: context.max_damage,
                })?;
                let calculated = if input < 0 {
                    calculated.wrapping_neg()
                } else {
                    calculated
                };
                let affects = warhead.affects(state.type_name());
                let absorb = if affects {
                    warhead
                        .absorb_percent
                        .unwrap_or_else(|| ty.absorb_percent())
                } else {
                    ty.absorb_percent()
                };
                let pass = if affects {
                    warhead.pass_percent.unwrap_or_else(|| ty.pass_percent())
                } else {
                    ty.pass_percent()
                };
                (
                    (f64::from(calculated) * absorb) as i32,
                    (f64::from(input) * pass) as i32,
                )
            } else {
                (0, 0)
            };
        let original_shield_damage = shield_damage;
        let minimum = warhead
            .received_minimum
            .unwrap_or_else(|| ty.received_damage_minimum());
        let maximum = warhead
            .received_maximum
            .unwrap_or_else(|| ty.received_damage_maximum());
        let shield_damage = clamp_native(
            shield_damage,
            safe_multiply(minimum, warhead.received_min_multiplier),
            safe_multiply(maximum, warhead.received_max_multiplier),
        );
        let remaining = if shield_damage > 0 {
            let residue = shield_damage.wrapping_sub(state.hp);
            if residue >= 0 {
                if ty.absorb_over_damage() {
                    break_shield(&mut state, ty, frame);
                    health_damage
                } else {
                    let raw = f64::from(original_shield_damage.wrapping_sub(state.hp))
                        * armor_multiplier
                        / arms;
                    let residue = raw.max(0.0) as i32;
                    break_shield(&mut state, ty, frame);
                    residue.wrapping_add(health_damage)
                }
            } else {
                state.hp = residue.wrapping_neg();
                health_damage
            }
        } else if shield_damage < 0 {
            let lost = ty.strength().wrapping_sub(state.hp);
            if lost == 0 {
                if verses(context, &context.entity.armor) * f64::from(input) > 0.0 {
                    0
                } else {
                    input
                }
            } else {
                let remaining = lost.wrapping_add(shield_damage);
                state.hp = if remaining < 0 {
                    ty.strength()
                } else {
                    state.hp.wrapping_sub(shield_damage)
                };
                0
            }
        } else {
            health_damage
        };
        Ok(remaining)
    })()?;
    // Native enclosing hook only writes a nonnegative result while active.
    let remaining = if active && shield_return >= 0 {
        shield_return
    } else {
        input
    };
    let armor = if state.is_active(frame)
        && !penetrated(&state, ty, warhead, context.warhead_psychedelic)
    {
        shield_armor(ty, &context.entity)
    } else {
        &context.entity.armor
    };
    let restart = active && remaining >= 0 && input > remaining
        || input > 0
            && !context.warhead_psychedelic
            && (input.wrapping_sub(remaining) > 0
                || verses(context, armor) * f64::from(input) >= 1.0);
    if restart {
        restart_recovery_in_combat(&mut state, ty, frame);
    }
    // This authored extension is enabled by the frozen INI selection.
    // It measures actually consumed shield points, not the unabsorbed packet.
    if overload_enabled {
        state.heat = state
            .heat
            .saturating_add(before_hp.saturating_sub(state.hp).max(0));
        if state.heat >= config.overload.limit && !state.overload.in_progress(frame) {
            state.overload.start(frame, config.overload.duration);
        }
        // A hit can restart either combat delay or recovery timer even while
        // absorption is disabled. Keep each restarted countdown frozen too.
        if state.overload.in_progress(frame) {
            state.timers.pause_all(frame);
        }
    }
    Ok(Some((state, remaining)))
}

/// ShieldClass::BreakShield and combat recovery timer setters, moved from
/// VERA20K sim/shield.rs (Phobos b90ab4bc Hooks.ReceiveDamage.cpp:153..177).
fn break_shield(state: &mut ShieldState, ty: &ShieldType, frame: i32) {
    state.hp = 0;
    if ty.respawn() != 0.0 {
        state.timers.respawn.start(frame, ty.respawn_rate());
    }
    state.timers.self_healing.stop();
    state.last_break_frame = frame as u32;
}

fn restart_recovery_in_combat(state: &mut ShieldState, ty: &ShieldType, frame: i32) {
    if ty.respawn_restart_in_combat() {
        let delay = ty.respawn_restart_in_combat_delay();
        if delay > 0 {
            state.timers.respawn_combat_restart.start(frame, delay);
            state.timers.respawn.stop();
        } else if state.hp <= 0 && ty.respawn_rate() != 0 {
            state.timers.respawn.start(frame, ty.respawn_rate());
        }
    }
    if ty.self_healing_restart_in_combat() {
        let delay = ty.self_healing_restart_in_combat_delay();
        if delay > 0 {
            state.timers.self_healing_combat_restart.start(frame, delay);
            state.timers.self_healing.stop();
        } else if state.hp > 0 && ty.self_healing_rate() != 0 {
            state
                .timers
                .self_healing
                .start(frame, ty.self_healing_rate());
        }
    }
}

fn tick(
    mut state: ShieldState,
    config: &Configuration,
    entity: &EntityContext,
    frame: i32,
    overload_enabled: bool,
) -> Option<ShieldState> {
    let Some(ty) = config.catalog.entry(state.type_name()) else {
        return Some(state);
    };
    if entity.in_limbo {
        return Some(state);
    }
    if entity.health <= 0 || !entity.alive {
        return None;
    }
    state.temporal = entity.temporal;
    if state.temporal {
        if state.hp <= 0 {
            state.timers.respawn.pause(frame);
        } else {
            state.timers.self_healing.pause(frame);
        }
        return Some(state);
    }
    if entity.chrono_warp_latch || entity.ai_frozen {
        return Some(state);
    }
    if overload_enabled {
        state.heat = state.heat.saturating_sub(config.overload.cooling).max(0);
        if state.overload.in_progress(frame) {
            return Some(state);
        }
        if state.overload.completed(frame) {
            state.overload.stop();
            state.timers.resume_all_if_paused(frame);
        }
    }
    if ty.powered() {
        let powered = !entity.deactivated && entity.powered;
        if !powered {
            state.online = false;
            if state.hp <= 0 {
                state.timers.respawn.pause(frame);
            } else {
                state.timers.self_healing.pause(frame);
            }
        } else if !state.online {
            state.online = true;
        }
    }
    if state.online && state.self_healing_enabled {
        if state.hp <= 0 {
            state.timers.respawn.resume_if_paused(frame);
            respawn(&mut state, ty, frame);
            self_healing(&mut state, ty, frame);
        } else {
            state.timers.self_healing.resume_if_paused(frame);
            self_healing(&mut state, ty, frame);
        }
    }
    Some(state)
}

fn self_healing(state: &mut ShieldState, ty: &ShieldType, frame: i32) {
    if state.hp <= 0 || state.timers.self_healing_combat_restart.in_progress(frame) {
        return;
    }
    let rate = ty.self_healing_rate();
    if state.timers.self_healing_combat_restart.completed(frame) {
        state.timers.self_healing.start(frame, rate);
        state.timers.self_healing_combat_restart.stop();
    }
    let amount = percentage_amount(ty.self_healing(), ty.strength());
    if amount != 0 {
        if !state.timers.self_healing.has_started() && (state.hp < ty.strength() || amount < 0) {
            state.timers.self_healing.start(frame, rate);
        }
        if state.timers.self_healing.completed(frame) {
            state.hp = state.hp.wrapping_add(amount);
            if state.hp > ty.strength() {
                state.hp = ty.strength();
                state.timers.self_healing.stop();
            } else if state.hp <= 0 {
                break_shield(state, ty, frame);
            } else {
                state.timers.self_healing.start(frame, rate);
            }
        }
    }
}

fn respawn(state: &mut ShieldState, ty: &ShieldType, frame: i32) {
    if state.hp > 0 || state.timers.respawn_combat_restart.in_progress(frame) {
        return;
    }
    let rate = ty.respawn_rate();
    if state.timers.respawn_combat_restart.completed(frame) {
        state.timers.respawn.start(frame, rate);
        state.timers.respawn_combat_restart.stop();
    }
    let amount = percentage_amount(ty.respawn(), ty.strength());
    if amount > 0 {
        if !state.timers.respawn.has_started() {
            state.timers.respawn.start(frame, rate);
        }
        if state.timers.respawn.completed(frame) {
            state.timers.respawn.stop();
            state.hp = amount;
        }
    }
}

fn pip_frame(building: bool, hp: i32, ty: &ShieldType, config: &Configuration) -> u32 {
    let local = ty.pips(building);
    let pips = if local[0] != -1 {
        local
    } else {
        config.globals.pips(building)
    };
    let strength = f64::from(ty.strength());
    let yellow = ty.condition_yellow(config.globals.condition_yellow());
    let red = ty.condition_red(config.globals.condition_red());
    let selected = if f64::from(hp) > yellow * strength && pips[0] != -1 {
        pips[0]
    } else if f64::from(hp) > red * strength && (pips[1] != -1 || pips[0] != -1) {
        if pips[1] == -1 { pips[0] } else { pips[1] }
    } else if pips[2] != -1 || pips[0] != -1 {
        if pips[2] == -1 { pips[0] } else { pips[2] }
    } else if building {
        5
    } else {
        16
    };
    selected.max(0) as u32
}

fn projection(
    state: &ShieldState,
    config: &Configuration,
    _entity: &EntityContext,
    frame: i32,
) -> Projection {
    let Some(ty) = config.catalog.entry(state.type_name()) else {
        return Projection::default();
    };
    let mut result = Projection::default();
    if !state.broken_non_respawning(ty) {
        result.meters.push(Meter {
            id: "shield".into(),
            current: state.hp,
            maximum: ty.strength(),
            visible: !(ty.pips_hide_if_no_strength() && state.hp <= 0),
            active: state.is_active(frame),
            color_rgb: [80, 170, 255],
            pip_frame: pip_frame(false, state.hp, ty, config),
            building_pip_frame: pip_frame(true, state.hp, ty, config),
            building_empty_frame: ty
                .pips_building_empty()
                .or_else(|| config.globals.pips_building_empty())
                .unwrap_or(0)
                .max(0) as u32,
            bracket_delta: ty.bracket_delta(),
            ..Default::default()
        });
    }
    if state.is_active(frame) && ty.has_tint() {
        let (r, g, b) = ty.tint_color().unwrap_or((0, 0, 0));
        result.tint = Some(Tint {
            color_rgb: [r, g, b],
            intensity: (ty.tint_intensity() * 1000.0) as i32,
            visible_to_houses: ty.tint_visible_to_houses().bits(),
            ..Default::default()
        });
    }
    result
}
