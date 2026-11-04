//! Decoded replies test the component path; fixed ART geometry never crosses
//! the guest ABI, including invalid dimensions and large outline backings.
use super::*;
use crate::component::tests::{FrozenHost, frozen_query};
use std::cell::Cell;
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, ConfigureContext, FixedIniSource,
    FoundationReadPoint, FoundationRegistryView, FoundationResourceContext,
    FoundationResourcePolicy as Policy, ImmutableIniSectionRef, ImmutableIniValue,
    ProductionObjectKind, ProductionTypeRef,
};

fn building(ordinal: u32) -> ProductionTypeRef {
    ProductionTypeRef {
        category: ProductionObjectKind::Building,
        ordinal,
    }
}

fn art(section: &str) -> ImmutableIniSectionRef {
    ImmutableIniSectionRef {
        source: FixedIniSource::FixedArt,
        source_hash: 37,
        section: section.into(),
    }
}

struct ArtHost {
    pass: u8,
    view_calls: Cell<usize>,
}
impl ArtHost {
    fn new(pass: u8) -> Self {
        Self {
            pass,
            view_calls: Cell::new(0),
        }
    }
}

impl Host for ArtHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        (matches!(self.pass, 0 | 2) && (section, key) == ("VeraFeatures", "AresFoundation"))
            .then(|| "decoded bool".into())
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if let Some(reply) =
            crate::component::tests::empty_super_sources(section, key, reader, current)
        {
            return reply;
        }
        if reader == "faction-registry-view" {
            // This domain fixture has no Side/Country reader dispatch.
            assert_eq!((section, key, current), ("", "", b"null".as_slice()));
            return serde_json::to_vec(&vera_mod_api::FactionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if reader == "fixed-art-view" {
            assert_ne!(self.pass, 9, "final Configure must not reread fixed ART");
            assert_eq!((section, key, current), ("", "", b"null".as_slice()));
            assert_eq!(
                self.view_calls.replace(1),
                0,
                "ART metadata is shared once per pass"
            );
            if self.pass == 3 {
                return Err("fixed ART metadata unavailable".into());
            }
            let point = |ordinal, type_name: &str, image: &str| FoundationReadPoint {
                building: building(ordinal),
                type_name: type_name.into(),
                art: art(image),
            };
            let reads = match self.pass {
                0 => vec![point(0, "A", "FIRST_INVALID"), point(1, "B", "STOCK")],
                1 => vec![point(0, "A", "NEW_IMAGE"), point(1, "B", "OTHER_CUSTOM")],
                4 => vec![point(0, "A", "FIRST_INVALID")],
                6 => vec![point(0, "A", "NEW_IMAGE")],
                _ => vec![],
            };
            return serde_json::to_vec(&FoundationRegistryView { reads })
                .map_err(|e| e.to_string());
        }
        if reader == "rules-registry-view" {
            return serde_json::to_vec(&vera_mod_api::ProductionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if (section, key, reader) == ("VeraFeatures", "AresFoundation", "bool") {
            return serde_json::to_vec(&(self.pass == 2)).map_err(|e| e.to_string());
        }
        if reader == "foundation-stock" {
            let input: ImmutableIniValue<u8> = serde_json::from_slice(current).unwrap();
            assert_eq!(key, "Foundation");
            assert_eq!(input.reference.source, FixedIniSource::FixedArt);
            assert_eq!(input.reference.source_hash, 37);
            // These are canonical host dual-read replies, not a guest parser.
            let (prior, selector) = match (self.pass, section, input.reference.section.as_str()) {
                (0 | 4, "A", "FIRST_INVALID") => (0, 4u8),
                (0, "B", "STOCK") => (0, 13),
                (1, "A", "NEW_IMAGE") => (4, 7),
                (1, "B", "OTHER_CUSTOM") => (13, 18),
                (6, "A", "NEW_IMAGE") => (0, 9),
                _ => panic!("unexpected stock ART read"),
            };
            assert_eq!(input.current, prior);
            return serde_json::to_vec(&selector).map_err(|e| e.to_string());
        }
        if reader == "source-string:2048" {
            let input: ImmutableIniValue<String> = serde_json::from_slice(current).unwrap();
            assert_eq!(section, input.reference.section);
            assert_eq!(
                key, "Foundation",
                "geometry slots must not cross the guest ABI"
            );
            assert_eq!(input.current, "");
            if self.pass == 4 {
                return Err("fixed ART scalar unavailable".into());
            }
            let header = match section {
                // Lock the reference without requesting dimensions or any of
                // the possible 1,048,576 outline slots from its fixed source.
                "FIRST_INVALID" => "Custom",
                "STOCK" => "4x3",
                "OTHER_CUSTOM" => "cUsToM",
                "NEW_IMAGE" if self.pass == 6 => "Custom",
                _ => panic!("a locked Custom must not reread the replacement Image"),
            };
            return serde_json::to_vec(header).map_err(|e| e.to_string());
        }
        panic!("unexpected fixed ART service {section}/{key}/{reader}");
    }

    fn registry(&self, registry: &str) -> Vec<String> {
        if registry == "$fixed-art" && self.pass != 5 {
            vec!["FixedArt".into()]
        } else {
            vec![]
        }
    }

    fn damage_kernel(&self, _: &vera_mod_api::DamageKernelInput) -> Result<i32, String> {
        panic!("foundation configuration has no damage kernel")
    }
}

fn request(config: Vec<u8>, pass: u8) -> Request {
    Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass: pass.into(),
            source: "decoded-fixed-ART-fixture".into(),
            final_pass: pass == 9,
            object_types: vec!["A".into(), "B".into()],
            warheads: vec![],
        }),
        config,
        state: vec![],
    }
}

fn configure(config: Vec<u8>, pass: u8) -> Vec<u8> {
    let request = request(config, pass);
    let plan = evaluate(&request, &ArtHost::new(pass)).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

fn policy(config: &[u8], ordinal: u32) -> Policy {
    frozen_query(
        config,
        Context::FoundationResource(FoundationResourceContext {
            building: building(ordinal),
        }),
    )
    .foundation_resource
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
fn foundation_latches_first_custom_while_off_and_freezes_only_small_references() {
    let first = configure(vec![], 0);
    assert_eq!(policy(&first, 0), Policy::Stock { selector: 4 });
    assert_eq!(policy(&first, 1), Policy::Stock { selector: 13 });
    let changed = configure(first, 1);
    assert_eq!(policy(&changed, 0), Policy::Stock { selector: 7 });
    assert_eq!(policy(&changed, 1), Policy::Stock { selector: 18 });
    let frozen = configure(configure(changed, 2), 9);
    assert_eq!(
        policy(&frozen, 0),
        Policy::ArtSection {
            reference: art("FIRST_INVALID")
        }
    );
    assert_eq!(
        policy(&frozen, 1),
        Policy::ArtSection {
            reference: art("OTHER_CUSTOM")
        }
    );
    let decoded: serde_json::Value = serde_json::from_slice(&frozen).unwrap();
    assert!(
        serde_json::to_vec(&decoded["foundation"]).unwrap().len() < 600,
        "source geometry size must not enlarge guest configuration"
    );
    assert!(serde_json::to_vec(&policy(&frozen, 0)).unwrap().len() < 200);
}

#[test]
fn foundation_boundaries_clear_latches_and_real_source_errors_are_not_hidden() {
    let frozen = configure(configure(configure(vec![], 0), 1), 2);
    let reset = boundary(frozen.clone(), ConfigurationBoundary::TypeReset);
    assert_eq!(policy(&reset, 0), Policy::Stock { selector: 0 });
    let rebuilt = configure(reset, 6);
    assert_eq!(
        policy(&rebuilt, 0),
        Policy::ArtSection {
            reference: art("NEW_IMAGE")
        }
    );
    let cleared = boundary(frozen, ConfigurationBoundary::ScenarioClear);
    assert_eq!(policy(&cleared, 1), Policy::Stock { selector: 0 });
    // Undiscovered ART is an empty domain for older decoded hosts; advertised
    // metadata/scalar services must instead propagate their real failures.
    assert_eq!(
        policy(&configure(vec![], 5), 0),
        Policy::Stock { selector: 0 }
    );
    assert!(
        evaluate(&request(vec![], 3), &ArtHost::new(3))
            .unwrap_err()
            .contains("fixed ART metadata unavailable")
    );
    assert!(
        evaluate(&request(vec![], 4), &ArtHost::new(4))
            .unwrap_err()
            .contains("fixed ART scalar unavailable")
    );
}
