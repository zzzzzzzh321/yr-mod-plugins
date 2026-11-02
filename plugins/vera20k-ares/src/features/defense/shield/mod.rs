//! Shield feature boundary. This module owns bindings and frozen shield
//! configuration; logic owns the per-object state and its ordered transitions.
mod config;
mod logic;

use crate::support::{ConfigView, Host};
use config::{ShieldGlobals, ShieldTypeCatalog, ShieldWarheadRules};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::ConfigureContext;

pub(crate) use logic::evaluate;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    catalog: ShieldTypeCatalog,
    globals: ShieldGlobals,
    objects: BTreeMap<String, String>,
    warheads: BTreeMap<String, ShieldWarheadRules>,
    overload: OverloadConfig,
}

#[derive(Serialize, Deserialize)]
struct OverloadConfig {
    limit: i32,
    duration: i32,
    cooling: i32,
}

impl Default for OverloadConfig {
    fn default() -> Self {
        Self {
            limit: 60,
            duration: 75,
            cooling: 1,
        }
    }
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
        overload_enabled: bool,
    ) {
        if context.final_pass {
            self.globals = ShieldGlobals::from_ini(view);
            self.objects.clear();
            for name in &context.object_types {
                if let Some(binding) = view.section_or_empty(name).read_name("ShieldType", 0x80)
                    && self.catalog.entry(&binding).is_some()
                {
                    self.objects.insert(name.to_ascii_uppercase(), binding);
                }
            }
            self.warheads.clear();
            for name in &context.warheads {
                let rules = ShieldWarheadRules::from_section(&view.section_or_empty(name));
                // Missing entries resolve to this same default in damage_stage.
                // Keep only semantic overrides in every entity's config message.
                if rules != ShieldWarheadRules::default() {
                    self.warheads.insert(name.to_ascii_uppercase(), rules);
                }
            }
            if overload_enabled {
                let section = view.section_or_empty("VeraOverload");
                self.overload.limit = section.read_int("HeatLimit", self.overload.limit).max(1);
                self.overload.duration =
                    section.read_int("Duration", self.overload.duration).max(1);
                self.overload.cooling = section.read_int("Cooling", self.overload.cooling).max(0);
            }
        } else {
            self.catalog.apply_pass(view);
        }
    }
}

#[cfg(test)]
mod tests;
