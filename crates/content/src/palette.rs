//! The colour palette (`assets/data/palette.ron`, ADR-0012): named colours
//! that all drawing code refers to by name.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::bundle;
use crate::error::ContentError;
use crate::ron_loader::parse_ron;

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

/// The validated palette: colour name → RGB.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PaletteDef {
    /// Every colour in the file, by name.
    pub colors: BTreeMap<String, [u8; 3]>,
}

impl PaletteDef {
    /// Looks up a colour by name.
    pub fn get(&self, name: &str) -> Option<[u8; 3]> {
        self.colors.get(name).copied()
    }

    /// Loads and validates the embedded palette of the default theme.
    pub fn load() -> Result<Self, Vec<ContentError>> {
        Self::load_theme(Theme::default())
    }

    /// Loads and validates the embedded palette of `theme`.
    pub fn load_theme(theme: Theme) -> Result<Self, Vec<ContentError>> {
        let path = theme.path();
        let display = bundle::display_path(path);
        let source = bundle::file(path).ok_or_else(|| {
            vec![ContentError::new(
                &display,
                "file not found in asset bundle",
            )]
        })?;
        Self::from_source(&display, source)
    }

    /// Parses and validates palette `source`, attributing errors to `file`.
    /// Reports every bad hex value and every missing required colour.
    pub fn from_source(file: &str, source: &str) -> Result<Self, Vec<ContentError>> {
        let raw: BTreeMap<String, String> = parse_ron(file, source).map_err(|e| vec![e])?;
        let mut errors = Vec::new();
        let mut colors = BTreeMap::new();
        for (name, value) in &raw {
            match parse_hex(value) {
                Ok(rgb) => {
                    colors.insert(name.clone(), rgb);
                }
                Err(why) => {
                    let mut e = ContentError::new(file, format!("colour \"{name}\": {why}"));
                    if let Some(line) = line_of_key(source, name) {
                        e = e.at(line, None);
                    }
                    errors.push(e);
                }
            }
        }
        for &name in REQUIRED_COLORS {
            if !raw.contains_key(name) {
                errors.push(ContentError::new(
                    file,
                    format!("missing required colour \"{name}\""),
                ));
            }
        }
        if errors.is_empty() {
            Ok(Self { colors })
        } else {
            Err(errors)
        }
    }
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
        let palette = match PaletteDef::load_theme(theme) {
            Ok(p) => p,
            Err(e) => {
                errors.extend(e);
                continue;
            }
        };
        let file = bundle::display_path(theme.path());
        for name in base
            .colors
            .keys()
            .filter(|n| !palette.colors.contains_key(*n))
        {
            errors.push(ContentError::new(
                &file,
                format!("missing colour \"{name}\""),
            ));
        }
        for name in palette
            .colors
            .keys()
            .filter(|n| !base.colors.contains_key(*n))
        {
            errors.push(ContentError::new(
                &file,
                format!("colour \"{name}\" is not in the default theme"),
            ));
        }
        themes.insert(theme, palette);
    }
    if errors.is_empty() {
        Ok(themes)
    } else {
        Err(errors)
    }
}

/// Parses `#RRGGBB` (hex digits in either case) into RGB.
pub fn parse_hex(s: &str) -> Result<[u8; 3], String> {
    let Some(digits) = s.strip_prefix('#') else {
        return Err(format!("\"{s}\" must start with '#' (format \"#RRGGBB\")"));
    };
    if digits.len() != 6 || !digits.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(format!(
            "\"{s}\" must be '#' followed by exactly 6 hex digits (format \"#RRGGBB\")"
        ));
    }
    let channel = |i: usize| u8::from_str_radix(&digits[i..i + 2], 16).map_err(|e| e.to_string());
    Ok([channel(0)?, channel(2)?, channel(4)?])
}

/// Formats RGB as lowercase `#rrggbb`.
pub fn format_hex([r, g, b]: [u8; 3]) -> String {
    format!("#{r:02x}{g:02x}{b:02x}")
}

/// 1-based line of the first line containing `"key"`, for error positions.
fn line_of_key(source: &str, key: &str) -> Option<u32> {
    let needle = format!("\"{key}\"");
    let index = source.lines().position(|l| l.contains(&needle))?;
    u32::try_from(index + 1).ok()
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    fn full_palette_except(skip: &str, extra: &str) -> String {
        let lines: Vec<String> = REQUIRED_COLORS
            .iter()
            .filter(|&&name| name != skip)
            .map(|name| format!("    \"{name}\": \"#010203\",\n"))
            .collect();
        format!("{{\n{}{extra}}}", lines.concat())
    }

    #[test]
    fn hex_valid_upper_and_lower() {
        assert_eq!(parse_hex("#1a2B3c"), Ok([0x1a, 0x2b, 0x3c]));
        assert_eq!(parse_hex("#FFFFFF"), Ok([255, 255, 255]));
        assert_eq!(parse_hex("#000000"), Ok([0, 0, 0]));
    }

    #[test]
    fn hex_missing_hash() {
        assert!(parse_hex("1a2b3c").is_err_and(|e| e.contains("'#'")));
    }

    #[test]
    fn hex_wrong_length() {
        assert!(parse_hex("#1a2b3").is_err());
        assert!(parse_hex("#1a2b3c4").is_err());
        assert!(parse_hex("#").is_err());
    }

    #[test]
    fn hex_non_hex_digits() {
        assert!(parse_hex("#1a2g3c").is_err());
        assert!(parse_hex("#+1+2+3").is_err());
        // Multi-byte chars must not panic on slicing.
        assert!(parse_hex("#ééé").is_err());
    }

    #[test]
    fn format_hex_is_lowercase_padded() {
        assert_eq!(format_hex([0x0a, 0xbc, 0x01]), "#0abc01");
    }

    proptest! {
        #[test]
        fn hex_round_trips(c in any::<[u8; 3]>()) {
            prop_assert_eq!(parse_hex(&format_hex(c)), Ok(c));
            prop_assert_eq!(parse_hex(&format_hex(c).to_uppercase()), Ok(c));
        }
    }

    #[test]
    fn valid_palette_loads_all_colours() {
        let src = full_palette_except("", "    \"grass\": \"#00FF00\",\n");
        let p = PaletteDef::from_source("p.ron", &src).unwrap_or_default();
        assert_eq!(p.get("grass"), Some([0, 255, 0]));
        assert_eq!(p.get("player"), Some([1, 2, 3]));
        assert_eq!(p.colors.len(), REQUIRED_COLORS.len() + 1);
    }

    #[test]
    fn missing_required_colour_is_error() {
        let src = full_palette_except("player", "");
        let errs = PaletteDef::from_source("p.ron", &src)
            .err()
            .unwrap_or_default();
        assert_eq!(errs.len(), 1);
        assert_eq!(
            errs[0].to_string(),
            "p.ron: missing required colour \"player\""
        );
    }

    #[test]
    fn bad_hex_is_error_with_line() {
        let src = full_palette_except("", "    \"grass\": \"00ff00\",\n");
        let errs = PaletteDef::from_source("p.ron", &src)
            .err()
            .unwrap_or_default();
        assert_eq!(errs.len(), 1);
        let line = u32::try_from(REQUIRED_COLORS.len() + 2).unwrap_or(0);
        assert_eq!(errs[0].line, Some(line));
        assert!(errs[0].message.contains("\"grass\""));
        assert!(errs[0].message.contains("must start with '#'"));
    }

    #[test]
    fn multiple_errors_reported_together() {
        let src = full_palette_except(
            "enemy",
            "    \"grass\": \"#12\",\n    \"road\": \"#zzzzzz\",\n",
        );
        let errs = PaletteDef::from_source("p.ron", &src)
            .err()
            .unwrap_or_default();
        assert_eq!(errs.len(), 3);
        let text: Vec<String> = errs.iter().map(ToString::to_string).collect();
        assert!(text.iter().any(|t| t.contains("\"enemy\"")));
        assert!(text.iter().any(|t| t.contains("\"grass\"")));
        assert!(text.iter().any(|t| t.contains("\"road\"")));
    }

    #[test]
    fn syntax_error_is_single_positioned_error() {
        let errs = PaletteDef::from_source("p.ron", "{\n  \"a\" \"#000000\"\n}")
            .err()
            .unwrap_or_default();
        assert_eq!(errs.len(), 1);
        assert_eq!(errs[0].line, Some(2));
    }

    #[test]
    fn line_of_key_finds_quoted_key() {
        let src = "{\n \"text_dim\": \"#000000\",\n \"text\": \"#000000\",\n}";
        assert_eq!(line_of_key(src, "text"), Some(3));
        assert_eq!(line_of_key(src, "text_dim"), Some(2));
        assert_eq!(line_of_key(src, "nope"), None);
    }

    #[test]
    fn every_theme_has_the_default_themes_colour_names() {
        let base = PaletteDef::load().unwrap_or_default();
        let themes = load_themes(Some(&base)).unwrap_or_default();
        assert_eq!(themes.len(), Theme::ALL.len());
        for (theme, palette) in &themes {
            let names: Vec<_> = palette.colors.keys().collect();
            assert_eq!(names, base.colors.keys().collect::<Vec<_>>(), "{theme:?}");
        }
    }

    #[test]
    fn themes_differ_and_the_default_is_earthy() {
        assert_eq!(Theme::default(), Theme::Earthy);
        assert_eq!(Theme::Earthy.path(), PALETTE_PATH);
        let base = PaletteDef::load().unwrap_or_default();
        let themes = load_themes(Some(&base)).unwrap_or_default();
        for a in Theme::ALL {
            for b in Theme::ALL.into_iter().filter(|&b| b != a) {
                assert_ne!(themes.get(&a), themes.get(&b), "{a:?} = {b:?}");
            }
        }
    }

    #[test]
    fn a_theme_with_other_names_is_reported() {
        let mut base = PaletteDef::load().unwrap_or_default();
        base.colors.insert("extra".into(), [1, 2, 3]);
        base.colors.remove("peak");
        let errors = load_themes(Some(&base)).unwrap_err();
        // Each of the four files lacks "extra" and has "peak" besides.
        assert_eq!(errors.len(), 8, "{errors:?}");
    }

    #[test]
    fn embedded_palette_loads() {
        let p = PaletteDef::load().unwrap_or_default();
        for &name in REQUIRED_COLORS {
            assert!(p.get(name).is_some(), "missing {name}");
        }
    }
}
