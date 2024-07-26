use super::{asset_from_contents, Error, Result, COMPRESS_LAZY_THEMES};

use std::collections::BTreeMap;
use std::convert::TryFrom;

use serde::Deserialize;
use serde::Serialize;

use once_cell::unsync::OnceCell;

use syntect::highlighting::{Theme, ThemeSet};

/// Same structure as a [`syntect::highlighting::ThemeSet`] but with themes
/// stored in raw serialized form, and deserialized on demand.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LazyThemeSet {
    /// This is a [`BTreeMap`] because that's what [`syntect::highlighting::ThemeSet`] uses
    themes: BTreeMap<String, LazyTheme>,
}

/// Stores raw serialized data for a theme with methods to lazily deserialize
/// (load) the theme.
#[derive(Debug, Serialize, Deserialize)]
struct LazyTheme {
    serialized: Vec<u8>,

    #[serde(skip, default = "OnceCell::new")]
    deserialized: OnceCell<Theme>,
}

impl LazyThemeSet {
    /// Lazily load the given theme
    pub fn get(&self, name: &str) -> Option<&Theme> {
        self.themes.get(name).and_then(|lazy_theme| {
            lazy_theme
                .deserialized
                .get_or_try_init(|| lazy_theme.deserialize())
                .ok()
        })
    }

    /// Returns the name of all themes.
    pub fn themes(&self) -> impl Iterator<Item = &str> {
        self.themes.keys().map(|name| name.as_ref())
    }
}

impl LazyTheme {
    fn deserialize(&self) -> Result<Theme> {
        asset_from_contents(
            &self.serialized[..],
            "lazy-loaded theme",
            COMPRESS_LAZY_THEMES,
        )
    }
}

impl TryFrom<LazyThemeSet> for ThemeSet {
    type Error = Error;

    /// Since the user might want to add custom themes to bat, we need a way to
    /// convert from a `LazyThemeSet` to a regular [`ThemeSet`] so that more
    /// themes can be added. This function does that pretty straight-forward
    /// conversion.
    fn try_from(lazy_theme_set: LazyThemeSet) -> Result<Self> {
        let mut theme_set = ThemeSet::default();

        for (name, lazy_theme) in lazy_theme_set.themes {
            theme_set.themes.insert(name, lazy_theme.deserialize()?);
        }

        Ok(theme_set)
    }
}
