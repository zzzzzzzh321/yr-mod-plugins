//! Scripted replies exercise the component and shared reader adapters; native
//! INI decoding, filename bounds and asset loading are host integration tests.
use super::*;
use crate::component::tests::{FrozenHost, frozen_query};
use std::cell::Cell;
use vera_mod_api::{
    CameoReadPoint, CameoRegistryView, CameoResourceContext, CameoResourcePolicy,
    ConfigurationBoundary, ConfigureBoundaryContext, ConfigureContext, DamageKernelInput,
    FixedIniSource, ImmutableIniSectionRef, ImmutableIniValue, ProductionObjectKind,
    ProductionTypeRef,
};

fn object(category: ProductionObjectKind, ordinal: u32) -> ProductionTypeRef {
    ProductionTypeRef { category, ordinal }
}

fn art(section: &str) -> ImmutableIniSectionRef {
    ImmutableIniSectionRef {
        source: FixedIniSource::FixedArt,
        source_hash: 37,
        section: section.into(),
    }
}

struct DecodedHost {
    pass: u8,
    view_calls: Cell<usize>,
}

impl Host for DecodedHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        (key == "SidebarPCX" && matches!(section, "POWER" | "LATER_POWER"))
            .then(|| "physical value decoded by the host".into())
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
            return serde_json::to_vec(&vera_mod_api::ProductionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if reader == "cameo-registry-view" {
            assert_eq!((section, key, current), ("", "", b"null".as_slice()));
            assert_eq!(
                self.view_calls.replace(1),
                0,
                "one catalog per physical pass"
            );
            if self.pass == 3 {
                return Err("cameo source service unavailable".into());
            }
            let techno = |category, ordinal, image: &str| CameoReadPoint::Techno {
                object: object(category, ordinal),
                art: art(image),
            };
            let super_weapon = |ordinal, section: &str| CameoReadPoint::SuperWeapon {
                ordinal,
                section: section.into(),
            };
            use ProductionObjectKind::{Building, Unit};
            let reads = match self.pass {
                0 => vec![
                    techno(Unit, 0, "FIRST"),
                    techno(Building, 0, "BUILDING"),
                    techno(Unit, 1, "EMPTY"),
                    super_weapon(0, "POWER"),
                ],
                1 => vec![
                    techno(Unit, 0, "LATER"),
                    techno(Building, 0, "BUILDING"),
                    super_weapon(0, "LATER_POWER"),
                    super_weapon(1, "POWER"),
                ],
                2 => vec![], // An unreached suffix cannot revisit an old ART source.
                5 => vec![techno(Unit, 0, "FIRST")],
                6 => vec![super_weapon(0, "POWER")],
                _ => panic!("unadvertised cameo domain was queried"),
            };
            return serde_json::to_vec(&CameoRegistryView { reads }).map_err(|e| e.to_string());
        }
        let value = match reader {
            "source-pcx-filename" => {
                let input: ImmutableIniValue<Option<String>> =
                    serde_json::from_slice(current).unwrap();
                assert_eq!(input.reference, art(section));
                assert_eq!(input.current, None, "this reader returns an update only");
                if self.pass == 5 {
                    return Err("fixed ART filename reader unavailable".into());
                }
                match (self.pass, section, key) {
                    (0, "FIRST", "CameoPCX") => Some("normal.pcx"),
                    (0, "FIRST", "AltCameoPCX") => None,
                    (0, "BUILDING", "CameoPCX") => Some("building.pcx"),
                    (0, "BUILDING", "AltCameoPCX") => Some("elite.pcx"),
                    (0, "EMPTY", "CameoPCX" | "AltCameoPCX") => None,
                    (1, "LATER", "CameoPCX") => None,
                    (1, "LATER", "AltCameoPCX") => Some("replacement.pcx"),
                    (1, "BUILDING", "CameoPCX") => Some("<none>"),
                    (1, "BUILDING", "AltCameoPCX") => None,
                    _ => panic!("unexpected fixed ART read {section}/{key}"),
                }
            }
            "pcx-filename" => {
                assert_eq!(key, "SidebarPCX");
                assert_eq!(current, b"null");
                if self.pass == 6 {
                    return Err("Rules filename reader unavailable".into());
                }
                match (self.pass, section) {
                    (0, "POWER") => Some("none"),
                    (1, "LATER_POWER") => None,
                    (1, "POWER") => Some("second.pcx"),
                    _ => panic!("unexpected SuperWeapon read"),
                }
            }
            _ => panic!("unexpected service {section}/{key}/{reader}"),
        };
        serde_json::to_vec(&value).map_err(|e| e.to_string())
    }

    fn registry(&self, registry: &str) -> Vec<String> {
        if registry == "$cameos" && self.pass != 4 {
            vec!["Cameos".into()]
        } else {
            vec![]
        }
    }

    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("no numeric service during cameo configuration")
    }
}

fn configure(config: Vec<u8>, pass: u8, final_pass: bool) -> Result<Vec<u8>, String> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass: pass.into(),
            source: "decoded-cameo-pass".into(),
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
            view_calls: Cell::new(0),
        },
    )?;
    plan.validate_for(&request)?;
    Ok(plan.config.unwrap())
}

fn query(config: &[u8]) -> CameoResourcePolicy {
    let plan = frozen_query(config, Context::CameoResource(CameoResourceContext {}));
    assert!(plan.config.is_none() && plan.state.is_empty());
    serde_json::from_slice(&serde_json::to_vec(&plan.cameo_resource.unwrap()).unwrap()).unwrap()
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
fn cameo_history_is_sparse_class_qualified_and_owned_by_retained_types() {
    let initial = serde_json::to_vec(&Configuration::default()).unwrap();
    let first = configure(initial, 0, false).unwrap();
    let changed = configure(first, 1, false).unwrap();
    let frozen = configure(configure(changed, 2, false).unwrap(), 3, true).unwrap();
    let catalog = query(&frozen);
    assert_eq!(
        catalog.technos.len(),
        2,
        "constructor-only names stay sparse"
    );
    let unit = catalog
        .technos
        .iter()
        .find(|entry| entry.object.category == ProductionObjectKind::Unit)
        .unwrap();
    assert_eq!(unit.object.ordinal, 0);
    assert_eq!(unit.normal.as_deref(), Some("normal.pcx"));
    assert_eq!(unit.promoted.as_deref(), Some("replacement.pcx"));
    let building = catalog
        .technos
        .iter()
        .find(|entry| entry.object.category == ProductionObjectKind::Building)
        .unwrap();
    assert_eq!(building.object.ordinal, 0);
    assert_eq!(building.normal.as_deref(), Some("<none>"));
    assert_eq!(building.promoted.as_deref(), Some("elite.pcx"));
    assert_eq!(catalog.super_weapons[0].ordinal, 0);
    assert_eq!(catalog.super_weapons[0].filename, "none");
    assert_eq!(catalog.super_weapons[1].ordinal, 1);
    assert_eq!(catalog.super_weapons[1].filename, "second.pcx");
    let config: serde_json::Value = serde_json::from_slice(&frozen).unwrap();
    assert!(
        !serde_json::to_string(&config["cameos"])
            .unwrap()
            .contains("source_hash")
    );
    // This is the host's intermediate configuration, not native SWTypeExt
    // survival: formal cameo consumers run only after the following TypeReset.
    let retained = boundary(frozen, ConfigurationBoundary::ScenarioClear);
    assert_eq!(query(&retained), catalog);
    let reset = boundary(retained, ConfigurationBoundary::TypeReset);
    assert_eq!(query(&reset), CameoResourcePolicy::default());
}

#[test]
fn absent_cameo_domain_is_optional_but_advertised_reader_errors_propagate() {
    let config = serde_json::to_vec(&Configuration::default()).unwrap();
    assert_eq!(
        query(&configure(config.clone(), 4, false).unwrap()),
        CameoResourcePolicy::default()
    );
    assert!(
        configure(config.clone(), 3, false)
            .unwrap_err()
            .contains("cameo source service")
    );
    assert!(
        configure(config.clone(), 5, false)
            .unwrap_err()
            .contains("fixed ART filename reader")
    );
    assert!(
        configure(config, 6, false)
            .unwrap_err()
            .contains("Rules filename reader")
    );
}
