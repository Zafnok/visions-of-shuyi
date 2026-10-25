//! Headless test driver for screens (ADR-0007 layer 4): scripted key and
//! controller-button presses in, screen names and snapshots out. No window,
//! no clock.
//!
//! The scripts, the waits, the audio recorded and the music player's
//! stand-in are the engine's (`shuyi_platform::harness`, ticket 1111,
//! moved by engine ticket 3003); this
//! [`Harness`] wraps it with this game's start-up (the embedded content,
//! the test chapter, the title) and its lookups (the game flow, the
//! battle and its map). Every method of the engine's Harness works on it.
//!
//! Available to unit tests and, with the `harness` feature, to integration
//! tests in `crates/ui/tests/`. See `crates/ui/README.md`.
//!
//! ```
//! use trpg_ui::harness::Harness;
//! use trpg_ui::input::RIGHT_HANDED;
//!
//! let mut h = Harness::with_layout(RIGHT_HANDED);
//! h.keys("f");
//! assert_eq!(h.top_screen(), "mode_select");
//! h.keys("d");
//! assert_eq!(h.top_screen(), "title");
//! ```

use core::ops::{Deref, DerefMut};

use crate::flow::FlowScreen;
use crate::game::Game;
use crate::game::system_menu::SystemMenuEntry;
use crate::glyph_buffer::Rect;
use crate::input::{Layout, PadKind, RIGHT_HANDED};
use crate::map_view::{MapScene, RangeName, TerrainLooks, UnitView, skin_named};
use crate::profiles::ProfileId;
use crate::screen::{Ctx, Screen, Shuyi, ShuyiCtx};
use crate::screens::BattleScreen;
use crate::settings::{SETTINGS_KEY, Settings};
use crate::storage::{MemoryStorage, Storage};
use trpg_core::Pos;

pub use shuyi_platform::harness::{FRAME_DT, MAX_FRAMES};

/// The engine's Harness on this game.
pub type Core = shuyi_platform::harness::Harness<Shuyi>;

/// Drives a [`Game`] with simulated key presses and time: the engine's
/// Harness ([`Core`], which it derefs to) with this game's start-up and
/// lookups. The scripts are forwarded, so they chain with this game's
/// methods (`h.use_pad(kind).with_tips()`) and take arguments that read
/// the Harness (`h.wait(2.0 - time(&h))`).
///
/// Its New Game starts the small test chapter
/// ([`NewGameDef::test_chapter`](trpg_content::NewGameDef::test_chapter)),
/// so tests of the game flow don't depend on the story's content;
/// [`Harness::with_shipped_story`] starts the story.
pub struct Harness(Core);

impl Deref for Harness {
    type Target = Core;

    fn deref(&self) -> &Core {
        &self.0
    }
}

impl DerefMut for Harness {
    fn deref_mut(&mut self) -> &mut Core {
        &mut self.0
    }
}

impl Harness {
    /// A first launch: the embedded content and empty storage, so no
    /// layout is chosen and the first key opens the layout picker.
    ///
    /// # Panics
    ///
    /// If the embedded content fails to load (the content tests catch that
    /// first).
    pub fn new() -> Self {
        Self::with_storage(Box::new(MemoryStorage::new()))
    }

    /// [`new`](Self::new), with `setup` changing the context before the
    /// game starts: e.g. a platform without a keyboard (the Playdate).
    pub fn first_launch_and(setup: impl FnOnce(&mut Ctx)) -> Self {
        let mut ctx = embedded_ctx().with_storage(Box::new(MemoryStorage::new()));
        setup(&mut ctx);
        Self::from_game(crate::game::start(ctx))
    }

    /// A later launch where `layout` was picked before: at the title screen
    /// with that layout's keys.
    pub fn with_layout(layout: Layout) -> Self {
        Self::with_layout_and(layout, |_| {})
    }

    /// [`with_layout`](Self::with_layout), with `setup` changing the
    /// context once the settings are read, before the game starts: e.g.
    /// the Playdate's console and palette (`cargo shuyi frame-png
    /// --playdate`).
    pub fn with_layout_and(layout: Layout, setup: impl FnOnce(&mut Ctx)) -> Self {
        let mut storage = MemoryStorage::new();
        let settings = Settings::default().with_layout(layout);
        if let Err(e) = storage.write(&ProfileId::FIRST.key(SETTINGS_KEY), &settings.to_ron()) {
            panic!("saving the layout: {e}");
        }
        let mut ctx = embedded_ctx().with_storage(Box::new(storage));
        setup(&mut ctx);
        Self::from_game(crate::game::start(ctx))
    }

    /// Makes New Game start the story as the game ships it
    /// (`data/new_game.ron`), not the test chapter (see [`Harness`]'s
    /// docs).
    #[must_use]
    pub fn with_shipped_story(mut self) -> Self {
        match trpg_content::load_embedded() {
            Ok(content) => self.ctx_mut().game.content.new_game = content.new_game,
            Err(e) => panic!("embedded content failed to load: {e}"),
        }
        self
    }

    /// Like [`new`](Self::new), as `app` launches the game: the title
    /// waits for a key or button press ([`Ctx::key_prompt`]).
    pub fn at_prompt() -> Self {
        Self::new().waiting()
    }

    /// Like [`with_layout`](Self::with_layout), as `app` launches the game
    /// ([`at_prompt`](Self::at_prompt)).
    pub fn at_prompt_with_layout(layout: Layout) -> Self {
        Self::with_layout(layout).waiting()
    }

    /// [`at_prompt_with_layout`](Self::at_prompt_with_layout), with
    /// `setup` as in [`with_layout_and`](Self::with_layout_and).
    pub fn at_prompt_with_layout_and(layout: Layout, setup: impl FnOnce(&mut Ctx)) -> Self {
        Self::with_layout_and(layout, setup).waiting()
    }

    /// Waits for a key ([`Core::at_prompt`]).
    fn waiting(self) -> Self {
        Self(self.0.at_prompt())
    }

    /// A launch with `storage` (e.g. from [`into_storage`](Self::into_storage)
    /// of an earlier run), starting as the real game does.
    pub fn with_storage(storage: Box<dyn Storage>) -> Self {
        Self::from_game(crate::game::start(embedded_ctx().with_storage(storage)))
    }

    /// The game with the embedded content and the right-handed layout,
    /// showing only `root`, for testing a screen on its own.
    pub fn with_screen(root: Box<dyn Screen<Shuyi>>) -> Self {
        let ctx = embedded_ctx().with_layout(RIGHT_HANDED);
        Self::from_game(crate::game::new(ctx, root))
    }

    /// Wraps `game`. Debug screens are always on, so F2 behaves the same
    /// in debug and release test runs.
    pub fn from_game(game: Game) -> Self {
        let crank = game.ctx().game.content.keymap.crank;
        let mut core = Core::new(game.with_debug_screens(true));
        core.set_crank(crank);
        Self(core)
    }

    /// [`Core::music_load_delay`].
    pub fn music_load_delay(&mut self, seconds: f32) -> &mut Self {
        self.0.music_load_delay(seconds);
        self
    }

    /// [`Core::without_music`].
    pub fn without_music(&mut self) -> &mut Self {
        self.0.without_music();
        self
    }

    /// [`Core::use_pad`].
    pub fn use_pad(&mut self, kind: PadKind) -> &mut Self {
        self.0.use_pad(kind);
        self
    }

    /// [`Core::keys`].
    pub fn keys(&mut self, script: &str) -> &mut Self {
        self.0.keys(script);
        self
    }

    /// [`Core::type_text`].
    pub fn type_text(&mut self, text: &str) -> &mut Self {
        self.0.type_text(text);
        self
    }

    /// [`Core::hold`].
    pub fn hold(&mut self, chord: &str, seconds: f32) -> &mut Self {
        self.0.hold(chord, seconds);
        self
    }

    /// The player chose `entry` in the platform's own menu (the Playdate's
    /// system menu, ticket 1035; [`Core::act`] with its action): the menu
    /// closes and the game goes on.
    pub fn system_menu(&mut self, entry: SystemMenuEntry) -> &mut Self {
        self.0.act(entry.action());
        self
    }

    /// [`Core::pad`].
    pub fn pad(&mut self, script: &str) -> &mut Self {
        self.0.pad(script);
        self
    }

    /// [`Core::hold_pad`].
    pub fn hold_pad(&mut self, button: &str, seconds: f32) -> &mut Self {
        self.0.hold_pad(button, seconds);
        self
    }

    /// [`Core::wait`].
    pub fn wait(&mut self, seconds: f32) -> &mut Self {
        self.0.wait(seconds);
        self
    }

    /// [`Core::clear_audio`].
    pub fn clear_audio(&mut self) -> &mut Self {
        self.0.clear_audio();
        self
    }

    /// The game flow on the stack (New Game or Quick Battle), if any.
    pub fn flow(&self) -> Option<&FlowScreen> {
        self.screen()
    }

    /// The battle on the stack (the game flow's, or one put there itself);
    /// `None` with no battle.
    pub fn battle(&self) -> Option<&BattleScreen> {
        self.screen()
    }

    /// The battle on the stack, to play it with scripted commands
    /// ([`BattleScreen::send`]).
    pub fn battle_mut(&mut self) -> Option<&mut BattleScreen> {
        self.screen_mut()
    }

    /// What the battle on the stack shows on its map (ADR-0038), as
    /// [`battle`](Self::battle) finds it. Assert on this (or on the
    /// battle's state), not on cells and colours, to check what happened.
    pub fn map_scene(&self) -> Option<MapScene> {
        map_scene(self)
    }

    /// [`map_scene`](Self::map_scene) for the tiles the map skin fits in
    /// `area` (cells) rather than in the battle's map area.
    pub fn map_scene_in(&self, area: Rect) -> Option<MapScene> {
        map_scene_in(self, area)
    }

    /// Paints battle maps with the skin called `name` from the next frame
    /// on (`"glyph"`, or `"sprite"` for the test tileset; ADR-0038), as
    /// the debug menu's "Map skin" does.
    ///
    /// # Panics
    ///
    /// If there is no such skin.
    pub fn with_map_skin(&mut self, name: &str) -> &mut Self {
        let ctx = self.ctx_mut();
        let skin = skin_named(&ctx.game.content, name);
        ctx.set_map_skin(skin.unwrap_or_else(|| panic!("no map skin {name:?}")));
        self
    }

    /// [`map_scene`](Self::map_scene) as text ([`MapScene::to_text`]);
    /// empty with no battle.
    pub fn map_text(&self) -> String {
        map_text(self)
    }

    /// The tile under the battle map's cursor, if one is shown
    /// ([`MapScene::cursor_tile`]).
    pub fn cursor_tile(&self) -> Option<Pos> {
        self.map_scene()?.cursor_tile()
    }

    /// The unit the battle map shows on `pos` ([`MapScene::unit_at`]).
    pub fn unit_at(&self, pos: Pos) -> Option<UnitView> {
        self.map_scene()?.unit_at(pos).cloned()
    }

    /// The ranges the battle map shows on `pos` ([`MapScene::tints_at`]).
    pub fn tints_at(&self, pos: Pos) -> Vec<RangeName> {
        let scene = self.map_scene();
        scene.map(|s| s.tints_at(pos)).unwrap_or_default()
    }

    /// The selected unit's path on the battle map, its own tile first;
    /// empty with none.
    pub fn path(&self) -> Vec<Pos> {
        self.map_scene().map(|s| s.path).unwrap_or_default()
    }

    /// Turns the one-time tips on (they are off in tests by default).
    pub fn with_tips(&mut self) -> &mut Self {
        self.ctx_mut().game.tips_enabled = true;
        self
    }

    /// Turns the checklist lessons on (they are off in tests by default,
    /// so a test can end its turns).
    pub fn with_lessons(&mut self) -> &mut Self {
        self.ctx_mut().game.lessons = crate::screen::Lessons::On;
        self
    }

    /// Ends the run, handing back its storage, to start the next launch
    /// with [`with_storage`](Self::with_storage).
    pub fn into_storage(self) -> Box<dyn Storage> {
        self.0.into_storage()
    }

    /// The engine's Harness it wraps, to hand to the engine's tools (the
    /// MCP session's `Headless`, ticket 1181).
    pub fn into_core(self) -> Core {
        self.0
    }
}

/// [`Harness::map_scene`] on the engine's Harness, for the tools that
/// hold one (ticket 1181).
pub fn map_scene(h: &Core) -> Option<MapScene> {
    Some(h.screen::<BattleScreen>()?.scene(h.ctx()))
}

/// [`Harness::map_scene_in`] on the engine's Harness.
pub fn map_scene_in(h: &Core, area: Rect) -> Option<MapScene> {
    let ctx = h.ctx();
    let size = ctx.game.map_skin.view_tiles(area);
    Some(h.screen::<BattleScreen>()?.scene_of_size(ctx, size))
}

/// [`Harness::map_text`] on the engine's Harness.
pub fn map_text(h: &Core) -> String {
    let terrains = TerrainLooks(&h.ctx().game.content.terrain.display);
    map_scene(h).map_or_else(String::new, |scene| scene.to_text(&terrains))
}

impl Default for Harness {
    fn default() -> Self {
        Self::new()
    }
}

fn embedded_ctx() -> Ctx {
    match Ctx::embedded() {
        Ok(mut ctx) => {
            ctx.debug_tools = true;
            // New Game plays the small test chapter, not the story.
            ctx.game.content.new_game = trpg_content::NewGameDef::test_chapter();
            ctx
        }
        Err(e) => panic!("embedded content failed to load: {e}"),
    }
}

/// The scripts, the audio and the music clock are tested on the engine's
/// toy game (Shuyi Engine's `crates/ui/tests/it`); these are this game's start-up,
/// its lookups and its title's music through them.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::console::ConsoleSizes;
    use crate::input::Device;
    use crate::input::LEFT_HANDED;
    use crate::map_view::range;
    use crate::screen::KeyPrompt;
    use std::time::Duration;

    #[test]
    fn use_pad_sets_the_kind_of_pad_pressed_and_snapshot_as_only_looks() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        let keys = h.snapshot();
        assert_eq!(h.game().ctx().device, Device::Keyboard);
        h.pad("RightStickPress");
        // Unbound: still the keyboard.
        assert_eq!(h.game().ctx().device, Device::Keyboard);
        h.pad("DpadDown DpadUp");
        assert_eq!(h.game().ctx().device, Device::Pad(PadKind::Generic));
        let pad = h.snapshot();
        assert_ne!(pad, keys);
        assert_eq!(h.snapshot_as(Device::Keyboard), keys);
        // Looking doesn't change the device or the frame.
        assert_eq!(h.game().ctx().device, Device::Pad(PadKind::Generic));
        assert_eq!(h.snapshot(), pad);
        h.use_pad(PadKind::PlayStation).pad("DpadDown DpadUp");
        assert_eq!(h.game().ctx().device, Device::Pad(PadKind::PlayStation));
        assert_ne!(h.snapshot(), pad);
        h.use_pad(PadKind::Nintendo).hold_pad("DpadDown", 0.0);
        assert_eq!(h.game().ctx().device, Device::Pad(PadKind::Nintendo));
        h.keys("Up");
        assert_eq!(h.game().ctx().device, Device::Keyboard);
        assert_eq!(h.snapshot(), keys);
    }

    #[test]
    fn new_is_a_first_launch_and_with_layout_a_later_one() {
        let h = Harness::default();
        assert_eq!(h.screens(), ["title"]);
        assert_eq!(h.game().ctx().layout(), None);
        let h = Harness::with_layout(LEFT_HANDED);
        assert_eq!(h.screens(), ["title"]);
        assert_eq!(h.game().ctx().layout(), Some(LEFT_HANDED));
    }

    #[test]
    fn storage_carries_over_between_runs() {
        let mut h = Harness::new();
        h.keys("z Enter");
        let h = Harness::with_storage(h.into_storage());
        assert_eq!(h.screens(), ["title"]);
        assert_eq!(h.game().ctx().layout(), Some(RIGHT_HANDED));
    }

    #[test]
    fn reports_game_state() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        assert_eq!(h.top_screen(), "title");
        assert_eq!(
            h.snapshot(),
            h.game().buffer().to_snapshot(&h.game().ctx().palette)
        );
        h.keys("Up f");
        assert!(h.quit_requested());
        let mut h = Harness::with_screen(Box::new(crate::screens::ModeSelectScreen::new(
            &crate::screen::tests::ctx(),
        )));

        h.keys("d");
        assert_eq!(h.top_screen(), "");
        assert_eq!(h.screens(), [] as [&str; 0]);
        assert!(h.quit_requested());
    }

    #[test]
    fn map_scene_is_the_battle_on_the_stack() {
        use crate::screens::BattleScreen;
        use crate::screens::battle::quick_battle;
        // No battle: no scene, no text.
        let mut h = Harness::with_layout(RIGHT_HANDED);
        assert_eq!(h.map_scene(), None);
        assert_eq!(h.map_text(), "");
        // The Quick Battle, inside the game flow: no map on its
        // Preparations, then the battle's (Left wraps to `Fight!`).
        h.keys("Down f");
        assert_eq!(h.screens(), ["title", "game_flow", "preparations"]);
        assert_eq!(h.map_scene(), None);
        h.keys("Left f");
        assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
        let scene = h.map_scene().unwrap();
        let battle = h.battle().unwrap();
        assert_eq!(scene, battle.scene(h.game().ctx()));
        assert_eq!(scene.units.len(), 8);
        let terrains = TerrainLooks(&h.game().ctx().game.content.terrain.display);
        assert_eq!(h.map_text(), scene.to_text(&terrains));
        assert!(h.map_text().starts_with("origin (-10,-11) size 35x30\n"));
        // A battle screen on the stack itself.
        let state = quick_battle(&embedded_ctx().game.content).unwrap();
        let h = Harness::with_screen(Box::new(BattleScreen::new(state)));
        let bare = h.map_scene().unwrap();
        // The same map; only the cursor's pulse is a frame behind.
        assert_eq!((bare.tiles, bare.units), (scene.tiles, scene.units));
        assert_eq!(bare.cursor.map(|c| c.pos), scene.cursor.map(|c| c.pos));
        assert!(h.battle().is_some());
    }

    #[test]
    fn the_map_skin_can_be_switched_by_name() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        assert!(h.battle().is_none());
        h.keys("Down f Left f");
        assert_eq!(h.battle().map(|b| b.state().units().len()), Some(8));
        h.with_map_skin("sprite").wait(FRAME_DT);
        assert_eq!(h.game().ctx().game.map_skin.name(), "sprite");
        // The test tileset's 24 px tiles at the zoom a new player has
        // (2×, 48 px): 11 × 10 of them.
        assert!(h.map_text().contains(" size 11x10\n"), "{}", h.map_text());
        h.with_map_skin("glyph");
        assert_eq!(h.game().ctx().game.map_skin.name(), "glyph");
    }

    #[test]
    #[should_panic(expected = "no map skin \"ascii\"")]
    fn an_unknown_map_skin_panics() {
        Harness::with_layout(RIGHT_HANDED).with_map_skin("ascii");
    }

    /// A scene of the map alone, as many tiles as the skin fits in an
    /// area, centred on the cursor.
    #[test]
    fn map_scene_in_is_sized_for_the_area() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        assert_eq!(h.map_scene_in(Rect::new(0, 0, 10, 4)), None);
        h.keys("Down f Left f f");
        let lord = Pos::new(3, 5);
        let small = h.map_scene_in(Rect::new(0, 0, 10, 4)).unwrap();
        assert_eq!(small.size, (5, 4));
        assert_eq!(small.cursor_tile(), Some(lord));
        assert_eq!(small.unit_at(lord).map(|u| u.label.as_str()), Some("Lo"));
        // The battle's own scene is unchanged.
        let own = h.map_scene().unwrap();
        assert_ne!(own.size, small.size);
        let area = crate::screens::battle::layout::MAP_VIEW;
        assert_eq!(h.map_scene_in(area), Some(own));
        h.with_map_skin("playdate");
        let pd = h.map_scene_in(Rect::new(0, 0, 50, 15)).unwrap();
        assert_eq!(pd.size, (24, 15));
        assert_eq!(pd.cursor_tile(), Some(lord));
    }

    #[test]
    fn scene_helpers_read_the_battle_map() {
        // No battle: nothing.
        let mut h = Harness::with_layout(RIGHT_HANDED);
        assert_eq!(h.cursor_tile(), None);
        assert_eq!(h.unit_at(Pos::new(3, 5)), None);
        assert_eq!(
            h.tints_at(Pos::new(3, 5)),
            [] as [shuyi_ui::grid::RangeName; 0]
        );
        assert_eq!(h.path(), [] as [trpg_core::Pos; 0]);
        // The Quick Battle: the cursor on the lord, nothing selected.
        h.keys("Down f Left f f");
        let (lord, plain, fort) = (Pos::new(3, 5), Pos::new(4, 5), Pos::new(5, 5));
        assert_eq!(h.cursor_tile(), Some(lord));
        let unit = h.unit_at(lord).unwrap();
        assert_eq!((unit.label.as_str(), unit.pos), ("Lo", lord));
        assert_eq!(h.unit_at(plain), None);
        assert_eq!(h.tints_at(plain), [] as [shuyi_ui::grid::RangeName; 0]);
        assert!(h.path().len() <= 1, "{:?}", h.path());
        // The lord selected and steered to the fort: its ranges and path.
        h.keys("f Right Right");
        assert_eq!(h.tints_at(plain), [range::MOVE]);
        assert_eq!(h.path(), [lord, plain, fort]);
        assert_eq!(h.cursor_tile(), None, "no cursor on the path's end");
        // Back on its own tile: the cursor shows again.
        h.keys("Left Left");
        assert_eq!(h.cursor_tile(), Some(lord));
    }

    #[test]
    fn debug_screens_are_always_on() {
        let ctx = embedded_ctx().with_layout(RIGHT_HANDED);
        let game = crate::game::start(ctx).with_debug_screens(false);
        let mut h = Harness::from_game(game);
        h.keys("F2");
        assert_eq!(h.top_screen(), "debug_menu");
    }

    /// The clock's cue and position, if the music sounds.
    fn clock(h: &Harness) -> Option<(&str, f32)> {
        h.music_clock().map(|c| (c.cue.as_str(), c.position))
    }

    /// Whether the title track sounds, `expected` seconds in (to 10 ms:
    /// frame times are `f32`s).
    fn title_at(h: &Harness, expected: f32) -> bool {
        clock(h).is_some_and(|(cue, at)| cue == "title" && (at - expected).abs() < 0.01)
    }

    /// Ticket 0227: the title's music clock.
    #[test]
    fn the_title_music_clock_counts_up_and_wraps_at_the_track_length() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        assert_eq!(h.music_clock(), None);
        // The title asks for its music in its first frame; the track
        // sounds from then, so the next frame is the first to see a clock.
        h.wait(0.0);
        assert_eq!(h.music_clock(), None);
        h.wait(1.0);
        assert!(title_at(&h, 1.0), "{:?}", h.music_clock());
        let length_ms = h.game().ctx().game.content.audio.music["title"].length_ms;
        let length = Duration::from_millis(length_ms.into()).as_secs_f32();
        assert!(length > 60.0, "{length}");
        let clock_length = h.music_clock().map(|c| c.length).unwrap();
        assert!((clock_length - length).abs() < 1e-4, "{clock_length}");
        // Key presses are frames too.
        h.wait(2.5).keys("Down Up");
        let so_far = 3.5 + 4.0 * FRAME_DT;
        assert!(title_at(&h, so_far), "{:?}", h.music_clock());
        // To a quarter of a second before the end of the track ...
        let mut left = length - 0.25 - so_far;
        while left > 0.0 {
            let step = left.min(30.0);
            h.wait(step);
            left -= step;
        }
        assert!(title_at(&h, length - 0.25), "{:?}", h.music_clock());
        // ... and round to its start again.
        h.wait(1.0);
        assert!(title_at(&h, 0.75), "{:?}", h.music_clock());
    }

    #[test]
    fn a_music_load_delay_starts_the_clock_late() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        h.music_load_delay(2.0).wait(0.0);
        h.wait(1.5);
        assert_eq!(h.music_clock(), None);
        h.wait(1.0);
        assert!(title_at(&h, 0.5), "{:?}", h.music_clock());
        // A negative or NaN delay is none.
        for delay in [-1.0, f32::NAN] {
            let mut h = Harness::with_layout(RIGHT_HANDED);
            h.music_load_delay(2.0).music_load_delay(delay).wait(0.0);
            h.wait(0.5);
            assert!(title_at(&h, 0.5), "{delay}: {:?}", h.music_clock());
        }
    }

    #[test]
    fn without_music_the_clock_never_starts() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        h.without_music().wait(0.0);
        h.wait(3.0);
        assert_eq!(h.music_clock(), None);
        // The game asked for its music all the same.
        let start = crate::audio::MusicCommand::Start {
            cue: "title".into(),
        };
        assert!(h.music_commands().contains(&start));
    }

    /// Ticket 1032: the setup changes the context before the game starts,
    /// so its frame is made with the setup's console; at the prompt too.
    #[test]
    fn a_setup_changes_the_context_before_the_game_starts() {
        let playdate = |ctx: &mut Ctx| ctx.console = crate::ConsoleSpec::PLAYDATE;
        let h = Harness::with_layout_and(RIGHT_HANDED, playdate);
        assert_eq!(h.game().buffer().spec(), crate::ConsoleSpec::PLAYDATE);
        assert_eq!(h.game().ctx().key_prompt, KeyPrompt::Off);
        let h = Harness::at_prompt_with_layout_and(RIGHT_HANDED, playdate);
        assert_eq!(h.game().buffer().spec(), crate::ConsoleSpec::PLAYDATE);
        assert_eq!(h.game().ctx().key_prompt, KeyPrompt::Waiting);
        assert_eq!(h.top_screen(), "title");
    }

    /// The waiting title asks for its music at the first key press
    /// (`docs/design/title-screen.md`), so its clock starts there.
    #[test]
    fn at_the_prompt_the_clock_starts_at_the_first_key_press() {
        let mut h = Harness::at_prompt_with_layout(RIGHT_HANDED);
        h.wait(1.0);
        assert_eq!(h.music_clock(), None);
        // The press starts the music; the release is one frame later.
        h.keys("f");
        assert!(title_at(&h, FRAME_DT), "{:?}", h.music_clock());
        h.wait(0.5);
        assert!(title_at(&h, 0.5 + FRAME_DT), "{:?}", h.music_clock());
    }

    /// During a fade the clock is still the old track's; the new track's
    /// starts when the fade ends (0.5 s, `audio.ron`).
    #[test]
    fn the_clock_follows_a_switch_of_track_once_the_fade_ends() {
        let mut h = Harness::with_layout(RIGHT_HANDED);
        h.wait(0.0).wait(1.0);
        // Quick Battle plays a track from the skirmish pool, from its
        // Preparations screen on.
        h.keys("Down f");
        assert_eq!(h.top_screen(), "preparations");
        assert!(title_at(&h, 1.0 + 4.0 * FRAME_DT), "{:?}", h.music_clock());
        h.wait(1.0);
        let (cue, position) = clock(&h).unwrap();
        let pool = &h.game().ctx().game.content.audio.pools["skirmish"];
        assert!(pool.iter().any(|c| c == cue), "{cue}");
        // The fade began with the `f` press, two frames before the wait.
        let expected = 1.0 + 2.0 * FRAME_DT - 0.5;
        assert!((position - expected).abs() < 0.05, "{position}");
    }
}
