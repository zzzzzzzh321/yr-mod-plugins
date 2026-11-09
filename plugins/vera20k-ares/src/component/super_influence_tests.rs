//! Predecoded physical reader replies exercise component retention/identity;
//! the engine tests own the canonical vector and numeric parser coverage.
use super::*;
use super::tests::{FrozenHost, empty_super_sources, frozen_query};
use std::cell::{Cell, RefCell};
use vera_mod_api::{
    ConfigurationBoundary, ConfigureBoundaryContext, ConfigureContext, ConfigureSuperReadContext,
    DamageKernelInput, ProductionObjectKind as Kind, ProductionReadPoint, ProductionReadSite,
    ProductionRegisteredType, ProductionRegistryPrefix, ProductionRegistryView, ProductionTypeRef,
    SuperReadValue, SuperWeaponReadPoint, SuperWeaponReadSite, SuperWeaponRegistryView,
    SuperWeaponResourceContext, SuperWeaponResourcePolicy,
};

fn object(category: Kind, ordinal: u32) -> ProductionTypeRef {
    ProductionTypeRef { category, ordinal }
}

struct DecodedHost {
    pass: u32,
    physical_calls: Cell<u32>,
    fields: RefCell<Vec<String>>,
}

impl DecodedHost {
    fn new(pass: u32) -> Self {
        Self {
            pass,
            physical_calls: Cell::new(0),
            fields: RefCell::new(vec![]),
        }
    }

    fn physical(&self) -> ProductionRegistryView {
        let technos: Vec<_> = [
            (Kind::Building, 0, "SAME"),
            (Kind::Unit, 0, "SAME"),
            (Kind::Unit, 1, "SAME"),
            (Kind::Unit, 2, "LATE"),
        ]
        .into_iter()
        .map(|(category, ordinal, name)| ProductionRegisteredType {
            identity: object(category, ordinal),
            name: name.into(),
        })
        .collect();
        let reached = match self.pass {
            0 => vec![object(Kind::Building, 0), object(Kind::Unit, 0)],
            1 => vec![object(Kind::Unit, 1)],
            _ => vec![object(Kind::Unit, 0)],
        };
        ProductionRegistryView {
            buildings: vec!["SAME".into()],
            countries: vec![],
            technos,
            reads: reached
                .into_iter()
                .map(|object| ProductionReadPoint {
                    prefix: ProductionRegistryPrefix {
                        buildings: 1,
                        countries: 0,
                        technos: if self.pass == 0 { 2 } else { 3 },
                    },
                    site: ProductionReadSite::TechnoExtension { object },
                })
                .collect(),
        }
    }
}

impl Host for DecodedHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        matches!(
            (section, key),
            ("BODY", "SW.AnyDesignator" | "SW.AnyInhibitor")
                | ("SAME" | "LATE", "DesignatorRange" | "InhibitorRange")
        )
        .then(|| "predecoded physical value".into())
    }

    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        if reader == "superweapon-registry-view" {
            return serde_json::to_vec(&SuperWeaponRegistryView {
                superweapons: vec!["STRIKE".into(), "LATE".into()],
                reads: (0..=self.pass)
                    .map(|_| SuperWeaponReadPoint {
                        site: SuperWeaponReadSite::Type {
                            ordinal: 0,
                            section: Some("BODY".into()),
                        },
                    })
                    .collect(),
            })
            .map_err(|e| e.to_string());
        }
        if let Some(reply) = empty_super_sources(section, key, reader, current) {
            return reply;
        }
        if reader == "rules-registry-view" {
            assert_eq!(
                self.physical_calls.replace(1),
                0,
                "share the one physical directory"
            );
            return serde_json::to_vec(&self.physical()).map_err(|e| e.to_string());
        }
        if reader == "faction-registry-view" {
            return serde_json::to_vec(&vera_mod_api::FactionRegistryView::default())
                .map_err(|e| e.to_string());
        }
        if reader == "ares-nullable-int" {
            assert_eq!(
                section, "SAME",
                "constructor-only LATE must not read a body"
            );
            self.fields.borrow_mut().push(key.into());
            return Ok(match (self.pass, key) {
                (0, "DesignatorRange") => b"13".to_vec(),
                (0, "InhibitorRange") => b"-2".to_vec(),
                (1, "DesignatorRange") => b"0".to_vec(),
                (1, "InhibitorRange") => b"-3".to_vec(),
                (2, _) => current.to_vec(), // decoded invalid/empty retains nullable state
                _ => panic!("unexpected nullable range read"),
            });
        }
        if reader == "ares-bool" {
            assert_eq!(section, "BODY");
            self.fields.borrow_mut().push(key.into());
            return Ok(if self.pass == 1 {
                current.to_vec()
            } else {
                serde_json::to_vec(&(self.pass == 0 && key == "SW.AnyDesignator")).unwrap()
            });
        }
        if reader == "comma-mask:2048" {
            // Section::read_comma_mask passes its token vocabulary directly,
            // unlike the scoped SuperReadValue envelope used below.
            assert_eq!(section, "BODY");
            let _: Vec<(String, u32)> = serde_json::from_slice(current).unwrap();
            return Ok(b"null".to_vec());
        }
        let input: SuperReadValue<serde_json::Value> = serde_json::from_slice(current).unwrap();
        assert_eq!((section, input.read), ("BODY", self.pass));
        if reader == "super-techno-list" {
            assert!(input.current.is_null(), "no name-based guest resolution");
            self.fields.borrow_mut().push(key.into());
            let reply = match (self.pass, key) {
                (0, "SW.Designators") => Some(vec![
                    object(Kind::Building, 0),
                    object(Kind::Unit, 0),
                    object(Kind::Unit, 0),
                ]),
                (0, "SW.Inhibitors") => Some(vec![object(Kind::Unit, 0)]),
                (1, "SW.Designators") => None, // decoded absent/empty vector retains
                (1, "SW.Inhibitors") => Some(vec![object(Kind::Unit, 1)]),
                (2, "SW.Designators" | "SW.Inhibitors") => Some(vec![]), // none/comma-only/unknown replaces
                _ => panic!("unexpected scoped list"),
            };
            return serde_json::to_vec(&reply).map_err(|e| e.to_string());
        }
        match reader {
            "super-country-mask" | "super-building-list" => Ok(b"null".to_vec()),
            "super-animation-reference" | "super-eva-reference" | "super-effective:ares-int" => {
                serde_json::to_vec(&input.current).map_err(|e| e.to_string())
            }
            _ => panic!("unexpected influence fixture reader {reader}"),
        }
    }

    fn registry(&self, _: &str) -> Vec<String> {
        vec![]
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("configuration only")
    }
}

fn visit(config: Vec<u8>, pass: u32) -> (Vec<u8>, Vec<String>) {
    let host = DecodedHost::new(pass);
    let request = Request {
        api_version: API_VERSION,
        context: Context::ConfigureSuperRead(ConfigureSuperReadContext {
            read: pass,
            ordinal: 0,
            stored_id: "STRIKE".into(),
            body_read: true,
            effective_type: Some("IronCurtain".into()),
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &host).unwrap();
    plan.validate_for(&request).unwrap();
    (plan.config.unwrap(), host.fields.into_inner())
}

fn configure(config: Vec<u8>, pass: u32) -> Vec<u8> {
    let host = DecodedHost::new(pass);
    let request = Request {
        api_version: API_VERSION,
        context: Context::Configure(ConfigureContext {
            pass: u64::from(pass),
            source: "decoded physical influence".into(),
            final_pass: false,
            object_types: vec![],
            warheads: vec![],
        }),
        config,
        state: vec![],
    };
    let plan = evaluate(&request, &host).unwrap();
    plan.validate_for(&request).unwrap();
    assert_eq!(host.physical_calls.get(), 1);
    let fields = host.fields.into_inner();
    for pair in fields.chunks_exact(2) {
        assert_eq!(pair, ["DesignatorRange", "InhibitorRange"]);
    }
    plan.config.unwrap()
}

fn query(config: &[u8]) -> SuperWeaponResourcePolicy {
    frozen_query(
        config,
        Context::SuperWeaponResource(SuperWeaponResourceContext {}),
    )
    .superweapon_resource
    .unwrap()
}

#[test]
fn influence_physical_history_keeps_exact_refs_and_nullable_ranges_through_frozen_query() {
    let (root, fields) = visit(vec![], 0);
    assert_eq!(
        fields,
        [
            "SW.Designators",
            "SW.AnyDesignator",
            "SW.Inhibitors",
            "SW.AnyInhibitor"
        ]
    );
    let root = configure(root, 0);
    let (mode, _) = visit(root, 1);
    let mode = configure(mode, 1);
    let policy = query(&mode);
    let influence = policy.definitions[0].influence.as_ref().unwrap();
    assert_eq!(
        influence.designators,
        [
            object(Kind::Building, 0),
            object(Kind::Unit, 0),
            object(Kind::Unit, 0)
        ]
    );
    assert_eq!(influence.inhibitors, [object(Kind::Unit, 1)]);
    assert!(influence.any_designator && !influence.any_inhibitor);
    assert_eq!(
        policy.definitions[1].influence,
        Some(Default::default()),
        "late SW constructor defaults"
    );
    let ranges = policy.techno_influence.unwrap();
    assert_eq!(ranges.len(), 3, "late constructor-only Techno stays unset");
    for (identity, designator, inhibitor) in [
        (object(Kind::Building, 0), 13, -2),
        (object(Kind::Unit, 0), 13, -2),
        (object(Kind::Unit, 1), 0, -3),
    ] {
        let range = ranges
            .iter()
            .find(|range| range.object == identity)
            .unwrap();
        assert_eq!(
            (range.designator_range, range.inhibitor_range),
            (Some(designator), Some(inhibitor))
        );
    }
    let (map, _) = visit(mode, 2);
    let map = configure(map, 2);
    let policy = query(&map);
    let influence = policy.definitions[0].influence.as_ref().unwrap();
    assert!(influence.designators.is_empty() && influence.inhibitors.is_empty());
    assert!(!influence.any_designator && !influence.any_inhibitor);
    assert_eq!(
        policy.techno_influence.unwrap(),
        ranges,
        "invalid/empty scalars retain exact per-slot overrides"
    );
}

#[test]
fn influence_cohort_does_not_upgrade_old_common_config_until_real_reset() {
    // A nonempty .37 configuration already has both older cohorts, but no
    // influence domain or Type fields. Ordinary Configure must preserve that.
    let (fresh, _) = visit(vec![], 0);
    let mut old: serde_json::Value = serde_json::from_slice(&fresh).unwrap();
    let domain = old["superweapons"].as_object_mut().unwrap();
    domain.remove("techno_influence");
    for state in domain["types"].as_object_mut().unwrap().values_mut() {
        state.as_object_mut().unwrap().remove("influence");
    }
    let (old, fields) = visit(serde_json::to_vec(&old).unwrap(), 1);
    assert!(
        fields.is_empty(),
        "old common/nuclear cohorts do not enable influence reads"
    );
    let old = configure(old, 1);
    let policy = query(&old);
    assert!(policy.techno_influence.is_none());
    assert!(
        policy
            .definitions
            .iter()
            .all(|definition| definition.influence.is_none())
    );
    let serialized = String::from_utf8(old.clone()).unwrap();
    assert!(!serialized.contains("techno_influence") && !serialized.contains("\"influence\""));
    for boundary in [
        ConfigurationBoundary::ScenarioClear,
        ConfigurationBoundary::TypeReset,
    ] {
        let request = Request {
            api_version: API_VERSION,
            context: Context::ConfigureBoundary(ConfigureBoundaryContext { boundary }),
            config: old.clone(),
            state: vec![],
        };
        let reset = evaluate(&request, &FrozenHost).unwrap().config.unwrap();
        assert_eq!(query(&reset).techno_influence, Some(vec![]));
        let (configured, fields) = visit(reset, 0);
        assert_eq!(fields.len(), 4);
        assert!(query(&configured).definitions[0].influence.is_some());
    }
}
