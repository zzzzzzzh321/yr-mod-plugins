//! Installed resource recipes run before INI configuration. No Host services
//! or match state are needed; existing host parsers retain all resource data.
use vera_mod_api::{
    CsfLanguageGate, IncludeSemantics, StartupAudioCatalog, StartupAudioPair, StartupCsfCandidate,
    StartupCsfComposition, StartupResourceContext, StartupResourcePolicy,
};

pub(crate) fn recipe(context: &StartupResourceContext) -> StartupResourcePolicy {
    match context {
        // ReadCCFile4741F0 hooks474200/474314, tail10064660. The host's existing
        // traversal retains the independent Phobos dialect and VERA inheritance.
        StartupResourceContext::SourceComposition => StartupResourcePolicy::SourceComposition {
            include: IncludeSemantics::HashIncludeOnce,
        },
        StartupResourceContext::AudioCatalog => {
            StartupResourcePolicy::AudioCatalog { catalog: audio() }
        }
        StartupResourceContext::StringSupplements { base_language } => {
            StartupResourcePolicy::StringSupplements {
                composition: strings(*base_language),
            }
        }
    }
}

fn audio() -> StartupAudioCatalog {
    // Installed hook4011C0 ->Audio_Load1005ADD0 appends audio, ares, audio01..99.
    // Header stride, bad-pair admission, replacement and lookup stay host-owned.
    let pairs = ["audio".to_owned(), "ares".to_owned()]
        .into_iter()
        .chain((1..100).map(|index| format!("audio{index:02}")))
        .map(|stem| StartupAudioPair {
            index: format!("{stem}.idx"),
            bag: format!("{stem}.bag"),
        })
        .collect();
    StartupAudioCatalog::OrderedPairs { pairs }
}

fn strings(base_language: u32) -> StartupCsfComposition {
    // Installed string-table hook6BD886 ->10060DF0 uses GetLanguage734640.
    //10060E13 falls back to us, independent of the host OS locale.
    let short = match base_language {
        1 => "uk",
        2 => "ge",
        3 => "fr",
        4 => "sp",
        5 => "it",
        6 => "ja",
        7 => "jb",
        8 => "ko",
        9 => "ch",
        _ => "us",
    };
    let candidates = [
        StartupCsfCandidate {
            name: "ares.csf".into(),
            language: CsfLanguageGate::Ignore,
        },
        StartupCsfCandidate {
            name: format!("ares_{short}.csf"),
            language: CsfLanguageGate::MatchBaseOrNeutral,
        },
    ]
    .into_iter()
    .chain((0..100).map(|index| StartupCsfCandidate {
        name: format!("stringtable{index:02}.csf"),
        language: CsfLanguageGate::MatchBaseOrNeutral,
    }))
    .collect();
    StartupCsfComposition::OrderedRecords { candidates }
}
