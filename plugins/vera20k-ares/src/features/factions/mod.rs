//! Shared faction configuration. One ordinal-indexed state owns initialization,
//! physical pass history and the parent copy for both gameplay and presentation.
mod country;
mod defaults;
mod side;

use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    ConfigureContext, FactionReadSite, FactionResourceContext, FactionResourcePolicy,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    sides: BTreeMap<u32, side::State>,
    countries: BTreeMap<u32, country::State>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        if context.final_pass {
            return Ok(());
        }
        self.configure_physical(view)
    }

    /// Country-only preparation reaches faction readers without replaying a
    /// full Rules Process or any other domain's feature/configuration reads.
    pub(crate) fn configure_physical<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
    ) -> Result<(), String> {
        let physical = view.faction_registry_view()?;
        for (read, point) in physical.reads.iter().enumerate() {
            let read = read as u32;
            match &point.site {
                FactionReadSite::SideSweep { side } => {
                    let id = &physical.sides[*side as usize];
                    self.sides
                        .entry(*side)
                        .or_default()
                        .read(read, *side, id, view);
                }
                FactionReadSite::CountrySuffix {
                    country,
                    primary_rules,
                    parent,
                    side,
                    ..
                } => {
                    let id = &physical.countries[*country as usize];
                    let mut state = self
                        .countries
                        .remove(country)
                        .unwrap_or_else(|| country::State::new(id));
                    if state.can_read() {
                        state.initialize_color(read, *side, view);
                        if *primary_rules {
                            state.read_primary(read, id, view);
                        }
                        //1001BDC0 runs after the primary GUI block. A later
                        // parent may still have only its constructor values.
                        if !state.parent_attempted
                            && !parent.is_empty()
                            && !parent.eq_ignore_ascii_case(id)
                        {
                            if let Some(index) = view.faction_find::<u32>(read, parent, "country") {
                                if index >= point.prefix.countries {
                                    return Err("parent Find escaped its Country prefix".into());
                                }
                                let inherited = self
                                    .countries
                                    .get(&index)
                                    .map(|s| s.policy.clone())
                                    .unwrap_or_else(|| {
                                        defaults::country(&physical.countries[index as usize])
                                    });
                                state.inherit(&inherited);
                            }
                            //1001C0CB consumes failed Find too. Empty/self did
                            // not enter this branch and do not consume it.
                            state.parent_attempted = true;
                        }
                        state.read_common(read, id, view);
                    }
                    self.countries.insert(*country, state);
                }
            }
        }
        Ok(())
    }

    pub(crate) fn query(
        &self,
        context: &FactionResourceContext,
        enabled: bool,
    ) -> FactionResourcePolicy {
        match context {
            FactionResourceContext::Side { ordinal, .. } => FactionResourcePolicy::Side {
                policy: enabled.then(|| {
                    self.sides
                        .get(ordinal)
                        .map(|s| s.policy.clone())
                        .unwrap_or_default()
                }),
            },
            FactionResourceContext::Country { ordinal, stored_id } => {
                FactionResourcePolicy::Country {
                    policy: enabled.then(|| {
                        self.countries
                            .get(ordinal)
                            .map(|s| s.policy.clone())
                            .unwrap_or_else(|| defaults::country(stored_id))
                    }),
                }
            }
        }
    }
}

fn fixed_name<H: Host>(
    view: &ConfigView<'_, H>,
    id: &str,
    key: &str,
    target: &mut Option<String>,
    file: bool,
) {
    if let Some(value) = view.section_or_empty(id).read_name(key, 2048) {
        *target = if file && is_none(&value) {
            None
        } else {
            Some(vera_mod_api::native_string::truncate_native_bytes(&value, 31).to_owned())
        };
    }
}
fn is_none(value: &str) -> bool {
    value.eq_ignore_ascii_case("none") || value.eq_ignore_ascii_case("<none>")
}

#[cfg(test)]
mod tests;
