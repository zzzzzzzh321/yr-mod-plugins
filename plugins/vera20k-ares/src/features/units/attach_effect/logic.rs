//! Installed Ares Attach10058520, Update10058E00, Remove100588E0 and
//! RemoveType100589F0. Actions address the host's sole live instance vector;
//! the temporary visit snapshot below is never retained by the component.
use super::{Configuration, Descriptor};
use vera_mod_api::{
    EffectAction, EffectContext, EffectControl, EffectDescriptor, EffectEvent, EffectInstanceView,
    EffectPlan, EffectResourceContext, EffectResourceKind, relationship::permits_house,
};

fn settings<'a>(
    context: &EffectResourceContext,
    config: &'a Configuration,
) -> Option<&'a Descriptor> {
    match context.kind.user() {
        Some(user) => config.type_effect(&context.name, user),
        None => config
            .warhead_effect(&context.name)
            .map(|warhead| &warhead.effect),
    }
}

fn set_control(plan: &mut EffectPlan, control: EffectControl) {
    if plan.control != control {
        plan.actions.push(EffectAction::SetControl { control });
        plan.control = control;
    }
}

fn set_hidden(plan: &mut EffectPlan, hidden: bool) {
    if plan.animations_hidden != hidden {
        plan.actions
            .push(EffectAction::SetAnimationsHidden { hidden });
        plan.animations_hidden = hidden;
    }
}

pub(crate) fn resource(
    context: &EffectResourceContext,
    config: &Configuration,
) -> Option<EffectDescriptor> {
    // A registered Techno whose reader suffix was never reached still has
    // the native constructor descriptor. Warheads with Duration0 need no
    // impact subscription (10052716..10052718's complete early return).
    let defaults = Descriptor::default();
    let settings = match context.kind.user() {
        Some(user) => config.type_effect(&context.name, user).unwrap_or(&defaults),
        None => {
            let value = &config.warhead_effect(&context.name)?.effect;
            if value.duration == 0 {
                return None;
            }
            value
        }
    };
    if settings.temporal_hides_anim {
        return None;
    }
    Some(EffectDescriptor {
        firepower_multiplier_bits: settings.firepower_multiplier_bits,
        armor_multiplier_bits: settings.armor_multiplier_bits,
        rof_multiplier_bits: settings.rof_multiplier_bits,
        speed_multiplier_bits: settings.speed_multiplier_bits,
        cloakable: settings.cloakable,
        animation: settings.animation.clone(),
    })
}

/// Shared Attach10058520: IC admission precedes refresh; only a new instance
/// aggregates, after animation creation. Refresh preserves the first source.
fn attach(
    context: &EffectContext,
    entries: &[EffectInstanceView],
    resource: &EffectResourceContext,
    source_owner: Option<String>,
    settings: &Descriptor,
    plan: &mut EffectPlan,
) -> bool {
    if context.entity.invulnerable && !settings.penetrates_iron_curtain {
        return false;
    }
    let repeated = (!settings.cumulative)
        .then(|| entries.iter().position(|entry| entry.resource == *resource))
        .flatten();
    let can_create = settings.animation.is_some() && !context.cloaked_or_cloaking;
    if let Some(index) = repeated {
        plan.actions.push(EffectAction::SetRemaining {
            index,
            remaining: settings.duration,
        });
        if can_create && settings.anim_reset_on_reapply {
            plan.actions.push(EffectAction::CreateAnimation { index });
        }
    } else {
        let index = entries.len();
        plan.actions.push(EffectAction::Append {
            resource: resource.clone(),
            remaining: settings.duration,
            source_owner,
        });
        if can_create {
            plan.actions.push(EffectAction::CreateAnimation { index });
        }
        plan.actions.push(EffectAction::RecalculateStats);
    }
    //100586A4..100586C0: the shared tail only changes cloak states1/2.
    if settings.force_decloak && context.cloaked_or_cloaking {
        plan.actions.push(EffectAction::Decloak);
    }
    true
}

pub(crate) fn evaluate(context: &EffectContext, config: &Configuration) -> EffectPlan {
    let mut plan = EffectPlan::unchanged(context);
    match &context.event {
        EffectEvent::Apply {
            resource,
            source_owner,
            source_is_owner,
            source_is_ally,
            affects_allies,
            verses_bits,
        } => {
            let Some(warhead) = config
                .warhead_effect(&resource.name)
                .filter(|_| resource.kind == EffectResourceKind::Warhead)
            else {
                return plan;
            };
            let settings = &warhead.effect;
            //10052700's victim rung. Retain the current engine's health<=0
            // boundary; the canonical host collector owns ground admission.
            if settings.duration == 0
                || settings.temporal_hides_anim
                || context.entity.in_limbo
                || !context.entity.alive
                || context.entity.health <= 0
                || !permits_house(
                    source_owner.is_some(),
                    *source_is_owner,
                    *source_is_ally,
                    *affects_allies,
                    warhead.affects_enemies,
                    warhead.affects_owner,
                )
                || f64::from_bits(*verses_bits).abs() < 0.001
            {
                return plan;
            }
            attach(
                context,
                &context.entries,
                resource,
                source_owner.clone(),
                settings,
                &mut plan,
            );
        }
        EffectEvent::Visit => visit(context, config, &mut plan),
        EffectEvent::EnterLimbo => {
            if context.entries.is_empty() {
                return plan;
            }
            // Remove100588E0 retires every animation before Discard removal.
            //100588F4 publishes the hidden flag before retiring references.
            set_hidden(&mut plan, true);
            for index in 0..context.entries.len() {
                plan.actions.push(EffectAction::RetireAnimation { index });
            }
            let mut removed = 0;
            for (index, entry) in context.entries.iter().enumerate() {
                if settings(&entry.resource, config).is_some_and(|value| value.discard_on_entry) {
                    plan.actions.push(EffectAction::Remove {
                        index: index - removed,
                    });
                    removed += 1;
                }
            }
            if removed != 0 {
                plan.actions.push(EffectAction::RecalculateStats);
            }
        }
        EffectEvent::TypeChanged { previous } => {
            // Conversion10043792 -> RemoveType100589F0 retains the counter,
            // resets applied/disabled, and removes only the first old type item.
            let counter = match context.control {
                EffectControl::Waiting(value)
                | EffectControl::Active(value)
                | EffectControl::Disabled(value) => value,
            };
            set_control(&mut plan, EffectControl::Waiting(counter));
            if let Some(index) = context
                .entries
                .iter()
                .position(|entry| entry.resource == *previous)
            {
                // Conversion's existing owner erases first, then releases
                // the removed item's animation through Remove's host owner.
                plan.actions.push(EffectAction::Remove { index });
            }
            // The conversion caller recomputes even if there was no old item.
            plan.actions.push(EffectAction::RecalculateStats);
        }
    }
    plan
}

fn visit(context: &EffectContext, config: &Configuration, plan: &mut EffectPlan) {
    //10058E11..10058E32: these gates pause the entire visit, not only timers.
    if context.entity.in_limbo || context.entity.chrono_warp_latch || context.inside_transport {
        return;
    }
    if context.entries.is_empty() && !matches!(context.control, EffectControl::Waiting(_)) {
        return;
    }
    if !context.entries.is_empty() && context.cloaked_or_cloaking != context.animations_hidden {
        for (index, entry) in context.entries.iter().enumerate() {
            if context.cloaked_or_cloaking {
                plan.actions.push(EffectAction::RetireAnimation { index });
            } else if settings(&entry.resource, config)
                .is_some_and(|value| value.animation.is_some())
            {
                plan.actions.push(EffectAction::CreateAnimation { index });
            }
        }
        //10058E7F/10058EAD follow the whole create/retire loop.
        set_hidden(plan, context.cloaked_or_cloaking);
    }

    let own_resource =
        EffectResourceKind::for_user(context.user).map(|kind| EffectResourceContext {
            kind,
            name: context.entity.type_name.clone(),
        });
    let defaults = Descriptor::default();
    let own_settings = own_resource.as_ref().map(|_| {
        config
            .type_effect(&context.entity.type_name, context.user)
            .unwrap_or(&defaults)
    });
    let mut entries = context.entries.clone();
    //10058EC0..10058EF7 updates all durations before expiry/compaction calls.
    for (index, entry) in entries.iter_mut().enumerate() {
        let mut remaining = entry.remaining;
        if remaining > 0 {
            remaining -= 1;
        }
        if own_resource.as_ref() == Some(&entry.resource) {
            if context.entity.deactivated {
                remaining = 0;
            }
            if remaining == 0
                && let Some(settings) = own_settings
            {
                //10058EE6..10058EEC precede this item's remaining write.
                set_control(plan, EffectControl::Waiting(settings.delay));
            }
        }
        if remaining != entry.remaining {
            plan.actions
                .push(EffectAction::SetRemaining { index, remaining });
            entry.remaining = remaining;
        }
    }
    let mut index = 0;
    let mut expired = false;
    while index < entries.len() {
        if entries[index].remaining == 0 {
            plan.actions.push(EffectAction::RetireAnimation { index });
            plan.actions.push(EffectAction::Remove { index });
            entries.remove(index);
            expired = true;
        } else {
            index += 1;
        }
    }
    if expired {
        plan.actions.push(EffectAction::RecalculateStats);
    }

    //10058F82..10058FE7 follows compaction and its one aggregate. Zero Delay
    // may reapply a self effect in this same visit, using the shortened list.
    let (EffectControl::Waiting(counter), Some(settings), Some(resource)) =
        (plan.control, own_settings, own_resource)
    else {
        return;
    };
    if settings.duration == 0 {
        set_control(plan, EffectControl::Disabled(counter));
        return;
    }
    if settings.temporal_hides_anim {
        return;
    }
    let delay = if counter == i32::MAX {
        settings.initial_delay
    } else {
        counter
    };
    if delay > 0 {
        set_control(plan, EffectControl::Waiting(delay - 1));
    } else if delay == 0
        && !context.entity.deactivated
        && attach(
            context,
            &entries,
            &resource,
            Some(context.entity.owner.clone()),
            settings,
            plan,
        )
    {
        //10058FCE follows the complete Attach call, including its callbacks.
        set_control(plan, EffectControl::Active(counter));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::units::attach_effect::WarheadDescriptor;
    use std::collections::BTreeMap;
    use vera_mod_api::{EntityContext, WEAPON_USER_UNIT};

    fn configured(types: &[(&str, Descriptor)], warheads: &[(&str, Descriptor)]) -> Configuration {
        let objects = BTreeMap::from([(
            WEAPON_USER_UNIT,
            types
                .iter()
                .map(|(name, value)| (name.to_string(), value.clone()))
                .collect::<BTreeMap<_, _>>(),
        )]);
        let warheads = warheads
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    WarheadDescriptor {
                        effect: value.clone(),
                        ..Default::default()
                    },
                )
            })
            .collect::<BTreeMap<_, _>>();
        serde_json::from_value(serde_json::json!({"objects":objects,"warheads":warheads})).unwrap()
    }

    fn id(kind: EffectResourceKind, name: &str) -> EffectResourceContext {
        EffectResourceContext {
            kind,
            name: name.into(),
        }
    }

    fn entry(resource: EffectResourceContext, remaining: i32) -> EffectInstanceView {
        EffectInstanceView {
            resource,
            remaining,
            source_owner: Some("FIRST".into()),
            has_animation: true,
        }
    }

    fn context(event: EffectEvent, entries: Vec<EffectInstanceView>) -> EffectContext {
        EffectContext {
            frame: 10,
            entity: EntityContext {
                id: 1,
                type_name: "TANK".into(),
                owner: "TARGET".into(),
                armor: "none".into(),
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
            },
            user: WEAPON_USER_UNIT,
            inside_transport: false,
            cloaked_or_cloaking: false,
            entries,
            control: EffectControl::Waiting(i32::MAX),
            animations_hidden: false,
            event,
        }
    }

    fn apply(resource: EffectResourceContext) -> EffectEvent {
        EffectEvent::Apply {
            resource,
            source_owner: Some("LATER".into()),
            source_is_owner: false,
            source_is_ally: false,
            affects_allies: false,
            verses_bits: 1.0f64.to_bits(),
        }
    }

    #[test]
    fn refresh_preserves_source_while_cumulative_hit_appends_before_aggregation() {
        let refresh = Descriptor {
            duration: 7,
            animation: Some("FX".into()),
            anim_reset_on_reapply: true,
            force_decloak: true,
            ..Default::default()
        };
        let cumulative = Descriptor {
            cumulative: true,
            ..refresh.clone()
        };
        let cfg = configured(&[], &[("HIT", refresh), ("STACK", cumulative)]);
        let hit = id(EffectResourceKind::Warhead, "HIT");
        let mut ctx = context(apply(hit.clone()), vec![entry(hit, 2)]);
        ctx.cloaked_or_cloaking = true;
        let plan = evaluate(&ctx, &cfg);
        plan.validate(&ctx).unwrap();
        assert_eq!(
            plan.actions,
            vec![
                EffectAction::SetRemaining {
                    index: 0,
                    remaining: 7
                },
                EffectAction::Decloak
            ]
        );

        let stack = id(EffectResourceKind::Warhead, "STACK");
        let ctx = context(apply(stack.clone()), vec![entry(stack.clone(), 2)]);
        let plan = evaluate(&ctx, &cfg);
        plan.validate(&ctx).unwrap();
        assert_eq!(
            plan.actions,
            vec![
                EffectAction::Append {
                    resource: stack,
                    remaining: 7,
                    source_owner: Some("LATER".into())
                },
                EffectAction::CreateAnimation { index: 1 },
                EffectAction::RecalculateStats,
            ]
        );
    }

    #[test]
    fn expiry_compacts_before_same_visit_self_reapply_and_transport_pauses_all_actions() {
        let effect = Descriptor {
            duration: 3,
            animation: Some("FX".into()),
            ..Default::default()
        };
        let cfg = configured(
            &[("TANK", effect.clone())],
            &[("EXPIRE", effect.clone()), ("KEEP", effect)],
        );
        let own = id(EffectResourceKind::Unit, "TANK");
        let mut ctx = context(
            EffectEvent::Visit,
            vec![
                entry(id(EffectResourceKind::Warhead, "EXPIRE"), 1),
                entry(own.clone(), 1),
                entry(id(EffectResourceKind::Warhead, "KEEP"), -1),
            ],
        );
        ctx.control = EffectControl::Active(31);
        let plan = evaluate(&ctx, &cfg);
        plan.validate(&ctx).unwrap();
        assert_eq!(plan.control, EffectControl::Active(0));
        assert_eq!(
            plan.actions,
            vec![
                EffectAction::SetRemaining {
                    index: 0,
                    remaining: 0
                },
                EffectAction::SetControl {
                    control: EffectControl::Waiting(0)
                },
                EffectAction::SetRemaining {
                    index: 1,
                    remaining: 0
                },
                EffectAction::RetireAnimation { index: 0 },
                EffectAction::Remove { index: 0 },
                EffectAction::RetireAnimation { index: 0 },
                EffectAction::Remove { index: 0 },
                EffectAction::RecalculateStats,
                EffectAction::Append {
                    resource: own,
                    remaining: 3,
                    source_owner: Some("TARGET".into())
                },
                EffectAction::CreateAnimation { index: 1 },
                EffectAction::RecalculateStats,
                EffectAction::SetControl {
                    control: EffectControl::Active(0)
                },
            ]
        );
        ctx.inside_transport = true;
        assert_eq!(evaluate(&ctx, &cfg), EffectPlan::unchanged(&ctx));
        // Registered types without a reached reader retain the constructor's
        // Duration0: one visit disables self work without a persistent guest state.
        ctx.inside_transport = false;
        ctx.entries.clear();
        ctx.entity.type_name = "CTOR_ONLY".into();
        ctx.control = EffectControl::Waiting(i32::MAX);
        let plan = evaluate(&ctx, &cfg);
        plan.validate(&ctx).unwrap();
        assert_eq!(plan.control, EffectControl::Disabled(i32::MAX));
        assert_eq!(
            resource(&id(EffectResourceKind::Unit, "CTOR_ONLY"), &cfg)
                .unwrap()
                .firepower_multiplier_bits,
            1.0f64.to_bits()
        );
    }

    #[test]
    fn limbo_discard_and_type_change_keep_domain_identity_and_counter() {
        let ordinary = Descriptor {
            duration: 3,
            ..Default::default()
        };
        let discard = Descriptor {
            discard_on_entry: true,
            ..ordinary.clone()
        };
        let cfg = configured(&[("TANK", discard)], &[("TANK", ordinary)]);
        let own = id(EffectResourceKind::Unit, "TANK");
        let mut ctx = context(
            EffectEvent::EnterLimbo,
            vec![
                entry(id(EffectResourceKind::Warhead, "TANK"), 3),
                entry(own.clone(), 3),
                entry(own.clone(), 4),
            ],
        );
        ctx.control = EffectControl::Disabled(9);
        let plan = evaluate(&ctx, &cfg);
        plan.validate(&ctx).unwrap();
        assert!(plan.animations_hidden);
        assert_eq!(plan.control, ctx.control);
        assert_eq!(
            plan.actions,
            vec![
                EffectAction::SetAnimationsHidden { hidden: true },
                EffectAction::RetireAnimation { index: 0 },
                EffectAction::RetireAnimation { index: 1 },
                EffectAction::RetireAnimation { index: 2 },
                EffectAction::Remove { index: 1 },
                EffectAction::Remove { index: 1 },
                EffectAction::RecalculateStats,
            ]
        );
        ctx.event = EffectEvent::TypeChanged { previous: own };
        let plan = evaluate(&ctx, &cfg);
        plan.validate(&ctx).unwrap();
        assert_eq!(plan.control, EffectControl::Waiting(9));
        assert_eq!(
            plan.actions,
            vec![
                EffectAction::SetControl {
                    control: EffectControl::Waiting(9)
                },
                EffectAction::Remove { index: 1 },
                EffectAction::RecalculateStats,
            ]
        );
    }
}
