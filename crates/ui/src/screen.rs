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
use alloc::rc::Rc;
use core::fmt;

use shuyi_ui::GameData;
use shuyi_ui::frame::color::Palette;
use shuyi_ui::input::help_keys::HelpKeys;
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

pub use shuyi_ui::ctx::{
    DEFAULT_MUSIC_SEED, DEFAULT_VOICE_VOLUME, IdleMotion, KEYBINDINGS_KEY, KeyPrompt, Keyboard,
    MAX_VOICE_VOLUME,
};
pub use shuyi_ui::screen::Screen;

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
    /// `None` with no campaign (the title). The game flow keeps it up to
    /// date, and takes a change to Casual here
    /// ([`switch_to_casual`](Self::switch_to_casual)) into its campaign.
    pub campaign_mode: Option<GameMode>,
    /// The tutorial level of the campaign being played (0835): `None` with
    /// no campaign. The game flow keeps it up to date, and takes a change
    /// made here (the Options screen's `Tutorial` row, 0838) into its
    /// campaign. Battle tips show
    /// only at [`TutorialLevel::Full`] (or with no campaign, as in screen
    /// tests).
    pub campaign_tutorial: Option<TutorialLevel>,
    /// The checklist lessons the campaign being played has finished, by id
    /// (0456): a battle opens none of them. The game flow fills it from
    /// the campaign and keeps the campaign up to date with what a battle
    /// adds; empty with no campaign (a battle then keeps what it finishes
    /// here, as in screen tests).
    pub campaign_lessons: BTreeSet<String>,
    /// Whether the campaign's mode may be switched now: only at
    /// Preparations, before a battle (`death-and-difficulty.md`). The game
    /// flow keeps it up to date.
    pub mode_switch: ModeSwitch,
    /// How battle maps look (ADR-0038): the glyph skin, unless a debug tool
    /// swaps in another. Screens build a `MapScene` and this paints it.
    /// Change it with [`set_map_skin`](ShuyiCtx::set_map_skin), which
    /// keeps the player's zoom.
    pub map_skin: Rc<dyn MapSkin>,
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

    /// The clip for the lead's gender, where the words differ.
    fn voice_variant(&self) -> Variant {
        Variant::of(self.lead.gender)
    }

    /// Validates the voice manifest `app` read (`voice/<lang>/voice.ron`,
    /// ADR-0046) against the dialogue, and keeps the clips that still say
    /// what their line says with today's names.
    fn voices(&self, source: &str) -> Result<Playable, String> {
        let file = format!(
            "{}/{SOURCE_LANG}/{}",
            voice::VOICE_DIR,
            voice::MANIFEST_FILE
        );
        let content = &self.content;
        match voice::from_source(&file, source, &content.dialogue) {
            Ok(manifest) => Ok(voice::playable(
                &manifest,
                &content.dialogue,
                &content.names,
            )),
            Err(errors) => {
                let errors: Vec<String> = errors.iter().map(ToString::to_string).collect();
                Err(errors.join("; "))
            }
        }
    }

    /// Paints the battle map with the skin of the map look in the settings
    /// (ticket 0824), in the [`skin_set`](Self::skin_set), at their zoom
    /// (ticket 0439): a new skin when the look changed (or the settings were
    /// just loaded), the same skin at the new zoom when only the zoom did.
    fn apply_settings(&mut self, now: &Settings, before: Option<&Settings>) {
        let zoom = now.game.map_zoom;
        let skin = match before {
            Some(b) if b.game.map_look == now.game.map_look => {
                if b.game.map_zoom == zoom {
                    return;
                }
                Rc::clone(&self.map_skin)
            }
            _ => crate::map_view::skin_for(self.skin_set, now.game.map_look, &self.content),
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
    fn set_map_skin(&mut self, skin: Rc<dyn MapSkin>);

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
        Self::from_content(trpg_content::load_embedded().map_err(LoadError::Content)?)
    }

    fn from_content(content: Content) -> Result<Self, LoadError> {
        Self::new(Shuyi::new(content)).map_err(LoadError::Palette)
    }

    fn words(&self) -> Words<'_> {
        Words::new(&self.game.content.lang, self.effective_lang())
    }

    fn language(&self) -> Language {
        Language::new(&self.game.content.lang, self.effective_lang())
    }

    fn set_map_skin(&mut self, skin: Rc<dyn MapSkin>) {
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
    use crate::input::{
        Button, Keymap, LEFT_HANDED, Layout, LayoutBindings, PadBindings, PlayerKeys, RIGHT_HANDED,
    };
    use std::rc::Rc;

    use super::*;
    use crate::settings::SETTINGS_KEY;
    use crate::storage::{MemoryStorage, Storage, StorageError};
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
        AudioRequest::PlayVoice {
            line: line.clone(),
            variant,
        }
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
        assert!(c.audio.pending().is_empty());
        c.set_voice_manifest(&manifest);
        assert!(c.take_warnings().is_empty());
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
        c.voices_on = false;
        assert!(!c.has_voice(&lines[0]));
        c.play_voice(&lines[0]);
        c.preload_voices(&lines);
        assert!(c.audio.pending().is_empty());
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
        assert!(c.audio.pending().is_empty());
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

    #[test]
    fn preload_lists_the_playable_clips_in_order() {
        let mut c = ctx();
        let (manifest, lines) = test_voices(&c, &[3, 1]);
        c.set_voice_manifest(&manifest);
        let all: Vec<LineId> = c.game.content.dialogue.scenes[crate::debug::TEST_SCENE]
            .lines()
            .iter()
            .map(|l| l.id.clone())
            .collect();
        c.preload_voices(&all);
        // Script order, not manifest order; unvoiced lines left out.
        let expected = vec![
            (lines[1].clone(), Variant::None),
            (lines[0].clone(), Variant::None),
        ];
        assert_eq!(
            c.audio.take(),
            [AudioRequest::PreloadVoices { lines: expected }]
        );
        c.preload_voices(&all[..1]);
        c.preload_voices(&[]);
        assert!(c.audio.pending().is_empty(), "nothing playable: no request");
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

    #[test]
    fn the_voice_volume_is_a_factor_of_ten_steps() {
        let mut c = ctx();
        assert!(c.voices_on);
        assert_eq!(c.voice_volume, DEFAULT_VOICE_VOLUME);
        assert!((c.voice_gain() - 0.8).abs() < f32::EPSILON);
        for (volume, gain) in [(0, 0.0), (5, 0.5), (10, 1.0), (200, 1.0)] {
            c.voice_volume = volume;
            assert!((c.voice_gain() - gain).abs() < f32::EPSILON, "{volume}");
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
        ctx
    }

    /// [`ctx`] with a manifest holding these sound and music cues too.
    /// Each music cue loops and is 10 s long (an existing cue, e.g.
    /// `title`, is replaced).
    pub(crate) fn ctx_with_cues(sounds: &[&str], music: &[&str]) -> Ctx {
        use trpg_content::{CreditRef, MusicCue, SoundCue};
        let mut ctx = ctx();
        let audio = &mut ctx.game.content.audio;
        for &cue in sounds {
            let files = vec![format!("sfx/{cue}.wav")];
            let sound = SoundCue {
                files,
                volume: 100,
                credit: CreditRef::Own,
            };
            audio.sounds.insert(cue.to_owned(), sound);
        }
        for &cue in music {
            let track = MusicCue {
                file: format!("{cue}.ogg"),
                volume: 100,
                looped: true,
                length_ms: 10_000,
                credit: CreditRef::Own,
            };
            audio.music.insert(cue.to_owned(), track);
        }
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

    #[test]
    fn text_comes_from_the_language_in_use() {
        let mut c = ctx();
        assert_eq!(c.lang, LangCode::english());
        assert_eq!(c.text("title.new_game"), "New Game");
        c.lang = LangCode::new(TEST).unwrap();
        assert_eq!(c.text("title.new_game"), "NEW GAME");
        // Stale and missing entries are English.
        assert_eq!(c.text("title.continue"), "Continue");
        assert_eq!(c.text("title.credits"), "Credits");
        // The test pack is only for builds with debug tools.
        c.debug_tools = false;
        assert_eq!(c.text("title.new_game"), "New Game");
        // A language with no pack is English.
        c.lang = LangCode::new("zz").unwrap();
        assert_eq!(c.text("title.new_game"), "New Game");
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
        assert_eq!(
            c.text_with("title.help", &[])
                .matches("! NOT MAPPED")
                .count(),
            3
        );
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
    #[cfg(debug_assertions)]
    #[should_panic(expected = "text key not in the English text: \"title.nope\"")]
    fn an_unknown_text_key_panics_in_debug_builds() {
        let c = ctx();
        let _ = c.text("title.nope");
    }

    #[test]
    fn a_new_ctx_has_no_layout_and_the_picker_keys() {
        let c = Ctx::embedded().unwrap();
        assert_eq!(c.layout(), None);
        assert_eq!(c.keymap, Keymap::layout_picker(&c.game.content.keymap));
        assert_eq!(c.saved_layout(), None);
    }

    #[test]
    fn use_layout_switches_without_saving() {
        let mut c = ctx();
        c.use_layout(LEFT_HANDED);
        assert_eq!(c.layout(), Some(LEFT_HANDED));
        assert_eq!(
            c.keymap,
            Keymap::for_layout(&c.game.content.keymap, LEFT_HANDED)
        );
        assert_eq!(c.storage.read(LAYOUT_KEY), Ok(None));
    }

    #[test]
    fn choose_layout_saves_and_saved_layout_reads_it_back() {
        for layout in [RIGHT_HANDED, LEFT_HANDED] {
            let mut c = Ctx::embedded().unwrap();
            assert_eq!(c.choose_layout(layout), Ok(()));
            assert_eq!(c.layout(), Some(layout));
            // In the settings (0805), no longer under its own key.
            assert_eq!(c.settings().layout(), Some(layout));
            assert_eq!(c.storage.read(LAYOUT_KEY), Ok(None));
            let saved = c.storage.read(SETTINGS_KEY).unwrap().unwrap();
            assert_eq!(saved, c.settings().to_ron());
            assert_eq!(c.saved_layout(), Some(layout));
        }
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

    #[test]
    fn settings_are_saved_on_change_and_loaded_with_the_storage() {
        let mut c = ctx();
        assert_eq!(*c.settings(), Settings::default());
        // Nothing changed: nothing written.
        assert_eq!(c.change_settings(|_| {}), Ok(()));
        assert_eq!(c.storage.read(SETTINGS_KEY), Ok(None));
        c.change_settings(|s| {
            s.music_volume = 3;
            s.game.combat_animations = false;
        })
        .unwrap();
        assert_eq!(c.settings().music_volume, 3);
        let storage = std::mem::take(&mut c.storage).into_computer();
        let mut again = Ctx::embedded().unwrap().with_storage(storage);
        assert_eq!(again.settings(), c.settings());
        assert!(again.take_warnings().is_empty());
    }

    /// Ticket 0839: a previewed change applies at once and is saved only
    /// when kept; restoring puts it back and saves nothing unless the
    /// storage holds something else.
    #[test]
    fn previewed_settings_are_saved_only_when_kept() {
        use crate::settings::MapLook;
        use trpg_content::Theme;
        let mut c = ctx();
        assert!(!c.preview_settings(|_| {}));
        let before = c.settings().clone();
        assert!(c.preview_settings(|s| {
            s.theme = Theme::Moonlit;
            s.game.map_look = MapLook::Glyphs;
            s.fullscreen = true;
        }));
        let moonlit = Palette::new::<UiColor>(&c.game.content.themes[&Theme::Moonlit]).unwrap();
        assert_eq!(c.palette.get(UiColor::Text), moonlit.get(UiColor::Text));
        assert_eq!(c.storage.read(SETTINGS_KEY), Ok(None));
        // Put back: as before, nothing written.
        c.restore_settings(&before).unwrap();
        assert_eq!(*c.settings(), before);
        assert_ne!(c.palette.get(UiColor::Text), moonlit.get(UiColor::Text));
        assert_eq!(c.storage.read(SETTINGS_KEY), Ok(None));
        // Kept: written; keeping again writes nothing new.
        c.preview_settings(|s| s.music_volume = 3);
        c.keep_settings().unwrap();
        let saved = c.storage.read(SETTINGS_KEY).unwrap().unwrap();
        assert_eq!(Settings::from_ron(&saved).unwrap().music_volume, 3);
        c.storage.write(SETTINGS_KEY, "marker").unwrap();
        c.keep_settings().unwrap();
        assert_eq!(c.storage.read(SETTINGS_KEY).unwrap().unwrap(), "marker");
        // A layout picked meanwhile saves only the layout.
        c.preview_settings(|s| s.music_volume = 9);
        c.choose_layout(LEFT_HANDED).unwrap();
        let saved = c.storage.read(SETTINGS_KEY).unwrap().unwrap();
        let saved = Settings::from_ron(&saved).unwrap();
        assert_eq!((saved.music_volume, saved.layout()), (3, Some(LEFT_HANDED)));
    }

    #[test]
    fn change_settings_cant_change_the_layout() {
        let mut c = ctx();
        c.choose_layout(LEFT_HANDED).unwrap();
        c.change_settings(|s| *s = Settings::default()).unwrap();
        assert_eq!(c.settings().layout(), Some(LEFT_HANDED));
        assert_eq!(c.layout(), Some(LEFT_HANDED));
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
        use crate::map_view::{GlyphSkin, skin_named};
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
        c.set_map_skin(Rc::new(GlyphSkin));
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
    fn unreadable_settings_give_the_defaults_and_a_warning() {
        let mut storage = MemoryStorage::new();
        storage.write(SETTINGS_KEY, "nonsense").unwrap();
        let mut c = Ctx::embedded().unwrap().with_storage(Box::new(storage));
        assert_eq!(*c.settings(), Settings::default());
        let warnings = c.take_warnings();
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].starts_with("settings: unreadable: ")
                && warnings[0].ends_with(", using the defaults"),
            "{warnings:?}"
        );
    }

    #[test]
    fn a_failed_save_still_changes_the_settings() {
        let mut c = Ctx::embedded().unwrap().with_storage(Box::new(Failing));
        assert!(c.change_settings(|s| s.sound_volume = 1).is_err());
        assert_eq!(c.settings().sound_volume, 1);
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
    fn a_bad_saved_layout_counts_as_none() {
        let mut c = Ctx::embedded().unwrap();
        c.storage.write(LAYOUT_KEY, "Vim").unwrap();
        assert_eq!(c.saved_layout(), None);
        c.storage
            .write(
                LAYOUT_KEY,
                " LeftHanded
",
            )
            .unwrap();
        assert_eq!(c.saved_layout(), Some(LEFT_HANDED));
    }

    /// A storage whose every operation fails.
    #[derive(Debug)]
    struct Failing;

    impl Storage for Failing {
        fn read(&self, _: &str) -> Result<Option<String>, StorageError> {
            Err(StorageError::Backend("down".into()))
        }
        fn write(&mut self, _: &str, _: &str) -> Result<(), StorageError> {
            Err(StorageError::Backend("down".into()))
        }
        fn delete(&mut self, _: &str) -> Result<(), StorageError> {
            Err(StorageError::Backend("down".into()))
        }
        fn list(&self) -> Result<Vec<String>, StorageError> {
            Err(StorageError::Backend("down".into()))
        }
    }

    #[test]
    fn a_failed_save_still_switches_layout() {
        let mut c = Ctx::embedded().unwrap().with_storage(Box::new(Failing));
        assert_eq!(c.saved_layout(), None);
        assert!(c.choose_layout(LEFT_HANDED).is_err());
        assert_eq!(c.layout(), Some(LEFT_HANDED));
    }

    #[test]
    fn unreadable_storage_gives_default_keys_and_a_warning() {
        let mut c = Ctx::embedded().unwrap().with_storage(Box::new(Failing));
        assert_eq!(c.player_keys(), &PlayerKeys::default());
        assert_eq!(
            c.take_warnings(),
            [
                "profiles: can't be read: storage error: down",
                "keybindings: can't be read, using the default keys: storage error: down",
                "settings: can't be read: storage error: down, using the defaults"
            ]
        );
        assert_eq!(*c.settings(), Settings::default());
        assert!(c.take_warnings().is_empty());
    }

    /// `Info` on `g` instead of its default key.
    fn info_on_g(c: &Ctx, layout: Layout) -> LayoutBindings {
        let mut b = c.layout_bindings(layout);
        let g = crate::input::Chord::plain(crate::input::Key::G);
        assert!(b.bind(Action::Info, 0, g).is_ok());
        b
    }

    #[test]
    fn set_layout_bindings_saves_and_switches_the_layout_in_use() {
        let mut c = ctx();
        let b = info_on_g(&c, RIGHT_HANDED);
        assert_eq!(c.set_layout_bindings(RIGHT_HANDED, b.clone()), Ok(()));
        assert_eq!(
            c.keymap,
            b.keymap(c.game.content.keymap.repeat)
                .with_default_pad(&c.game.content.keymap)
        );
        assert_eq!(c.layout_bindings(RIGHT_HANDED), b);
        let saved = c.storage.read(KEYBINDINGS_KEY).unwrap().unwrap();
        assert_eq!(saved, c.player_keys().to_ron());
        // The other layout's keys change only the saved config.
        let before = c.keymap.clone();
        let left = info_on_g(&c, LEFT_HANDED);
        c.set_layout_bindings(LEFT_HANDED, left.clone()).unwrap();
        assert_eq!(c.keymap, before);
        c.use_layout(LEFT_HANDED);
        assert_eq!(
            c.keymap,
            left.keymap(c.game.content.keymap.repeat)
                .with_default_pad(&c.game.content.keymap)
        );
        assert_eq!(c.keymap_for(RIGHT_HANDED), before);
    }

    /// Unit info also on the right stick pressed in.
    fn info_on_right_trigger(c: &Ctx) -> PadBindings {
        let mut b = c.pad_bindings();
        assert!(b.bind(Action::Info, 1, Button::RightStickPress).is_ok());
        b
    }

    #[test]
    fn set_pad_bindings_saves_and_applies_in_every_layout_and_before_one_is_chosen() {
        let mut c = ctx();
        assert_eq!(
            c.pad_bindings(),
            PadBindings::defaults(&c.game.content.keymap)
        );
        let b = info_on_right_trigger(&c);
        assert_eq!(c.set_pad_bindings(b.clone()), Ok(()));
        assert_eq!(c.pad_bindings(), b);
        assert_eq!(
            c.keymap.pad_action(Button::RightStickPress),
            Some(Action::Info)
        );
        let saved = c.storage.read(KEYBINDINGS_KEY).unwrap().unwrap();
        assert_eq!(saved, c.player_keys().to_ron());
        // The keys are the layout's own; the other layout has the buttons too.
        assert_eq!(c.keymap, c.keymap_for(RIGHT_HANDED));
        assert_eq!(
            c.keymap.chords_for(Action::Info),
            Keymap::for_layout(&c.game.content.keymap, RIGHT_HANDED).chords_for(Action::Info)
        );
        c.use_layout(LEFT_HANDED);
        assert_eq!(
            c.keymap.pad_action(Button::RightStickPress),
            Some(Action::Info)
        );
        // Before a layout is chosen: the layout picker's keys, these buttons.
        let mut none = Ctx::embedded().unwrap();
        assert_eq!(none.set_pad_bindings(b.clone()), Ok(()));
        assert_eq!(none.layout(), None);
        assert_eq!(
            none.keymap,
            Keymap::layout_picker(&none.game.content.keymap).with_pad(b.pairs())
        );
        // They load with the storage, with or without a layout.
        let storage = std::mem::take(&mut none.storage).into_computer();
        let again = Ctx::embedded().unwrap().with_storage(storage);
        assert_eq!(again.pad_bindings(), b);
        assert_eq!(
            again.keymap.pad_action(Button::RightStickPress),
            Some(Action::Info)
        );
        // A failed save still applies them.
        let mut failing = ctx().with_storage(Box::new(Failing));
        assert!(failing.set_pad_bindings(b.clone()).is_err());
        assert_eq!(failing.pad_bindings(), b);
        assert_eq!(
            failing.keymap.pad_action(Button::RightStickPress),
            Some(Action::Info)
        );
    }

    #[test]
    fn saved_keys_load_with_the_storage() {
        let mut c = ctx();
        let b = info_on_g(&c, RIGHT_HANDED);
        c.set_layout_bindings(RIGHT_HANDED, b.clone()).unwrap();
        let storage = std::mem::take(&mut c.storage).into_computer();
        // A layout already in use picks up the loaded keys.
        let mut again = ctx().with_storage(storage);
        assert_eq!(again.layout_bindings(RIGHT_HANDED), b);
        assert_eq!(
            again.keymap,
            b.keymap(again.game.content.keymap.repeat)
                .with_default_pad(&again.game.content.keymap)
        );
        assert!(again.take_warnings().is_empty());
    }

    #[test]
    fn repaired_keys_leave_a_warning() {
        let mut storage = MemoryStorage::new();
        storage.write(KEYBINDINGS_KEY, "nonsense").unwrap();
        let mut c = Ctx::embedded().unwrap().with_storage(Box::new(storage));
        let warnings = c.take_warnings();
        assert_eq!(warnings.len(), 1);
        assert!(
            warnings[0].starts_with("keybindings: unreadable"),
            "{warnings:?}"
        );
    }

    #[test]
    fn a_failed_save_still_switches_keys() {
        let mut c = Ctx::embedded()
            .unwrap()
            .with_storage(Box::new(Failing))
            .with_layout(LEFT_HANDED);
        let b = info_on_g(&c, LEFT_HANDED);
        assert!(c.set_layout_bindings(LEFT_HANDED, b.clone()).is_err());
        assert_eq!(
            c.keymap,
            b.keymap(c.game.content.keymap.repeat)
                .with_default_pad(&c.game.content.keymap)
        );
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
