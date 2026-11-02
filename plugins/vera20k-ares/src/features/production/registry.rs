//! Rebuilt indexes over one host-owned physical catalog. Prefix admission is
//! checked after lookup so later registrations never resolve an earlier read.
use std::collections::BTreeMap;
use vera_mod_api::{ProductionRegistryPrefix, ProductionRegistryView, ProductionTypeRef};

pub(super) struct Registry<'a> {
    buildings: BTreeMap<String, u32>,
    countries: BTreeMap<String, u32>,
    technos: BTreeMap<String, (u32, ProductionTypeRef)>,
    names: BTreeMap<ProductionTypeRef, &'a str>,
    source: &'a ProductionRegistryView,
}

impl<'a> Registry<'a> {
    pub(super) fn new(source: &'a ProductionRegistryView) -> Self {
        let mut buildings = BTreeMap::new();
        let mut countries = BTreeMap::new();
        let mut technos = BTreeMap::new();
        let mut names = BTreeMap::new();
        for (index, name) in source.buildings.iter().enumerate() {
            buildings
                .entry(name.to_ascii_lowercase())
                .or_insert(index as u32);
        }
        for (index, name) in source.countries.iter().enumerate() {
            countries
                .entry(name.to_ascii_lowercase())
                .or_insert(index as u32);
        }
        for (index, entry) in source.technos.iter().enumerate() {
            technos
                .entry(entry.name.to_ascii_lowercase())
                .or_insert((index as u32, entry.identity));
            names.entry(entry.identity).or_insert(entry.name.as_str());
        }
        Self {
            buildings,
            countries,
            technos,
            names,
            source,
        }
    }

    pub(super) fn validate(&self, prefix: ProductionRegistryPrefix) -> Result<(), String> {
        if prefix.buildings as usize > self.source.buildings.len()
            || prefix.countries as usize > self.source.countries.len()
            || prefix.technos as usize > self.source.technos.len()
        {
            return Err("production read prefix exceeds its physical registry".into());
        }
        Ok(())
    }

    pub(super) fn name(&self, identity: ProductionTypeRef) -> Result<&'a str, String> {
        self.names
            .get(&identity)
            .copied()
            .ok_or_else(|| "production reader has no registered type identity".into())
    }

    // Prereqs::Parse100085F0 tries the Building array directly before groups.
    pub(super) fn building(&self, name: &str, prefix: ProductionRegistryPrefix) -> Option<u32> {
        self.buildings
            .get(&name.to_ascii_lowercase())
            .copied()
            .filter(|index| *index < prefix.buildings)
    }

    // Building10007E50 and Country1001BA90 apply none10003950 first.
    pub(super) fn existing_building(
        &self,
        name: &str,
        prefix: ProductionRegistryPrefix,
    ) -> Option<u32> {
        (!is_none(name))
            .then(|| self.building(name, prefix))
            .flatten()
    }

    pub(super) fn country(&self, name: &str, prefix: ProductionRegistryPrefix) -> Option<u32> {
        if is_none(name) {
            return None;
        }
        self.countries
            .get(&name.to_ascii_lowercase())
            .copied()
            .filter(|index| *index < prefix.countries)
    }

    // GenericPrerequisite Alternate10008470 uses the first cross-family
    // stored name in actual native construction order, without allocation.
    pub(super) fn techno(
        &self,
        name: &str,
        prefix: ProductionRegistryPrefix,
    ) -> Option<ProductionTypeRef> {
        self.technos
            .get(&name.to_ascii_lowercase())
            .copied()
            .filter(|(index, _)| *index < prefix.technos)
            .map(|(_, identity)| identity)
    }
}

fn is_none(name: &str) -> bool {
    name.eq_ignore_ascii_case("none") || name.eq_ignore_ascii_case("<none>")
}
