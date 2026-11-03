//! Techno ART1003E81F..1003E856 and SW Rules10033B50..10033B66
//! share the host's PCXFile100762C0 filename reader. Only reached suffixes
//! update these names; missing/empty reads retain the same Type's declaration.
use crate::support::{ConfigView, Host};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use vera_mod_api::{
    CameoReadPoint, CameoResourcePolicy, ConfigureContext, ProductionObjectKind, ProductionTypeRef,
    SuperWeaponCameoDeclaration, TechnoCameoDeclaration,
};

#[derive(Default, Serialize, Deserialize)]
struct Names {
    normal: Option<String>,
    promoted: Option<String>,
}

#[derive(Default, Serialize, Deserialize)]
pub(crate) struct Configuration {
    // Class-local ordinals distinguish same-name or same-index Type families.
    // Only declarations containing a filename occupy this sparse catalog.
    technos: BTreeMap<ProductionObjectKind, BTreeMap<u32, Names>>,
    super_weapons: BTreeMap<u32, String>,
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
        for point in view.cameo_view()?.reads {
            match point {
                CameoReadPoint::Techno { object, art } => {
                    let normal = view.read_source_pcx_filename(&art, "CameoPCX");
                    let promoted = view.read_source_pcx_filename(&art, "AltCameoPCX");
                    if normal.is_some() || promoted.is_some() {
                        let names = self
                            .technos
                            .entry(object.category)
                            .or_default()
                            .entry(object.ordinal)
                            .or_default();
                        if let Some(name) = normal {
                            names.normal = Some(name);
                        }
                        if let Some(name) = promoted {
                            names.promoted = Some(name);
                        }
                    }
                }
                CameoReadPoint::SuperWeapon { ordinal, section } => {
                    if let Some(name) = view
                        .section_or_empty(&section)
                        .read_pcx_filename("SidebarPCX")
                    {
                        self.super_weapons.insert(ordinal, name);
                    }
                }
            }
        }
        Ok(())
    }

    pub(crate) fn query(&self) -> CameoResourcePolicy {
        CameoResourcePolicy {
            technos: self
                .technos
                .iter()
                .flat_map(|(&category, types)| {
                    types
                        .iter()
                        .map(move |(&ordinal, names)| TechnoCameoDeclaration {
                            object: ProductionTypeRef { category, ordinal },
                            normal: names.normal.clone(),
                            promoted: names.promoted.clone(),
                        })
                })
                .collect(),
            super_weapons: self
                .super_weapons
                .iter()
                .map(|(&ordinal, filename)| SuperWeaponCameoDeclaration {
                    ordinal,
                    filename: filename.clone(),
                })
                .collect(),
        }
    }
}
