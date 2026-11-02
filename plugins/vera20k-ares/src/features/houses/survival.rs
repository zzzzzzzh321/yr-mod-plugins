//! Ares TypeExt reader1003F47C and HouseExt tracking10021F70. This component
//! retains only authored nullable overrides. The canonical host type facts
//! supply unset defaults; HouseTracking owns every event count and defeat gate.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::ConfigureContext;

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    overrides: BTreeMap<String, bool>,
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) {
        self.overrides.clear();
        for name in &context.object_types {
            if let Some(value) = view
                .section_or_empty(name)
                .read_ares_nullable_bool("KeepAlive", None)
            {
                self.overrides.insert(name.clone(), value);
            }
        }
    }

    pub(crate) fn contribution(&self, name: &str) -> Option<bool> {
        self.overrides.get(name).copied()
    }
}
