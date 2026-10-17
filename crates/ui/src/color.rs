//! The game's colours (ADR-0012): the [`UiColor`]s its screens and skins
//! draw with, each a name its palette must have. Colours by name ([`Rgb`],
//! [`Palette`]) are Shuyi Engine's (`shuyi_ui::frame::color`, ticket 1108);
//! [`UiColor`] is this game's [`ColorSet`], and says which of its colours
//! the engine's own colours are ([`engine_color`]).

pub use shuyi_ui::frame::color::{
    ColorSet, DIM_DARK, DIM_GREY, EngineColor, Palette, Rgb, required_names, to_channel,
};

/// Declares [`UiColor`] with its palette names, in `REQUIRED_COLORS` order.
macro_rules! ui_colors {
    ($($variant:ident => $name:literal,)*) => {
        /// A colour the UI needs. Every variant's [`name`](UiColor::name) is a
        /// required palette colour (`trpg_content::palette::REQUIRED_COLORS`),
        /// so looking one up can't fail.
        #[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
        pub enum UiColor {
            $(
                #[doc = concat!("Palette colour `", $name, "`.")]
                $variant,
            )*
        }

        impl UiColor {
            /// Every variant, in `REQUIRED_COLORS` order.
            pub const ALL: &[UiColor] = &[$(UiColor::$variant,)*];

            /// The colour's name in the palette file.
            pub const fn name(self) -> &'static str {
                match self {
                    $(UiColor::$variant => $name,)*
                }
            }
        }

        impl ColorSet for UiColor {
            const ALL: &'static [Self] = UiColor::ALL;

            fn name(self) -> &'static str {
                UiColor::name(self)
            }

            fn index(self) -> usize {
                self as usize
            }

            fn engine_name(color: EngineColor) -> &'static str {
                engine_color(color).name()
            }
        }
    };
}

// check-text: not player text (the colours' names in the palette file)
ui_colors! {
    Black => "black",
    White => "white",
    Text => "text",
    TextDim => "text_dim",
    TextHighlight => "text_highlight",
    PanelBg => "panel_bg",
    PanelBorder => "panel_border",
    PanelBorderFocus => "panel_border_focus",
    Player => "player",
    Enemy => "enemy",
    Ally => "ally",
    Neutral => "neutral",
    PlayerOutline => "player_outline",
    MoveRange => "move_range",
    AttackRange => "attack_range",
    HealRange => "heal_range",
    DangerZone => "danger_zone",
    Effect => "effect",
    Cursor => "cursor",
    Path => "path",
    HpHigh => "hp_high",
    HpMid => "hp_mid",
    HpLow => "hp_low",
    ExpBar => "exp_bar",
}

/// The game's colour each of the engine's is painted in.
pub const fn engine_color(color: EngineColor) -> UiColor {
    match color {
        EngineColor::Text => UiColor::Text,
        EngineColor::TextDim => UiColor::TextDim,
        EngineColor::Panel => UiColor::PanelBg,
        EngineColor::Border => UiColor::PanelBorder,
        EngineColor::Highlight => UiColor::TextHighlight,
        // HP running low is the game's warning.
        EngineColor::Warning => UiColor::HpLow,
        EngineColor::Focus => UiColor::PanelBorderFocus,
        EngineColor::Background => UiColor::Black,
        // The player's own color: what moves their cursor.
        EngineColor::Accent => UiColor::Player,
        EngineColor::Bright => UiColor::White,
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use trpg_content::PaletteDef;
    use trpg_content::palette::REQUIRED_COLORS;

    use super::*;

    /// The embedded game palette.
    pub(crate) fn game_palette() -> Palette {
        Palette::new::<UiColor>(&trpg_content::palette::load().unwrap()).unwrap()
    }

    fn def(pairs: &[(&str, [u8; 3])]) -> PaletteDef {
        PaletteDef {
            colors: pairs.iter().map(|&(n, c)| (n.to_owned(), c)).collect(),
        }
    }

    #[test]
    fn ui_color_names_are_exactly_required_colors() {
        let names: Vec<&str> = UiColor::ALL.iter().map(|c| c.name()).collect();
        assert_eq!(names, REQUIRED_COLORS);
        // The engine's colours are among them: the palette check is the same.
        assert_eq!(required_names::<UiColor>(), REQUIRED_COLORS);
    }

    #[test]
    fn ui_color_index_matches_all_order() {
        for (i, c) in UiColor::ALL.iter().enumerate() {
            assert_eq!(*c as usize, i);
            assert_eq!(ColorSet::index(*c), i);
            assert_eq!(ColorSet::name(*c), c.name());
        }
        assert_eq!(<UiColor as ColorSet>::ALL, UiColor::ALL);
    }

    #[test]
    fn the_engines_colours_are_the_games_panel_colours() {
        let p = game_palette();
        let names: Vec<&str> = EngineColor::ALL
            .iter()
            .map(|&e| <UiColor as ColorSet>::engine_name(e))
            .collect();
        assert_eq!(
            names,
            [
                "text",
                "text_dim",
                "panel_bg",
                "panel_border",
                "text_highlight",
                "hp_low",
                "panel_border_focus",
                "black",
                "player",
                "white"
            ]
        );
        for &e in EngineColor::ALL {
            assert_eq!(p.engine(e), p.get(engine_color(e)));
        }
    }

    #[test]
    fn palette_get_and_lookup() {
        let p = game_palette();
        let def = trpg_content::palette::load().unwrap();
        for c in UiColor::ALL {
            assert_eq!(p.get(*c), Rgb::from(def.get(c.name()).unwrap()));
        }
        assert_eq!(p.lookup("grass"), def.get("grass").map(Rgb::from));
        assert_eq!(p.lookup("no_such_colour"), None);
        let all: Vec<(&str, Rgb)> = p.iter().collect();
        assert_eq!(all.len(), def.colors.len());
        assert!(all.windows(2).all(|w| w[0].0 < w[1].0));
        assert!(all.iter().all(|&(n, c)| p.lookup(n) == Some(c)));
    }

    #[test]
    fn palette_new_reports_missing_ui_colors() {
        assert_eq!(
            Palette::new::<UiColor>(&PaletteDef::default()),
            Err(REQUIRED_COLORS.to_vec())
        );
        let mut d = trpg_content::palette::load().unwrap();
        d.colors.remove("cursor");
        d.colors.remove("text");
        assert_eq!(Palette::new::<UiColor>(&d), Err(vec!["text", "cursor"]));
    }

    #[test]
    fn name_of_prefers_ui_names_then_alphabetical() {
        let mut pairs: Vec<(&str, [u8; 3])> =
            REQUIRED_COLORS.iter().map(|&n| (n, [9, 9, 9])).collect();
        pairs.push(("zeta", [1, 2, 3]));
        pairs.push(("alpha", [1, 2, 3]));
        pairs.push(("aaa", [9, 9, 9]));
        pairs.push(("text_dim", [4, 4, 4]));
        let p = Palette::new::<UiColor>(&def(&pairs)).unwrap();
        assert_eq!(p.name_of(Rgb::new(9, 9, 9)), Some("black"));
        assert_eq!(p.name_of(Rgb::new(1, 2, 3)), Some("alpha"));
        assert_eq!(p.name_of(Rgb::new(4, 4, 4)), Some("text_dim"));
        assert_eq!(p.name_of(Rgb::new(7, 7, 7)), None);
    }
}
