//! Decoded Host replies exercise real configuration and JSON freezing. This
//! fixture does not parse INI, resolve names or duplicate native vector readers.
use super::*;
use crate::support::Host;
use serde_json::{Value, json};
use std::{cell::RefCell, collections::BTreeMap};
use vera_mod_api::{
    DamageKernelInput, FactionLookup, FactionNullableRef, FactionReadPoint, FactionReadValue,
    FactionRegistryPrefix, FactionRegistryView, ProductionObjectKind, ProductionTypeRef,
};

#[derive(Default)]
struct DecodedHost {
    physical: FactionRegistryView,
    scalars: BTreeMap<(String, String, String), Value>,
    scoped: BTreeMap<(u32, String, String, String), Value>,
    finds: BTreeMap<(u32, String, String), Value>,
    calls: RefCell<Vec<String>>,
}
impl DecodedHost {
    fn scalar(&mut self, owner: &str, key: &str, reader: &str, value: Value) {
        self.scalars
            .insert((owner.into(), key.into(), reader.into()), value);
    }
    fn scoped(&mut self, read: u32, owner: &str, key: &str, reader: &str, value: Value) {
        self.scoped
            .insert((read, owner.into(), key.into(), reader.into()), value);
    }
    fn find(&mut self, read: u32, family: &str, name: &str, value: Value) {
        self.finds.insert((read, family.into(), name.into()), value);
    }
}
impl Host for DecodedHost {
    fn config(&self, section: &str, key: &str) -> Option<String> {
        (self
            .scalars
            .keys()
            .any(|(s, k, _)| s == section && k == key)
            || self
                .scoped
                .keys()
                .any(|(_, s, k, _)| s == section && k == key))
        .then(|| "opaque physical value; the native Host has decoded it".into())
    }
    fn config_read(
        &self,
        section: &str,
        key: &str,
        reader: &str,
        current: &[u8],
    ) -> Result<Vec<u8>, String> {
        self.calls
            .borrow_mut()
            .push(format!("{section}/{key}/{reader}"));
        let value = if reader == "faction-registry-view" {
            return serde_json::to_vec(&self.physical).map_err(|e| e.to_string());
        } else if let Some(family) = reader.strip_prefix("faction-find:") {
            let query: FactionLookup = serde_json::from_slice(current).unwrap();
            self.finds
                .get(&(query.read, family.into(), query.name))
                .expect("unexpected literal lookup")
        } else if reader.starts_with("faction-") {
            let query: FactionReadValue<Value> = serde_json::from_slice(current).unwrap();
            self.scoped
                .get(&(query.read, section.into(), key.into(), reader.into()))
                .expect("unexpected scoped reader")
        } else {
            self.scalars
                .get(&(section.into(), key.into(), reader.into()))
                .expect("unexpected ordinary reader")
        };
        serde_json::to_vec(value).map_err(|e| e.to_string())
    }
    fn registry(&self, _: &str) -> Vec<String> {
        panic!("no duplicate registry lookup")
    }
    fn damage_kernel(&self, _: &DamageKernelInput) -> Result<i32, String> {
        panic!("no simulation numeric call")
    }
}
fn prefix(sides: u32, countries: u32) -> FactionRegistryPrefix {
    FactionRegistryPrefix {
        sides,
        countries,
        infantry: 8,
        units: 8,
        aircraft: 8,
        buildings: 8,
        animations: 8,
        colors: 16,
    }
}
fn point(site: FactionReadSite, prefix: FactionRegistryPrefix) -> FactionReadPoint {
    FactionReadPoint { prefix, site }
}
fn country_point(
    country: u32,
    parent: &str,
    side: i32,
    primary_rules: bool,
    count: u32,
) -> FactionReadPoint {
    point(
        FactionReadSite::CountrySuffix {
            country,
            primary_rules,
            body_read: true,
            parent: parent.into(),
            side,
        },
        prefix(0, count),
    )
}
fn pass(config: &mut Configuration, host: &DecodedHost) {
    let view = ConfigView::new(host);
    config
        .configure(
            &ConfigureContext {
                pass: 0,
                source: "actual-physical-pass".into(),
                final_pass: false,
                object_types: vec![],
                warheads: vec![],
            },
            &view,
        )
        .unwrap();
    view.finish().unwrap();
}
fn country_policy(
    config: &Configuration,
    ordinal: u32,
    id: &str,
) -> vera_mod_api::FactionCountryPolicy {
    let query = FactionResourceContext::Country {
        ordinal,
        stored_id: id.into(),
    };
    let policy = config.query(&query, true);
    policy.validate_for(&query).unwrap();
    match policy {
        FactionResourcePolicy::Country { policy: Some(p) } => p,
        _ => unreachable!(),
    }
}

#[test]
fn side_physical_defaults_nullable_identity_and_frozen_off_policy_share_one_state() {
    let mut first = DecodedHost {
        physical: FactionRegistryView {
            sides: vec!["Nod".into()],
            countries: vec![],
            reads: vec![point(FactionReadSite::SideSweep { side: 0 }, prefix(1, 0))],
        },
        ..Default::default()
    };
    first.find(0, "aircraft", "PDPLANE", Value::Null);
    first.scoped(
        0,
        "Nod",
        "Crew",
        "faction-nullable-ref:infantry",
        json!(FactionNullableRef::<ProductionTypeRef>::Null),
    );
    first.scoped(
        0,
        "Nod",
        "AI.BaseDefenses",
        "faction-nullable-vector:buildings",
        json!([]),
    );
    let mut config = Configuration::default();
    pass(&mut config, &first);
    // ID selects Nod defaults; ordinal0 independently selects Allied score art.
    let p = &config.sides[&0].policy;
    assert_eq!((p.eva_voice_index, p.sidebar_mix_file_index), (1, 2));
    assert_eq!(
        p.presentation.campaign_score_background.as_deref(),
        Some("ASCRBKMD.SHP")
    );
    assert_eq!(p.paradrop_aircraft, FactionNullableRef::Null);
    assert_eq!(p.crew, FactionNullableRef::Null);
    assert_eq!(p.engineer, FactionNullableRef::Unset);
    assert_eq!(p.base_defenses, Some(vec![]));
    let mut second = DecodedHost {
        physical: first.physical.clone(),
        ..Default::default()
    };
    second.scoped(
        0,
        "Nod",
        "Crew",
        "faction-nullable-ref:infantry",
        json!(FactionNullableRef::Resolved(ProductionTypeRef {
            category: ProductionObjectKind::Infantry,
            ordinal: 2
        })),
    );
    // No second PDPLANE reply: phase2 cannot repeat the initializer's Find.
    pass(&mut config, &second);
    let frozen = serde_json::to_vec(&config).unwrap();
    let restored: Configuration = serde_json::from_slice(&frozen).unwrap();
    let query = FactionResourceContext::Side {
        ordinal: 0,
        stored_id: "Nod".into(),
    };
    assert_eq!(
        restored.query(&query, false),
        FactionResourcePolicy::Side { policy: None }
    );
    match restored.query(&query, true) {
        FactionResourcePolicy::Side { policy: Some(p) } => {
            assert_eq!(
                p.crew,
                FactionNullableRef::Resolved(ProductionTypeRef {
                    category: ProductionObjectKind::Infantry,
                    ordinal: 2
                })
            );
            assert_eq!(p.engineer, FactionNullableRef::Unset);
            assert_eq!(p.paradrop_aircraft, FactionNullableRef::Null);
        }
        _ => unreachable!(),
    }
    // A final projection is not another native visit, even with a physical Host.
    let view = ConfigView::new(&second);
    let before = second.calls.borrow().len();
    let final_context = ConfigureContext {
        pass: 9,
        source: "projection".into(),
        final_pass: true,
        object_types: vec![],
        warheads: vec![],
    };
    config.configure(&final_context, &view).unwrap();
    assert_eq!(second.calls.borrow().len(), before);
}

#[test]
fn country_primary_before_one_shot_parent_copy_and_late_names_survive_freezing() {
    let mut first = DecodedHost {
        physical: FactionRegistryView {
            sides: vec![],
            countries: vec!["Child".into(), "Parent".into(), "Orphan".into()],
            reads: vec![
                country_point(0, "Parent", 0, true, 3),
                country_point(1, "Parent", 1, true, 3),
                country_point(2, "Missing", -1, false, 3),
            ],
        },
        ..Default::default()
    };
    first.find(0, "color", "AlliedLoad", json!(6));
    first.find(0, "country", "Parent", json!(1));
    first.find(1, "color", "SovietLoad", Value::Null);
    first.find(2, "country", "Missing", Value::Null);
    first.scalar("Child", "File.LoadScreen", "name:2048", json!("child.shp"));
    first.scoped(0, "Child", "File.Flag", "faction-pcx", json!("child.pcx"));
    first.scalar(
        "Child",
        "AIRandomSelectionWeight",
        "ares-nullable-int",
        json!(5),
    );
    first.scalar(
        "Parent",
        "File.LoadScreen",
        "name:2048",
        json!("parent.shp"),
    );
    first.scalar(
        "Parent",
        "LoadScreenText.Name",
        "name:2048",
        json!("Name:Parent"),
    );
    first.scalar("Parent", "RandomSelectionWeight", "ares-int", json!(7));
    first.scalar("Parent", "ListIndex", "ares-int", json!(i32::MAX));
    first.scoped(
        1,
        "Parent",
        "VeteranBuildings",
        "faction-valueable-vector:buildings",
        json!([4]),
    );
    first.scoped(
        1,
        "Parent",
        "Parachute.Anim",
        "faction-valueable-ref:animation",
        json!(3),
    );
    first.scoped(
        1,
        "Parent",
        "ParaDrop.Types",
        "faction-valueable-vector:paradroppables",
        json!([ProductionTypeRef {
            category: ProductionObjectKind::Infantry,
            ordinal: 2
        }]),
    );
    first.scoped(
        1,
        "Parent",
        "ParaDrop.Num",
        "faction-valueable-ints",
        json!([-3]),
    );
    let mut config = Configuration::default();
    pass(&mut config, &first);
    let child = country_policy(&config, 0, "Child");
    assert_eq!(
        child.loading.file, "ls%sobs.shp",
        "copy observes the later parent's constructor, after overriding primary child art"
    );
    assert_eq!(child.presentation.flag.as_deref(), Some("rani.pcx"));
    assert_eq!(
        (
            child.presentation.list_index,
            child.presentation.ai_random_weight
        ),
        (101, Some(5))
    );
    assert!(child.veteran_buildings.is_empty());
    let mut second = DecodedHost {
        physical: FactionRegistryView {
            sides: vec![],
            countries: vec![
                "Child".into(),
                "Parent".into(),
                "Orphan".into(),
                "Later".into(),
                "Missing".into(),
            ],
            reads: vec![
                country_point(0, "Parent", 0, false, 5),
                country_point(2, "Missing", -1, false, 5),
                country_point(3, "Parent", -1, false, 5),
            ],
        },
        ..Default::default()
    };
    second.find(2, "country", "Parent", json!(1));
    // No repeat lookup for Child or Orphan, even though Missing now exists.
    pass(&mut config, &second);
    let frozen = serde_json::to_vec(&config).unwrap();
    let restored: Configuration = serde_json::from_slice(&frozen).unwrap();
    assert_eq!(country_policy(&restored, 0, "Child"), child);
    let later = country_policy(&restored, 3, "Later");
    assert_eq!(
        (later.loading.file.as_str(), later.loading.name.as_str()),
        ("parent.shp", "Name:Parent")
    );
    assert_eq!(
        (
            later.presentation.list_index,
            later.presentation.random_weight,
            later.presentation.ai_random_weight
        ),
        (i32::MIN, 7, None)
    );
    assert_eq!(later.veteran_buildings, [4]);
    assert_eq!(later.paradrop.counts, [-3]);
    assert!(
        country_policy(&restored, 2, "Orphan")
            .veteran_buildings
            .is_empty()
    );
    let query = FactionResourceContext::Country {
        ordinal: 3,
        stored_id: "Later".into(),
    };
    assert_eq!(
        restored.query(&query, false),
        FactionResourcePolicy::Country { policy: None }
    );
    assert!(
        Configuration::default().countries.is_empty(),
        "TypeReset/ScenarioClear use the same domain reset"
    );
}
