//! Decoded Host responses exercise composition, not a second INI parser.
use super::*;
use crate::component::tests::{FrozenHost, frozen_query};
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, ProductionObjectKind as Kind,
    ProductionReadPoint, ProductionReadSite as Site, ProductionRegisteredType,
    ProductionRegistryPrefix as Prefix, ProductionRegistryView, ProductionRequirement as Req,
    ProductionResourceContext as Query, ProductionResourcePolicy as Policy,
    ProductionTypeRef as Ref,
};

struct ProductionHost(u8);

impl ProductionHost {
    fn value(&self, section: &str, key: &str) -> Option<serde_json::Value> {
        use serde_json::json;
        match (self.0, section, key) {
            (0 | 3, "VeraFeatures", "AresProduction") => Some(json!(false)),
            (1, "VeraFeatures", "AresProduction") => Some(json!(true)),
            (0..=2, "GlobalControls", "AllowBypassBuildLimit") => Some(json!("host prefix")),
            (0, "GenericPrerequisites", "custom") => Some(json!(["future"])),
            (0, "GenericPrerequisites", "future") => Some(json!(["A"])),
            (0, "General", "PrerequisiteCustom") => Some(json!(["A"])),
            (0, "General", "PrerequisiteCustomAlternate") => Some(json!(["CONTROL"])),
            (0, "CONTROL", "Prerequisite") => Some(json!(["LATE", "custom", "POWER", "A"])),
            (0, "CONTROL", "PrerequisiteOverride") => Some(json!(["custom"])),
            (0, "CONTROL", "BuiltAt") => Some(json!(["LATE", "A", "A"])),
            (0, "CONTROL", "FactoryOwners") => Some(json!(["LATE_COUNTRY", "C1", "C1"])),
            (0, "CONTROL", "FactoryOwners.Forbidden") => Some(json!(["C1"])),
            (0 | 1, "CONTROL", "FactoryOwners.HaveAllPlans") => Some(json!(true)),
            (0, "CONTROL", "FactoryOwners.Permanent") => Some(json!(false)),
            (1, "CONTROL", "FactoryOwners.Permanent") => Some(json!("native invalid retains")),
            (0, "CONTROL", "FactoryOwners.HasAllPlans") => Some(json!(true)),
            (1, "CONTROL", "FactoryOwners") => Some(serde_json::Value::Null),
            (1, "CONTROL", "BuiltAt") => Some(json!([])),
            (0, "SOURCE", "FactoryOwners") => Some(json!(["C1"])),
            (0, "SOURCE", "Factory.ExplicitOnly") => Some(json!(true)),
            (1, "SOURCE", "Factory.ExplicitOnly") => Some(json!(false)),
            _ => None,
        }
    }

    fn view(&self) -> ProductionRegistryView {
        let identity = |category, ordinal| Ref { category, ordinal };
        let typed = |category, ordinal, name: &str| ProductionRegisteredType {
            identity: identity(category, ordinal),
            name: name.into(),
        };
        let early = Prefix {
            buildings: 1,
            countries: 1,
            technos: 3,
        };
        let later = Prefix {
            buildings: 2,
            countries: 1,
            technos: 4,
        };
        let full = Prefix {
            buildings: 4,
            countries: 2,
            technos: 6,
        };
        let read = |prefix, site| ProductionReadPoint { prefix, site };
        let control = identity(Kind::Unit, 0);
        let infantry = identity(Kind::Infantry, 0);
        let source = identity(Kind::Building, 2);
        let reads = match self.0 {
            0 => vec![
                read(early, Site::ProcessEntry),
                read(early, Site::BeforeTypeData),
                read(early, Site::TechnoPrerequisite { object: control }),
                read(early, Site::TechnoExtension { object: control }),
                read(later, Site::TechnoPrerequisite { object: infantry }),
                read(later, Site::TechnoExtension { object: infantry }),
                read(full, Site::TechnoExtension { object: source }),
                read(
                    full,
                    Site::BuildingExtension {
                        object: source,
                        factory: Some(Kind::Unit),
                        unit_repair: false,
                    },
                ),
            ],
            1 => vec![
                read(full, Site::ProcessEntry),
                read(full, Site::BeforeTypeData),
                read(full, Site::TechnoPrerequisite { object: control }),
                read(full, Site::TechnoExtension { object: control }),
                read(
                    full,
                    Site::BuildingExtension {
                        object: source,
                        factory: Some(Kind::Aircraft),
                        unit_repair: true,
                    },
                ),
            ],
            2 => vec![read(full, Site::ProcessEntry)],
            _ => vec![],
        };
        ProductionRegistryView {
            buildings: ["A", "POWER", "SOURCE", "LATE"].map(str::to_owned).into(),
            countries: ["C1", "LATE_COUNTRY"].map(str::to_owned).into(),
            technos: vec![
                typed(Kind::Building, 0, "A"),
                typed(Kind::Unit, 0, "CONTROL"),
                typed(Kind::Infantry, 0, "CONTROL"),
                typed(Kind::Building, 1, "POWER"),
                typed(Kind::Building, 2, "SOURCE"),
                typed(Kind::Building, 3, "LATE"),
            ],
            reads,
        }
    }
}

impl Host for ProductionHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        self.value(section, key)
            .map(|_| "decoded host fixture".into())
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "faction-registry-view" {
            // This domain fixture has no Side/Country reader dispatch.
            assert_eq!((section, key, current), ("", "", b"null".as_slice()));
            return serde_json::to_vec(&vera_mod_api::FactionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if reader == "rules-registry-view" {
            assert!(section.is_empty() && key.is_empty());
            return serde_json::to_vec(&self.view()).map_err(|e| e.to_string());
        }
        if reader == "bool-prefix-reverse3" {
            assert_eq!((section, key), ("GlobalControls", "AllowBypassBuildLimit"));
            let prior: [bool; 3] = serde_json::from_slice(current).unwrap();
            let flags = match self.0 {
                0 => {
                    assert_eq!(prior, [false; 3]);
                    [true; 3]
                }
                1 => {
                    assert_eq!(prior, [true; 3]);
                    [true, true, false]
                }
                2 => {
                    assert_eq!(prior, [true, true, false]);
                    [true, false, false]
                }
                _ => panic!("unexpected global prefix"),
            };
            return serde_json::to_vec(&flags).map_err(|e| e.to_string());
        }
        if self.0 == 1 && key == "FactoryOwners.Permanent" {
            assert_eq!(reader, "ares-bool");
            return Ok(current.to_vec());
        }
        let expected = if key == "AresProduction" {
            "bool"
        } else if key.starts_with("FactoryOwners.") && key != "FactoryOwners.Forbidden"
            || key == "Factory.ExplicitOnly"
        {
            "ares-bool"
        } else {
            "comma-list:2048"
        };
        assert_eq!(reader, expected, "{section}/{key}");
        serde_json::to_vec(&self.value(section, key).unwrap()).map_err(|e| e.to_string())
    }
    fn registry(&self, name: &str) -> Vec<String> {
        if self.0 == 0 && name == "GenericPrerequisites" {
            vec!["custom".into(), "future".into()]
        } else {
            vec![]
        }
    }
    fn damage_kernel(&self, _: &vera_mod_api::DamageKernelInput) -> Result<i32, String> {
        panic!("production declarations never compute damage")
    }
}

fn configure(bytes: Vec<u8>, pass: u8) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(vera_mod_api::ConfigureContext {
            pass: pass.into(),
            source: "decoded-production-fixture".into(),
            final_pass: pass == 5,
            object_types: vec!["CONTROL".into(), "SOURCE".into()],
            warheads: vec![],
        }),
        config: bytes,
        state: vec![],
    };
    let plan = evaluate(&request, &ProductionHost(pass)).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

fn boundary(bytes: Vec<u8>, boundary: ConfigurationBoundary) -> Vec<u8> {
    let request = Request {
        api_version: API_VERSION,
        context: Context::ConfigureBoundary(ConfigureBoundaryContext { boundary }),
        config: bytes,
        state: vec![],
    };
    let plan = evaluate(&request, &FrozenHost).unwrap();
    plan.validate_for(&request).unwrap();
    plan.config.unwrap()
}

fn query(bytes: &[u8], context: Query) -> Option<Policy> {
    frozen_query(bytes, Context::ProductionResource(context)).production_resource
}

#[test]
fn production_layers_phases_and_process_boundaries_keep_their_own_state() {
    let mut initial = serde_json::to_value(decode_config(&[]).unwrap()).unwrap();
    initial["production"]["objects"] = serde_json::json!({
        "infantry":{"CONTROL":{"techno_phase":3}},
        "building":{"SOURCE":{"techno_phase":3}},
    });
    let first = configure(serde_json::to_vec(&initial).unwrap(), 0);
    let second = configure(first, 1);
    let frozen = configure(second, 5);
    let Some(Policy::Catalog {
        groups,
        ai_bypass_build_limit,
        semantics,
    }) = query(&frozen, Query::Catalog)
    else {
        panic!("missing production catalog")
    };
    assert_eq!(ai_bypass_build_limit, [true, true, false]);
    assert!(semantics.require_factory_for_build && semantics.factory_country_plans);
    assert_eq!(groups[6].members, [Req::Group(7)]);
    assert_eq!(
        groups[6].alternates,
        [Ref {
            category: Kind::Unit,
            ordinal: 0
        }]
    );
    let object = |category, name: &str| Query::Object {
        category,
        name: name.into(),
    };
    let Some(Policy::Object { policy }) = query(&frozen, object(Kind::Unit, "CONTROL")) else {
        panic!("missing unit")
    };
    assert_eq!(
        policy.prerequisites,
        [Req::Group(6), Req::Group(0), Req::Building(0)]
    );
    assert_eq!(policy.prerequisite_override, [Req::Group(6)]);
    assert_eq!(policy.factory_owners, [0, 0]);
    assert_eq!(policy.forbidden_factory_owners, [0]);
    assert!(policy.built_at.is_empty());
    assert!(policy.permanent_factory_plan && policy.factory_has_all_plans);
    let Some(Policy::Object { policy }) = query(&frozen, object(Kind::Infantry, "CONTROL")) else {
        panic!("missing infantry")
    };
    assert_eq!(
        policy.prerequisites,
        [Req::Group(6), Req::Building(1), Req::Building(0)]
    );
    assert!(
        policy.factory_owners.is_empty(),
        "non-readable TypeExt phase remains skipped"
    );
    let Some(Policy::Object { policy }) = query(&frozen, object(Kind::Building, "SOURCE")) else {
        panic!("missing factory")
    };
    assert!(
        policy.factory_explicit_only && policy.factory_owners.is_empty(),
        "independent Building phase reads then aircraft-repair skip retains"
    );
    let disabled = configure(frozen, 3);
    assert!(query(&disabled, Query::Catalog).is_none());
    let reset = boundary(disabled, ConfigurationBoundary::TypeReset);
    let Some(Policy::Catalog {
        groups,
        ai_bypass_build_limit,
        ..
    }) = query(&reset, Query::Catalog)
    else {
        panic!("TypeReset must restore new-match feature selection")
    };
    assert_eq!(groups.len(), 6);
    assert_eq!(ai_bypass_build_limit, [true, true, false]);
    let Some(Policy::Object { policy }) = query(&reset, object(Kind::Building, "SOURCE")) else {
        panic!("missing constructor policy")
    };
    assert_eq!(policy, vera_mod_api::ProductionObjectPolicy::default());
    let after = configure(reset, 2);
    let Some(Policy::Catalog {
        ai_bypass_build_limit,
        ..
    }) = query(&after, Query::Catalog)
    else {
        panic!("missing retained controls")
    };
    assert_eq!(ai_bypass_build_limit, [true, false, false]);
    // Boundary routing precedes validation of the outgoing feature selection.
    let mut invalid: serde_json::Value = serde_json::from_slice(&after).unwrap();
    invalid["features"]["overload"] = true.into();
    let cleared = boundary(
        serde_json::to_vec(&invalid).unwrap(),
        ConfigurationBoundary::ScenarioClear,
    );
    let Some(Policy::Catalog {
        ai_bypass_build_limit,
        ..
    }) = query(&cleared, Query::Catalog)
    else {
        panic!("missing reconstructed controls")
    };
    assert_eq!(ai_bypass_build_limit, [false; 3]);
    let state: serde_json::Value = serde_json::from_slice(&cleared).unwrap();
    assert_eq!(state["production"]["globals"]["phase"], 0);
}
