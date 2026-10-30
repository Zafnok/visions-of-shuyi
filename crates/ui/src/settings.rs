//! The player's settings (ticket 0805): Shuyi Engine's
//! (`shuyi_ui::settings`, ticket 1109: text speed, fullscreen, the color
//! theme, the volumes and the layout picked) and this game's own
//! ([`ShuyiSettings`]: animation and enemy-phase speeds, combat
//! animations, auto-end, the cursor style and the map's look and zoom).
//! Saved as one RON record under [`SETTINGS_KEY`], whose form this game
//! keeps (ADR-0050); [`Ctx`] loads them with its storage and saves them on
//! every change ([`Ctx::change_settings`]). Key bindings are saved apart
//! (ADR-0031). Each player profile has its own (ADR-0069).
//!
//! [`Ctx`]: crate::screen::Ctx
//! [`Ctx::change_settings`]: shuyi_ui::Ctx::change_settings

use crate::prelude::*;
use serde::{Deserialize, Serialize};

use crate::input::Layout;
use trpg_content::Theme;

use crate::map_view::CursorStyle;

pub use shuyi_ui::settings::{DEFAULT_VOLUME, GameSettings, MAX_VOLUME, SETTINGS_KEY, TextSpeed};

/// This game's settings: the engine's, with this game's
/// ([`Settings::game`](shuyi_ui::settings::Settings::game)).
pub type Settings = shuyi_ui::settings::Settings<ShuyiSettings>;

/// The saved format's version.
pub const SETTINGS_VERSION: u32 = 1;

/// How much faster animations play at [`AnimSpeed::Fast`]. *Tunable.*
pub const FAST_ANIM: f32 = 2.0;

/// How fast a battle's animations play while Confirm is held
/// (`docs/design/controls.md`): instead of the settings' speed, not on top
/// of it.
pub const HELD_SPEED: f32 = 4.0;

/// How much faster the Enemy and Other phases play at
/// [`EnemyPhaseSpeed::Fast`], on top of the animation speed. *Tunable.*
pub const FAST_ENEMY_PHASE: f32 = 2.0;

/// How fast battle animations play: a fight's playback and the EXP bar,
/// and under a sprite skin a unit's walk (`docs/design/options.md`).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum AnimSpeed {
    /// The speed the game always had.
    #[default]
    Normal,
    /// [`FAST_ANIM`] times as fast.
    Fast,
}

impl AnimSpeed {
    /// Every speed, slowest first.
    pub const ALL: [Self; 2] = [Self::Normal, Self::Fast];

    /// The multiplier on animation time.
    pub const fn factor(self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Fast => FAST_ANIM,
        }
    }

    /// The key of the name the Options screen shows
    /// ([`Ctx::text`](shuyi_ui::Ctx::text)).
    pub const fn key(self) -> &'static str {
        match self {
            Self::Normal => "options.value.normal",
            Self::Fast => "options.value.fast",
        }
    }
}

/// How fast the Enemy and Other phases play.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnemyPhaseSpeed {
    /// The speed the game always had.
    #[default]
    Normal,
    /// [`FAST_ENEMY_PHASE`] times as fast.
    Fast,
}

impl EnemyPhaseSpeed {
    /// Every speed, slowest first.
    pub const ALL: [Self; 2] = [Self::Normal, Self::Fast];

    /// The multiplier on animation time in the AI's phases.
    pub const fn factor(self) -> f32 {
        match self {
            Self::Normal => 1.0,
            Self::Fast => FAST_ENEMY_PHASE,
        }
    }

    /// The key of the name the Options screen shows
    /// ([`Ctx::text`](shuyi_ui::Ctx::text)).
    pub const fn key(self) -> &'static str {
        match self {
            Self::Normal => "options.value.normal",
            Self::Fast => "options.value.fast",
        }
    }
}

/// How the battle map looks (ticket 0824, ADR-0038): the bought pictures,
/// or the glyphs the public repository shows. Terrain and units switch
/// together.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapLook {
    /// The bought tiles and unit sprites (a build without them has only
    /// glyphs).
    #[default]
    Pictures,
    /// Letters for units, symbols for terrain.
    Glyphs,
}

impl MapLook {
    /// Every look, the default first.
    pub const ALL: [Self; 2] = [Self::Pictures, Self::Glyphs];

    /// The key of the name the Options screen shows
    /// ([`Ctx::text`](shuyi_ui::Ctx::text)).
    pub const fn key(self) -> &'static str {
        match self {
            Self::Pictures => "options.map_look.pictures",
            Self::Glyphs => "options.map_look.glyphs",
        }
    }
}

/// How big the battle map's pictures are drawn (ticket 0439,
/// `docs/design/look-and-feel.md`, *Zoom*). The Zoom map key in battle
/// flips it. Only a skin that paints the whole map from pictures zooms.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MapZoom {
    /// The art's own size: a 16-pixel tile.
    X1,
    /// Twice that: a 32-pixel tile. What a new player starts at (Nick,
    /// 2026-10-12).
    #[default]
    X2,
}

impl MapZoom {
    /// Every zoom, smallest first.
    pub const ALL: [Self; 2] = [Self::X1, Self::X2];

    /// Console pixels per pixel of the art.
    pub const fn factor(self) -> i32 {
        match self {
            Self::X1 => 1,
            Self::X2 => 2,
        }
    }

    /// The zoom drawing `factor` console pixels per pixel of the art, if
    /// there is one.
    pub fn from_factor(factor: i32) -> Option<Self> {
        Self::ALL.into_iter().find(|zoom| zoom.factor() == factor)
    }

    /// The other zoom.
    #[must_use]
    pub const fn toggled(self) -> Self {
        match self {
            Self::X1 => Self::X2,
            Self::X2 => Self::X1,
        }
    }

    /// The key of the name the Options screen shows
    /// ([`Ctx::text`](shuyi_ui::Ctx::text)).
    pub const fn key(self) -> &'static str {
        match self {
            Self::X1 => "options.map_zoom.x1",
            Self::X2 => "options.map_zoom.x2",
        }
    }
}

/// What the Options screen sets that is this game's: the battle's speeds
/// and look. The defaults are how the game behaved before it had options.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShuyiSettings {
    /// How fast battle animations play.
    pub anim_speed: AnimSpeed,
    /// Whether a fight's playback is shown; off, every fight is skipped as
    /// the Cancel key skips one.
    pub combat_animations: bool,
    /// How fast the Enemy and Other phases play.
    pub enemy_phase_speed: EnemyPhaseSpeed,
    /// Auto-end: the player phase ends when its last unit has acted
    /// (`docs/design/turn-structure.md`; off by default). The Auto-end key
    /// in battle flips it too.
    pub auto_end_turn: bool,
    /// How the battle cursor is drawn (`docs/design/look-and-feel.md`).
    pub cursor_style: CursorStyle,
    /// How the battle map looks (shown only when pictures exist).
    pub map_look: MapLook,
    /// How big the battle map's pictures are drawn. The Zoom map key in
    /// battle flips it too.
    pub map_zoom: MapZoom,
    /// Whether the title plays its intro cinematic when left alone (ticket
    /// 0844); off, it stays on the logo and menu.
    pub intro: bool,
    /// When a town on the world map opens its shops (Nick, ticket 1007).
    pub towns: TownsOpen,
}

/// When a town on the world map opens its shops (Nick, ticket 1007).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TownsOpen {
    /// When the army arrives there (the default).
    #[default]
    OnArrival,
    /// Only on Confirm on the town where the army is.
    OnConfirm,
}

impl Default for ShuyiSettings {
    fn default() -> Self {
        Self {
            anim_speed: AnimSpeed::default(),
            combat_animations: true,
            enemy_phase_speed: EnemyPhaseSpeed::default(),
            auto_end_turn: false,
            cursor_style: CursorStyle::default(),
            map_look: MapLook::default(),
            map_zoom: MapZoom::default(),
            intro: true,
            towns: TownsOpen::OnArrival,
        }
    }
}

impl ShuyiSettings {
    /// The multiplier on a battle's animation time: the animation speed,
    /// and in the AI's phases (`ai_phase`) the enemy phase speed too.
    pub fn battle_speed(&self, ai_phase: bool) -> f32 {
        let phase = if ai_phase {
            self.enemy_phase_speed.factor()
        } else {
            1.0
        };
        self.anim_speed.factor() * phase
    }

    /// [`battle_speed`](Self::battle_speed) while Confirm is held
    /// (`held`): [`HELD_SPEED`] instead, never slower than the settings
    /// already play.
    pub fn battle_speed_held(&self, ai_phase: bool, held: bool) -> f32 {
        let speed = self.battle_speed(ai_phase);
        if held { speed.max(HELD_SPEED) } else { speed }
    }
}

/// The saved form (ADR-0050): the engine's settings and this game's in
/// one record, as it always was. A field missing from the text takes its
/// default, so an older save loads.
#[derive(Serialize, Deserialize)]
#[serde(default, rename = "Settings")]
struct SettingsFile {
    version: u32,
    text_speed: TextSpeed,
    anim_speed: AnimSpeed,
    combat_animations: bool,
    enemy_phase_speed: EnemyPhaseSpeed,
    auto_end_turn: bool,
    fullscreen: bool,
    /// [`Layout::name`].
    layout: Option<String>,
    cursor_style: CursorStyle,
    map_look: MapLook,
    /// [`MapZoom::factor`].
    map_zoom: i32,
    /// `Some(false)` when the intro is off, otherwise left out, so a save
    /// of the default is as it always was.
    #[serde(skip_serializing_if = "Option::is_none")]
    intro: Option<bool>,
    /// `Some(false)` when towns open only on Confirm, otherwise left out,
    /// as `intro` (ticket 1007).
    #[serde(skip_serializing_if = "Option::is_none")]
    towns_on_arrival: Option<bool>,
    theme: Theme,
    music_volume: u8,
    sound_volume: u8,
}

impl Default for SettingsFile {
    fn default() -> Self {
        file(&Settings::default())
    }
}

/// The saved form of `s`.
fn file(s: &Settings) -> SettingsFile {
    let game = &s.game;
    SettingsFile {
        version: SETTINGS_VERSION,
        text_speed: s.text_speed,
        anim_speed: game.anim_speed,
        combat_animations: game.combat_animations,
        enemy_phase_speed: game.enemy_phase_speed,
        auto_end_turn: game.auto_end_turn,
        fullscreen: s.fullscreen,
        layout: s.layout().map(|l| l.name().to_owned()),
        cursor_style: game.cursor_style,
        map_look: game.map_look,
        map_zoom: game.map_zoom.factor(),
        intro: (!game.intro).then_some(false),
        towns_on_arrival: (game.towns == TownsOpen::OnConfirm).then_some(false),
        theme: s.theme,
        music_volume: s.music_volume,
        sound_volume: s.sound_volume,
    }
}

impl GameSettings for ShuyiSettings {
    type Theme = Theme;

    /// The saved form (RON).
    fn to_ron(settings: &Settings) -> String {
        let config = ron::ser::PrettyConfig::new().struct_names(true);
        // Numbers, booleans, names and options always serialise.
        ron::ser::to_string_pretty(&file(settings), config).unwrap_or_default()
    }

    /// Reads the saved form. Text that can't be read, or of another
    /// version, gives `Err` with why (the caller then uses the defaults);
    /// a missing field takes its default, an unknown layout counts as none
    /// picked and a volume over [`MAX_VOLUME`] is brought down to it.
    // check-text: not player text (warnings for the log)
    fn from_ron(text: &str) -> Result<Settings, String> {
        let file: SettingsFile = ron::from_str(text).map_err(|e| format!("unreadable: {e}"))?;
        if file.version != SETTINGS_VERSION {
            let version = file.version;
            return Err(format!("version {version} isn't {SETTINGS_VERSION}"));
        }
        let mut s = Settings::default();
        s.text_speed = file.text_speed;
        s.fullscreen = file.fullscreen;
        s.theme = file.theme;
        s.music_volume = file.music_volume.min(MAX_VOLUME);
        s.sound_volume = file.sound_volume.min(MAX_VOLUME);
        s.game = ShuyiSettings {
            anim_speed: file.anim_speed,
            combat_animations: file.combat_animations,
            enemy_phase_speed: file.enemy_phase_speed,
            auto_end_turn: file.auto_end_turn,
            cursor_style: file.cursor_style,
            map_look: file.map_look,
            map_zoom: MapZoom::from_factor(file.map_zoom).unwrap_or_default(),
            intro: file.intro.unwrap_or(true),
            towns: if file.towns_on_arrival == Some(false) {
                TownsOpen::OnConfirm
            } else {
                TownsOpen::OnArrival
            },
        };
        Ok(match file.layout.as_deref().and_then(Layout::from_name) {
            Some(layout) => s.with_layout(layout),
            None => s,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::LEFT_HANDED;

    /// Every setting changed from its default.
    fn changed() -> Settings {
        let mut s = Settings::default().with_layout(LEFT_HANDED);
        s.text_speed = TextSpeed::Fast;
        s.fullscreen = true;
        s.theme = Theme::Moonlit;
        s.music_volume = 35;
        s.sound_volume = 64;
        s.game = ShuyiSettings {
            anim_speed: AnimSpeed::Fast,
            combat_animations: false,
            enemy_phase_speed: EnemyPhaseSpeed::Fast,
            auto_end_turn: true,
            cursor_style: CursorStyle::TileGlow,
            map_look: MapLook::Glyphs,
            map_zoom: MapZoom::X1,
            // On, so the saved form below is the one before ticket 0844.
            intro: true,
            // On arrival too, as before ticket 1007.
            towns: TownsOpen::OnArrival,
        };
        s
    }

    /// Ticket 0844: the intro is on by default, saved only when off, and an
    /// older save without it loads with it on.
    #[test]
    fn the_intro_setting_defaults_on_and_round_trips() {
        let s = Settings::default();
        assert!(s.game.intro);
        assert!(!s.to_ron().contains("intro"));
        let mut off = Settings::default();
        off.game.intro = false;
        assert!(off.to_ron().contains("intro: Some(false),"));
        assert_eq!(Settings::from_ron(&off.to_ron()), Ok(off));
        assert!(
            Settings::from_ron("Settings(version: 1)")
                .unwrap()
                .game
                .intro
        );
    }

    /// Ticket 1007: towns open on arrival by default, saved only when off,
    /// and an older save without it loads with it on.
    #[test]
    fn the_towns_setting_defaults_on_and_round_trips() {
        let s = Settings::default();
        assert_eq!(s.game.towns, TownsOpen::OnArrival);
        assert!(!s.to_ron().contains("towns_on_arrival"));
        let mut off = Settings::default();
        off.game.towns = TownsOpen::OnConfirm;
        assert!(off.to_ron().contains("towns_on_arrival: Some(false),"));
        assert_eq!(Settings::from_ron(&off.to_ron()), Ok(off));
        let old = Settings::from_ron("Settings(version: 1)").unwrap();
        assert_eq!(old.game.towns, TownsOpen::OnArrival);
    }

    /// `text` (kept in the repository with LF) in this platform's line
    /// endings, as `ron` writes them (CRLF on Windows).
    fn as_written(text: &str) -> String {
        let lf = text.replace("\r\n", "\n");
        if cfg!(windows) {
            lf.replace('\n', "\r\n")
        } else {
            lf
        }
    }

    /// Ticket 1109: settings saved before the engine had the settings
    /// (written by `main` at `6677932e`, every setting changed, and the
    /// defaults) load as they were, and are written again byte for byte.
    #[test]
    fn settings_saved_before_the_split_load_unchanged() {
        let saved = as_written(include_str!("settings/saved_before_1109.ron"));
        assert_eq!(Settings::from_ron(&saved), Ok(changed()));
        assert_eq!(changed().to_ron(), saved);
        let defaults = as_written(include_str!("settings/defaults_before_1109.ron"));
        assert_eq!(Settings::from_ron(&defaults), Ok(Settings::default()));
        assert_eq!(Settings::default().to_ron(), defaults);
    }

    #[test]
    fn defaults_are_how_the_game_behaved_before() {
        let s = Settings::default();
        assert_eq!(s.text_speed.chars_per_s(), Some(60.0));
        assert!((s.game.battle_speed(false) - 1.0).abs() < f32::EPSILON);
        assert!((s.game.battle_speed(true) - 1.0).abs() < f32::EPSILON);
        let game = &s.game;
        assert!(game.combat_animations && !game.auto_end_turn && !s.fullscreen);
        assert_eq!(game.cursor_style, CursorStyle::Corners);
        assert_eq!(game.map_look, MapLook::Pictures);
        // But the map zoom: a new player starts at 2× (Nick, ticket 0439).
        assert_eq!(game.map_zoom, MapZoom::X2);
        assert_eq!(s.theme, Theme::Earthy);
        assert_eq!((s.music_volume, s.sound_volume), (80, 80));
        assert_eq!(s.layout(), None);
    }

    #[test]
    fn settings_round_trip_through_ron() {
        for s in [Settings::default(), changed()] {
            assert_eq!(Settings::from_ron(&s.to_ron()), Ok(s));
        }
        let text = changed().to_ron();
        assert!(text.starts_with("Settings("), "{text}");
        assert!(text.contains("layout: Some(\"LeftHanded\")"), "{text}");
        // The zoom is saved as its number, either one.
        assert!(text.contains("map_zoom: 1,"), "{text}");
        let mut big = Settings::default();
        big.game.map_zoom = MapZoom::X2;
        assert!(big.to_ron().contains("map_zoom: 2,"));
        assert_eq!(Settings::from_ron(&big.to_ron()), Ok(big));
    }

    #[test]
    fn the_two_zooms_are_one_and_two_pixels_a_pixel() {
        assert_eq!(MapZoom::ALL, [MapZoom::X1, MapZoom::X2]);
        assert_eq!(MapZoom::ALL.map(MapZoom::factor), [1, 2]);
        assert_eq!(
            MapZoom::ALL.map(MapZoom::toggled),
            [MapZoom::X2, MapZoom::X1]
        );
        let keys = ["options.map_zoom.x1", "options.map_zoom.x2"];
        assert_eq!(MapZoom::ALL.map(MapZoom::key), keys);
        for zoom in MapZoom::ALL {
            assert_eq!(MapZoom::from_factor(zoom.factor()), Some(zoom));
        }
        for other in [0, 3, 4, -1, i32::MAX] {
            assert_eq!(MapZoom::from_factor(other), None, "{other}");
        }
    }

    /// A saved zoom the game doesn't have falls back to the default, and
    /// the rest of the settings are kept.
    #[test]
    fn a_zoom_out_of_range_falls_back_to_the_default() {
        for saved in ["0", "3", "-2", "200"] {
            let text = format!("Settings(version: 1, map_zoom: {saved}, sound_volume: 3)");
            let mut expected = Settings::default();
            expected.sound_volume = 3;
            assert_eq!(Settings::from_ron(&text), Ok(expected), "{saved}");
        }
        let kept = Settings::from_ron("Settings(version: 1, map_zoom: 1)").unwrap();
        assert_eq!(kept.game.map_zoom, MapZoom::X1);
        // Without one (a save from before the zoom): the default.
        let old = Settings::from_ron("Settings(version: 1)").unwrap();
        assert_eq!(old.game.map_zoom, MapZoom::X2);
        // Not a number at all: unreadable, so every default (the rule for
        // any unreadable settings), never a crash.
        assert!(Settings::from_ron("Settings(version: 1, map_zoom: \"big\")").is_err());
    }

    #[test]
    fn missing_fields_take_their_defaults() {
        let s = Settings::from_ron("Settings(version: 1, sound_volume: 3)").unwrap();
        let mut expected = Settings::default();
        expected.sound_volume = 3;
        assert_eq!(s, expected);
    }

    #[test]
    fn bad_text_and_other_versions_are_refused() {
        let unreadable = Settings::from_ron("nonsense").unwrap_err();
        assert!(unreadable.starts_with("unreadable: "), "{unreadable}");
        assert_eq!(
            Settings::from_ron("Settings(version: 2)"),
            Err("version 2 isn't 1".to_owned())
        );
    }

    #[test]
    fn odd_values_are_repaired() {
        let text = "Settings(version: 1, music_volume: 200, sound_volume: 101, layout: \
                    Some(\"Two words\"))";
        let s = Settings::from_ron(text).unwrap();
        assert_eq!((s.music_volume, s.sound_volume), (100, 100));
        assert_eq!(s.layout(), None);
        // A layout's name is read whatever the keymap lists: the keymap's
        // owner (`Ctx::saved_layout`) drops one it doesn't have.
        let vim = Settings::from_ron("Settings(version: 1, layout: Some(\"Vim\"))").unwrap();
        assert_eq!(vim.layout().map(|l| l.to_string()), Some("Vim".to_owned()));
    }

    #[test]
    fn speeds_multiply_in_the_ai_phases_only() {
        let mut s = ShuyiSettings {
            anim_speed: AnimSpeed::Fast,
            ..ShuyiSettings::default()
        };
        assert!((s.battle_speed(false) - 2.0).abs() < f32::EPSILON);
        assert!((s.battle_speed(true) - 2.0).abs() < f32::EPSILON);
        s.enemy_phase_speed = EnemyPhaseSpeed::Fast;
        assert!((s.battle_speed(false) - 2.0).abs() < f32::EPSILON);
        assert!((s.battle_speed(true) - 4.0).abs() < f32::EPSILON);
        s.anim_speed = AnimSpeed::Normal;
        assert!((s.battle_speed(true) - 2.0).abs() < f32::EPSILON);
    }

    /// Nick: holding Confirm turns the ×2 into ×4, not ×8.
    #[test]
    fn holding_confirm_is_four_times_whatever_the_speeds() {
        let close = |a: f32, b: f32| (a - b).abs() < f32::EPSILON;
        let mut s = ShuyiSettings::default();
        assert!(close(s.battle_speed_held(false, false), 1.0));
        assert!(close(s.battle_speed_held(false, true), 4.0));
        s.anim_speed = AnimSpeed::Fast;
        assert!(close(s.battle_speed_held(false, false), 2.0));
        assert!(close(s.battle_speed_held(false, true), 4.0));
        s.enemy_phase_speed = EnemyPhaseSpeed::Fast;
        assert!(close(s.battle_speed_held(true, false), 4.0));
        assert!(close(s.battle_speed_held(true, true), 4.0));
        assert!(close(s.battle_speed_held(false, true), 4.0));
    }

    #[test]
    fn speed_names_are_the_options_values() {
        let value = |name: &str| format!("options.value.{name}");
        assert_eq!(
            AnimSpeed::ALL.map(AnimSpeed::key),
            ["normal", "fast"].map(value)
        );
        assert_eq!(
            EnemyPhaseSpeed::ALL.map(EnemyPhaseSpeed::key),
            ["normal", "fast"].map(value)
        );
    }

    #[test]
    fn restoring_keeps_the_layout() {
        let restored = changed().restored();
        assert_eq!(restored, Settings::default().with_layout(LEFT_HANDED));
    }
}
