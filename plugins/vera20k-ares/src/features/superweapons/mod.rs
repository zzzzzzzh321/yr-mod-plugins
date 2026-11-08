//! Retained SWTypeExt configuration. The native registry owns reference
//! allocation; the engine keeps all live Super/timer/wallet state.
mod config;
mod nuclear;
mod targeting;

use crate::support::{ConfigView, Host};
pub(crate) use config::{DeliveryOwner, Kind, TypeState};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    self as api, ConfigureContext, ConfigureSuperReadContext, ProductionObjectKind,
    SuperReferenceRead, SuperWeaponReadSite,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    pub(crate) types: BTreeMap<u32, TypeState>,
    /// BuildingType ordinals; ordered vectors keep duplicates.
    pub(crate) providers: BTreeMap<u32, Vec<u32>>,
    pub(crate) sound_source_hash: Option<u64>,
    pub(crate) eva_source_hash: Option<u64>,
    /// Independent of .35's custom_payloads flag. Historical nonempty
    /// configuration does not acquire the Nuclear handler on later reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub(crate) nuclear: Option<bool>,
}

impl Configuration {
    pub(crate) fn for_new_match() -> Self {
        Self {
            nuclear: Some(true),
            ..Self::default()
        }
    }

    fn retain_sources<H: Host>(&mut self, view: &ConfigView<'_, H>) -> Result<(), String> {
        let source = view.required_sound_source_hash()?;
        if self
            .sound_source_hash
            .is_some_and(|previous| previous != source)
        {
            return Err("Super sound source changed without a configuration boundary".into());
        }
        self.sound_source_hash = Some(source);
        let source = view.eva_source_hash()?;
        if self
            .eva_source_hash
            .is_some_and(|previous| previous != source)
        {
            return Err("Super EVA source changed without a configuration boundary".into());
        }
        self.eva_source_hash = Some(source);
        Ok(())
    }

    fn register(&mut self, ordinal: u32, stored_id: &str) -> Result<&mut TypeState, String> {
        let state = self
            .types
            .entry(ordinal)
            .or_insert_with(|| TypeState::new(stored_id));
        if state.stored_id != stored_id {
            return Err("SuperWeapon read identity changed without a Type reset".into());
        }
        Ok(state)
    }

    /// Only this synchronous original traversal visit reads Type extensions.
    /// A successful ordinary Configure never replays these visits.
    pub(crate) fn configure_read<H: Host>(
        &mut self,
        context: &ConfigureSuperReadContext,
        view: &ConfigView<'_, H>,
    ) -> Result<SuperReferenceRead, String> {
        let physical = view.superweapon_registry_view()?;
        let point = physical
            .reads
            .get(context.read as usize)
            .ok_or("missing Super read point")?;
        let SuperWeaponReadSite::Type { ordinal, section } = &point.site else {
            return Err("Super Type read selected a Building point".into());
        };
        if *ordinal != context.ordinal
            || section.is_some() != context.body_read
            || physical.superweapons.get(*ordinal as usize) != Some(&context.stored_id)
        {
            return Err("Super Type read differs from its physical identity".into());
        }
        self.retain_sources(view)?;
        let nuclear_enabled = self.nuclear == Some(true);
        let state = self.register(context.ordinal, &context.stored_id)?;
        state.initialize_eva_default(view)?;
        let initialize_nuclear = nuclear_enabled
            && !state.initialized
            && context.body_read
            && context.effective_type.as_deref().and_then(Kind::parse) == Some(Kind::Retail(0));
        //10033284 ->10033347 consumes a first absent body with mode0.
        if !state.initialized {
            if context.body_read {
                state.targeting.mode = context
                    .effective_type
                    .as_deref()
                    .map_or(0, targeting::handler_initial_mode);
                if initialize_nuclear {
                    state.nuclear = Some(nuclear::Nuclear::initialize(
                        context,
                        section.as_deref().ok_or("missing initial Nuclear body")?,
                        view,
                    )?);
                }
            }
            state.initialized = true;
        }
        if let Some(owner) = section {
            state.read(context, owner, view)?;
            let selected = nuclear_enabled && state.selects_nuclear();
            if let Some(nuclear) = &mut state.nuclear {
                nuclear.read(context, owner, selected, view)?;
            }
        }
        // Common10033701 reads WH before handler dispatch10033ABD. The new
        // cohort must retain it even while a custom Type temporarily selects
        // another handler. Only the Nuclear-specific payload read depends on
        // that selection; historical cohorts keep their earlier Generic gate.
        Ok(
            if context.body_read && nuclear_enabled && state.selects_nuclear() {
                SuperReferenceRead::NuclearFromCurrentSection {
                    initialize: initialize_nuclear,
                }
            } else if context.body_read
                && (nuclear_enabled
                    || context
                        .effective_type
                        .as_deref()
                        .is_some_and(|kind| kind.eq_ignore_ascii_case("GenericWarhead")))
            {
                SuperReferenceRead::WarheadFromCurrentSection
            } else {
                SuperReferenceRead::Retain
            },
        )
    }

    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        if context.final_pass {
            return Ok(());
        }
        let physical = view.superweapon_registry_view()?;
        if !physical.superweapons.is_empty() {
            self.retain_sources(view)?;
        }
        // Late constructor-only definitions have defaults but have not
        // reached Initialize. This does not invent a Type body/visit.
        for (ordinal, name) in physical.superweapons.iter().enumerate() {
            self.register(u32::try_from(ordinal).map_err(|e| e.to_string())?, name)?
                .initialize_eva_default(view)?;
        }
        for (read, point) in physical.reads.iter().enumerate() {
            let SuperWeaponReadSite::Building { object, section } = &point.site else {
                continue;
            };
            if object.category != ProductionObjectKind::Building {
                return Err("Super provider read is not a Building".into());
            }
            let value: Option<Vec<u32>> = view.superweapon_read(
                u32::try_from(read).map_err(|e| e.to_string())?,
                section,
                "SuperWeapons",
                "super-provider-list",
                (),
            )?;
            if let Some(value) = value {
                self.providers.insert(object.ordinal, value);
            }
        }
        Ok(())
    }

    pub(crate) fn query(&self) -> Result<api::SuperWeaponResourcePolicy, String> {
        let mut definitions = Vec::with_capacity(self.types.len());
        for (&ordinal, state) in &self.types {
            let payload = match state.kind {
                Kind::GenericWarhead => {
                    let (automatic_target, automatic_house) = state.targeting.automatic()?;
                    api::SuperWeaponPayloadPolicy::GenericWarhead {
                        damage: state.generic.damage,
                        targeting: api::SuperWeaponTargetingPolicy {
                            manual_target: state.targeting.manual_target,
                            manual_house: state.targeting.manual_house,
                            automatic_target,
                            automatic_house,
                            offensive_ai: state.targeting.offensive(),
                            fire_into_shroud: state.generic.fire_into_shroud,
                            range_minimum: state.generic.range_minimum_bits,
                            range_maximum: state.generic.range_maximum_bits,
                        },
                    }
                }
                Kind::UnitDelivery => {
                    let delivery = state
                        .delivery
                        .as_ref()
                        .ok_or("UnitDelivery dispatch lacks a reached descriptor")?;
                    api::SuperWeaponPayloadPolicy::UnitDelivery {
                        types: delivery.types.clone(),
                        deferment: delivery.deferment,
                        owner: match delivery.owner {
                            DeliveryOwner::Invoker => api::SuperWeaponDeliveryOwner::Invoker,
                            DeliveryOwner::Neutral => api::SuperWeaponDeliveryOwner::Neutral,
                            DeliveryOwner::Special => api::SuperWeaponDeliveryOwner::Special,
                            DeliveryOwner::Civilian => api::SuperWeaponDeliveryOwner::Civilian,
                        },
                        base_normal: delivery.base_normal,
                        action: state
                            .action
                            .clone()
                            .ok_or("UnitDelivery dispatch lacks an action")?,
                        activation_sound: state.activation_sound.clone(),
                    }
                }
                Kind::Retail(_) if self.nuclear == Some(true) && state.selects_nuclear() => state
                    .nuclear
                    .as_ref()
                    .ok_or("Nuclear dispatch lacks its reached descriptor")?
                    .policy(state.activation_sound.clone()),
                Kind::Retail(_) | Kind::NoEffect => api::SuperWeaponPayloadPolicy::Retail,
            };
            definitions.push(api::SuperWeaponDeclaration {
                ordinal,
                stored_id: state.stored_id.clone(),
                availability: api::SuperWeaponAvailabilityPolicy {
                    allow_player: state.availability.allow_player,
                    allow_ai: state.availability.allow_ai,
                    required_houses: state.availability.required_houses,
                    forbidden_houses: state.availability.forbidden_houses,
                    shots: state.availability.shots,
                    auxiliary: state.availability.auxiliary.clone(),
                    negative: state.availability.negative.clone(),
                },
                charge: api::SuperWeaponChargePolicy {
                    initial_ready: state.charge.initial_ready,
                    virtual_charge: state.charge.virtual_charge,
                },
                constraints: state.targeting.effective_constraints()?,
                use_ai_targeting: state.use_ai_targeting,
                money_amount: state.money_amount,
                insufficient_funds_eva: state.insufficient_funds_eva.clone(),
                presentation: api::SuperWeaponPresentationPolicy {
                    animation: state.presentation.animation.clone(),
                    height: state.presentation.height,
                    visibility: state.presentation.visibility,
                    sound: state.presentation.sound.clone(),
                },
                payload,
            });
        }
        let policy = api::SuperWeaponResourcePolicy {
            semantics: api::SuperWeaponSemantics {
                additional_providers: true,
                extended_availability: true,
                definition_building_animation: true,
                grant_charge_history: true,
                paid_launches: true,
                custom_payloads: true,
                launch_presentation: true,
            },
            sound_source_hash: self.sound_source_hash,
            eva_source_hash: self.eva_source_hash,
            definitions,
            providers: self
                .providers
                .iter()
                .map(
                    |(&ordinal, additional)| api::SuperWeaponProviderDeclaration {
                        object: api::ProductionTypeRef {
                            category: ProductionObjectKind::Building,
                            ordinal,
                        },
                        additional: additional.clone(),
                    },
                )
                .collect(),
        };
        policy.validate()?;
        Ok(policy)
    }
}
