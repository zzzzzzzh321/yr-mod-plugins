//! Retained common SWTypeExt references and exact TechnoTypeExt ranges.
//! Live observations and targeting checks remain with the host's one collector.
use crate::support::{ConfigView, Host, Section};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    ProductionObjectKind, ProductionReadSite, ProductionRegistryView, ProductionTypeRef,
    SuperWeaponInfluencePolicy, TechnoInfluenceDeclaration,
};

pub(super) fn read<H: Host>(
    policy: &mut SuperWeaponInfluencePolicy,
    read: u32,
    owner: &str,
    section: &Section<'_, H>,
    view: &ConfigView<'_, H>,
) -> Result<(), String> {
    // SWTypeExt ctor1003191B..10031945; common ReadINI1003396E..100339C6.
    // The scoped vector reader owns Find10007EA0 against this visit's common
    // construction prefix. It preserves order/duplicates and empty retention.
    if let Some(value) = view.superweapon_read::<_, Option<Vec<ProductionTypeRef>>>(
        read,
        owner,
        "SW.Designators",
        "super-techno-list",
        (),
    )? {
        policy.designators = value;
    }
    policy.any_designator = section.read_ares_bool("SW.AnyDesignator", policy.any_designator);
    if let Some(value) = view.superweapon_read::<_, Option<Vec<ProductionTypeRef>>>(
        read,
        owner,
        "SW.Inhibitors",
        "super-techno-list",
        (),
    )? {
        policy.inhibitors = value;
    }
    policy.any_inhibitor = section.read_ares_bool("SW.AnyInhibitor", policy.any_inhibitor);
    Ok(())
}

#[derive(Default, Serialize, Deserialize)]
struct Ranges {
    designator: Option<i32>,
    inhibitor: Option<i32>,
}

/// Presence of this domain is independent of Nuclear/common targeting. Missing
/// in a historical frozen config, it is created only for a fresh match/reset.
#[derive(Default, Serialize, Deserialize)]
pub(super) struct Technos {
    types: BTreeMap<ProductionObjectKind, BTreeMap<u32, Ranges>>,
}

impl Technos {
    pub(super) fn configure<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
        physical: &ProductionRegistryView,
    ) -> Result<(), String> {
        // Temporary indexing uses the shared host directory; no second Type
        // registry or name-based owner survives in frozen configuration.
        let mut names = BTreeMap::new();
        for (index, entry) in physical.technos.iter().enumerate() {
            names
                .entry(entry.identity)
                .or_insert((index, entry.name.as_str()));
        }
        for read in &physical.reads {
            let ProductionReadSite::TechnoExtension { object } = read.site else {
                continue;
            };
            let prefix = read.prefix.technos as usize;
            if prefix > physical.technos.len() {
                return Err("influence prefix exceeds its physical registry".into());
            }
            let name = names
                .get(&object)
                .filter(|(index, _)| *index < prefix)
                .map(|(_, name)| *name)
                .ok_or("influence Type is outside its construction prefix")?;
            let members = self.types.entry(object.category).or_default();
            let mut ranges = members.remove(&object.ordinal).unwrap_or_default();
            let section = view.section_or_empty(name);
            // TechnoTypeExt ctor1003CCA3: both nullable values unset. Reader
            //1003EE18..1003EE44 reaches DesignatorRange before InhibitorRange;
            //10008CC0 retains absent/empty/invalid and accepts signed zero.
            ranges.designator =
                section.read_ares_nullable_int("DesignatorRange", ranges.designator);
            ranges.inhibitor = section.read_ares_nullable_int("InhibitorRange", ranges.inhibitor);
            if ranges.designator.is_some() || ranges.inhibitor.is_some() {
                members.insert(object.ordinal, ranges);
            }
        }
        self.types.retain(|_, members| !members.is_empty());
        Ok(())
    }

    pub(super) fn query(&self) -> Vec<TechnoInfluenceDeclaration> {
        // The API's explicit all-unset base covers constructor-only late Types
        // without inventing a body visit or emitting duplicate exact keys.
        self.types
            .iter()
            .flat_map(|(&category, members)| {
                members
                    .iter()
                    .map(move |(&ordinal, ranges)| TechnoInfluenceDeclaration {
                        object: ProductionTypeRef { category, ordinal },
                        designator_range: ranges.designator,
                        inhibitor_range: ranges.inhibitor,
                    })
            })
            .collect()
    }
}
