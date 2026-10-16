//! The colour palette (`assets/data/palette.ron`, ADR-0012): named colours
//! that all drawing code refers to by name. Named colours, hex and theme
//! files are Shuyi Engine's (`shuyi_assets::palette`, ticket 1104); this
//! module holds the game's themes and the colours its UI needs.
use crate::prelude::*;

use alloc::collections::BTreeMap;

use serde::{Deserialize, Serialize};
pub use shuyi_assets::palette::{PaletteDef, format_hex, line_of_key, parse_hex, theme_problems};

use crate::bundle;
use crate::error::ContentError;

/// Path of the default theme's palette inside the asset bundle.
pub const PALETTE_PATH: &str = "data/palette.ron";

/// A colour theme the player can pick (ticket 0806,
/// `docs/design/look-and-feel.md`): a palette file of its own. A theme
/// recolours glyphs and the UI, never a picture.
#[derive(
    Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
)]
pub enum Theme {
    /// C, "Rich & painterly".
    Painterly,
    /// D, "Earthy painterly": the look the game was made in.
    #[default]
    Earthy,
    /// E, "War-table parchment": a light theme.
    Parchment,
    /// G, "Moonlit".
    Moonlit,
}

impl Theme {
    /// Every theme, in the order the Options
    /// screen steps through them.
    pub const ALL: [Self; 4] = [
        Self::Painterly,
        Self::Earthy,
        Self::Parchment,
        Self::Moonlit,
    ];

    /// The palette file inside the asset bundle.
    pub const fn path(self) -> &'static str {
        match self {
            Self::Painterly => "data/palettes/painterly.ron",
            Self::Earthy => PALETTE_PATH,
            Self::Parchment => "data/palettes/parchment.ron",
            Self::Moonlit => "data/palettes/moonlit.ron",
        }
    }

    /// The key of the name the Options screen shows.
    pub const fn key(self) -> &'static str {
        match self {
            Self::Painterly => "options.theme.painterly",
            Self::Earthy => "options.theme.earthy",
            Self::Parchment => "options.theme.parchment",
            Self::Moonlit => "options.theme.moonlit",
        }
    }
}

/// Colour names the UI relies on; the palette must define every one.
pub const REQUIRED_COLORS: &[&str] = &[
    "black",
    "white",
    "text",
    "text_dim",
    "text_highlight",
    "panel_bg",
    "panel_border",
    "panel_border_focus",
    "player",
    "enemy",
    "ally",
    "neutral",
    "player_outline",
    "move_range",
    "attack_range",
    "heal_range",
    "danger_zone",
    "effect",
    "cursor",
    "path",
    "hp_high",
    "hp_mid",
    "hp_low",
    "exp_bar",
];

/// Loads and validates the embedded palette of the default theme.
pub fn load() -> Result<PaletteDef, Vec<ContentError>> {
    load_theme(Theme::default())
}

/// Loads and validates the embedded palette of `theme`.
pub fn load_theme(theme: Theme) -> Result<PaletteDef, Vec<ContentError>> {
    let path = theme.path();
    let display = bundle::display_path(path);
    let source = bundle::file(path).ok_or_else(|| {
        vec![ContentError::new(
            &display,
            "file not found in asset bundle",
        )]
    })?;
    from_source(&display, source)
}

/// Parses and validates palette `source`, attributing errors to `file`.
/// Reports every bad hex value and every missing [`REQUIRED_COLORS`] one.
pub fn from_source(file: &str, source: &str) -> Result<PaletteDef, Vec<ContentError>> {
    PaletteDef::from_source(file, source, REQUIRED_COLORS)
}

/// Loads every theme's palette. Each must define exactly the colour
/// names of the default theme's `base` (a name missing would leave
/// something unpainted; an extra one is a typo). Without a `base`
/// (it failed to load, and said so) there are none to check.
pub fn load_themes(
    base: Option<&PaletteDef>,
) -> Result<BTreeMap<Theme, PaletteDef>, Vec<ContentError>> {
    let Some(base) = base else {
        return Ok(BTreeMap::new());
    };
    let mut errors = Vec::new();
    let mut themes = BTreeMap::new();
    for theme in Theme::ALL {
        let palette = match load_theme(theme) {
            Ok(p) => p,
            Err(e) => {
                errors.extend(e);
                continue;
            }
        };
        let file = bundle::display_path(theme.path());
        errors.extend(theme_problems(&file, base, &palette));
        themes.insert(theme, palette);
    }
    if errors.is_empty() {
        Ok(themes)
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_games_colours_are_required() {
        let lines: Vec<String> = REQUIRED_COLORS
            .iter()
            .filter(|&&name| name != "exp_bar")
            .map(|name| format!("    \"{name}\": \"#010203\",\n"))
            .collect();
        let src = format!("{{\n{}}}", lines.concat());
        let errs = from_source("p.ron", &src).err().unwrap_or_default();
        let text: Vec<String> = errs.iter().map(ToString::to_string).collect();
        assert_eq!(text, ["p.ron: missing required colour \"exp_bar\""]);
    }

    #[test]
    fn every_theme_has_the_default_themes_colour_names() {
        let base = load().unwrap_or_default();
        let themes = load_themes(Some(&base)).unwrap_or_default();
        assert_eq!(themes.len(), Theme::ALL.len());
        for (theme, palette) in &themes {
            let names: Vec<_> = palette.colors.keys().collect();
            assert_eq!(names, base.colors.keys().collect::<Vec<_>>(), "{theme:?}");
        }
        assert_eq!(load_themes(None), Ok(BTreeMap::new()));
    }

    #[test]
    fn themes_differ_and_the_default_is_earthy() {
        assert_eq!(Theme::default(), Theme::Earthy);
        assert_eq!(Theme::Earthy.path(), PALETTE_PATH);
        let base = load().unwrap_or_default();
        let themes = load_themes(Some(&base)).unwrap_or_default();
        for a in Theme::ALL {
            for b in Theme::ALL.into_iter().filter(|&b| b != a) {
                assert_ne!(themes.get(&a), themes.get(&b), "{a:?} = {b:?}");
            }
        }
    }

    #[test]
    fn each_theme_has_its_own_text_key() {
        let keys: Vec<_> = Theme::ALL.iter().map(|t| t.key()).collect();
        assert_eq!(
            keys,
            [
                "options.theme.painterly",
                "options.theme.earthy",
                "options.theme.parchment",
                "options.theme.moonlit"
            ]
        );
    }

    #[test]
    fn a_theme_with_other_names_is_reported() {
        let mut base = load().unwrap_or_default();
        base.colors.insert("extra".into(), [1, 2, 3]);
        base.colors.remove("peak");
        let errors = load_themes(Some(&base)).unwrap_err();
        // Each of the four files lacks "extra" and has "peak" besides.
        assert_eq!(errors.len(), 8, "{errors:?}");
    }

    #[test]
    fn embedded_palette_loads() {
        let p = load().unwrap_or_default();
        for &name in REQUIRED_COLORS {
            assert!(p.get(name).is_some(), "missing {name}");
        }
    }
}
