//! Ares AddDefaults10007DB0, LoadFromINI10008470 and Parse100085F0.
//! Groups retain immediate declarations; host production owns evaluation.
use super::registry::Registry;
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    BASE_PRODUCTION_GROUPS, ProductionGroup, ProductionRegistryPrefix, ProductionRequirement,
    ProductionTypeRef, native_string::truncate_native_bytes,
};

#[derive(Default, Serialize, Deserialize)]
pub(super) struct Groups {
    custom_names: Vec<String>,
    members: BTreeMap<u32, Vec<ProductionRequirement>>,
    alternates: BTreeMap<u32, Vec<ProductionTypeRef>>,
}

impl Groups {
    fn names(&self) -> impl Iterator<Item = &str> {
        BASE_PRODUCTION_GROUPS
            .into_iter()
            .chain(self.custom_names.iter().map(String::as_str))
    }

    pub(super) fn lookup(&self) -> BTreeMap<String, u32> {
        let mut lookup = BTreeMap::new();
        for (index, name) in self.names().enumerate() {
            lookup
                .entry(name.to_ascii_lowercase())
                .or_insert(index as u32);
        }
        lookup
    }

    pub(super) fn read<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
        registry: &Registry<'_>,
        prefix: ProductionRegistryPrefix,
    ) -> BTreeMap<String, u32> {
        let custom = view.section_or_empty("GenericPrerequisites");
        let mut lookup = self.lookup();
        // All full raw keys are FindOrAllocate10007FD0'd before member reads.
        for name in custom.physical_keys() {
            if !lookup.contains_key(&name.to_ascii_lowercase()) {
                let stored = truncate_native_bytes(&name, 31).to_owned();
                let index = BASE_PRODUCTION_GROUPS.len() + self.custom_names.len();
                lookup
                    .entry(stored.to_ascii_lowercase())
                    .or_insert(index as u32);
                self.custom_names.push(stored);
            }
        }
        let general = view.section_or_empty("General");
        for index in 0..BASE_PRODUCTION_GROUPS.len() + self.custom_names.len() {
            let name = if index < BASE_PRODUCTION_GROUPS.len() {
                BASE_PRODUCTION_GROUPS[index]
            } else {
                self.custom_names[index - BASE_PRODUCTION_GROUPS.len()].as_str()
            };
            let mut suffix = name.to_ascii_lowercase();
            if let Some(first) = suffix.chars().next() {
                suffix.replace_range(..first.len_utf8(), &first.to_ascii_uppercase().to_string());
            }
            let key = format!("Prerequisite{suffix}");
            for (section, key) in [(&general, key.as_str()), (&custom, name)] {
                if let Some(tokens) = section.read_comma_list(key, 2048) {
                    let members = parse(tokens, &lookup, registry, prefix);
                    if members.is_empty() {
                        self.members.remove(&(index as u32));
                    } else {
                        self.members.insert(index as u32, members);
                    }
                }
            }
            if let Some(tokens) = general.read_comma_list(&format!("{key}Alternate"), 2048) {
                let alternates: Vec<_> = tokens
                    .into_iter()
                    .filter_map(|token| registry.techno(&token, prefix))
                    .collect();
                if alternates.is_empty() {
                    self.alternates.remove(&(index as u32));
                } else {
                    self.alternates.insert(index as u32, alternates);
                }
            }
        }
        lookup
    }

    pub(super) fn query(&self) -> Vec<ProductionGroup> {
        self.names()
            .enumerate()
            .map(|(index, name)| ProductionGroup {
                name: name.into(),
                members: self
                    .members
                    .get(&(index as u32))
                    .cloned()
                    .unwrap_or_default(),
                alternates: self
                    .alternates
                    .get(&(index as u32))
                    .cloned()
                    .unwrap_or_default(),
            })
            .collect()
    }
}

pub(super) fn parse(
    tokens: Vec<String>,
    groups: &BTreeMap<String, u32>,
    registry: &Registry<'_>,
    prefix: ProductionRegistryPrefix,
) -> Vec<ProductionRequirement> {
    tokens
        .into_iter()
        .filter_map(|token| {
            registry
                .building(&token, prefix)
                .map(ProductionRequirement::Building)
                .or_else(|| {
                    groups
                        .get(&token.to_ascii_lowercase())
                        .copied()
                        .map(ProductionRequirement::Group)
                })
        })
        .collect()
}
