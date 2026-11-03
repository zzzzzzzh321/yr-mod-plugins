//! Building ART461225..46125D supplies stock dual reads. Ares suffix
//!464A49/464A56 ->1000DD40 latches the first Custom header, including an
//! invalid geometry. The existing host Foundation owner parses that source.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    ConfigureContext, FoundationResourceContext, FoundationResourcePolicy, ImmutableIniSectionRef,
    ProductionObjectKind,
};

#[derive(Default, Serialize, Deserialize)]
struct Building {
    stock: u8,
    first_custom: Option<ImmutableIniSectionRef>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    // Only Building identities enter this domain, so the native category-local
    // ordinal is sufficient. A zero-stock/no-custom constructor stays sparse.
    buildings: BTreeMap<u32, Building>,
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
        for read in view.fixed_art_view()?.reads {
            if read.building.category != ProductionObjectKind::Building {
                return Err("foundation ART reader requires a Building identity".into());
            }
            let mut state = self
                .buildings
                .remove(&read.building.ordinal)
                .unwrap_or_default();
            // This shadow is independent of both the first Custom latch and
            // module activation. The host performs the original Image/Type
            // dual read with their current defaults; never infer a selector
            // from a previously parsed Custom geometry.
            state.stock = view.read_stock_foundation(&read.art, &read.type_name, state.stock);
            if state.first_custom.is_none()
                && view
                    .read_source_string(&read.art, "Foundation", "", 2048)
                    .eq_ignore_ascii_case("Custom")
            {
                state.first_custom = Some(read.art);
            }
            if state.stock != 0 || state.first_custom.is_some() {
                self.buildings.insert(read.building.ordinal, state);
            }
        }
        Ok(())
    }

    pub(crate) fn query(
        &self,
        context: &FoundationResourceContext,
        enabled: bool,
    ) -> FoundationResourcePolicy {
        let state = self.buildings.get(&context.building.ordinal);
        if enabled && let Some(reference) = state.and_then(|state| state.first_custom.as_ref()) {
            FoundationResourcePolicy::ArtSection {
                reference: reference.clone(),
            }
        } else {
            FoundationResourcePolicy::Stock {
                selector: state.map_or(0, |state| state.stock),
            }
        }
    }
}
