//! Screens and the screen stack: the UI architecture every screen uses.
//! See `crates/ui/README.md` for how to add a screen.
//!
//! The mechanism is Shuyi Engine's (`shuyi_ui::screen`, `shuyi_ui::Ctx`,
//! ticket 1109): a [`Screen`] reads the frame's [`Action`]s in
//! [`update`](Screen::update) and answers with a [`Transition`]; it paints
//! itself into a [`GlyphBuffer`](crate::GlyphBuffer) in
//! [`draw`](Screen::draw). The [`ScreenStack`] routes input to the top
//! screen only and draws from the top-most opaque screen up, so overlays
//! (menus, dialogs) show the screen below them.
//!
//! This module is the game's part: [`Shuyi`], what this game's screens
//! share besides the engine's services (its content, the campaign being
//! played, the map's skin, the lead), the aliases its screens are written
//! with ([`Ctx`], [`FrameInput`], [`Transition`]), and [`ShuyiCtx`], the
//! lookups that need both. A screen is `impl Screen<Shuyi> for …`.

use crate::prelude::*;
use alloc::collections::BTreeSet;
use core::fmt;

use shuyi_ui::GameData;
use shuyi_ui::frame::color::Palette;
use shuyi_ui::frame::glyph_buffer::PxRect;
use shuyi_ui::frame::key_cap::KeyCapStyle;
use shuyi_ui::input::help_keys::HelpKeys;
use shuyi_ui::text::StyledText;
use trpg_content::dialogue;
use trpg_content::image::KEY_ARROWS_PATH;
use trpg_content::lang::TEST;
use trpg_content::voice::{self, SOURCE_LANG};
use trpg_content::{
    AudioManifest, Content, ContentErrors, KeymapDef, PaletteDef, Playable, Theme, Variant,
};
use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{GameMode, LeadGender, LeadProfile, TutorialLevel};

use crate::color::UiColor;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::Cell;
use crate::input::Action;
use crate::map_view::MapSkin;
use crate::settings::{Settings, ShuyiSettings};
use crate::words::{Language, Words};

pub use shuyi_ui::ctx::{DEFAULT_MUSIC_SEED, IdleMotion, KEYBINDINGS_KEY, KeyPrompt, Keyboard};
pub use shuyi_ui::screen::{Screen, ScreenResult};

/// The clock a looping motion (a pulse, a blink, units stepping in place)
/// reads: `ctx`'s, or 0 when [`Ctx::idle_motion`](shuyi_ui::Ctx::idle_motion)
/// is off, so a test's frame shows every loop at its start (1290).
pub fn loop_clock_s(ctx: &Ctx) -> f64 {
    match ctx.idle_motion {
        IdleMotion::On => ctx.clock_s,
        IdleMotion::Off => 0.0,
    }
}

/// Whole milliseconds of [`loop_clock_s`]: a map scene's clock, on the
/// battle map and behind a conversation alike (its moving tiles, 1384).
pub fn loop_clock_ms(ctx: &Ctx) -> u64 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ms = (loop_clock_s(ctx).max(0.0) * 1000.0) as u64;
    ms
}

/// What every screen shares: Shuyi Engine's services and this game's part,
/// [`Ctx::game`](shuyi_ui::Ctx::game) ([`Shuyi`]).
pub type Ctx = shuyi_ui::Ctx<Shuyi>;

/// One frame's input as this game's screens see it.
pub type FrameInput = shuyi_ui::FrameInput<Action>;

/// What a screen asks the stack to do after an update.
pub type Transition = shuyi_ui::Transition<Shuyi>;

/// The stack of open screens.
pub type ScreenStack = shuyi_ui::ScreenStack<Shuyi>;

/// [`Storage`](crate::storage::Storage) key under which the chosen
/// [`Layout`](crate::input::Layout) was saved (its name, e.g.
/// `LeftHanded`) before the settings held it (ticket 0805). Still read,
/// for a player who picked a layout back then
/// ([`Ctx::saved_layout`](shuyi_ui::Ctx::saved_layout)); never written.
pub const LAYOUT_KEY: &str = "layout";

/// Whether this build offers debug tools: debug builds, and release builds
/// with the `debug-tools` feature (ADR-0023).
pub const DEBUG_TOOLS: bool = cfg!(any(debug_assertions, feature = "debug-tools"));

/// Whether battles open their checklist lessons ([`Shuyi::lessons`], 0456).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lessons {
    /// No lesson opens, so a test can end its turns.
    #[default]
    Off,
    /// Lessons open as the campaign's tutorial level says.
    On,
}

/// Whether the Options screen may switch the campaign's mode
/// ([`Shuyi::mode_switch`]).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ModeSwitch {
    /// Not now: no campaign, or a battle is under way.
    #[default]
    Closed,
    /// At Preparations, before a battle.
    Open,
}

/// Which set of skins paints the game (ADR-0038, ADR-0054): the map skin and
/// each screen's. A screen's skin reads it in its `draw`
/// ([`Shuyi::skin_set`]); the map skin is chosen with it
/// ([`map_view::skin_for`](crate::map_view::skin_for)).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum SkinSet {
    /// The PC's: coloured glyphs, and the map in the player's map look
    /// (glyphs, or the bought art's pictures).
    #[default]
    Glyph,
    /// The Playdate's one-bit look (ADR-0070 §4, `docs/design/playdate.md`).
    Playdate,
}

/// This game's part of the context ([`Ctx::game`](shuyi_ui::Ctx::game)):
/// its content, the campaign being played, how battle maps look and who
/// the lead is. A plain struct: add fields as later tickets need them.
#[derive(Debug)]
pub struct Shuyi {
    /// All validated game content.
    pub content: Content,
    /// The mode of the campaign being played, for the Options screen:
    /// `None` with no campaign (the title). This, with
    /// [`campaign_tutorial`](Self::campaign_tutorial) and
    /// [`campaign_lessons`](Self::campaign_lessons), is the one copy while
    /// a campaign runs: the game flow fills them when it begins or loads
    /// one, takes them into its campaign when a screen over it pops (so a
    /// change to Casual here,
    /// [`switch_to_casual`](Self::switch_to_casual), goes into its saves)
    /// and clears them when it ends.
    pub campaign_mode: Option<GameMode>,
    /// The tutorial level of the campaign being played (0835): `None` with
    /// no campaign. The Options screen's `Tutorial` row changes it (0838).
    /// Battle tips show
    /// only at [`TutorialLevel::Full`] (or with no campaign, as in screen
    /// tests).
    pub campaign_tutorial: Option<TutorialLevel>,
    /// The checklist lessons the campaign being played has finished, by id
    /// (0456): a battle opens none of them, and adds those it finishes.
    /// Empty with no campaign (a battle then keeps what it finishes here,
    /// as in screen tests).
    pub campaign_lessons: BTreeSet<String>,
    /// Whether the campaign's mode may be switched now: only at
    /// Preparations, before a battle (`death-and-difficulty.md`). The game
    /// flow keeps it up to date.
    pub mode_switch: ModeSwitch,
    /// How battle maps look (ADR-0038): the glyph skin, unless a debug tool
    /// swaps in another. Screens build a `MapScene` and this paints it.
    /// Change it with [`set_map_skin`](ShuyiCtx::set_map_skin), which
    /// keeps the player's zoom.
    pub map_skin: MapSkin,
    /// Which set of skins paints the game: [`SkinSet::Glyph`] unless a
    /// front end says otherwise
    /// ([`use_playdate_skins`](ShuyiCtx::use_playdate_skins)).
    pub skin_set: SkinSet,
    /// Whether battles show their one-time tips (0406). Off here, so
    /// screen tests aren't interrupted by them; `app` turns it on.
    pub tips_enabled: bool,
    /// Whether battles open their checklist lessons (0456). Off here, so
    /// screen tests can end their turns; `app` turns it on.
    pub lessons: Lessons,
    /// Who the player made the lead, for dialogue's name and pronoun
    /// tokens and the lead's portrait: a placeholder until a campaign
    /// starts (New Game asks the player), then the campaign's
    /// ([`crate::flow`]).
    pub lead: LeadProfile,
    /// Whether this is the Chapter 1 demo (ticket 0914,
    /// `docs/design/demo.md`): its bundle says so
    /// ([`trpg_content::bundle::is_demo`]). Chapter 1 then ends on
    /// "Thanks for playing" and "Export save?", and the title has no Load
    /// Game.
    pub demo: bool,
    /// Whether a key's name in a hint is drawn as a key cap (ticket 0254,
    /// [`key_caps`](GameData::key_caps)): on in the game. The test
    /// harness turns them off, so a test of what a hint says reads its
    /// words; a test of the look turns them on
    /// ([`Harness::with_key_caps`](crate::harness::Harness::with_key_caps)).
    pub key_caps: bool,
    /// The places' sound bed playing (ticket 1332), one for every screen.
    pub ambience: crate::ambience::AmbiencePlayer,
}

impl Shuyi {
    /// This game's part of a new context over `content`: no campaign, the
    /// default map skin at a new player's zoom, a placeholder lead.
    pub fn new(content: Content) -> Self {
        let map_skin = crate::map_view::default_skin(&content);
        let zoom = ShuyiSettings::default().map_zoom;
        let map_skin = map_skin.at_zoom(zoom).unwrap_or(map_skin);
        Self {
            content,
            campaign_mode: None,
            campaign_tutorial: None,
            campaign_lessons: BTreeSet::new(),
            mode_switch: ModeSwitch::Closed,
            map_skin,
            skin_set: SkinSet::Glyph,
            tips_enabled: false,
            lessons: Lessons::Off,
            lead: LeadProfile::new(DEFAULT_NAME, LeadGender::Male),
            demo: trpg_content::bundle::is_demo(),
            key_caps: true,
            ambience: crate::ambience::AmbiencePlayer::default(),
        }
    }

    /// The Options screen's switch to Casual: one way, only from a Classic
    /// campaign and only while the switch is open
    /// ([`mode_switch`](Self::mode_switch);
    /// `docs/design/death-and-difficulty.md`). Returns whether the mode
    /// changed.
    pub fn switch_to_casual(&mut self) -> bool {
        let classic =
            self.campaign_mode == Some(GameMode::Classic) && self.mode_switch == ModeSwitch::Open;
        if classic {
            self.campaign_mode = Some(GameMode::Casual);
        }
        classic
    }
}

impl GameData for Shuyi {
    type Action = Action;
    type Colors = UiColor;
    type Voice = Variant;
    type Settings = ShuyiSettings;

    fn lang(&self) -> &shuyi_assets::lang::Lang {
        &self.content.lang
    }

    fn keymap_def(&self) -> &KeymapDef {
        &self.content.keymap
    }

    fn audio_config(&self) -> shuyi_ui::audio::AudioConfig {
        crate::audio::AUDIO
    }

    fn audio_manifest(&self) -> &AudioManifest {
        &self.content.audio
    }

    fn palette_def(&self) -> &PaletteDef {
        &self.content.palette
    }

    fn console(&self) -> ConsoleSpec {
        ConsoleSpec::PC
    }

    /// The colour theme's palette (ticket 0806); content validation makes
    /// sure every theme has one.
    fn theme_def(&self, theme: Theme) -> Option<&PaletteDef> {
        self.content.themes.get(&theme)
    }

    /// The test pack (ADR-0057) counts only with debug tools on.
    fn debug_lang(&self) -> Option<&str> {
        Some(TEST)
    }

    /// `{Cursor}` the cursor keys, `{Select}` the key that picks on the
    /// map, `{Action}` that action's key ([`crate::tips::fill_text`]).
    fn key_text(&self, name: &str, keys: HelpKeys<'_, Action>) -> Option<String> {
        crate::tips::key_text(name, keys)
    }

    /// [`key_text`](GameData::key_text) in spans as the engine gives them
    /// (the arrow keys one span, drawn as their picture), but plain on a
    /// controller and for a key not mapped
    /// ([`for_player`](crate::widgets::help::for_player)).
    fn key_text_styled(&self, name: &str, keys: HelpKeys<'_, Action>) -> Option<StyledText> {
        let text = self.key_text(name, keys)?;
        let mut styled = shuyi_ui::input::help_keys::cursor_keys_styled(keys);
        if styled.plain() != text {
            styled = StyledText::new();
            styled.push_key(&text);
        }
        Some(crate::widgets::help::for_player(keys, &styled))
    }

    /// Key caps as Nick decided them in 0250 (`docs/design/look-and-feel.md`,
    /// *Key names in hints*): bold, a single letter in capitals, 2 px round
    /// the letters, in the theme's `key`, `key_bg` and `key_edge`; the
    /// arrow keys as the picture of the arrow cluster
    /// ([`KEY_ARROWS_PATH`]). None on the Playdate's one-bit screen (0256).
    fn key_caps(&self) -> Option<KeyCapStyle> {
        if !self.key_caps {
            return None;
        }
        let caps = KeyCapStyle::new()
            .with_bold(true)
            .with_capitals(true)
            .with_margin_px(2)
            .with_one_bit(false);
        let images = &self.content.images;
        let arrows = images.id(KEY_ARROWS_PATH).and_then(|id| {
            let info = images.info(id)?;
            let (w, h) = (
                i32::try_from(info.width).ok()?,
                i32::try_from(info.height).ok()?,
            );
            Some((id, PxRect::new(0, 0, w, h)))
        });
        Some(match arrows {
            Some((id, src)) => caps.with_arrows(id, src),
            None => caps,
        })
    }

    /// The clip for the lead's gender, where the words differ.
    fn voice_variant(&self) -> Variant {
        Variant::of(self.lead.gender)
    }

    /// Validates the voice manifest `app` read (`voice/<lang>/voice.ron`,
    /// ADR-0046) against the dialogue and the guide's lines (0476), and
    /// keeps the clips that still say what their line says with today's
    /// names.
    fn voices(&self, source: &str) -> Result<Playable, String> {
        let file = format!(
            "{}/{SOURCE_LANG}/{}",
            voice::VOICE_DIR,
            voice::MANIFEST_FILE
        );
        let content = &self.content;
        let guide = |table| voice::with_guide(table, &content.lessons, &content.tips);
        match voice::from_source(&file, source, &guide(content.dialogue.clone())) {
            Ok(manifest) => {
                // The lines as written: what the voice says for the name.
                let written = guide(dialogue::written(&content.dialogue, &content.stand_ins));
                Ok(voice::playable(&manifest, &written, &content.names))
            }
            Err(errors) => {
                let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
                Err(errors.join("; "))
            }
        }
    }

    /// Paints the battle map with the skin of the map look in the settings
    /// (ticket 0824), in the [`skin_set`](Self::skin_set), with their
    /// threat arrows (ticket 0908) and range look (ticket 0923), at their
    /// zoom (ticket 0439): a new skin when the look, the arrows or the
    /// ranges changed (or the settings were just loaded), the same skin at
    /// the new zoom when only the zoom did.
    fn apply_settings(&mut self, now: &Settings, before: Option<&Settings>) {
        let zoom = now.game.map_zoom;
        let (look, arrows, ranges) = (
            now.game.map_look,
            now.game.threat_arrows,
            now.game.range_style,
        );
        let skin = match before {
            Some(b)
                if b.game.map_look == look
                    && b.game.threat_arrows == arrows
                    && b.game.range_style == ranges =>
            {
                if b.game.map_zoom == zoom {
                    return;
                }
                self.map_skin.clone()
            }
            _ => crate::map_view::skin_for(self.skin_set, look, &self.content)
                .with_marks(arrows, ranges),
        };
        self.map_skin = skin.at_zoom(zoom).unwrap_or(skin);
    }

    /// Text on black.
    fn blank_cell(&self, palette: &Palette) -> Cell {
        let text = palette.get(UiColor::Text);
        Cell::new(' ', text, palette.get(UiColor::Black))
    }

    fn keys_before_profiles(&self) -> Vec<String> {
        crate::profiles::keys_from_before_profiles()
    }

    fn old_layout_key(&self) -> Option<&'static str> {
        Some(LAYOUT_KEY)
    }
}

/// What this game's screens ask of the [`Ctx`] that needs its part and the
/// engine's together. It is in the crate's prelude; elsewhere, import it.
pub trait ShuyiCtx: Sized {
    /// The context for the content embedded in the binary.
    fn embedded() -> Result<Self, LoadError>;

    /// The context over loaded `content`. Fails with the names of any UI
    /// colours the palette lacks. Starts with a
    /// [`MemoryStorage`](crate::storage::MemoryStorage) and no layout
    /// chosen (so the layout picker's keys are active).
    fn from_content(content: Content) -> Result<Self, LoadError>;

    /// The data's text (names, tips, titles, dialogue) in the player's
    /// language: what a screen shows any of it through. Like
    /// [`text`](shuyi_ui::Ctx::text), the test pack counts only with
    /// [`debug_tools`](shuyi_ui::Ctx::debug_tools) on.
    fn words(&self) -> Words<'_>;

    /// [`words`](Self::words), to keep: for a screen that names things
    /// where it has no context.
    fn language(&self) -> Language;

    /// Paints the battle map with `skin`, at the zoom in the settings if
    /// it can zoom (ticket 0439): the zoom is kept from one skin to the
    /// next.
    fn set_map_skin(&mut self, skin: MapSkin);

    /// Makes the keymap file's `playdate` table the default buttons (ticket
    /// 1034), for the Playdate front end, before the player's bindings are
    /// loaded: its pad is the only one there is.
    fn use_playdate_buttons(&mut self);

    /// Paints the game with the Playdate's skins ([`SkinSet::Playdate`]),
    /// for its front end and for pictures of its screen: the map in the
    /// Playdate's map skin, whatever the map look in the settings.
    fn use_playdate_skins(&mut self);
}

impl ShuyiCtx for Ctx {
    fn embedded() -> Result<Self, LoadError> {
        let content = trpg_content::load_embedded();
        let keys = crate::lang_keys::check_ui_text_keys(trpg_content::bundle::install());
        match (content, keys) {
            (Ok(content), Ok(())) => Self::from_content(content),
            (content, keys) => {
                let mut errors = content.err().map_or_else(Vec::new, |e| e.0);
                errors.extend(keys.err().unwrap_or_default());
                Err(LoadError::Content(ContentErrors(errors)))
            }
        }
    }

    fn from_content(content: Content) -> Result<Self, LoadError> {
        Self::new(Shuyi::new(content)).map_err(LoadError::Palette)
    }

    fn words(&self) -> Words<'_> {
        Words::new(&self.game.content.lang, self.effective_lang()).with_widths(self.widths())
    }

    fn language(&self) -> Language {
        Language::new(&self.game.content.lang, self.effective_lang()).with_widths(self.widths())
    }

    fn set_map_skin(&mut self, skin: MapSkin) {
        let zoom = self.settings().game.map_zoom;
        self.game.map_skin = skin.at_zoom(zoom).unwrap_or(skin);
    }

    fn use_playdate_buttons(&mut self) {
        let def = core::mem::take(&mut self.game.content.keymap);
        self.game.content.keymap = def.for_playdate();
        self.refresh_keymap();
    }

    fn use_playdate_skins(&mut self) {
        self.game.skin_set = SkinSet::Playdate;
        let look = self.settings().game.map_look;
        let skin = crate::map_view::skin_for(SkinSet::Playdate, look, &self.game.content);
        self.set_map_skin(skin);
        // No sound beds there until the bench says they run well (1332).
        self.game.ambience = crate::ambience::AmbiencePlayer::off();
    }
}

/// Why the game's content could not be loaded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadError {
    /// One or more assets failed to load or validate.
    Content(ContentErrors),
    /// The palette lacks these UI colours.
    Palette(Vec<&'static str>),
}

// check-text: not player text (a start-up error, before any language is loaded)
impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Content(errors) => write!(f, "{errors}"),
            Self::Palette(missing) => write!(f, "palette lacks {}", missing.join(", ")),
        }
    }
}

impl core::error::Error for LoadError {}

#[cfg(test)]
pub(crate) mod tests {
    use crate::input::{Keymap, LEFT_HANDED, RIGHT_HANDED};

    use super::*;
    use crate::audio::groups::VOICE;
    use crate::settings::SETTINGS_KEY;
    use crate::storage::{MemoryStorage, Storage};
    use crate::{AudioRequest, Rgb};
    use trpg_content::{LangCode, LineId};

    /// A voice manifest with a clip for each of the test scene's lines
    /// numbered `picks` (in script order), and those lines' ids.
    pub(crate) fn test_voices(ctx: &Ctx, picks: &[usize]) -> (String, Vec<LineId>) {
        let lines = ctx.game.content.dialogue.scenes[crate::debug::TEST_SCENE].lines();
        let picked: Vec<_> = picks.iter().map(|&i| lines[i]).collect();
        let clip = |line: &trpg_content::dialogue::Line| {
            format!(
                "(line: \"{}\", spoken: \"{}\", voice: \"v\", \
                 made_by: Recorded(actor: \"a\")),\n",
                line.id, line.text
            )
        };
        let clips: String = picked.iter().map(clip).collect();
        let ids = picked.iter().map(|line| line.id.clone()).collect();
        (format!("(clips: [\n{clips}])"), ids)
    }

    fn play(line: &LineId, variant: Variant) -> AudioRequest {
        AudioRequest::play_voice(line.clone(), variant)
    }

    /// Acceptance (0238): a line with a clip is asked for; one without,
    /// or with voices off, is not.
    #[test]
    fn play_voice_asks_only_for_lines_with_a_playable_clip() {
        let mut c = ctx();
        let (manifest, lines) = test_voices(&c, &[0, 1]);
        let all = c.game.content.dialogue.scenes[crate::debug::TEST_SCENE].lines();
        let unvoiced = all[2].id.clone();
        // No manifest yet: nothing has a voice.
        c.play_voice(&lines[0]);
        assert!(!c.has_voice(&lines[0]));
        assert_eq!(c.audio.pending(), []);
        c.set_voice_manifest(&manifest);
        assert_eq!(c.take_warnings(), [] as [std::string::String; 0]);
        assert!(c.has_voice(&lines[0]));
        assert!(!c.has_voice(&unvoiced));
        c.play_voice(&lines[0]);
        c.play_voice(&unvoiced);
        c.play_voice(&LineId::new("test_00000000"));
        c.play_voice(&lines[1]);
        assert_eq!(
            c.audio.take(),
            [
                play(&lines[0], Variant::None),
                play(&lines[1], Variant::None)
            ]
        );
        c.change_settings(|s| s.voices = false).unwrap();
        // Turning them off stops the voice playing (ticket 0826).
        assert_eq!(c.audio.take(), [AudioRequest::StopVoice]);
        assert!(!c.has_voice(&lines[0]));
        c.play_voice(&lines[0]);
        c.preload_voices(&lines);
        assert_eq!(c.audio.pending(), []);
        // Stopping is always passed on: a voice may still be playing.
        c.stop_voice();
        assert_eq!(c.audio.take(), [AudioRequest::StopVoice]);
    }

    /// The first narration line of the test scene, as a manifest clip
    /// that says `spoken`.
    fn narration_clip(c: &Ctx, spoken: &str) -> (String, LineId) {
        let (manifest, lines) = test_voices(c, &[0]);
        let text = c.game.content.dialogue.scenes[crate::debug::TEST_SCENE].lines()[0].text;
        (manifest.replace(text, spoken), lines[0].clone())
    }

    #[test]
    fn a_stale_clip_is_never_asked_for() {
        let mut c = ctx();
        let (manifest, line) = narration_clip(&c, "Words the line no longer says.");
        c.set_voice_manifest(&manifest);
        assert!(c.take_warnings().is_empty(), "stale is not invalid");
        assert!(!c.has_voice(&line));
        c.play_voice(&line);
        c.preload_voices(std::slice::from_ref(&line));
        assert_eq!(c.audio.pending(), []);
    }

    #[test]
    fn an_invalid_voice_manifest_is_a_warning_and_no_voices() {
        let mut c = ctx();
        let (good, lines) = test_voices(&c, &[0]);
        c.set_voice_manifest(&good);
        assert!(c.has_voice(&lines[0]));
        let bad = good.replace(lines[0].as_str(), "test_00000000");
        c.set_voice_manifest(&bad);
        assert_eq!(
            c.take_warnings(),
            ["no voices: voice/en/voice.ron:2: no dialogue line has the id \"test_00000000\""]
        );
        assert!(!c.has_voice(&lines[0]), "the earlier manifest is gone");
        c.set_voice_manifest("not ron");
        let warnings = c.take_warnings();
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].starts_with("no voices: voice/en/voice.ron:1"),
            "{warnings:?}"
        );
    }

    /// A line whose words change with the lead's gender plays the clip
    /// for the lead the player made.
    #[test]
    fn a_gendered_line_plays_the_clip_for_the_leads_gender() {
        let mut c = ctx();
        let script = "@scene g\n> {They} left {their} sword.\n@end\n";
        let table =
            trpg_content::dialogue::from_sources(&[("g.dlg", script)], None, None, None, None);
        c.game.content.dialogue = table.unwrap();
        let line = c.game.content.dialogue.scenes["g"].lines()[0].id.clone();
        let clip = |variant: &str, spoken: &str| {
            format!(
                "(line: \"{line}\", variant: {variant}, spoken: \"{spoken}\", voice: \"v\", \
                 made_by: Recorded(actor: \"a\")),"
            )
        };
        let manifest = format!(
            "(clips: [{}{}])",
            clip("M", "He left his sword."),
            clip("F", "She left her sword.")
        );
        c.set_voice_manifest(&manifest);
        assert_eq!(c.take_warnings(), [] as [&str; 0]);
        c.play_voice(&line);
        c.game.lead.gender = LeadGender::Female;
        c.play_voice(&line);
        assert_eq!(
            c.audio.take(),
            [play(&line, Variant::M), play(&line, Variant::F)]
        );
    }

    /// Ticket 0826: the voices are settings, on until changed, at the
    /// volume Nick picked: the top (0879, the audio manifest's `groups`).
    #[test]
    fn the_voice_settings_start_on_at_the_top() {
        let mut c = ctx();
        assert!(c.settings().voices);
        assert_eq!(c.volume(VOICE), 100);
        for volume in [0, 50, 100] {
            c.change_settings(|s| s.volumes.set(VOICE, volume)).unwrap();
            assert_eq!(c.volume(VOICE), volume);
        }
    }

    /// The context for the embedded content, with the right-handed layout
    /// already chosen.
    /// Debug tools are on whatever the build profile, as in the harness.
    pub(crate) fn ctx() -> Ctx {
        let mut ctx = Ctx::embedded().unwrap().with_layout(RIGHT_HANDED);
        ctx.debug_tools = true;
        // New Game plays the small test chapter, as in the harness.
        ctx.game.content.new_game = trpg_content::NewGameDef::test_chapter();
        // The tests' tips too, as in the harness (1304).
        crate::harness::with_test_tips(&mut ctx.game.content.tips);
        ctx
    }

    #[test]
    fn ctx_uses_the_content_palette_and_keymap() {
        let c = ctx();
        assert_eq!(
            c.palette,
            Palette::new::<UiColor>(&c.game.content.palette).unwrap()
        );
        assert_eq!(
            c.keymap,
            Keymap::for_layout(&c.game.content.keymap, RIGHT_HANDED)
        );
        assert_eq!(c.layout(), Some(RIGHT_HANDED));
        assert_eq!(c.palette.get(UiColor::Black), Rgb::new(0, 0, 0));
    }

    /// Ticket 0243: `! not mapped` and `arrows` come from the language.
    #[test]
    fn help_keys_name_keys_with_the_languages_words() {
        let mut c = ctx();
        assert_eq!(c.help_keys().not_mapped(), "! not mapped");
        assert_eq!(c.help_keys().device(), c.device);
        let cursor_keys_name = crate::widgets::help::cursor_keys_name;
        assert_eq!(cursor_keys_name(c.help_keys()), "arrows");
        c.lang = LangCode::new(TEST).unwrap();
        assert_eq!(c.help_keys().not_mapped(), "! NOT MAPPED");
        assert_eq!(cursor_keys_name(c.help_keys()), "ARROWS");
        c.keymap = Keymap::new([], trpg_content::RepeatDef::default());
        let name = crate::widgets::help::key_name(c.help_keys(), Action::Confirm);
        assert_eq!(name, "! NOT MAPPED");
        // Filled into text, a key's name keeps its words together with
        // no-break spaces (engine 2302), which print as spaces.
        let help = c.text_with("title.help", &[]);
        assert_eq!(help.matches("!\u{a0}NOT\u{a0}MAPPED").count(), 3);
    }

    #[test]
    fn text_with_fills_values_and_key_names() {
        let mut c = ctx();
        assert_eq!(
            c.text_with("title.help", &[]),
            "arrows move · f select · d back"
        );
        c.lang = LangCode::new(TEST).unwrap();
        assert_eq!(
            c.text_with("title.help", &[]),
            "ARROWS MOVE · f SELECT · d BACK"
        );
        // A value named like a placeholder in the text fills it.
        assert_eq!(
            c.text_with("title.help", &[("Cursor", &7)]),
            "7 MOVE · f SELECT · d BACK"
        );
    }

    #[test]
    fn a_saved_layout_the_keymap_doesnt_list_is_ignored() {
        let mut storage = MemoryStorage::new();
        let settings = "Settings(version: 1, layout: Some(\"Vim\"))";
        storage.write(SETTINGS_KEY, settings).unwrap();
        storage.write(LAYOUT_KEY, "Dvorak").unwrap();
        let c = Ctx::embedded().unwrap().with_storage(Box::new(storage));
        assert_eq!(c.saved_layout(), None);
        assert_eq!(c.settings().layout(), None);
        assert_eq!(c.shown_layout(), Some(RIGHT_HANDED));
        // The old key still counts when the settings name an unknown one.
        let mut storage = MemoryStorage::new();
        storage.write(SETTINGS_KEY, settings).unwrap();
        storage.write(LAYOUT_KEY, "LeftHanded").unwrap();
        let c = Ctx::embedded().unwrap().with_storage(Box::new(storage));
        assert_eq!(c.saved_layout(), Some(LEFT_HANDED));
        assert_eq!(c.settings().layout(), Some(LEFT_HANDED));
        assert_eq!(c.with_layout(LEFT_HANDED).shown_layout(), Some(LEFT_HANDED));
    }

    #[test]
    fn every_layout_of_the_keymap_has_a_name_in_every_language() {
        let mut c = ctx();
        for lang in [LangCode::new("en").unwrap(), LangCode::new(TEST).unwrap()] {
            c.lang = lang;
            for (layout, key) in &c.game.content.keymap.layout_names {
                assert_ne!(c.text(key), key.as_str(), "{layout}");
                assert_eq!(
                    crate::screens::layout_picker::label(&c, *layout),
                    c.text(key)
                );
            }
        }
    }

    #[test]
    fn a_layout_saved_before_the_settings_is_still_found() {
        let mut storage = MemoryStorage::new();
        storage.write(LAYOUT_KEY, "LeftHanded").unwrap();
        let mut c = Ctx::embedded().unwrap().with_storage(Box::new(storage));
        assert_eq!(c.saved_layout(), Some(LEFT_HANDED));
        assert_eq!(c.settings().layout(), Some(LEFT_HANDED));
        // The next save of the settings carries it.
        c.change_settings(|s| s.fullscreen = true).unwrap();
        let saved = c.storage.read(SETTINGS_KEY).unwrap().unwrap();
        let saved = Settings::from_ron(&saved).unwrap();
        assert_eq!(saved.layout(), Some(LEFT_HANDED));
        // The settings' layout beats the old key.
        c.choose_layout(RIGHT_HANDED).unwrap();
        assert_eq!(c.saved_layout(), Some(RIGHT_HANDED));
    }

    /// Ticket 0439: the map skin is drawn at the zoom in the settings,
    /// whichever changes: the zoom, or the skin (which keeps the zoom).
    /// The PC paints with the glyph set; the Playdate's front end asks for
    /// its own, and its map skin stays whatever the map look or the zoom
    /// in the settings, and when the settings are loaded.
    #[test]
    fn the_playdate_skins_stay_whatever_the_settings() {
        use crate::settings::{MapLook, MapZoom};
        let mut c = ctx();
        assert_eq!(c.game.skin_set, SkinSet::Glyph);
        assert_eq!(SkinSet::default(), SkinSet::Glyph);
        assert_eq!(c.game.map_skin.name(), "glyph");
        c.use_playdate_skins();
        assert_eq!(c.game.skin_set, SkinSet::Playdate);
        assert_eq!(c.game.map_skin.name(), "playdate");
        for look in [MapLook::Glyphs, MapLook::Pictures, MapLook::Glyphs] {
            c.change_settings(|s| s.game.map_look = look).unwrap();
            assert_eq!(c.game.map_skin.name(), "playdate", "{look:?}");
        }
        c.change_settings(|s| s.game.map_zoom = MapZoom::X1)
            .unwrap();
        assert_eq!(c.game.map_skin.name(), "playdate");
        let loaded = c.with_storage(Box::new(crate::storage::MemoryStorage::new()));
        assert_eq!(loaded.game.map_skin.name(), "playdate");
        // On the PC the look picks the skin, as before.
        let mut pc = ctx();
        pc.change_settings(|s| s.game.map_look = MapLook::Glyphs)
            .unwrap();
        assert_eq!(pc.game.map_skin.name(), "glyph");
    }

    #[test]
    fn the_map_skin_follows_the_zoom_in_the_settings() {
        use crate::map_view::{MapSkin, skin_named};
        use crate::screens::battle::layout::MAP_VIEW;
        use crate::settings::MapZoom;
        let mut c = ctx();
        let sprite = |c: &Ctx| skin_named(&c.game.content, "sprite").unwrap();
        // A skin that zooms starts at the player's zoom (2× for a new one).
        assert_eq!(sprite(&c).zoom(), Some(MapZoom::X1));
        c.set_map_skin(sprite(&c));
        assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X2));
        assert_eq!(c.game.map_skin.view_tiles(MAP_VIEW), (11, 10));
        // Changing the zoom redraws the same skin at it, and saves it.
        c.change_settings(|s| s.game.map_zoom = MapZoom::X1)
            .unwrap();
        assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X1));
        assert_eq!(c.game.map_skin.tileset_id(), Some("test"));
        assert_eq!(c.game.map_skin.view_tiles(MAP_VIEW), (23, 20));
        let saved = c.storage.read(SETTINGS_KEY).unwrap().unwrap();
        assert_eq!(
            Settings::from_ron(&saved).unwrap().game.map_zoom,
            MapZoom::X1
        );
        // Another setting leaves the skin as it is.
        c.change_settings(|s| s.fullscreen = true).unwrap();
        assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X1));
        // A skin that can't zoom is taken as it is, and the zoom is kept
        // for the next that can.
        c.set_map_skin(MapSkin::glyph());
        assert_eq!(
            (c.game.map_skin.name(), c.game.map_skin.zoom()),
            ("glyph", None)
        );
        c.change_settings(|s| s.game.map_zoom = MapZoom::X2)
            .unwrap();
        assert_eq!(c.game.map_skin.name(), "glyph");
        c.set_map_skin(skin_named(&c.game.content, "test_auto").unwrap());
        assert_eq!(c.game.map_skin.zoom(), Some(MapZoom::X2));
        assert_eq!(c.game.map_skin.tileset_id(), Some("test_auto"));
        // The next launch reads the zoom back with the settings, and a
        // game that starts with a picture skin starts at it.
        c.change_settings(|s| s.game.map_zoom = MapZoom::X1)
            .unwrap();
        let mut content = Ctx::embedded().unwrap().game.content;
        let mut tileset = content.tilesets["test"].clone();
        tileset.id = crate::map_view::GAME_TILESET.to_owned();
        content.tilesets.insert(tileset.id.clone(), tileset);
        let fresh = Ctx::from_content(content.clone()).unwrap();
        assert_eq!(fresh.game.map_skin.zoom(), Some(MapZoom::X2));
        let again = Ctx::from_content(content)
            .unwrap()
            .with_storage(c.storage.into_computer());
        assert_eq!(again.settings().game.map_zoom, MapZoom::X1);
        assert_eq!(again.game.map_skin.zoom(), Some(MapZoom::X1));
    }

    #[test]
    fn only_a_classic_campaign_switches_to_casual() {
        let mut c = ctx();
        assert!(!c.game.switch_to_casual());
        assert_eq!(c.game.campaign_mode, None);
        c.game.campaign_mode = Some(GameMode::Classic);
        // Not in the middle of a battle: only at Preparations.
        assert!(!c.game.switch_to_casual());
        assert_eq!(c.game.campaign_mode, Some(GameMode::Classic));
        c.game.mode_switch = ModeSwitch::Open;
        assert!(c.game.switch_to_casual());
        assert_eq!(c.game.campaign_mode, Some(GameMode::Casual));
        assert!(!c.game.switch_to_casual());
        assert_eq!(c.game.campaign_mode, Some(GameMode::Casual));
    }

    #[test]
    fn ctx_rejects_an_incomplete_palette() {
        let mut content = ctx().game.content;
        content.palette.colors.remove("text");
        let err = Ctx::from_content(content).unwrap_err();
        assert_eq!(err, LoadError::Palette(vec!["text"]));
        assert_eq!(err.to_string(), "palette lacks text");
    }

    #[test]
    fn content_error_display() {
        let errors = ContentErrors(vec![trpg_content::ContentError::new("a.ron", "bad")]);
        let err = LoadError::Content(errors.clone());
        assert_eq!(err.to_string(), errors.to_string());
    }
}
