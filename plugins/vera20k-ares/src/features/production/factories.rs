//! Ares TechnoTypeExt1003DEC0/1003E8D9/1003E973 and independent
//! BuildingExt1000DCF0/1000DD40 retained declarations, without live factories.
use super::registry::Registry;
use crate::support::{Host, Section};
use serde::{Deserialize, Serialize};
use vera_mod_api::{
    ProductionObjectKind, ProductionObjectPolicy, ProductionRegistryPrefix, ProductionRequirement,
};

#[derive(Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Declarations {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) prerequisites: Vec<ProductionRequirement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub(super) prerequisite_override: Vec<ProductionRequirement>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    built_at: Vec<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    factory_owners: Vec<u32>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    forbidden_factory_owners: Vec<u32>,
    #[serde(skip_serializing_if = "is_false")]
    permanent_factory_plan: bool,
    #[serde(skip_serializing_if = "is_false")]
    factory_has_all_plans: bool,
    #[serde(skip_serializing_if = "is_false")]
    factory_explicit_only: bool,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct ObjectState {
    techno_phase: i32,
    building_phase: i32,
    #[serde(flatten)]
    pub(super) declarations: Declarations,
}

impl ObjectState {
    pub(super) fn is_default(&self) -> bool {
        // Both native first-phase initializers leave these ctor fields alone
        // (Type1003DEC0, Building100105A0). For this mechanism phases0/1/2
        // read identically, so an all-default phase2 need not occupy a sparse
        // record. Non-readable phases remain retained even with empty fields.
        (0..=2).contains(&self.techno_phase)
            && (0..=2).contains(&self.building_phase)
            && self.declarations == Declarations::default()
    }

    pub(super) fn read_techno<H: Host>(
        &mut self,
        section: &Section<'_, H>,
        registry: &Registry<'_>,
        prefix: ProductionRegistryPrefix,
    ) {
        if !(0..=2).contains(&self.techno_phase) {
            return;
        }
        if self.techno_phase == 0 {
            self.techno_phase = 1;
        }
        //1003E8D9 precedes Country vectors and both permanent-plan aliases.
        if let Some(tokens) = section.read_comma_list("BuiltAt", 2048) {
            self.declarations.built_at = tokens
                .into_iter()
                .filter_map(|token| registry.existing_building(&token, prefix))
                .collect();
        }
        for (key, destination) in [
            ("FactoryOwners", &mut self.declarations.factory_owners),
            (
                "FactoryOwners.Forbidden",
                &mut self.declarations.forbidden_factory_owners,
            ),
        ] {
            if let Some(tokens) = section.read_comma_list(key, 2048) {
                *destination = tokens
                    .into_iter()
                    .filter_map(|token| registry.country(&token, prefix))
                    .collect();
            }
        }
        self.declarations.permanent_factory_plan = section.read_ares_bool(
            "FactoryOwners.HaveAllPlans",
            self.declarations.permanent_factory_plan,
        );
        self.declarations.permanent_factory_plan = section.read_ares_bool(
            "FactoryOwners.Permanent",
            self.declarations.permanent_factory_plan,
        );
        self.declarations.factory_has_all_plans = section.read_ares_bool(
            "FactoryOwners.HasAllPlans",
            self.declarations.factory_has_all_plans,
        );
        self.techno_phase = 2;
    }

    pub(super) fn read_building<H: Host>(
        &mut self,
        section: &Section<'_, H>,
        factory: Option<ProductionObjectKind>,
        unit_repair: bool,
    ) {
        if !(0..=2).contains(&self.building_phase) {
            return;
        }
        if self.building_phase == 0 {
            self.building_phase = 1;
        }
        //1000DD79..1000DD89 skips common fields but still completes phase2.
        if !(unit_repair && factory == Some(ProductionObjectKind::Aircraft)) {
            self.declarations.factory_explicit_only = section.read_ares_bool(
                "Factory.ExplicitOnly",
                self.declarations.factory_explicit_only,
            );
        }
        self.building_phase = 2;
    }

    pub(super) fn query(&self) -> ProductionObjectPolicy {
        let d = &self.declarations;
        ProductionObjectPolicy {
            prerequisites: d.prerequisites.clone(),
            prerequisite_override: d.prerequisite_override.clone(),
            built_at: d.built_at.clone(),
            factory_owners: d.factory_owners.clone(),
            forbidden_factory_owners: d.forbidden_factory_owners.clone(),
            permanent_factory_plan: d.permanent_factory_plan,
            factory_has_all_plans: d.factory_has_all_plans,
            factory_explicit_only: d.factory_explicit_only,
        }
    }
}
