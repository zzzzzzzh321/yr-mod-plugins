//! Decoded sound-reader replies exercise physical configuration and frozen
//! resource binding without reimplementing the INI parser or sound lookup.
use super::*;
use crate::component::tests::{FrozenHost, frozen_query};
use std::cell::Cell;
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, ConfigureContext, DamageKernelInput,
    NullableResourceName, ProductionObjectKind, ProductionReadPoint, ProductionReadSite,
    ProductionRegisteredType, ProductionRegistryPrefix, ProductionRegistryView, ProductionTypeRef,
    TypeSoundResourceContext, TypeSoundResourcePolicy,
};

fn identity(category: ProductionObjectKind, ordinal: u32) -> ProductionTypeRef {
    ProductionTypeRef { category, ordinal }
}

struct DecodedHost {
    pass: u8,
    physical_calls: Cell<usize>,
    sound_calls: Cell<usize>,
}

impl Host for DecodedHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        matches!(
            (self.pass, section, key),
            (
                0 | 1 | 5,
                "UNIT",
                "Promote.VeteranSound" | "Promote.EliteSound"
            ) | (0, "BLDG", "Promote.VeteranSound" | "Promote.EliteSound")
                | (1, "BLDG", "Promote.VeteranSound")
        )
        .then(|| "decoded physical sound reference".into())
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "faction-registry-view" {
            return serde_json::to_vec(&vera_mod_api::FactionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if reader == "rules-registry-view" {
            assert_eq!(self.physical_calls.replace(1), 0, "share one physical view");
            let technos = [
                (ProductionObjectKind::Unit, 0, "UNIT"),
                (ProductionObjectKind::Building, 0, "BLDG"),
                (ProductionObjectKind::Unit, 1, "EMPTY"),
            ]
            .map(|(category, ordinal, name)| ProductionRegisteredType {
                identity: identity(category, ordinal),
                name: name.into(),
            });
            let reads = technos
                .iter()
                .map(|entry| ProductionReadPoint {
                    prefix: ProductionRegistryPrefix {
                        buildings: 1,
                        countries: 0,
                        technos: 3,
                    },
                    site: ProductionReadSite::TechnoExtension {
                        object: entry.identity,
                    },
                })
                .collect();
            return serde_json::to_vec(&ProductionRegistryView {
                buildings: vec!["BLDG".into()],
                countries: vec![],
                technos: technos.into(),
                reads,
            })
            .map_err(|e| e.to_string());
        }
        if reader == "sound-source-hash" {
            assert_eq!((section, key, current), ("", "", b"null".as_slice()));
            assert_eq!(
                self.sound_calls.replace(1),
                0,
                "one catalog identity per pass"
            );
            if self.pass == 4 {
                return Err("sound source service unavailable".into());
            }
            assert_ne!(self.pass, 3, "missing marker must not query the source");
            if self.pass == 6 {
                return Ok(br#""not a catalog hash""#.to_vec());
            }
            return serde_json::to_vec(&if self.pass == 2 { 42u64 } else { 41 })
                .map_err(|e| e.to_string());
        }
        assert_eq!(reader, "sound-reference");
        if self.pass == 5 {
            return Err("canonical sound reference reader unavailable".into());
        }
        use NullableResourceName::{Clear, Inherit, Named};
        let prior: NullableResourceName = serde_json::from_slice(current).unwrap();
        let cue = match (self.pass, section, key) {
            (0, "UNIT", "Promote.VeteranSound") => {
                assert_eq!(prior, Inherit);
                Named("HeroVeteran".into())
            }
            (0, "UNIT", "Promote.EliteSound") => {
                assert_eq!(prior, Inherit);
                Named("HeroElite".into())
            }
            (0, "BLDG", "Promote.VeteranSound") => {
                assert_eq!(prior, Inherit);
                Named("BuildingVeteran".into())
            }
            (0, "BLDG", "Promote.EliteSound") => {
                assert_eq!(prior, Inherit);
                Named("none".into()) // Registered literal wins before the none test.
            }
            (1, "UNIT", "Promote.VeteranSound") => {
                assert_eq!(prior, Named("HeroVeteran".into()));
                prior // Unknown physical name retains the already resolved cue.
            }
            (1, "UNIT", "Promote.EliteSound") => {
                assert_eq!(prior, Named("HeroElite".into()));
                prior // A reached empty value retains the same cue too.
            }
            (1, "BLDG", "Promote.VeteranSound") => {
                assert_eq!(prior, Named("BuildingVeteran".into()));
                Clear
            }
            _ => panic!("unexpected sound reader {section}/{key}"),
        };
        serde_json::to_vec(&cue).map_err(|e| e.to_string())
    }

    fn registry(&self, registry: &str) -> Vec<String> {
        if registry == "$sounds" && self.pass != 3 {
            vec!["FixedSounds".into()]
        } else {
            vec![]
        }
    }

    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("no numeric service during promotion sound configuration")
    }
}

fn configure(config: Vec<u8>, pass: u8, final_pass: bool) -> Result<Vec<u8>, String> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass: pass.into(),
            source: "decoded-promotion-sound-pass".into(),
            final_pass,
            object_types: vec![],
            warheads: vec![],
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(
        &request,
        &DecodedHost {
            pass,
            physical_calls: Cell::new(0),
            sound_calls: Cell::new(0),
        },
    )?;
    plan.validate_for(&request)?;
    Ok(plan.config.unwrap())
}

fn query(config: &[u8]) -> TypeSoundResourcePolicy {
    let plan = frozen_query(
        config,
        Context::TypeSoundResource(TypeSoundResourceContext {}),
    );
    assert!(plan.config.is_none() && plan.state.is_empty());
    serde_json::from_slice(&serde_json::to_vec(&plan.type_sound_resource.unwrap()).unwrap())
        .unwrap()
}

fn boundary(config: Vec<u8>, boundary: ConfigurationBoundary) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::ConfigureBoundary(ConfigureBoundaryContext { boundary }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &FrozenHost).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

#[test]
fn promotion_sound_history_binds_once_to_fixed_source_and_follows_type_reset() {
    use NullableResourceName::{Clear, Named};
    let initial = serde_json::to_vec(&Configuration::default()).unwrap();
    let root = configure(initial.clone(), 0, false).unwrap();
    let mode = configure(root, 1, false).unwrap();
    let frozen = configure(mode, 4, true).unwrap(); // A final replay would fail its source service.
    let catalog = query(&frozen);
    assert_eq!(catalog.source_hash, Some(41));
    assert_eq!(
        catalog.promotions.len(),
        2,
        "all-inherited Types stay sparse"
    );
    let unit = catalog
        .promotions
        .iter()
        .find(|entry| entry.object == identity(ProductionObjectKind::Unit, 0))
        .unwrap();
    assert_eq!(unit.veteran, Named("HeroVeteran".into()));
    assert_eq!(unit.elite, Named("HeroElite".into()));
    let building = catalog
        .promotions
        .iter()
        .find(|entry| entry.object == identity(ProductionObjectKind::Building, 0))
        .unwrap();
    assert_eq!(building.veteran, Clear);
    assert_eq!(building.elite, Named("none".into()));
    let retained = boundary(frozen.clone(), ConfigurationBoundary::ScenarioClear);
    assert_eq!(query(&retained), catalog);
    let reset = boundary(retained, ConfigurationBoundary::TypeReset);
    assert_eq!(
        query(&reset),
        TypeSoundResourcePolicy {
            source_hash: Some(41),
            promotions: vec![]
        }
    );
    assert!(
        configure(reset, 2, false)
            .unwrap_err()
            .contains("source changed")
    );
    assert_eq!(
        query(&configure(initial.clone(), 3, false).unwrap()),
        TypeSoundResourcePolicy::default()
    );
    assert!(
        configure(initial.clone(), 4, false)
            .unwrap_err()
            .contains("sound source service")
    );
    assert!(
        configure(initial, 6, false)
            .unwrap_err()
            .contains("fixed sound source")
    );
    assert!(
        configure(frozen, 5, false)
            .unwrap_err()
            .contains("sound reference reader")
    );
}
