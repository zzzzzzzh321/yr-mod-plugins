//! Installed TypeExt1003E232..1003E33A configures every Techno family.
//! Building45FEE5..45FF0D separately supplies retail capability on opt-out;
//! the host retains the only live searchlight and General parameter owners.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use vera_mod_api::{
    ConfigureContext, ProductionObjectKind, ProductionReadSite, ProductionRegistryView,
    SpotlightAttachment, SpotlightPolicy, WEAPON_USER_AIRCRAFT, WEAPON_USER_BUILDING,
    WEAPON_USER_INFANTRY, WEAPON_USER_UNIT,
};

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    types: BTreeMap<u8, BTreeMap<String, SpotlightPolicy>>,
    // None means historical configuration lacks the retail read history. An
    // admitted physical pass knows the constructor's false default; only true
    // buildings need an entry. Never infer this from the Ares-trimmed reader.
    stock_enabled: Option<BTreeSet<String>>,
}

fn type_default() -> SpotlightPolicy {
    SpotlightPolicy {
        has_spotlight: Some(false),
        use_type_rules: true,
        ..Default::default()
    }
}

impl Configuration {
    pub(crate) fn configure<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
        physical: &ProductionRegistryView,
    ) -> Result<(), String> {
        for (user, registry) in [
            (WEAPON_USER_BUILDING, "$buildings"),
            (WEAPON_USER_AIRCRAFT, "$aircraft"),
            (WEAPON_USER_UNIT, "$vehicles"),
            (WEAPON_USER_INFANTRY, "$infantry"),
        ] {
            for name in view.registered_names(registry) {
                let section = view.section_in_registry(&name, registry);
                let mut policy = self
                    .types
                    .get_mut(&user)
                    .and_then(|types| types.remove(&name))
                    .unwrap_or_else(type_default);
                policy.has_spotlight = Some(
                    section.read_ares_bool("HasSpotlight", policy.has_spotlight.unwrap_or(false)),
                );
                policy.start_height =
                    section.read_ares_int("Spotlight.StartHeight", policy.start_height);
                policy.distance = section.read_ares_int("Spotlight.Distance", policy.distance);
                //1003E260..1003E2E7 is a trimmed, lookup-only enum. Empty and
                // unknown copies retain the old value, including Barrel.
                if section.is_present("Spotlight.AttachedTo") {
                    let value = section.read_string("Spotlight.AttachedTo", "", 2048);
                    policy.attached_to = if value.eq_ignore_ascii_case("body") {
                        SpotlightAttachment::Body
                    } else if value.eq_ignore_ascii_case("turret") {
                        SpotlightAttachment::Turret
                    } else if value.eq_ignore_ascii_case("barrel") {
                        SpotlightAttachment::Barrel
                    } else {
                        policy.attached_to
                    };
                }
                for (key, bit) in [
                    ("Spotlight.DisableRed", 2),
                    ("Spotlight.DisableGreen", 4),
                    ("Spotlight.DisableBlue", 8),
                    ("Spotlight.DisableColor", 1),
                ] {
                    let disabled = section.read_ares_bool(key, policy.glow_flags & bit != 0);
                    policy.glow_flags = (policy.glow_flags & !bit) | if disabled { bit } else { 0 };
                }
                if policy != type_default() {
                    self.types.entry(user).or_default().insert(name, policy);
                }
            }
        }
        self.types.retain(|_, types| !types.is_empty());

        let stock = self.stock_enabled.get_or_insert_with(BTreeSet::new);
        // Building extension visits have independent admission from the common
        // Techno suffix. Index real typed identities once, never final-merge
        // membership or same-name Foot entries.
        let names: BTreeMap<_, _> = physical
            .technos
            .iter()
            .filter(|entry| entry.identity.category == ProductionObjectKind::Building)
            .map(|entry| (entry.identity, entry.name.as_str()))
            .collect();
        for read in &physical.reads {
            let ProductionReadSite::BuildingExtension { object, .. } = &read.site else {
                continue;
            };
            let name = names
                .get(object)
                .copied()
                .ok_or("spotlight building reader has no registered building identity")?;
            // Retail ReadBool does not trim leading whitespace, unlike the
            // Ares Valueable<bool>. Retain its own current value across layers.
            let enabled = view
                .section_or_empty(name)
                .read_bool("HasSpotlight", stock.contains(name));
            if enabled {
                stock.insert(name.into());
            } else {
                stock.remove(name);
            }
        }
        Ok(())
    }

    pub(crate) fn finalize(&mut self, context: &ConfigureContext) {
        if self.types.is_empty() && self.stock_enabled.as_ref().is_none_or(BTreeSet::is_empty) {
            return;
        }
        let names: BTreeSet<_> = context.object_types.iter().map(String::as_str).collect();
        for types in self.types.values_mut() {
            types.retain(|name, _| names.contains(name.as_str()));
        }
        self.types.retain(|_, types| !types.is_empty());
        if let Some(stock) = &mut self.stock_enabled {
            stock.retain(|name| names.contains(name.as_str()));
        }
    }

    pub(crate) fn query(&self, name: &str, user: u8, enabled: bool) -> SpotlightPolicy {
        if enabled {
            self.types
                .get(&user)
                .and_then(|types| types.get(name))
                .copied()
                .unwrap_or_else(type_default)
        } else {
            SpotlightPolicy {
                has_spotlight: if user == WEAPON_USER_BUILDING {
                    self.stock_enabled
                        .as_ref()
                        .map(|stock| stock.contains(name))
                } else {
                    Some(false)
                },
                ..Default::default()
            }
        }
    }
}
