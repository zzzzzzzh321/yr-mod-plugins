//! TechnoTypeExt1003F0B4..1003F0DB reads both NullableIdx<Voc> cues
//! through1000E9D0. The host resolves against the one fixed sound catalog;
//! the component retains each reached Type's inheritance/silence/name state.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    NullableResourceName, ProductionObjectKind, ProductionReadSite, ProductionRegistryView,
    ProductionTypeRef, PromotionSoundDeclaration, TypeSoundResourcePolicy,
};

#[derive(Default, Serialize, Deserialize)]
struct Cues {
    veteran: NullableResourceName,
    elite: NullableResourceName,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    source_hash: Option<u64>,
    types: BTreeMap<ProductionObjectKind, BTreeMap<u32, Cues>>,
}

impl Configuration {
    pub(crate) fn reset_types(&mut self) {
        // Type destruction does not replace the process-selected SOUNDMD.
        self.types.clear();
    }

    pub(crate) fn configure<H: Host>(
        &mut self,
        view: &ConfigView<'_, H>,
        physical: &ProductionRegistryView,
    ) -> Result<(), String> {
        let Some(source_hash) = view.sound_source_hash() else {
            return Ok(());
        };
        if self.source_hash.is_some_and(|old| old != source_hash) {
            return Err("promotion sound source changed during configuration".into());
        }
        self.source_hash = Some(source_hash);
        // This temporary index names the host's class-qualified identities;
        // admission still checks their actual construction prefix per read.
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
                return Err("promotion sound prefix exceeds its physical registry".into());
            }
            let name = names
                .get(&object)
                .filter(|(index, _)| *index < prefix)
                .map(|(_, name)| *name)
                .ok_or("promotion sound Type is outside its construction prefix")?;
            let members = self.types.entry(object.category).or_default();
            let mut cues = members.remove(&object.ordinal).unwrap_or_default();
            let section = view.section_or_empty(name);
            cues.veteran = section.read_sound_reference("Promote.VeteranSound", cues.veteran);
            cues.elite = section.read_sound_reference("Promote.EliteSound", cues.elite);
            if cues.veteran != NullableResourceName::Inherit
                || cues.elite != NullableResourceName::Inherit
            {
                members.insert(object.ordinal, cues);
            }
        }
        self.types.retain(|_, members| !members.is_empty());
        Ok(())
    }

    pub(crate) fn query(&self) -> TypeSoundResourcePolicy {
        TypeSoundResourcePolicy {
            source_hash: self.source_hash,
            promotions: self
                .types
                .iter()
                .flat_map(|(&category, types)| {
                    types
                        .iter()
                        .map(move |(&ordinal, cues)| PromotionSoundDeclaration {
                            object: ProductionTypeRef { category, ordinal },
                            veteran: cues.veteran.clone(),
                            elite: cues.elite.clone(),
                        })
                })
                .collect(),
        }
    }
}
