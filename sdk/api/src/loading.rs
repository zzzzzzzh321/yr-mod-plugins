//! Read-only loading-theme selection at the original physical source frontier.
//! These names are not inherited faction state or a startup catalog recipe.
use crate::{FactionResourceContext, FactionResourcePolicy};
use serde::{Deserialize, Serialize};

pub const MAX_LOADING_THEME_NAME: usize = 127;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LoadingThemeResourceContext {
    /// The retained Side and Country select sections in the same frozen root.
    /// A missing/unallocated identity skips that one native ReadTheme call.
    Rules {
        side: Option<FactionResourceContext>,
        country: Option<FactionResourceContext>,
    },
    /// The already parsed campaign map supplies its Basic section.
    Campaign,
}
impl LoadingThemeResourceContext {
    pub fn validate(&self) -> Result<(), String> {
        if let Self::Rules { side, country } = self {
            if let Some(identity) = side {
                if !matches!(identity, FactionResourceContext::Side { .. }) {
                    return Err("loading Side has the wrong identity family".into());
                }
                FactionResourcePolicy::Side { policy: None }.validate_for(identity)?;
            }
            if let Some(identity) = country {
                if !matches!(identity, FactionResourceContext::Country { .. }) {
                    return Err("loading Country has the wrong identity family".into());
                }
                FactionResourcePolicy::Country { policy: None }.validate_for(identity)?;
            }
        }
        Ok(())
    }
    pub fn sections(&self) -> Vec<String> {
        match self {
            Self::Rules { side, country } => side
                .iter()
                .chain(country.iter())
                .map(|identity| match identity {
                    FactionResourceContext::Side { stored_id, .. }
                    | FactionResourceContext::Country { stored_id, .. } => stored_id.clone(),
                })
                .collect(),
            Self::Campaign => vec!["Basic".into()],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub enum LoadingThemePolicy {
    Retail,
    /// Ordered ReadTheme values: Side then Country, or campaign Basic alone.
    /// Missing and empty values remain distinct from an unknown nonempty name;
    /// the original Theme owner applies its retained -2/-1/default behavior.
    Names {
        values: Vec<Option<String>>,
    },
}
impl LoadingThemePolicy {
    pub fn validate_for(&self, context: &LoadingThemeResourceContext) -> Result<(), String> {
        context.validate()?;
        if let Self::Names { values } = self {
            let count = match context {
                LoadingThemeResourceContext::Rules { .. } => 2,
                LoadingThemeResourceContext::Campaign => 1,
            };
            if values.len() != count
                || values
                    .iter()
                    .flatten()
                    .any(|value| value.chars().count() > MAX_LOADING_THEME_NAME)
            {
                return Err("invalid loading theme name count or ReadString128 capacity".into());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{API_VERSION, Context, Plan, Request};

    #[test]
    fn loading_names_observe_config_but_cannot_write_or_carry_entity_state() {
        let mut request = Request {
            api_version: API_VERSION,
            context: Context::LoadingThemeResource(LoadingThemeResourceContext::Campaign),
            config: vec![1],
            state: Vec::new(),
        };
        request.validate().unwrap();
        let mut plan = Plan::unchanged(&request);
        plan.loading_theme = Some(LoadingThemePolicy::Names {
            values: vec![Some(String::new())],
        });
        plan.validate_for(&request).unwrap();
        plan.config = Some(vec![2]);
        assert!(plan.validate_for(&request).is_err());
        plan.config = None;
        plan.loading_theme = Some(LoadingThemePolicy::Names {
            values: vec![None, None],
        });
        assert!(plan.validate_for(&request).is_err());
        plan.loading_theme = Some(LoadingThemePolicy::Names {
            values: vec![Some("x".repeat(128))],
        });
        assert!(plan.validate_for(&request).is_err());
        request.state.push(1);
        assert!(request.validate().is_err());

        let wrong_family = LoadingThemeResourceContext::Rules {
            side: Some(FactionResourceContext::Country {
                ordinal: 0,
                stored_id: "Country".into(),
            }),
            country: None,
        };
        assert!(wrong_family.validate().is_err());
    }
}
