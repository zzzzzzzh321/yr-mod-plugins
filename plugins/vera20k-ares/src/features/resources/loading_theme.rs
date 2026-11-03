//! Ares1002FB80/683C70/683D05 reads names at the original loading frontier.
//! The host supplies the physical source and native identities. Theme lookup,
//! retained defaults and playback remain in the engine's existing Theme owner.
use crate::support::{ConfigView, Host};
use vera_mod_api::{FactionResourceContext, LoadingThemePolicy, LoadingThemeResourceContext};

pub(crate) fn query<H: Host>(
    view: &ConfigView<'_, H>,
    context: &LoadingThemeResourceContext,
    enabled: bool,
) -> LoadingThemePolicy {
    if !enabled {
        return LoadingThemePolicy::Retail;
    }
    let read = |id: &str| view.section_or_empty(id).read_name("LoadingTheme", 128);
    let read_identity = |identity: &Option<FactionResourceContext>| {
        identity.as_ref().and_then(|identity| match identity {
            FactionResourceContext::Side { stored_id, .. }
            | FactionResourceContext::Country { stored_id, .. } => read(stored_id),
        })
    };
    LoadingThemePolicy::Names {
        values: match context {
            LoadingThemeResourceContext::Rules { side, country } => {
                vec![read_identity(side), read_identity(country)]
            }
            LoadingThemeResourceContext::Campaign => vec![read("Basic")],
        },
    }
}
