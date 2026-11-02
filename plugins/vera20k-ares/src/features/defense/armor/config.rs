//! Ares ArmorTypes1002B013..1002B123/defaults10006890, common Armor
//! reader10006E10 and WarheadExt10051D80/10006460. These are immutable
//! configuration records; combat and numeric parsing stay host-owned.
use crate::support::{ConfigView, Host, Section};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vera_mod_api::{
    ArmorObjectKind, ArmorResourceContext, ArmorResourcePolicy, ArmorVersePolicy, BASE_ARMOR_NAMES,
    ConfigureContext, native_string::truncate_native_bytes,
};

/// Compact frozen encoding only: native parsing and the public SDK retain
/// the four independent fields. No numeric conversion changes their bits.
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
struct Record(u64, u8);

impl Default for Record {
    fn default() -> Self {
        ArmorVersePolicy::default().into()
    }
}

impl From<ArmorVersePolicy> for Record {
    fn from(value: ArmorVersePolicy) -> Self {
        Self(
            value.multiplier_bits,
            u8::from(value.force_fire)
                | u8::from(value.retaliate) << 1
                | u8::from(value.passive_acquire) << 2,
        )
    }
}

impl From<Record> for ArmorVersePolicy {
    fn from(value: Record) -> Self {
        Self {
            multiplier_bits: value.0,
            force_fire: value.1 & 1 != 0,
            retaliate: value.1 & 2 != 0,
            passive_acquire: value.1 & 4 != 0,
        }
    }
}

#[derive(Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
struct Definition {
    parent: Option<usize>,
    fallback: Record,
}

/// A missing index denotes the frozen constructor record, not a live parent.
/// Length is retained even for all-default custom slots: later layers must
/// not inherit again merely because the original record happened to be one.
#[derive(Serialize, Deserialize)]
struct Warhead {
    len: usize,
    entries: BTreeMap<usize, Record>,
}

impl Default for Warhead {
    fn default() -> Self {
        Self {
            len: BASE_ARMOR_NAMES.len(),
            entries: BTreeMap::new(),
        }
    }
}

impl Warhead {
    fn entry(&self, index: usize) -> Record {
        self.entries.get(&index).copied().unwrap_or_default()
    }

    fn set(&mut self, index: usize, value: Record) {
        if value == Record::default() {
            self.entries.remove(&index);
        } else {
            self.entries.insert(index, value);
        }
    }

    fn read_pass<H: Host>(
        &mut self,
        section: &Section<'_, H>,
        catalog: &Configuration,
        view: &ConfigView<'_, H>,
    ) {
        //10051D80 first replaces at most the original eleven records.
        if let Some(tokens) = section.read_comma_list("Verses", 0x800) {
            for (index, token) in tokens.into_iter().take(BASE_ARMOR_NAMES.len()).enumerate() {
                self.set(index, view.parse_verse(&token).into());
            }
        }
        // A WH has far fewer keys than a Techno section. Import its physical
        // key names once, preserving exact matches: armor names may themselves
        // contain dots or flag suffixes. Do not parse a key as an armor name.
        let keys = section
            .physical_keys()
            .into_iter()
            .filter(|key| key.starts_with("Versus."))
            .collect::<BTreeSet<_>>();
        let mut key = if keys.is_empty() {
            String::new()
        } else {
            String::from("Versus.")
        };
        let prefix_len = key.len();
        //10006460 expands and overrides one index before visiting the next:
        // backward parents see current overrides; forward parents do not.
        for (index, name) in catalog.names().enumerate() {
            if index == self.len {
                let definition = catalog.defaults.get(&index);
                let inherited = definition
                    .and_then(|definition| definition.parent)
                    .filter(|parent| *parent < self.len)
                    .map(|parent| self.entry(parent))
                    .unwrap_or_else(|| definition.map_or_else(Record::default, |d| d.fallback));
                self.set(index, inherited);
                self.len += 1;
            }
            if keys.is_empty() {
                continue;
            }
            // Reuse this buffer for all indices and suffixes; absent keys need
            // neither a reader request nor per-index format allocations.
            key.truncate(prefix_len);
            key.push_str(name);
            if keys.contains(&key)
                && let Some(token) = section.read_name(&key, 32)
            {
                self.set(index, view.parse_verse(&token).into());
            }
            let base_len = key.len();
            let previous = self.entry(index);
            let mut entry = ArmorVersePolicy::from(previous);
            key.push_str(".ForceFire");
            if keys.contains(&key) {
                entry.force_fire = section.read_bool(&key, entry.force_fire);
            }
            key.truncate(base_len);
            key.push_str(".Retaliate");
            if keys.contains(&key) {
                entry.retaliate = section.read_bool(&key, entry.retaliate);
            }
            key.truncate(base_len);
            key.push_str(".PassiveAcquire");
            if keys.contains(&key) {
                entry.passive_acquire = section.read_bool(&key, entry.passive_acquire);
            }
            let updated = entry.into();
            if updated != previous {
                self.set(index, updated);
            }
        }
    }
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    custom_names: Vec<String>,
    defaults: BTreeMap<usize, Definition>,
    objects: BTreeMap<ArmorObjectKind, BTreeMap<String, u32>>,
    warheads: BTreeMap<String, Warhead>,
}

impl Configuration {
    fn names(&self) -> impl Iterator<Item = &str> {
        BASE_ARMOR_NAMES
            .into_iter()
            .chain(self.custom_names.iter().map(String::as_str))
    }

    fn read_catalog<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
        lookup: &mut BTreeMap<String, usize>,
    ) {
        let section = view.section_or_empty("ArmorTypes");
        //1002B013 allocates every full raw key before reading any defaults.
        // Find precedes the31-byte store, so truncated identities can collide.
        for name in section.physical_keys() {
            if !lookup.contains_key(&name.to_ascii_lowercase()) {
                let stored = truncate_native_bytes(&name, 31).to_owned();
                let index = BASE_ARMOR_NAMES.len() + self.custom_names.len();
                // Find returns the first stored collision. Never index the
                // full raw key: it remains unknown after a truncated store.
                lookup.entry(stored.to_ascii_lowercase()).or_insert(index);
                self.custom_names.push(stored);
            }
        }
        for index in 0..BASE_ARMOR_NAMES.len() + self.custom_names.len() {
            let name = if index < BASE_ARMOR_NAMES.len() {
                BASE_ARMOR_NAMES[index]
            } else {
                self.custom_names[index - BASE_ARMOR_NAMES.len()].as_str()
            };
            if let Some(token) = section.read_name(name, 0x40) {
                let definition = Definition {
                    parent: lookup.get(&token.to_ascii_lowercase()).copied(),
                    fallback: view.parse_verse(&token).into(),
                };
                if definition == Definition::default() {
                    self.defaults.remove(&index);
                } else {
                    self.defaults.insert(index, definition);
                }
            }
        }
    }

    pub(crate) fn configure<H: Host>(
        &mut self,
        context: &ConfigureContext,
        view: &ConfigView<'_, H>,
    ) {
        // Final projection has no physical reader entry and cannot replay
        // names against a later catalog or re-inherit existing WH records.
        if context.final_pass {
            return;
        }
        // Rebuild this derived lookup once per physical pass; only the ordered
        // names above are persisted. Native Find is ASCII-insensitive and
        // returns the first stored identity when truncation caused duplicates.
        let mut lookup = BTreeMap::new();
        for (index, name) in self.names().enumerate() {
            lookup.entry(name.to_ascii_lowercase()).or_insert(index);
        }
        if !view.registered_names("$armor-before-type-data").is_empty() {
            self.read_catalog(view, &mut lookup);
        }
        for (category, registry) in [
            (ArmorObjectKind::Unit, "$armor-units"),
            (ArmorObjectKind::Infantry, "$armor-infantry"),
            (ArmorObjectKind::Building, "$armor-buildings"),
            (ArmorObjectKind::Aircraft, "$armor-aircraft"),
            (ArmorObjectKind::Terrain, "$armor-terrain"),
        ] {
            for name in view.registered_names(registry) {
                let section = view.section_or_empty(&name);
                let Some(binding) = section.read_name("Armor", 32) else {
                    continue;
                };
                //10006E10 unknown names immediately become None0, even when
                // another pass registers that name before the final query.
                let armor = lookup
                    .get(&binding.to_ascii_lowercase())
                    .copied()
                    .unwrap_or(0) as u32;
                let constructor = constructor_armor(category);
                if armor == constructor {
                    if let Some(members) = self.objects.get_mut(&category) {
                        members.remove(&name);
                    }
                } else {
                    self.objects
                        .entry(category)
                        .or_default()
                        .insert(name, armor);
                }
            }
        }
        self.objects.retain(|_, members| !members.is_empty());
        // This list includes reached WH readers whose section is absent.
        // Their newly allocated custom slots still inherit this pass.
        for name in view.registered_names("$armor-warheads") {
            let section = view.section_or_empty(&name);
            let mut warhead = self.warheads.remove(&name).unwrap_or_default();
            warhead.read_pass(&section, self, view);
            if warhead.len != BASE_ARMOR_NAMES.len() || !warhead.entries.is_empty() {
                self.warheads.insert(name, warhead);
            }
        }
    }

    pub(crate) fn query(&self, context: &ArmorResourceContext) -> ArmorResourcePolicy {
        match context {
            ArmorResourceContext::Catalog => ArmorResourcePolicy::Catalog {
                names: self.names().map(str::to_owned).collect(),
            },
            ArmorResourceContext::Object { category, name } => ArmorResourcePolicy::Object {
                armor: self
                    .objects
                    .get(category)
                    .and_then(|members| members.get(name))
                    .copied()
                    .unwrap_or_else(|| constructor_armor(*category)),
            },
            ArmorResourceContext::Warhead { name } => {
                let warhead = self.warheads.get(name);
                ArmorResourcePolicy::Warhead {
                    entries: (0..warhead.map_or(BASE_ARMOR_NAMES.len(), |wh| wh.len))
                        .map(|index| {
                            warhead
                                .map_or_else(ArmorVersePolicy::default, |wh| wh.entry(index).into())
                        })
                        .collect(),
                    // Ares10051D80 keeps its vector separate from WH+A0 and
                    // never derives the original WH+149 shortcut flag.
                    use_medium_wood_target_mask: false,
                }
            }
        }
    }
}

fn constructor_armor(category: ArmorObjectKind) -> u32 {
    if category == ArmorObjectKind::Terrain {
        6
    } else {
        0
    }
}
