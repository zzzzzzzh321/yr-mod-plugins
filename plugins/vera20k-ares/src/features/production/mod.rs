//! Frozen Ares production declarations. Live House counters, installed slots,
//! factory queues and permanent plan acquisition have their sole host owners.
mod factories;
mod groups;
mod registry;

use crate::support::{ConfigView, Host};
use factories::ObjectState;
use groups::Groups;
use registry::Registry;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    ConfigureContext, ProductionBuildLimitRule, ProductionObjectKind, ProductionObjectPolicy,
    ProductionPrerequisiteRule, ProductionReadSite, ProductionResourceContext,
    ProductionResourcePolicy, ProductionSemantics,
};

#[derive(Default, Serialize, Deserialize)]
struct GlobalControls {
    phase: i32,
    ai_bypass_build_limit: [bool; 3],
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    globals: GlobalControls,
    groups: Groups,
    objects: BTreeMap<ProductionObjectKind, BTreeMap<String, ObjectState>>,
}

impl Configuration {
    pub(crate) fn reset_types(&mut self) {
        // Scenario TypeReset6876AC deletes Type-bound declarations while
        // RulesExt+14A/+14B/+14C and its independent phase survive.
        self.groups = Groups::default();
        self.objects.clear();
    }

    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        if context.final_pass {
            return Ok(());
        }
        let physical = view.rules_registry_view()?;
        let registry = Registry::new(&physical);
        let mut group_lookup = self.groups.lookup();
        for read in &physical.reads {
            registry.validate(read.prefix)?;
            match read.site {
                ProductionReadSite::ProcessEntry => {
                    if (0..=2).contains(&self.globals.phase) {
                        if self.globals.phase == 0 {
                            self.globals.phase = 1;
                        }
                        // RulesExt1002B490/1002B370: the host's canonical
                        // prefix parser writes2/1/0 and stops at first invalid.
                        self.globals.ai_bypass_build_limit = view
                            .section_or_empty("GlobalControls")
                            .read_bool_prefix_reverse3(
                                "AllowBypassBuildLimit",
                                self.globals.ai_bypass_build_limit,
                            );
                        self.globals.phase = 2;
                    }
                }
                ProductionReadSite::BeforeTypeData => {
                    group_lookup = self.groups.read(view, &registry, read.prefix);
                }
                ProductionReadSite::TechnoPrerequisite { object }
                | ProductionReadSite::TechnoExtension { object }
                | ProductionReadSite::BuildingExtension { object, .. } => {
                    let name = registry.name(object)?;
                    let section = view.section_or_empty(name);
                    let mut state = self
                        .objects
                        .get_mut(&object.category)
                        .and_then(|members| members.remove(name))
                        .unwrap_or_default();
                    match read.site {
                        ProductionReadSite::TechnoPrerequisite { .. } => {
                            //7141C5/71424B run after Dock/DeploysInto, using
                            // that exact Building prefix, before the TypeExt.
                            if let Some(tokens) = section.read_comma_list("Prerequisite", 2048) {
                                state.declarations.prerequisites =
                                    groups::parse(tokens, &group_lookup, &registry, read.prefix);
                            }
                            if let Some(tokens) =
                                section.read_comma_list("PrerequisiteOverride", 2048)
                            {
                                state.declarations.prerequisite_override =
                                    groups::parse(tokens, &group_lookup, &registry, read.prefix);
                            }
                        }
                        ProductionReadSite::TechnoExtension { .. } => {
                            state.read_techno(&section, &registry, read.prefix)
                        }
                        ProductionReadSite::BuildingExtension {
                            factory,
                            unit_repair,
                            ..
                        } => state.read_building(&section, factory, unit_repair),
                        _ => unreachable!("object read site matched above"),
                    }
                    if !state.is_default() {
                        self.objects
                            .entry(object.category)
                            .or_default()
                            .insert(name.into(), state);
                    }
                }
            }
        }
        self.objects.retain(|_, members| !members.is_empty());
        Ok(())
    }

    pub(crate) fn query(&self, context: &ProductionResourceContext) -> ProductionResourcePolicy {
        match context {
            ProductionResourceContext::Catalog => ProductionResourcePolicy::Catalog {
                groups: self.groups.query(),
                semantics: ProductionSemantics {
                    prerequisite_rule: ProductionPrerequisiteRule::ActiveAndInstalled,
                    build_limit_rule: ProductionBuildLimitRule::TrackedCapacity,
                    override_uses_groups: true,
                    require_factory_for_build: true,
                    ai_checks_build_limit: true,
                    negative_one_tech_level_human_only: true,
                    campaign_ignores_shell_superweapon_option: true,
                    factory_checks_limit_before_search: true,
                    factory_respects_explicit_sources: true,
                    factory_retains_offline_status: true,
                    deactivated_factory_is_offline: true,
                    factory_honors_stop_first: true,
                    factory_country_plans: true,
                },
                ai_bypass_build_limit: self.globals.ai_bypass_build_limit,
            },
            ProductionResourceContext::Object { category, name } => {
                ProductionResourcePolicy::Object {
                    policy: self
                        .objects
                        .get(category)
                        .and_then(|members| members.get(name))
                        .map_or_else(ProductionObjectPolicy::default, ObjectState::query),
                }
            }
        }
    }
}
