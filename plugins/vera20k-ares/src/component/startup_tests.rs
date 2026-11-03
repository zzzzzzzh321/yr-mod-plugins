//! Real component evaluation with decoded wire requests; FrozenHost rejects any
//! attempted INI, registry, or numeric service access before source admission.
use super::*;
use crate::component::tests::FrozenHost;
use vera_mod_api::{
    CsfLanguageGate, IncludeSemantics, StartupAudioCatalog, StartupCsfComposition,
    StartupResourceContext as Query, StartupResourcePolicy as Policy,
};

fn query(context: Query) -> Plan {
    let request = Request {
        api_version: API_VERSION,
        context: Context::StartupResources(context),
        config: vec![],
        state: vec![],
    };
    let request: Request = serde_json::from_slice(&serde_json::to_vec(&request).unwrap()).unwrap();
    let plan = evaluate(&request, &FrozenHost).unwrap();
    let plan: Plan = serde_json::from_slice(&serde_json::to_vec(&plan).unwrap()).unwrap();
    plan.validate_for(&request).unwrap();
    assert!(plan.config.is_none() && plan.state.is_empty());
    plan
}

#[test]
fn startup_source_and_audio_recipes_are_ordered_installation_declarations() {
    assert!(manifest().hooks.contains(&HookKind::StartupResources));
    // The frozen schema now retains the additional faction configuration.
    assert_eq!(
        (manifest().state_version, manifest().config_version),
        (1, 15)
    );
    assert_eq!(
        query(Query::SourceComposition).startup_resources,
        Some(Policy::SourceComposition {
            include: IncludeSemantics::HashIncludeOnce
        })
    );
    let Some(Policy::AudioCatalog {
        catalog: StartupAudioCatalog::OrderedPairs { pairs },
    }) = query(Query::AudioCatalog).startup_resources
    else {
        panic!("missing ordered audio recipe");
    };
    assert_eq!(pairs.len(), 101);
    assert_eq!(
        (pairs[0].index.as_str(), pairs[0].bag.as_str()),
        ("audio.idx", "audio.bag")
    );
    assert_eq!(
        (pairs[1].index.as_str(), pairs[1].bag.as_str()),
        ("ares.idx", "ares.bag")
    );
    assert_eq!(
        (pairs[2].index.as_str(), pairs[2].bag.as_str()),
        ("audio01.idx", "audio01.bag")
    );
    assert_eq!(
        (pairs[100].index.as_str(), pairs[100].bag.as_str()),
        ("audio99.idx", "audio99.bag")
    );
}

#[test]
fn startup_csf_recipe_uses_base_language_and_keeps_supplement_admission_order() {
    let Some(Policy::StringSupplements {
        composition: StartupCsfComposition::OrderedRecords { candidates },
    }) = query(Query::StringSupplements { base_language: 9 }).startup_resources
    else {
        panic!("missing ordered CSF recipe");
    };
    assert_eq!(candidates.len(), 102);
    assert_eq!(candidates[0].name, "ares.csf");
    assert_eq!(candidates[0].language, CsfLanguageGate::Ignore);
    assert_eq!(candidates[1].name, "ares_ch.csf");
    assert_eq!(candidates[1].language, CsfLanguageGate::MatchBaseOrNeutral);
    assert_eq!(candidates[2].name, "stringtable00.csf");
    assert_eq!(candidates[101].name, "stringtable99.csf");
    assert_eq!(
        candidates[101].language,
        CsfLanguageGate::MatchBaseOrNeutral
    );
    let Some(Policy::StringSupplements {
        composition:
            StartupCsfComposition::OrderedRecords {
                candidates: unknown,
            },
    }) = query(Query::StringSupplements {
        base_language: u32::MAX,
    })
    .startup_resources
    else {
        panic!("missing unknown-language fallback");
    };
    assert_eq!(unknown[1].name, "ares_us.csf");
    assert_eq!(unknown[1].language, CsfLanguageGate::MatchBaseOrNeutral);
}
