//! The battle screen (ADR-0018, `docs/design/look-and-feel.md`): the map
//! viewport on the left, the side panel on the right and the help bar at the
//! bottom. The player browses with the cursor (ticket 0402), selects a unit,
//! steers its path, moves it and picks an action (0403, [`mode`]), and
//! attacks: picks a weapon and a target, reads the [`forecast`] and watches
//! the combat's [`playback`] (0404), or casts a spell on an enemy, an ally
//! or a tile ([`magic`], 0410); a tile whose terrain changes flashes.
//! Around that (0405): the unit
//! [`info`] screen, the danger zone, the [`map_menu`], ending the turn (and
//! auto-end), and the phase and outcome [`banner`]s. Every command sent is
//! kept in a [`BattleHistory`], and Rewind opens the [`rewind`] screen
//! over the battle (0307), which pops with the point to go back to.
//! One-time [`tips`] pop up over it the first time something new happens
//! (0406), and checklist [`lessons`] (0456) wait, the turn with
//! them, until the player has done what they ask. The battle's [`notes`] (0411) show in a box at its start
//! and on the map menu's `Objective` page. Scenes the battle's triggers fire (0705) play as a
//! [`DialogueScreen`] overlay over the map, in order with the banners, or
//! inside a combat's playback (a boss's line before the combat, a death
//! quote before the fall). In the Enemy and Other phases the AI acts
//! ([`ai_phase`], 0502): each action is shown one at a time, and the player
//! can only speed it up or skip fights.
//!
//! The map itself isn't drawn here (ADR-0038): the screen says what is on it
//! ([`BattleScreen::scene`]) and the map skin paints that. Panels, menus,
//! boxes and the help bar are this screen's own cells, painted from its
//! [`view`] by the skin of the skin set ([`SKINS`]): [`glyph`] on the PC,
//! [`playdate`] on the Playdate.

pub mod ai_phase;
pub mod art_list;
pub mod attack;
pub mod banner;
pub mod camera;
pub mod cursor;
pub mod doable;
pub mod effects;
pub mod event_sounds;
pub mod flight;
pub mod forecast;
pub mod glyph;
pub mod guide;
pub mod guides;
pub mod info;
pub mod items;
pub mod layout;
pub mod lessons;
pub mod loot;
pub mod magic;
pub mod map_menu;
pub mod mode;
pub mod notes;
pub mod objective;
pub mod panel;
pub mod path;
pub mod playback;
pub mod playdate;
pub mod progress;
pub mod rewind;
mod scene_voices;
pub mod shop;
pub mod skills;
mod sounds;
pub mod split_keys;
pub mod stage;
pub mod threat;
pub mod tips;
pub mod triggers;
pub mod units;
pub mod view;
pub mod walk;

use crate::prelude::*;
use alloc::collections::VecDeque;
use shuyi_assets::lang::text_width;
use trpg_content::{Content, DoneWhen, MapLook, Present, TipTrigger, battle_campaign};

use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{
    BattleHistory, BattleState, CastTarget, Command, Equipped, Event, Faction, GameMode, ItemId,
    LeadGender, LeadProfile, Objective, Outcome, Phase, Pos, StatValue, TileSet, Turn, Unit,
    UnitId, next_command,
};

use self::ai_phase::{AiAction, PACING};
use self::banner::{Banner, BannerKind};

use self::attack::{Targeting, aimed_first, aimed_options};
use self::camera::Camera;
use self::cursor::Cursor;
use self::effects::Effects;
use self::flight::Flight;
use self::guide::{Guide, Marks, Moment, Over, Took};
use self::guides::GuidesScreen;
use self::layout::{HELP_BAR, MAP_VIEW, map_area};
use self::lessons::LessonState;
use self::mode::{Effect, Line, MenuEntry, Mode, Selection, list_entry};
use self::playback::{Playback, SceneCue, TIMINGS};
use self::progress::{PROGRESS_TIMINGS, Progress};
use self::rewind::{MapFrame, RewindEffect, RewindScreen};
use self::shop::{ShopScreen, ShopVisit};
use self::threat::ThreatDisplay;
use self::view::{
    BannerView, BattleMenuView, BattleView, DialogView, ForecastView, InfoView, MapInfoView,
    MenuHeader, MenuPlace, MessageKind, MessageView, NotesView, ObjectiveView, PanelPreviewView,
    PanelView, QuestionView, TogglesView,
};
use self::walk::Gait;
use super::dialogue::DialogueScreen;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::map_view::{CursorView, MapScene, MapSkin, Place, UnitView, range};
use crate::screen::{
    Ctx, FrameInput, IdleMotion, Screen, ScreenResult, Shuyi, SkinSet, Transition, loop_clock_s,
};
use crate::screens::Skins;
use crate::screens::game_over::{GameOverChoice, GameOverScreen};
use crate::settings::{HELD_SPEED, MapZoom};
use crate::tips::TipView;
use crate::widgets::help::{
    action_key, frame_widths, hint_column, join_hints, key_pair, pad_last_hint, text_with_keys,
};
use crate::words::{Language, Words};
use shuyi_ui::grid::MapEdges;
use shuyi_ui::grid::scene::RangesShown;
use shuyi_ui::text::StyledText;

pub use self::effects::{HealPopup, PopupKind, TERRAIN_FLASH_S, TerrainFlash};
pub use self::threat::{danger_tiles, threat_enemies};

/// The battle screen's skins (ADR-0083): the PC's glyph look, the
/// Playdate's map over the whole screen with its corner pop-ups (ticket
/// 1038).
pub const SKINS: Skins<BattleView> = Skins {
    glyph: glyph::paint,
    playdate: playdate::paint,
};

/// Battle id of the debug Quick Battle (`assets-test/battles/quick.ron`).
pub const QUICK_BATTLE: &str = "quick";

/// The debug Quick Battle (`assets-test/battles/quick.ron`, which the title
/// screen plays through the game flow) at its start, with its own
/// characters in Classic: `test_small.map` with the placeholder characters
/// (a caster among them, unit 8) against generic enemies (rout), one
/// brigand close enough to fight on turn 1, a Frost Elemental holding its
/// tile, and a trigger of each kind about a rogue who arrives on turn 2.
/// Its pack holds [`QUICK_PACK_POTIONS`] Potions from its stock, as if
/// packed on the Preparations screen the title's Quick Battle opens first.
/// Fails with a message if the content lacks it.
pub fn quick_battle(content: &Content) -> Result<BattleState, String> {
    let def = content
        .battles
        .get(QUICK_BATTLE)
        // check-text: not player text (a developer's error)
        .ok_or_else(|| format!("no battle \"{QUICK_BATTLE}\""))?;
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = battle_campaign(content, def, GameMode::Classic, lead);
    let mut setup = campaign.battle_setup(def, &content.tables());
    for _ in 0..QUICK_PACK_POTIONS {
        setup
            .pack_from_stock(&ItemId::new("potion"))
            // check-text: not player text (a developer's error)
            .map_err(|e| format!("packing a potion: {e}"))?;
    }
    Ok(BattleState::new(setup).0)
}

/// Potions in the pack of [`quick_battle`].
pub const QUICK_PACK_POTIONS: usize = 3;

/// How long the `Auto-end: ON/OFF` message stays, in seconds. *Tunable.*
pub const TOAST_S: f32 = 1.5;

/// The music of a battle's outcome (0809): asked for once per ending.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum OutcomeMusic {
    /// Not asked for: the battle is on, or its outcome's banner isn't up
    /// yet.
    Unheard,
    /// Asked for, at the outcome's banner.
    Heard,
}

/// How the battle screen closed: it pops with this, for the game flow
/// below it.
#[derive(Debug)]
pub enum BattleEnd {
    /// Won: the battle as it ended, and the rewind charges left unused.
    Won(Box<BattleState>, u8),
    /// `Restart Battle` (the map menu) or `Retry Battle` (Game Over): the
    /// battle again.
    Restart,
    /// `Suspend`: the screen as it was, for the flow to save its battle,
    /// and to put back if it can't ([`BattleScreen::suspend_failed`]).
    Suspend(Box<BattleScreen>),
    /// `Title` on Game Over.
    Title,
}

/// The music pool a won battle picks its sting from (`audio.md`, 0022).
pub const VICTORY_MUSIC_POOL: &str = "victory";

/// The music cue of a lost battle, from its banner on (`audio.md`, 0023).
pub const DEFEAT_MUSIC: &str = "defeat";

/// The terrain a foot unit wades (`terrain.ron`): a river, for the Wading
/// tip (0483).
pub const WATER: &str = "water";

/// Something the battle screen shows once no combat is playing, one at a
/// time in order.
#[derive(Debug, Clone, PartialEq)]
pub enum Queued {
    /// The battle's notes (0411), at its start: closed by Confirm.
    Notes,
    /// A phase or outcome banner.
    Banner(Banner),
    /// A scene a trigger fired: played as a [`DialogueScreen`] overlay,
    /// for those on the map when it fired.
    Scene(SceneCue),
    /// The enemies left on the map running as the battle is won (0845):
    /// closed by Confirm, or once they are gone.
    Flight(Flight),
}

/// Why the battle screen closes before the battle is over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Leaving {
    /// `Restart Battle`: the game flow starts the battle again.
    Restart,
    /// `Suspend`: the game flow saves the battle and goes back to the
    /// title.
    Suspend,
}

/// A screen the battle opens over itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opened {
    /// The Options screen, from the map menu or the Playdate's system menu.
    Options,
    /// The Guides list, from the map menu (0458).
    Guides,
    /// The rewind screen (0307).
    Rewind,
    /// The shop screen for this unit on this shop tile (0409), from its
    /// action menu.
    Shop(UnitId, Pos),
}

/// The battle screen: the player browses the map with the cursor, selects
/// and moves units ([`Mode`]), opens the map menu, ends the turn; phase and
/// outcome banners show as the battle goes on, and the outcome's closes the
/// screen. Rewind while browsing in the player phase opens the
/// [`RewindScreen`] over it. In the Enemy and Other phases, once their banner has
/// closed, the AI's actions play one by one ([`AiAction`]), then the phase
/// ends.
#[derive(Debug, Clone)]
pub struct BattleScreen {
    state: BattleState,
    history: BattleHistory,
    /// The command the rules are applying, until they return: in a crash
    /// report's record, so a panic inside the rules replays to the panic
    /// (ticket 1245).
    applying: Option<Command>,
    camera: Camera,
    /// The viewport's size in tiles, across and down, that the camera is
    /// kept for: the map skin's, taken at the start of every frame.
    view: (i32, i32),
    cursor: Cursor,
    mode: Mode,
    /// The danger zone and the threat arrows (0405, 0448).
    threat: ThreatDisplay,
    /// Where and when the selected unit's ranges were opened, for a map
    /// skin whose ranges spread out from it (ticket 0923): kept while the
    /// same unit stays selected ([`note_ranges`](Self::note_ranges)).
    ranges_shown: Option<RangesShown>,
    /// Auto-end: end the player phase when its last unit has acted. Off by
    /// default (`docs/design/turn-structure.md`, ticket 0420). The
    /// player's setting ([`Settings::auto_end_turn`]), read at the start
    /// of every frame.
    ///
    /// [`Settings::auto_end_turn`]: crate::settings::Settings::auto_end_turn
    auto_end: bool,
    /// Whether a fight's playback is shown, or skipped as the Cancel key
    /// skips one: the player's Combat animations setting, read at the
    /// start of every frame.
    show_fights: bool,
    /// A command was applied: auto-end is checked once the screen is back
    /// to browsing.
    end_armed: bool,
    /// A short message (a note or a warning) and the seconds it has left.
    toast: Option<(MessageView, f32)>,
    /// Banners and scenes waiting to be shown, in the order of their
    /// events, the first on screen (after a combat's playback).
    queue: VecDeque<Queued>,
    /// A map menu shortcut just chosen (ticket 1035): its action, handled
    /// next as if its key had been pressed.
    pressed: Option<Action>,
    /// Where the outcome's music (0809) is.
    outcome_music: OutcomeMusic,
    /// The battle's own music cue, asked for again when a rewind undoes
    /// its ending ([`with_music`](Self::with_music)).
    music: Option<String>,
    /// The guide: her tips, lessons and voice (0406, 0456, 0476).
    guide: Guide,
    /// Trigger rows of this battle's after the registry's
    /// ([`add_trigger_row`](Self::add_trigger_row), 1310).
    extra_rows: Vec<triggers::Row>,
    /// The id of the battle (its file's), which the game flow gives
    /// ([`in_battle`](Self::in_battle)): a lesson may start only in some
    /// battles. `None` for a battle made any other way.
    battle_id: Option<String>,
    /// The heal numbers, terrain flashes, event sounds and the walks'
    /// and AI's speeds.
    effects: Effects,
    /// The EXP bar and level-up pages of the last command (0602), shown
    /// once its combat has played.
    progress: Option<Progress>,
    /// The info screen's page, on a skin that shows it a page at a time
    /// (the Playdate's, ticket 1039): 0 when it opens.
    info_page: u8,
    /// The player chose `Restart Battle` or `Suspend`: the screen closes,
    /// and the game flow does it.
    leaving: Option<Leaving>,
    /// Where the cursor and camera were when the player phase ended: they
    /// go back there when the next one starts.
    player_view: Option<(Pos, Camera)>,
    /// Seconds the battle notes have been up (0411): the units they are
    /// about blink.
    notes_t: f32,
    /// How the map looks, as its file says (ADR-0052). The game flow gives
    /// it ([`with_look`](Self::with_look)); a battle made any other way
    /// has the look of a map that names none.
    look: MapLook,
    /// The battle file's name for its objective's place, named on an
    /// escape or seize map's objective line (0324, 0331).
    place_name: Option<String>,
    /// The language of the last frame: what a command applied between
    /// frames names things in (a combat's fighters, a level up's skills).
    /// English until the first frame.
    language: Language,
}

/// The attack forecast `mode` shows, if any.
pub(super) fn forecast_of(mode: &Mode) -> Option<&Targeting> {
    match mode {
        Mode::Targeting(t) => Some(t),
        Mode::CastTarget(t) => t.forecast(),
        _ => None,
    }
}

/// Whether the step from `before` to `after` with `effect` ends the
/// player's phase or asks whether to: `EndPhase`, or the End turn prompt
/// opening.
fn ends_turn(before: &Mode, after: &Mode, effect: &Effect) -> bool {
    let asks = matches!(after, Mode::EndTurnPrompt { .. })
        && !matches!(before, Mode::EndTurnPrompt { .. });
    asks || matches!(effect, Effect::Apply(Command::EndPhase))
}

impl BattleScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "battle";

    /// A screen showing `state` (a battle just started: its history, and
    /// every rewind charge, start here), with the cursor on the first player lord
    /// (else the first player unit, else the map's centre) and the camera
    /// centred on it, in a view as big as the default map skin shows (the
    /// first frame fits it to the skin in use).
    pub fn new(state: BattleState) -> Self {
        let tiles = &state.map().tiles;
        let (w, h) = (tiles.width(), tiles.height());
        let player = |u: &&Unit| u.faction == Faction::Player;
        let units = state.units();
        let start = units
            .iter()
            .filter(player)
            .find(|u| u.is_lord)
            .or_else(|| units.iter().find(player))
            .map_or_else(|| Pos::new(i32::from(w) / 2, i32::from(h) / 2), |u| u.pos);
        // Refitted on the first frame if the game's skin shows another.
        let view = MapSkin::glyph().view_tiles(MAP_VIEW);
        Self {
            camera: Camera::centred_on(start, w, h, view),
            view,
            cursor: Cursor::new(start),
            mode: Mode::after_command(&state),
            history: BattleHistory::new(state.clone()),
            applying: None,
            pressed: None,
            outcome_music: OutcomeMusic::Unheard,
            music: None,
            state,
            threat: ThreatDisplay::default(),
            ranges_shown: None,
            auto_end: false,
            show_fights: true,
            end_armed: false,
            toast: None,
            queue: VecDeque::new(),
            guide: Guide::default(),
            extra_rows: Vec::new(),
            battle_id: None,
            effects: Effects::default(),
            progress: None,
            info_page: 0,
            leaving: None,
            player_view: None,
            notes_t: 0.0,
            look: MapLook::default(),
            place_name: None,
            language: Language::default(),
        }
    }

    /// This screen with its map looking as `look` says.
    #[must_use]
    pub fn with_look(mut self, look: MapLook) -> Self {
        self.look = look;
        self
    }

    /// This screen with the battle file's name for its objective's
    /// place, `name`.
    #[must_use]
    pub fn with_place_name(mut self, name: Option<String>) -> Self {
        self.place_name = name;
        self
    }

    /// This screen as battle `id`'s (its file's id), for the lessons that
    /// start only in some battles.
    #[must_use]
    pub fn in_battle(mut self, id: &str) -> Self {
        self.battle_id = Some(id.to_owned());
        self
    }

    /// This screen with the battle's music `cue`, as the game flow asked
    /// for it: a rewind that undoes the ending asks for it again (0809).
    #[must_use]
    pub fn with_music(mut self, cue: Option<String>) -> Self {
        self.music = cue;
        self
    }

    /// [`BattleScreen::new`] for a battle just started with `events`: its
    /// notes (0411) show first, if it has any and isn't already decided,
    /// then the banners of the events (turn 1's `PLAYER PHASE`; the outcome
    /// of a battle decided at its start) and the scenes they fire (a
    /// turn-1 trigger), in the order of the events.
    pub fn start(state: BattleState, events: &[Event]) -> Self {
        let mut screen = Self::new(state);
        if !screen.state.battle_notes().is_empty() && screen.state.outcome().is_none() {
            screen.queue.push_back(Queued::Notes);
        }
        if screen.state.outcome().is_none() {
            screen.queue.push_back(Queued::Banner(Banner::OBJECTIVE));
        }
        screen.queue.extend(events.iter().filter_map(|e| match e {
            Event::SceneTriggered { .. } => SceneCue::of(e).map(Queued::Scene),
            _ => Banner::for_event(e).map(Queued::Banner),
        }));
        screen
    }

    /// A suspended battle carrying on (0802): the battle as `history`'s
    /// commands left it, with its rewind points and the charges left.
    /// `history` must have its content tables
    /// ([`BattleHistory::restore_tables`]).
    pub fn resume(history: BattleHistory) -> Self {
        let mut screen = Self::new(history.state_at(history.len()));
        screen.history = history;
        screen
    }

    /// The scene waiting to play next, once no combat is playing, no EXP or
    /// level up is shown and no banner is before it.
    pub fn queued_scene(&self) -> Option<&str> {
        if self.playing() || self.progress.is_some() {
            return None;
        }
        match self.queue.front() {
            Some(Queued::Scene(cue)) => Some(&cue.id),
            _ => None,
        }
    }

    /// The heal numbers on screen.
    pub fn popups(&self) -> &[HealPopup] {
        &self.effects.popups
    }

    /// The tiles flashing after their terrain changed.
    pub fn flashes(&self) -> &[TerrainFlash] {
        &self.effects.flashes
    }

    /// Whether auto-end is on.
    pub fn auto_end(&self) -> bool {
        self.auto_end
    }

    /// The danger zone, while shown.
    pub fn danger(&self) -> Option<&TileSet> {
        self.threat.danger()
    }

    /// The EXP bar or level-up page on screen, if any: once the command's
    /// combat has played.
    pub fn progress(&self) -> Option<&Progress> {
        if self.playing() {
            return None;
        }
        self.progress.as_ref()
    }

    /// Gives `action` to the EXP bar or level-up page on screen, if any
    /// (Confirm or Cancel finishes or closes a page; other keys wait). Returns
    /// whether it took it.
    fn progress_key(&mut self, action: Action) -> bool {
        if self.playing() {
            return false;
        }
        let Some(progress) = self.progress.as_mut() else {
            return false;
        };
        if matches!(action, Action::Confirm | Action::Cancel) {
            progress.confirm();
        }
        if progress.done() {
            self.progress = None;
        }
        true
    }

    /// Plays the EXP bar or level-up page on screen for `dt` seconds.
    fn tick_progress(&mut self, dt: f32, confirm_held: bool) {
        if self.playing() {
            return;
        }
        if let Some(progress) = self.progress.as_mut() {
            progress.tick(dt, confirm_held);
            if progress.done() {
                self.progress = None;
            }
        }
    }

    /// Whether a combat's playback or an AI action is playing: banners,
    /// scenes, tips and EXP wait for it.
    fn playing(&self) -> bool {
        matches!(self.mode, Mode::Combat(_) | Mode::AiAction(_))
    }

    /// The banner on screen, if any: the first waiting, once no combat or
    /// AI action is playing and no EXP or level up is shown.
    pub fn banner(&self) -> Option<&Banner> {
        if self.playing() || self.progress.is_some() {
            return None;
        }
        match self.queue.front() {
            Some(Queued::Banner(b)) => Some(b),
            _ => None,
        }
    }

    /// The leftovers running (0845), while on screen: the first waiting,
    /// once no combat or AI action is playing and no EXP or level up is
    /// shown.
    pub fn flight(&self) -> Option<&Flight> {
        if self.playing() || self.progress.is_some() {
            return None;
        }
        match self.queue.front() {
            Some(Queued::Flight(f)) => Some(f),
            _ => None,
        }
    }

    /// Plays the leftovers' running on screen (0845) for `dt` seconds; it
    /// closes once they are gone.
    fn tick_flight(&mut self, dt: f32) {
        if self.flight().is_none() {
            return;
        }
        if let Some(Queued::Flight(flight)) = self.queue.front_mut() {
            flight.tick(dt);
            if flight.done() {
                self.queue.pop_front();
            }
        }
    }

    /// The leftovers as drawn (0845): running while their moment is on
    /// screen, standing where they were while it waits.
    fn flight_views(&self) -> Vec<UnitView> {
        let waiting = self.queue.iter().find_map(|q| match q {
            Queued::Flight(f) => Some(f),
            _ => None,
        });
        waiting.map_or_default(|f| f.views(self.effects.pace.0))
    }

    /// Asks for the outcome's music as its banner comes on screen (0809,
    /// `docs/design/audio.md`): one of the `victory` pool, played once, or
    /// `defeat`, which the Game Over screen leaves playing.
    fn play_outcome_music(&mut self, ctx: &mut Ctx) {
        let Some(BannerKind::Outcome(outcome)) = self.banner().map(|b| b.kind) else {
            return;
        };
        if core::mem::replace(&mut self.outcome_music, OutcomeMusic::Heard) == OutcomeMusic::Heard {
            return;
        }
        let cue = match outcome {
            Outcome::Victory => ctx.pick_music(VICTORY_MUSIC_POOL),
            Outcome::Defeat => Some(DEFEAT_MUSIC.to_owned()),
        };
        if let Some(cue) = cue {
            ctx.audio.play_music(&cue);
        }
    }

    /// Whether the battle's notes are on screen in their box (at the start
    /// of the battle, until Confirm closes it).
    pub fn notes_open(&self) -> bool {
        self.queue.front() == Some(&Queued::Notes)
    }

    /// Whether the battle's notes are on screen, in their box or on the
    /// `Objective` page: the units they are about blink.
    fn notes_shown(&self) -> bool {
        self.notes_open() || self.mode == Mode::Objective
    }

    /// Gives `action` to the battle notes box or the guide's box on
    /// screen, if any: each takes the keys until it is closed (the notes
    /// by Confirm; the guide's, [`Guide::take_key`]). Returns whether one
    /// took it.
    fn overlay_key(&mut self, ctx: &mut Ctx, action: Action) -> bool {
        if self.notes_open() {
            if action == Action::Confirm {
                self.queue.pop_front();
            }
            return true;
        }
        let (m, battle) = (self.moment(), self.battle_id.as_deref());
        match self.guide.take_key(m, ctx, action, battle) {
            Took::No => false,
            Took::Yes => true,
            Took::Restart => {
                self.leaving = Some(Leaving::Restart);
                true
            }
        }
    }

    /// Counts the `dt` seconds the battle notes have been up, from 0 each
    /// time they come up.
    fn tick_notes(&mut self, dt: f32) {
        self.notes_t = if self.notes_shown() {
            self.notes_t + dt
        } else {
            0.0
        };
    }

    /// The message shown for a moment, if any.
    pub fn toast(&self) -> Option<&str> {
        self.toast.as_ref().map(|(t, _)| t.text.as_str())
    }

    /// Shows or hides the danger zone.
    fn toggle_danger(&mut self) {
        self.threat.toggle_danger(&self.state);
    }

    /// Switches the map between its two zooms and saves the choice with
    /// the other options, if the map skin can zoom and the map is what
    /// the player is looking at ([`Mode::shows_map`]). The
    /// next frame fits the cameras to the tiles the skin now shows
    /// ([`refit`](Self::refit)): the cursor's tile stays in view. Never a
    /// `Command`: the zoom isn't game state.
    /// Notes which way it went, for the zoom lesson (1304).
    fn toggle_zoom(&mut self, ctx: &mut Ctx) {
        if !self.mode.shows_map() || ctx.game.map_skin.zoom().is_none() {
            return;
        }
        // If saving fails it still holds for this session.
        let _ = ctx.change_settings(|s| s.game.map_zoom = s.game.map_zoom.toggled());
        self.guide
            .notice(if ctx.settings().game.map_zoom == MapZoom::X1 {
                DoneWhen::ZoomedOut
            } else {
                DoneWhen::ZoomedIn
            });
    }

    /// Flips auto-end, saves it with the other options and says so.
    fn toggle_auto_end(&mut self, ctx: &mut Ctx) {
        self.auto_end = !self.auto_end;
        let on = self.auto_end;
        // If saving fails it still holds for this session.
        let _ = ctx.change_settings(|s| s.game.auto_end_turn = on);
        let text = ctx.text(if on {
            "battle.toast.auto_end_on"
        } else {
            "battle.toast.auto_end_off"
        });
        self.toast = Some((MessageView::note(text.to_owned()), TOAST_S));
    }

    /// What is on screen over the map now, as the guide's boxes wait for
    /// it.
    fn moment(&self) -> Moment {
        let busy = self.playing()
            || self.progress.is_some()
            || self.queued_scene().is_some()
            || self.notes_open();
        let over = if busy {
            Over::Busy
        } else if self.banner().is_some() {
            Over::Banner
        } else {
            Over::Nothing
        };
        Moment {
            over,
            phase: self.state.phase(),
            decided: self.state.outcome().is_some(),
            browsing_clear: matches!(self.mode, Mode::Idle { .. }) && self.queue.is_empty(),
            paged: self.mode.covers_guide(),
        }
    }

    /// The tip on screen, if any ([`Guide`]'s, 0406).
    pub fn shown_tip(&self) -> Option<TipTrigger> {
        self.guide.shown_tip(self.moment())
    }

    /// The box of the tip on screen ([`Self::shown_tip`]), if any.
    pub fn tip_view(&self, ctx: &Ctx) -> Option<TipView> {
        self.guide.tip_view(self.moment(), ctx)
    }

    /// The id of the lesson on screen, if any (0456).
    pub fn shown_lesson(&self) -> Option<&str> {
        self.guide.shown_lesson(self.moment())
    }

    /// Whether the lesson in play is open: shown once and not done yet.
    /// While it is, the turn waits for it unless an item ends it.
    pub fn lesson_open(&self) -> bool {
        self.guide.lessons.is_open()
    }

    /// Each item of the lesson in play, ticked yet, in order.
    pub fn lesson_ticks(&self) -> &[bool] {
        self.guide.lessons.ticked()
    }

    /// Whether the box at the end of the lessons is on screen (0465).
    pub fn lessons_end_shown(&self, ctx: &Ctx) -> bool {
        let battle = self.battle_id.as_deref();
        self.guide.lessons_end_shown(self.moment(), ctx, battle)
    }

    /// Closes the banner on screen. A victory's leaves the battle; a
    /// defeat's opens Game Over over it.
    fn close_banner(&mut self, ctx: &Ctx) -> Transition {
        let Some(&banner) = self.banner() else {
            return Transition::None;
        };
        self.queue.pop_front();
        match banner.kind {
            BannerKind::Outcome(Outcome::Victory) => {
                let state = Box::new(self.state.clone());
                Transition::pop_with(BattleEnd::Won(state, self.history.charges_left()))
            }
            BannerKind::Outcome(Outcome::Defeat) => self.game_over(ctx, false),
            BannerKind::Phase { .. } | BannerKind::Objective => Transition::None,
        }
    }

    /// Game Over over the lost battle, with `Rewind` while a charge is
    /// left (0822). `again`: back from the rewind screen with no charge
    /// spent, it is as it was left, without its opening animation (0830).
    /// Game Over asks for no music: `defeat`, on since the banner, plays on
    /// (0809).
    fn game_over(&self, ctx: &Ctx, again: bool) -> Transition {
        let mut screen = GameOverScreen::new(ctx, self.history.charges_left() > 0);
        if again {
            screen.skip();
        }
        Transition::Push(Box::new(screen))
    }

    /// Whether the AI is playing its phase (Enemy or Other): the player
    /// can only speed it up or skip its fights.
    pub fn ai_phase(&self) -> bool {
        self.state.phase() != Phase::Player && self.state.outcome().is_none()
    }

    /// In an AI phase, once nothing else is on screen (banner, scene, tip,
    /// EXP, a previous action), shows the AI's next action, or ends the
    /// phase when it has none.
    fn drive_ai(&mut self, ctx: &Ctx) {
        if !self.ai_phase() || self.state.outcome().is_some() {
            return;
        }
        // An AI unit that may move after its attack: the AI decides that,
        // not the player.
        if matches!(self.mode, Mode::MoveAfter { .. }) {
            self.mode = Mode::default();
        }
        if !matches!(self.mode, Mode::Idle { .. })
            || !self.queue.is_empty()
            || self.progress.is_some()
            || self.shown_tip().is_some()
        {
            return;
        }
        match next_command(&self.state, &ctx.game.content.ai) {
            Some(cmd) => self.apply_ai(&cmd),
            None => {
                self.apply(&Command::EndPhase);
            }
        }
    }

    /// During an AI action, the camera pans to its unit, then follows its
    /// walk; the cursor (hidden) goes with it, so the side panel shows it.
    fn follow_ai_action(&mut self) {
        let Mode::AiAction(a) = &self.mode else {
            return;
        };
        if a.walking() {
            let at = a.walker_pos();
            self.follow(at);
            if self.cursor.pos != at {
                self.cursor.jump(at);
            }
        } else {
            self.camera.origin = a.camera();
        }
    }

    /// Applies the AI's `cmd` and shows it ([`AiAction`]): the camera pans
    /// to its unit, the cursor marks it, it walks its move, then the
    /// command's combat plays, if any.
    fn apply_ai(&mut self, cmd: &Command) {
        let before = self.state.units().to_vec();
        let unit = match cmd {
            Command::Act { unit, .. } | Command::MoveAfter { unit, .. } => Some(*unit),
            _ => None,
        };
        // Its steps are heard as the walk is shown.
        self.effects.walked = unit;
        let Some(events) = self.apply(cmd) else {
            return;
        };
        let Some(unit) = unit else {
            return;
        };
        // An AI command moves only its own unit.
        let path = events
            .iter()
            .find_map(|e| match e {
                Event::UnitMoved { path, .. } => Some(path.clone()),
                _ => None,
            })
            .unwrap_or_default();
        let Some(start) = before.iter().find(|u| u.id == unit).map(|u| u.pos) else {
            return;
        };
        let from = self.camera.origin;
        self.follow(start);
        let to = self.camera.origin;
        self.camera.origin = from;
        self.cursor.jump(start);
        let then = core::mem::take(&mut self.mode);
        let pacing = PACING.at_speed(self.effects.ai_speed, self.effects.pace);
        let action = AiAction::new(unit, before, (from, to), path, then, pacing);
        self.mode = Mode::AiAction(Box::new(action));
    }

    /// The scene to play now, taken from the combat's playback where its
    /// clock has stopped, or from the front of the queue, as the overlay
    /// that plays it for those on the map when it fired (ADR-0055): a
    /// character whose unit had fallen by then isn't there. A scene
    /// missing from the content (validation rules it out) is dropped, and
    /// so is one with nothing to say for them.
    fn next_scene(&mut self, ctx: &Ctx) -> Option<DialogueScreen> {
        let cue = if let Mode::Combat(pb) = &mut self.mode {
            pb.take_scene()?
        } else {
            self.queued_scene()?;
            match self.queue.pop_front() {
                Some(Queued::Scene(cue)) => cue,
                _ => return None,
            }
        };
        let scene = ctx.game.content.dialogue.get(&cue.id)?;
        let present = Present::Only(cue.present);
        let screen = DialogueScreen::told_overlay(ctx, scene, ctx.game.lead.clone(), &present);
        if !screen.has_text() {
            return None;
        }
        self.guide.pushing_scene(screen.first_lines());
        Some(screen.with_battle(self.place()))
    }

    /// The battle as it stands, for a scene played over it to show
    /// (`@place battle`): the whole map in its look, with the units as the
    /// map shows them now.
    pub fn place(&self) -> Place {
        let tiles = &self.state.map().tiles;
        let size = (i32::from(tiles.width()), i32::from(tiles.height()));
        let mut scene = terrain_scene(&self.state, Pos::new(0, 0), size);
        self.add_units(&mut scene, self.notes_t);
        Place {
            tiles: tiles.clone(),
            look: self.look.clone(),
            units: scene.units,
        }
    }

    /// With auto-end on, ends the player phase once the screen is back to
    /// browsing after the command that left no player unit ready.
    fn check_auto_end(&mut self, ctx: &Ctx) {
        // A lesson in play holds the turn (0456), and one about ending it
        // (0457) is the player's to do: auto-end waits for it, and for the
        // box at the end of the lessons (0465).
        if !self.end_armed
            || LessonState::end_due(ctx, self.battle_id.as_deref())
            || !matches!(self.mode, Mode::Idle { .. })
            || !self.queue.is_empty()
            || self.progress.is_some()
            || self.guide.lesson_in_play()
        {
            return;
        }
        self.end_armed = false;
        let player = self.state.phase() == Phase::Player;
        if self.auto_end
            && player
            && self.state.outcome().is_none()
            && map_menu::ready_players(&self.state) == 0
        {
            self.apply(&Command::EndPhase);
        }
    }

    /// The pop for `Restart Battle` or `Suspend` (the game flow restarts
    /// the battle, or saves it and goes back to the title).
    fn leave(&self, leaving: Leaving) -> Transition {
        match leaving {
            Leaving::Restart => Transition::pop_with(BattleEnd::Restart),
            Leaving::Suspend => Transition::pop_with(BattleEnd::Suspend(Box::new(self.clone()))),
        }
    }

    /// The suspend save couldn't be written: the battle goes on, showing
    /// `message`.
    pub fn suspend_failed(&mut self, message: String) {
        self.leaving = None;
        self.toast = Some((MessageView::warning(message), TOAST_S));
    }

    /// Applies `cmd` as if the player (or the AI) had sent it: scripted
    /// tests play a battle with it.
    #[cfg(any(test, feature = "harness"))]
    pub fn send(&mut self, cmd: &Command) {
        self.apply(cmd);
    }

    /// The commands sent so far and the rewind charges left.
    pub fn history(&self) -> &BattleHistory {
        &self.history
    }

    /// The command the rules were applying when they didn't return (a
    /// panic): `None` between commands.
    pub fn applying(&self) -> Option<&Command> {
        self.applying.as_ref()
    }

    /// What the player is doing.
    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    /// The battle shown.
    pub fn state(&self) -> &BattleState {
        &self.state
    }

    /// The camera.
    pub fn camera(&self) -> Camera {
        self.camera
    }

    /// The cursor.
    pub fn cursor(&self) -> Cursor {
        self.cursor
    }

    // engine-friction: engine#3206, moves (game 1217 deletes it)
    /// Scrolls the camera to keep `target` [`Camera::MARGIN`] tiles from the
    /// viewport's edges.
    pub fn follow(&mut self, target: Pos) {
        let tiles = &self.state.map().tiles;
        let (w, h) = (tiles.width(), tiles.height());
        self.camera.follow(target, w, h, self.view, Camera::MARGIN);
    }

    // engine-friction: engine#3206, moves (game 1217 deletes it)
    /// The camera for a viewport of `view` tiles: the screen's own, or, if
    /// that was kept for another size (the map skin changed), one centred on
    /// the cursor.
    fn camera_in(&self, view: (i32, i32)) -> Camera {
        fit_camera(self.camera, self.view, self.cursor.pos, &self.state, view)
    }

    /// Starts a frame of `dt` seconds: keeps the cameras for as many tiles
    /// as the map skin shows ([`refit`](Self::refit)), takes the player's
    /// settings as they are now (the Options screen may have changed
    /// them) and the walking speed for the walks that start this frame,
    /// and advances the cursor's pulse.
    fn begin_frame(&mut self, ctx: &Ctx, dt: f32) {
        if !self
            .language
            .is(&ctx.game.content.lang, ctx.effective_lang())
        {
            self.language = ctx.language();
        }
        self.auto_end = ctx.settings().game.auto_end_turn;
        self.show_fights = ctx.settings().game.combat_animations;
        let view = ctx.game.map_skin.view_tiles(map_area(ctx));
        if view != self.view {
            self.refit(view);
        }
        self.effects.take_speeds(ctx);
        self.cursor.tick(dt);
    }

    /// The map skin changed (a debug tool) and the viewport now shows
    /// `view` tiles: the camera centres on the cursor, the camera kept for
    /// the player's next phase centres on the cursor kept with it, and an
    /// AI action's pan is worked out again, taking as long as before.
    fn refit(&mut self, view: (i32, i32)) {
        let tiles = &self.state.map().tiles;
        let (w, h) = (tiles.width(), tiles.height());
        self.camera = self.camera_in(view);
        self.view = view;
        if let Some((pos, camera)) = &mut self.player_view {
            *camera = Camera::centred_on(*pos, w, h, view);
        }
        if let Mode::AiAction(a) = &mut self.mode {
            let mut to = self.camera;
            to.follow(a.start(), w, h, view, Camera::MARGIN);
            a.repan((self.camera.origin, to.origin));
        }
    }

    /// The unit drawn under the cursor, if any.
    pub fn hovered(&self) -> Option<&Unit> {
        let pos = self.cursor.pos;
        self.state.units().iter().find(|u| self.drawn_pos(u) == pos)
    }

    /// The unit drawn under the cursor to look at: [`hovered`](Self::hovered),
    /// or a unit down on the tile (0472). Only the panel reads it; commands
    /// see units on the field alone.
    pub fn looked_at(&self) -> Option<&Unit> {
        let pos = self.cursor.pos;
        let down = || self.state.down().iter().find(|u| u.pos == pos);
        self.hovered().or_else(down)
    }

    /// Where `unit` is drawn: its tile, or where it walks or stands before
    /// its move is sent ([`Mode::drawn_pos`]).
    fn drawn_pos(&self, unit: &Unit) -> Pos {
        self.mode.drawn_pos(unit.id).unwrap_or(unit.pos)
    }

    /// What the combat stage shows now (ticket 0413): the fight playing
    /// back, if one is and combat animations are on.
    pub fn stage_view(&self, ctx: &Ctx) -> Option<stage::view::StageView> {
        let Mode::Combat(pb) = &self.mode else {
            return None;
        };
        if !self.show_fights {
            return None;
        }
        let ground = pb
            .bout()
            .and_then(|bout| self.state.map().tiles.get(bout.defender.pos))
            .and_then(|id| ctx.game.content.terrain.display.get(*id))
            .map(|terrain| terrain.id.as_str());
        stage::view(ctx, pb, ground, &self.look.light)
    }

    /// Applies `cmd` (built by [`mode::step`] from legal choices) and
    /// records it in the history, then plays its combat if it had one, and
    /// continues browsing (or with the unit's move after its attack); queues
    /// the banners of its events, and its scenes unless its combat plays
    /// them, and updates the danger zone. Leaving the player phase keeps
    /// where the cursor and camera were; they go back there when it comes
    /// round again. Returns the events; a refused command changes nothing
    /// and returns `None`.
    fn apply(&mut self, cmd: &Command) -> Option<Vec<Event>> {
        let before = self.state.units().to_vec();
        let was_player = !self.ai_phase();
        let walked = self.effects.walked.take();
        // Refused: the battle is unchanged and the player browses again.
        self.applying = Some(cmd.clone());
        let events = self.state.apply(cmd).ok();
        self.applying = None;
        let playback = events.as_ref().and_then(|events| {
            let words = self.language.words();
            Playback::new(words, events, &before, self.state.fallen(), TIMINGS)
        });
        if let Some(events) = &events {
            self.history.push(cmd.clone());
            self.note_level_ups(events);
            if let Some(line) = loot::line(&self.state, self.language.words(), events) {
                self.toast = Some((MessageView::note(line), TOAST_S));
            }
            if was_player {
                self.guide.notice_events(cmd, events);
            }
            let in_playback = playback.is_some();
            self.queue.extend(events.iter().filter_map(|e| match e {
                Event::SceneTriggered { .. } if !in_playback => SceneCue::of(e).map(Queued::Scene),
                Event::Fled { units } => Flight::new(&self.state, units).map(Queued::Flight),
                _ => Banner::for_event(e).map(Queued::Banner),
            }));
            self.effects.show_events(events, &self.state);
            self.threat.battle_changed(&self.state);
            self.guide.battle_changed();
            // Checked (phase, units ready) once back to browsing.
            self.end_armed = true;
            let told = (&self.state, self.language.words());
            self.progress = Progress::new(events, &before, told, PROGRESS_TIMINGS);
            if was_player && self.ai_phase() {
                self.player_view = Some((self.cursor.pos, self.camera));
            } else if !was_player
                && !self.ai_phase()
                && let Some((pos, camera)) = self.player_view.take()
            {
                self.cursor.jump(pos);
                self.camera = camera;
            }
        }
        let played = events.clone();
        let playback = events.and_then(|events| {
            let words = self.language.words();
            let banner = art_list::playback_banner(&self.state, words, &events, &before);
            let playback = playback.map(|p| p.with_banner(banner));
            let cues = event_sounds::event_cues(&events, walked, playback.is_some(), &self.state);
            self.effects.cues.extend(cues);
            let attacks = event_sounds::combat_attacks(&events, &before, &self.state);
            playback.map(|p| p.with_sounds(&attacks, event_sounds::heals(&events)))
        });
        self.mode = match playback {
            Some(mut p) => {
                if !self.show_fights {
                    // Combat animations are off.
                    p.skip();
                }
                Mode::Combat(Box::new(p))
            }
            None => Mode::after_command(&self.state),
        };
        played
    }

    /// Applies `cmd`, which leaves the unit's action open (`Equip`), and
    /// reopens the action menu for the unit at the end of its path (its
    /// ranges recomputed: the weapon changed), on `Equip`.
    fn apply_stay(&mut self, cmd: &Command, sel: Selection) {
        // Back on what was chosen (`Talk` is gone once its talk played).
        let entry = match cmd {
            Command::Talk { .. } => MenuEntry::Talk,
            _ => MenuEntry::Equip,
        };
        self.apply(cmd);
        let Some(mut fresh) = Selection::new(&self.state, sel.unit) else {
            return;
        };
        if self.state.unit(sel.unit).is_some_and(|u| u.acted) {
            return;
        }
        fresh.path = sel.path;
        self.mode = mode::back_to_entry(fresh, &self.state, entry);
    }

    /// The units as drawn, each with how far it has faded out: the
    /// battle's units (those down too), except during a combat's playback, where its
    /// fighters show the HP it has reached (the attacker not dimmed yet)
    /// and the units that fell stay until they have faded; and during an
    /// AI action, before its combat, the units as they were before it.
    pub fn shown_units(&self) -> Vec<(Unit, f32)> {
        // Units that are down (0319) stay on their tiles.
        let down = self.state.down().iter().cloned().map(|u| (u, 0.0));
        if let Mode::AiAction(a) = &self.mode {
            let units = a.units().into_iter().map(|u| (u, 0.0));
            return units.chain(down).collect();
        }
        let Mode::Combat(pb) = &self.mode else {
            let units = self.state.units().iter().map(|u| (u.clone(), 0.0));
            return units.chain(down).collect();
        };
        let attacker = pb.bouts().first().map(|b| b.attacker.unit);
        self.state
            .units()
            .iter()
            .chain(self.state.down())
            .chain(pb.falls())
            .map(|u| {
                let mut u = u.clone();
                if let Some(hp) = pb.hp(u.id) {
                    u.hp = hp;
                }
                if Some(u.id) == attacker {
                    u.acted = false;
                }
                let fade = pb.fade(u.id).unwrap_or(0.0);
                (u, fade)
            })
            .collect()
    }

    /// Whether Rewind opens the rewind screen: browsing in the player phase
    /// of a battle still running.
    fn can_open_rewind(&self) -> bool {
        matches!(self.mode, Mode::Idle { .. })
            && self.state.phase() == Phase::Player
            && self.state.outcome().is_none()
    }

    /// The rewind screen on this battle, with the lesson on screen if it
    /// is about rewinding: it stays over the map there.
    fn rewind_screen(&self, ctx: &Ctx) -> RewindScreen {
        let frame = MapFrame {
            camera: self.camera,
            view: self.view,
            cursor: self.cursor.pos,
            look: self.look.clone(),
        };
        let lesson = self
            .guide
            .rewind_lesson(self.moment(), ctx, self.cursor.pos);
        RewindScreen::new(&self.history, &self.state, frame, ctx.words()).with_lesson(lesson)
    }

    /// Opens the rewind screen on a battle that was lost (0822), clearing
    /// what the defeat left pending. Rewinding plays on from the point
    /// picked; closing the screen opens Game Over again.
    fn rewind_from_defeat(&mut self, ctx: &Ctx) -> Transition {
        self.queue.clear();
        self.progress = None;
        self.mode = Mode::default();
        self.end_armed = false;
        Transition::Push(Box::new(self.rewind_screen(ctx)))
    }

    /// Carries out what the rewind screen popped with: closed, or a point
    /// to go back to. Closed on a battle still decided, it came from Game
    /// Over (0822), which opens again.
    fn rewound(&mut self, ctx: &mut Ctx, effect: RewindEffect) -> Transition {
        match effect {
            RewindEffect::None => {}
            RewindEffect::Close => {
                self.guide.notice(DoneWhen::RewindClosed);
                self.guide.tick_lessons(ctx);
                if self.state.outcome().is_some() {
                    return self.game_over(ctx, true);
                }
            }
            RewindEffect::Rewind(point) => {
                // The screen only confirms a listed point with a charge left.
                if let Ok(state) = self.history.rewind_to(point) {
                    self.state = state;
                    self.threat.battle_changed(&self.state);
                    self.guide.battle_changed();
                    self.mode = Mode::after_command(&self.state);
                    self.queue.clear();
                    self.progress = None;
                    self.end_armed = false;
                    // The ending is undone: the battle's own track again.
                    if self.outcome_music == OutcomeMusic::Heard {
                        self.outcome_music = OutcomeMusic::Unheard;
                        if let Some(cue) = &self.music {
                            ctx.audio.play_music(cue);
                        }
                    }
                }
            }
        }
        Transition::None
    }

    /// Gives `action` to the mode ([`mode::step`], after [`Mode::route`]
    /// applies the optional split keys), plays its menu sound and carries
    /// out its effect. Returns the screen the map menu opens, if any:
    /// `Options` or `Guides`.
    fn step_mode(&mut self, ctx: &mut Ctx, action: Action) -> Option<Opened> {
        // One device per frame: a key and a button pressed in the same frame
        // are both routed by the device pressed last (ADR-0036).
        let action = self.mode.route(action, &ctx.keymap, ctx.device)?;
        if let Mode::Info { .. } = self.mode
            && let Some(page) = info::turn_page(action, self.info_page, layout::info_pages(ctx))
        {
            self.info_page = page;
            ctx.audio.menu(MenuSound::Move);
            return None;
        }
        let before = self.mode.clone();
        let mode = core::mem::take(&mut self.mode);
        let shortcuts = map_menu::shortcuts(ctx.platform.keyboard, &ctx.keymap);
        let here = (&self.state, ctx.words());
        let (mode, effect) = mode::step_with(mode, action, self.cursor.pos, here, &shortcuts);
        if self.guide.holds_turn() && ends_turn(&before, &mode, &effect) {
            // An open lesson holds the turn (0456): nothing happens.
            self.mode = before;
            ctx.audio.menu(MenuSound::Denied);
            return None;
        }
        let mode = mode.at_pace(self.effects.pace.0);
        if let Some(sound) = sounds::step_sound(action, &before, &mode, &effect) {
            ctx.audio.menu(sound);
        }
        self.mode = mode;
        if !matches!(before, Mode::Info { .. }) {
            // The info screen opens on its first page.
            self.info_page = 0;
        }
        match effect {
            Effect::None => {}
            Effect::Apply(cmd) => {
                self.apply(&cmd);
            }
            Effect::ApplyStay(cmd, sel) => self.apply_stay(&cmd, *sel),
            Effect::Cursor(to) => {
                self.cursor.jump(to);
                self.follow(to);
            }
            Effect::Restart => self.leaving = Some(Leaving::Restart),
            Effect::Suspend => self.leaving = Some(Leaving::Suspend),
            Effect::Options => return Some(Opened::Options),
            Effect::Guides => return Some(Opened::Guides),
            Effect::Press(action) => self.pressed = Some(action),
            Effect::Shop { unit, dest } => return Some(Opened::Shop(unit, dest)),
        }
        None
    }

    /// Gives `action` to the map: opens the rewind screen, toggles the
    /// danger zone, cycles the units, moves the cursor, or steps the mode
    /// ([`step_mode`](Self::step_mode)). Notes what the player did for the
    /// lessons (0456). Returns the screen to open over the battle, if any.
    fn play_key(&mut self, ctx: &mut Ctx, action: Action) -> Option<Opened> {
        let was = Marks::of(&self.mode);
        let swaps = matches!(action, Action::CursorLeft | Action::CursorRight)
            && matches!(&self.mode, Mode::Targeting(t) if t.can_swap());
        let mut opened = None;
        match action {
            // The crank wound back (ticket 1035) opens it on the newest
            // action; its next notches step the open screen.
            Action::Rewind | Action::RewindBack if self.can_open_rewind() => {
                ctx.audio.menu(MenuSound::Select);
                self.guide.notice(DoneWhen::RewindOpened);
                opened = Some(Opened::Rewind);
            }
            Action::DangerZone if self.mode.cursor_free() => self.toggle_danger(),
            // The Playdate's system menu (ticket 1035): where the map menu
            // and its Options are a press away.
            Action::Options if matches!(self.mode, Mode::Idle { .. } | Mode::MapMenu { .. }) => {
                ctx.audio.menu(MenuSound::Select);
                opened = Some(Opened::Options);
            }
            Action::NextUnit | Action::PrevUnit if matches!(self.mode, Mode::Idle { .. }) => {
                let from = self.cursor.pos;
                self.cycle(action == Action::NextUnit);
                if self.cursor.pos != from {
                    ctx.audio.cursor_tick();
                }
            }
            Action::CursorLeft | Action::CursorRight | Action::CursorUp | Action::CursorDown
                if self.mode.cursor_free() =>
            {
                if self.move_cursor(action) {
                    ctx.audio.cursor_tick();
                    self.guide.notice(DoneWhen::CursorMoved);
                }
            }
            _ => opened = self.step_mode(ctx, action),
        }
        if swaps && matches!(self.mode, Mode::Targeting(_)) {
            self.guide.notice(DoneWhen::WeaponSwapped);
        }
        self.guide.notice_mode(was, &self.mode);
        self.guide.notice_pressed(action, self.ai_phase());
        opened
    }

    /// Opens `opened` over the battle, once the lessons have ticked what
    /// the player did.
    fn open(&mut self, ctx: &mut Ctx, opened: Opened) -> Transition {
        if opened == Opened::Options {
            self.guide.notice(DoneWhen::OptionsOpened);
        }
        self.guide.tick_lessons(ctx);
        Transition::Push(match opened {
            Opened::Options => Box::new(crate::screens::options::screen()),
            Opened::Guides => Box::new(GuidesScreen::new(&ctx.game.content)),
            Opened::Rewind => Box::new(self.rewind_screen(ctx)),
            Opened::Shop(unit, dest) => match ShopScreen::new(&self.state, unit, dest) {
                Some(shop) => Box::new(shop),
                None => return Transition::None,
            },
        })
    }

    /// Plays the frame's sounds and advances what is animated by `dt`
    /// seconds (`held`: Confirm is down). A fight's playback and the EXP
    /// bar play at the player's speeds (0805); holding Confirm plays them
    /// at [`HELD_SPEED`] instead, not on top (Nick,
    /// `docs/design/controls.md`): their clocks multiply a held frame by
    /// their own ×4, so the frame's time is divided by it first. A walk
    /// and an AI unit's action keep real time: their speeds were set when
    /// they started (the walk's pace, the action's [`ai_phase::Pacing`]).
    fn animate(&mut self, ctx: &mut Ctx, dt: f32, held: bool) {
        let ai = self.ai_phase();
        let settings = ctx.settings();
        let scaled = if held {
            dt * settings.game.battle_speed_held(ai, true) / HELD_SPEED
        } else {
            dt * settings.game.battle_speed(ai)
        };
        let own_pace = matches!(self.mode, Mode::Moving { .. } | Mode::AiAction(_));
        let dt = if own_pace { dt } else { scaled };
        let here = (&self.mode, &self.state);
        self.effects.play_sounds(ctx, here, dt, held);
        let mode = core::mem::take(&mut self.mode);
        self.mode = mode.tick(dt, held, &self.state, self.language.words());
        self.follow_ai_action();
        // A walk aimed at an enemy (0428) ends in its forecast: the cursor
        // goes onto the target.
        if let Mode::Targeting(t) = &self.mode
            && let Some(at) = self.state.unit(t.target()).map(|u| u.pos)
            && self.cursor.pos != at
        {
            self.cursor.jump(at);
            self.follow(at);
        }
        self.tick_progress(scaled, held);
    }

    /// Moves the cursor one tile for a cursor key; the camera and a
    /// selected unit's path follow. Returns whether it moved (not at the
    /// map's edge).
    fn move_cursor(&mut self, action: Action) -> bool {
        let tiles = &self.state.map().tiles;
        let (w, h) = (tiles.width(), tiles.height());
        let moved = cursor::dir(action).is_some_and(|dir| self.cursor.step(dir, w, h));
        if moved {
            let to = self.cursor.pos;
            self.follow(to);
            self.mode.cursor_moved(to, &self.state);
        }
        moved
    }

    /// Whether `unit` can still act this phase: its faction's phase and it
    /// hasn't acted.
    fn is_ready(&self, unit: &Unit) -> bool {
        Phase::of(unit.faction) == self.state.phase() && !unit.acted
    }

    /// The acting faction's ready units, ordered by `(y, x)`.
    pub fn ready_units(&self) -> Vec<Pos> {
        let mut ready: Vec<Pos> = self
            .state
            .units()
            .iter()
            .filter(|u| self.is_ready(u))
            .map(|u| u.pos)
            .collect();
        ready.sort_by_key(|p| (p.y, p.x));
        ready
    }

    /// Moves the cursor (and camera) to the next ready unit after the
    /// cursor in `(y, x)` order, or the previous one before it; wraps
    /// around. Nothing happens with no ready units.
    fn cycle(&mut self, forward: bool) {
        let ready = self.ready_units();
        let here = (self.cursor.pos.y, self.cursor.pos.x);
        let key = |p: &&Pos| (p.y, p.x);
        let to = if forward {
            ready.iter().find(|p| key(p) > here).or(ready.first())
        } else {
            ready.iter().rev().find(|p| key(p) < here).or(ready.last())
        };
        if let Some(&to) = to {
            self.cursor.jump(to);
            self.follow(to);
        }
    }

    /// The help line of a tip, the battle notes, EXP or level-up page, or
    /// banner on screen.
    fn overlay_help(&self, ctx: &Ctx) -> Option<StyledText> {
        if self.shown_tip().is_some() || self.notes_open() {
            return Some(ctx.text_styled("battle.help.close", &[]));
        }
        if let Some(p) = self.progress() {
            return Some(progress::help(ctx, p));
        }
        let key = match self.banner()?.kind {
            BannerKind::Outcome(_) | BannerKind::Objective => "battle.help.continue",
            BannerKind::Phase { .. } => "battle.help.skip",
        };
        Some(ctx.text_styled(key, &[]))
    }

    /// The help line for the mode and what is under the cursor, e.g. `f
    /// select · e info · s next unit · r rewind · d back` over a ready unit while
    /// browsing. Key names come from the keymap; [`help_styled`] has each
    /// in a key span.
    ///
    /// [`help_styled`]: Self::help_styled
    pub fn help(&self, ctx: &Ctx) -> String {
        self.help_styled(ctx).plain()
    }

    /// [`help`](Self::help), each key's name in a key span, drawn as a
    /// key cap (ticket 0254).
    pub fn help_styled(&self, ctx: &Ctx) -> StyledText {
        self.help_parts(ctx).0
    }

    /// [`help_styled`](Self::help_styled), and the cell (from the line's
    /// start, as the frame measures it) where its end-turn hint starts, if
    /// it has one: only the browsing help does (0445).
    fn help_parts(&self, ctx: &Ctx) -> (StyledText, Option<usize>) {
        if let Some(line) = self.overlay_help(ctx) {
            return (line, None);
        }
        if self.lessons_end_shown(ctx) {
            return (ctx.text_styled("battle.lessons_end.help", &[]), None);
        }
        let key = match &self.mode {
            Mode::Idle { threat } => {
                let back = if threat.is_some() {
                    "battle.hint.hide_range"
                } else {
                    "battle.hint.menu"
                };
                let (line, end) = self.help_idle(ctx, back);
                return (line, Some(end));
            }
            Mode::Objective => "battle.help.objective",
            Mode::RestartPrompt => "battle.help.restart",
            Mode::SuspendPrompt => "battle.help.suspend",
            Mode::EndTurnPrompt { .. } => {
                let km = ctx.help_keys();
                let [accept, also] = split_keys::help_end_turn_accept_actions(km);
                let (accept, also) = (action_key(km, accept), action_key(km, also));
                let keys = [("accept", &accept), ("also", &also)];
                return (text_with_keys(ctx, "battle.help.end_turn", &keys), None);
            }
            Mode::Info { .. } => "battle.help.info",
            Mode::Selected(sel) => {
                let aimed = sel
                    .target
                    .and_then(|t| self.state.unit(t))
                    .is_some_and(|u| u.pos == self.cursor.pos);
                if aimed {
                    // The forecast opens with a weapon or a spell (0430); an
                    // ally nothing attacks is healed (0451).
                    let options = sel.target.map_or_else(Vec::new, |t| {
                        aimed_options(&self.state, sel.unit, sel.dest(), t)
                    });
                    match aimed_first(&self.state, sel.unit, &options) {
                        Some(Equipped::Spell(_)) => "battle.help.aim_cast",
                        Some(Equipped::Weapon(_)) => "battle.help.aim_attack",
                        None => "battle.help.aim_heal",
                    }
                } else if self.cursor.pos == sel.dest() && sel.reach.is_stoppable(sel.dest()) {
                    "battle.help.move_here"
                } else {
                    "battle.help.selected"
                }
            }
            Mode::Moving { .. } => "battle.help.skip",
            Mode::List { menu, .. } => match list_entry(menu) {
                MenuEntry::Magic => "battle.help.spells",
                MenuEntry::Equip => "battle.help.equip",
                _ => "battle.help.menu",
            },
            Mode::ActionMenu { .. } | Mode::MapMenu { .. } | Mode::UnitList { .. } => {
                "battle.help.menu"
            }
            Mode::CastTarget(t) => match t.forecast() {
                Some(forecast) => return (Self::help_targeting(ctx, forecast), None),
                None => "battle.help.cast_target",
            },
            Mode::ItemTarget(_) | Mode::SkillTarget(_) => "battle.help.use_target",
            Mode::TalkTarget(_) => "battle.help.talk_target",
            Mode::Targeting(t) => return (Self::help_targeting(ctx, t), None),
            Mode::Combat(_) => "battle.help.combat",
            Mode::AiAction(_) => "battle.help.ai",
            Mode::MoveAfter { unit, tiles } => {
                let here = self.state.unit(*unit).map(|u| u.pos);
                if here == Some(self.cursor.pos) {
                    "battle.help.stay"
                } else if tiles.contains(&self.cursor.pos) {
                    "battle.help.move_after"
                } else {
                    "battle.help.move_only"
                }
            }
        };
        (self.with_info_hint(ctx, ctx.text_styled(key, &[])), None)
    }

    /// `line`, then the Info key's hint where Info opens a unit over the
    /// mode (a unit selected or its action menu, 1307): `<name>'s info`,
    /// or `unit info` when the line with the name would not fit the help
    /// row beside the debug hint.
    fn with_info_hint(&self, ctx: &Ctx, line: StyledText) -> StyledText {
        let info = self.mode.info_unit(self.cursor.pos, &self.state);
        let Some(unit) = info.and_then(|id| self.state.unit(id)) else {
            return line;
        };
        let name = ctx.words().unit(unit).to_owned();
        let named = ctx.text_styled("battle.hint.info_of", &[("name", &name)]);
        let named = join_hints([line.clone(), named]);
        let debug = super::debug_hint(ctx).map_or(0, |h| text_width(&h, ctx.widths()) + 1);
        let room = usize::try_from(HELP_BAR.w - 1).unwrap_or(0);
        if named.width(&frame_widths(ctx)) <= room.saturating_sub(debug) {
            return named;
        }
        join_hints([line, ctx.text_styled("battle.hint.unit_info", &[])])
    }

    /// The help line while picking an attack's target: the cursor keys
    /// pick the target, or left/right swap the weapon or spell and up/down
    /// pick the target when several can make the attack (0430, 0442).
    /// Confirm reads `cast` for a spell.
    fn help_targeting(ctx: &Ctx, t: &Targeting) -> StyledText {
        let km = ctx.help_keys();
        let plain = |key| ctx.text_styled(key, &[]);
        let mut hints = if t.can_swap() {
            let target = key_pair(km, Action::CursorUp, Action::CursorDown);
            vec![
                Self::swap_hint(ctx, "battle.hint.swap"),
                text_with_keys(ctx, "battle.hint.target", &[("keys", &target)]),
            ]
        } else {
            vec![plain("battle.hint.next_target")]
        };
        hints.push(plain(match t.with {
            Equipped::Weapon(_) => "battle.hint.attack",
            Equipped::Spell(_) => "battle.hint.cast",
        }));
        hints.push(plain("battle.hint.back"));
        join_hints(hints)
    }

    /// The text `key` with `{keys}` the keys that swap the weapon, as two
    /// keys that are not one action: `Left/Right swap` in the help line,
    /// `Left/Right weapon` on the forecast (0463).
    fn swap_hint(ctx: &Ctx, key: &str) -> StyledText {
        let keys = key_pair(ctx.help_keys(), Action::CursorLeft, Action::CursorRight);
        text_with_keys(ctx, key, &[("keys", &keys)])
    }

    /// The help line while browsing, over what is under the cursor; `back`
    /// is the hint of the Cancel key. Its last hint, end turn, starts in
    /// the same cell whatever is under the cursor (Nick, 0445): where the
    /// longest of the four forms puts it, the others padded with spaces up
    /// to it. Returns the line and that cell.
    fn help_idle(&self, ctx: &Ctx, back: &str) -> (StyledText, usize) {
        // Each hint's text once: the four forms share them, and this runs
        // every frame.
        let text = |key: &str| ctx.text_styled(key, &[]);
        let [moves, select, range, info, menu, next, back, end] = [
            "battle.hint.move",
            "battle.hint.select",
            "battle.hint.range",
            "battle.hint.info",
            "battle.hint.open_menu",
            "battle.hint.next_unit",
            back,
            "battle.hint.end_turn",
        ]
        .map(|key| Some(text(key)));
        let rewind = self.can_open_rewind().then(|| text("battle.hint.rewind"));
        let tail = [&next, &rewind, &back, &end];
        // Over a ready unit of ours, an enemy, another unit, an empty tile.
        let forms = [
            [&None, &select, &info],
            [&moves, &range, &info],
            [&moves, &None, &info],
            [&moves, &menu, &None],
        ]
        .map(|head| -> Vec<StyledText> {
            let hints = head.into_iter().chain(tail).flatten();
            hints.cloned().collect()
        });
        let form = match self.hovered() {
            Some(u) if self.is_ready(u) && u.faction == Faction::Player => 0,
            Some(u) if u.faction != Faction::Player => 1,
            Some(_) => 2,
            None => 3,
        };
        let widths = frame_widths(ctx);
        let end_of = |hints: &Vec<StyledText>| hint_column(hints, hints.len() - 1, &widths);
        let column = forms.iter().map(end_of).max().unwrap_or(0);
        (pad_last_hint(&forms[form], column, &widths), column)
    }

    /// The toggles' hints at the left of the message row: the Zoom map
    /// key and `zoom: 2×` (only under a map skin that can zoom), then the
    /// Danger zone key and `danger zone: OFF`, each key named from the
    /// active keymap.
    fn danger_hint(&self, ctx: &Ctx) -> StyledText {
        let zoom = ctx.game.map_skin.zoom().map(|zoom| match zoom {
            MapZoom::X1 => "battle.status.zoom_x1",
            MapZoom::X2 => "battle.status.zoom_x2",
        });
        let danger = if self.threat.danger().is_some() {
            "battle.status.danger_on"
        } else {
            "battle.status.danger_off"
        };
        let hints = zoom.into_iter().chain([danger]);
        join_hints(hints.map(|key| ctx.text_styled(key, &[])))
    }

    /// The Auto-end key and `auto-end: ON`, the key named from the active
    /// keymap.
    fn auto_end_hint(&self, ctx: &Ctx) -> StyledText {
        let key = if self.auto_end {
            "battle.status.auto_end_on"
        } else {
            "battle.status.auto_end_off"
        };
        ctx.text_styled(key, &[])
    }

    /// The toggles' hints in one line, as they read left to right: zoom,
    /// danger zone, auto-end.
    #[cfg(test)]
    pub fn status(&self, ctx: &Ctx) -> StyledText {
        join_hints([self.danger_hint(ctx), self.auto_end_hint(ctx)])
    }

    /// The toggles' row (0445): shown only while the help has its
    /// end-turn hint (starting in cell `end_turn`) and no combat or
    /// preview line is up, auto-end above end turn.
    fn toggles(
        &self,
        ctx: &Ctx,
        end_turn: Option<usize>,
        message: Option<&MessageView>,
    ) -> Option<TogglesView> {
        let column = end_turn?;
        if message.is_some_and(|m| m.kind == MessageKind::Said) {
            return None;
        }
        let row = (self.danger_hint(ctx), self.auto_end_hint(ctx));
        Some(TogglesView::new(row.0, row.1, column, &frame_widths(ctx)))
    }

    /// What is on the visible map (ADR-0038), for the map skin to paint and
    /// for tests to read: the terrain, the tiles flashing after their
    /// terrain changed, the ranges the mode shows, the units as drawn, the
    /// cursor, a selected unit's path and, while the player browses, the
    /// sides the map goes on past.
    pub fn scene(&self, ctx: &Ctx) -> MapScene {
        self.scene_of_size(ctx, ctx.game.map_skin.view_tiles(map_area(ctx)))
    }

    /// [`scene`](Self::scene) for a view of `size` tiles instead of the
    /// map area's: the screen's camera if it is that size, else one
    /// centred on the cursor. For a picture of the map alone
    /// (`cargo shuyi frame-png --map`).
    pub fn scene_of_size(&self, ctx: &Ctx, size: (i32, i32)) -> MapScene {
        let origin = self.camera_in(size).origin;
        let mut scene = terrain_scene(&self.state, origin, size);
        scene.clock_ms = clock_ms(ctx);
        scene.look.clone_from(&self.look);
        for flash in &self.effects.flashes {
            if let Some(tile) = scene.tile_mut(flash.pos) {
                tile.flashes.push(flash.strength());
            }
        }
        self.add_ranges(&mut scene);
        scene.ranges_shown = self.ranges_shown;
        let notes_t = match ctx.idle_motion {
            IdleMotion::On => self.notes_t,
            IdleMotion::Off => 0.0,
        };
        self.add_units(&mut scene, notes_t);
        animate(&mut scene, self.mode.gait());
        scene.cursor = self.cursor_view(ctx).filter(|c| scene.contains(c.pos));
        if let Mode::Selected(sel) = &self.mode {
            scene.path.clone_from(&sel.path);
        }
        scene.threat = self.threat_view();
        // The arrows at the edges the map goes on past: only while the
        // player browses (ticket 0449).
        if self.mode.cursor_free() && !self.ai_phase() {
            let tiles = &self.state.map().tiles;
            let map = (i32::from(tiles.width()), i32::from(tiles.height()));
            scene.more = MapEdges::of_view(origin, size, map);
        }
        scene
    }

    /// Notes where and when the selected unit's ranges opened: its tile,
    /// at this frame's clock, when a unit is newly selected; kept while
    /// it stays selected; dropped otherwise (ticket 0923). Without idle
    /// motion the clock stands still, so the ranges show at once.
    fn note_ranges(&mut self, ctx: &Ctx) {
        let from = match &self.mode {
            Mode::Selected(sel) if ctx.idle_motion == IdleMotion::On => {
                self.state.unit(sel.unit).map(|u| u.pos)
            }
            _ => None,
        };
        self.ranges_shown = ranges_opened(from, self.ranges_shown, clock_ms(ctx));
    }

    /// Lays an escape map's exit area on `scene` (always), the danger zone
    /// over it, if shown, then the mode's ranges over them: a selected unit's move and attack ranges, a shown threat
    /// area, where a unit may move after its attack, or the targets being
    /// chosen from. While a spell's target is picked (0410): the enemies it
    /// can hit as an attack range or the allies it can heal as a heal
    /// range, and each tile it can change marked with the terrain it would
    /// become (Nick: units and terrain must look different).
    fn add_ranges(&self, scene: &mut MapScene) {
        let all = |set: &TileSet| -> Vec<Pos> { set.iter().collect() };
        // An escape map's exit, always, under everything else.
        if let Objective::Escape { area } = self.state.objective() {
            let tiles = self.state.map().tiles.positions();
            scene.tint(tiles.filter(|&p| area.contains(p)), range::EXIT);
        }
        if let Some(zone) = self.threat.danger() {
            scene.tint(all(zone), range::DANGER);
        }
        match &self.mode {
            Mode::Selected(sel) => {
                scene.tint(all(&sel.moves), range::MOVE);
                scene.tint(all(&sel.attack), range::ATTACK);
            }
            Mode::Idle {
                threat: Some(threat),
            } => scene.tint(all(&threat.area), range::ATTACK),
            Mode::MoveAfter { tiles, .. } => scene.tint(tiles.clone(), range::MOVE),
            Mode::Targeting(t) => {
                let at = t.targets.iter().filter_map(|&id| self.state.unit(id));
                scene.tint(at.map(|u| u.pos), range::ATTACK);
            }
            Mode::CastTarget(t) => {
                let units = t.targets().iter().filter_map(|target| match target {
                    CastTarget::Unit(id) => self.state.unit(*id).map(|u| u.pos),
                    CastTarget::Tile(_) => None,
                });
                let attack = self
                    .state
                    .spells()
                    .get(t.spell())
                    .is_some_and(trpg_core::SpellDef::is_attack);
                let kind = if attack { range::ATTACK } else { range::HEAL };
                scene.tint(units, kind);
                for (pos, becomes) in t.tile_changes(&self.state) {
                    if let Some(tile) = scene.tile_mut(pos) {
                        tile.becomes = Some(becomes);
                    }
                }
            }
            Mode::SkillTarget(t) => {
                let at = t.targets().iter().filter_map(|&id| self.state.unit(id));
                scene.tint(at.map(|u| u.pos), range::ATTACK);
            }
            Mode::ItemTarget(t) => {
                let at = t.targets().iter().filter_map(|&id| self.state.unit(id));
                let dest = t.sel.dest();
                let tiles = at.map(|u| if u.id == t.sel.unit { dest } else { u.pos });
                scene.tint(tiles, range::HEAL);
            }
            // The focused area skill's reach (0482): the allies standing in
            // it are the ones it would reach.
            Mode::List { .. } => scene.tint(self.reach_shown().iter().copied(), range::HEAL),
            _ => {}
        }
    }

    /// Adds the units to `scene`, each where it is drawn; during a combat's
    /// playback or an AI action, as [`shown_units`](Self::shown_units) has
    /// them. While the battle notes are up, the units they are about blink:
    /// they are highlighted in the on half of the blink, `notes_t` seconds
    /// after they came up.
    fn add_units(&self, scene: &mut MapScene, notes_t: f32) {
        if !self.playing() {
            let blink = self.notes_shown() && notes::blink_on(notes_t);
            for unit in self.state.units().iter().chain(self.state.down()) {
                let noted = blink && notes::is_noted(self.state.battle_notes(), unit.id);
                let view = UnitView::of(unit).at(self.drawn_pos(unit));
                scene.push_unit(view.highlighted(noted));
            }
            for view in self.flight_views() {
                scene.push_unit(view);
            }
            return;
        }
        for (unit, fade) in self.shown_units() {
            scene.push_unit(UnitView::of(&unit).fading(fade));
        }
        // An AI action shows the units as they were before it, the
        // leftovers among them.
        if !matches!(self.mode, Mode::AiAction(_)) {
            for view in self.flight_views() {
                scene.push_unit(view);
            }
        }
    }

    /// The cursor as the mode shows it (in the player's cursor style): on
    /// its tile while browsing or on a selected unit; none on the tile the
    /// path's arrowhead points at (a unit selected and the cursor away from
    /// it), during a walk or in a menu. During an AI action, on its unit
    /// until it walks.
    fn cursor_view(&self, ctx: &Ctx) -> Option<CursorView> {
        let pos = self.cursor.pos;
        match &self.mode {
            Mode::Moving { .. }
            | Mode::ActionMenu { .. }
            | Mode::List { .. }
            | Mode::Combat(_)
            | Mode::UnitList { .. }
            | Mode::Objective
            | Mode::EndTurnPrompt { .. }
            | Mode::RestartPrompt
            | Mode::SuspendPrompt
            | Mode::Info { .. } => return None,
            Mode::AiAction(a) if !a.shows_cursor() => return None,
            Mode::Selected(sel) if pos == sel.dest() && pos != sel.origin() => return None,
            Mode::Selected(_)
            | Mode::Idle { .. }
            | Mode::MoveAfter { .. }
            | Mode::Targeting(_)
            | Mode::CastTarget(_)
            | Mode::SkillTarget(_)
            | Mode::ItemTarget(_)
            | Mode::TalkTarget(_)
            | Mode::AiAction(_)
            | Mode::MapMenu { .. } => {}
        }
        // Its pulse is held at its start when idle motion is off (1290).
        let brightness = match ctx.idle_motion {
            IdleMotion::On => self.cursor.brightness(),
            IdleMotion::Off => 1.0,
        };
        Some(CursorView::new(
            pos,
            brightness,
            ctx.settings().game.cursor_style,
        ))
    }

    /// What the screen shows this frame (ADR-0054): the map's
    /// [`scene`](Self::scene) and everything around and over it, for the
    /// skin to paint and for tests to read.
    pub fn view(&self, ctx: &Ctx) -> BattleView {
        let words = ctx.words();
        let told = (&self.state, words);
        let stage = self.stage_view(ctx);
        let fight_box = match (&self.mode, &stage) {
            (Mode::Combat(pb), None) => playback::box_view(words, pb),
            _ => None,
        };
        let faces = (&ctx.game.content.portraits, &ctx.game.lead);
        let info = match self.mode {
            Mode::Info { unit, .. } => self.state.unit(unit).map(|u| InfoView {
                page: self.info_page,
                face: info::face(faces, u),
                supports: info::supports(told, &ctx.game.content.names, u),
                ..info::view(told, u)
            }),
            _ => None,
        };
        let forecast = self.forecast().and_then(|t| {
            let swap = t
                .can_swap()
                .then(|| Self::swap_hint(ctx, "battle.forecast.swap"));
            forecast::view(told, t).map(|f| ForecastView { swap, ..f })
        });
        let progress = self
            .progress()
            .and_then(|p| progress::view(words, faces, p));
        let notes = self.notes_open().then(|| NotesView {
            title: ctx.text(notes::NOTES_TITLE).to_owned(),
            lines: notes::note_lines(self.state.battle_notes(), words),
        });
        let limit = map_menu::shown_limit(&self.state);
        // The Playdate's look has no help bar (ADR-0070 §4): building it
        // there would only spend its frame budget.
        let bar = ctx.game.skin_set == SkinSet::Glyph;
        let (help, end_turn) = if bar {
            self.help_parts(ctx)
        } else {
            (StyledText::new(), None)
        };
        let message = self.message();
        let (m, battle) = (self.moment(), self.battle_id.as_deref());
        BattleView {
            menu: self.menu_view(words),
            popups: self.effects.popups.clone(),
            panel: forecast
                .is_none()
                .then(|| self.panel_view(words, ctx.game.skin_set == SkinSet::Glyph)),
            stage,
            dialog: self.dialog_view(ctx),
            notes,
            noted: self.noted(),
            banner: self.banner().map(|b| self.banner_view(b, words, limit)),
            tip: self.guide.tip_view(m, ctx),
            lesson: self.guide.lesson_view(m, ctx, self.cursor.pos),
            lessons_end: self.guide.lessons_end_view(m, ctx, battle, self.cursor.pos),
            toggles: self.toggles(ctx, end_turn, message.as_ref()),
            message,
            scene: self.scene(ctx),
            help,
            debug_hint: super::debug_hint(ctx),
            forecast,
            info,
            fight_box,
            progress,
            map_info: self.map_info(words),
        }
    }

    /// The turn and the rewind charges left, while the map menu is open.
    fn map_info(&self, words: Words<'_>) -> Option<MapInfoView> {
        matches!(self.mode, Mode::MapMenu { .. }).then(|| MapInfoView {
            turn: map_menu::turn_text(&self.state, words),
            charges: rewind::charges_text(
                words,
                self.history.charges_left(),
                self.state.rewind_charges(),
            ),
        })
    }

    /// The menu open, if any: the action menu or a list it opened, beside
    /// its unit (where it will stand); the map menu beside the cursor; the
    /// unit list in the middle. Its headers are written in `words`.
    fn menu_view(&self, words: Words<'_>) -> Option<BattleMenuView> {
        let (place, menu) = match &self.mode {
            Mode::ActionMenu { sel, menu, .. } => (MenuPlace::Beside(sel.dest()), menu.view()),
            Mode::List { sel, menu } => (MenuPlace::Beside(sel.dest()), menu.view()),
            Mode::MapMenu { menu } => (MenuPlace::Beside(self.cursor.pos), menu.view()),
            Mode::UnitList { menu } => (MenuPlace::Centred, menu.view()),
            _ => return None,
        };
        let header = match &self.mode {
            Mode::List { sel, menu } => match menu.value(menu.focus()) {
                // The weapon the focused art would be used with, and its
                // durability.
                Some(Line::Art(line)) => {
                    let weapon = (sel.unit, &Equipped::Weapon(line.slot));
                    art_list::header((&self.state, words), weapon)
                }
                // The pack's size.
                Some(Line::Item(_)) => Some(MenuHeader {
                    name: items::pack_header(&self.state, words),
                    uses: None,
                }),
                _ => None,
            },
            _ => None,
        };
        Some(BattleMenuView {
            menu,
            place,
            header,
            clear_of: self.reach_shown().to_vec(),
        })
    }

    /// The focused area skill's reach in the Skill list (0482), which the
    /// map tints and the list keeps clear of (0492); none otherwise.
    fn reach_shown(&self) -> &[Pos] {
        match &self.mode {
            Mode::List { menu, .. } => match menu.value(menu.focus()) {
                Some(Line::Skill(c)) => &c.area,
                _ => &[],
            },
            _ => &[],
        }
    }

    /// The side panel: the terrain and the unit under the cursor (as
    /// drawn, during a playback); with its `numbers` (stats, weapon,
    /// combat numbers) where the skin paints them, the PC's (0484).
    fn panel_view(&self, words: Words<'_>, numbers: bool) -> PanelView {
        let pos = self.cursor.pos;
        let told = (&self.state, words);
        let selected = self.mode.selection().is_some();
        let mut view = if self.playing() {
            let shown = self.shown_units();
            let hovered = shown.iter().find(|(u, f)| u.pos == pos && *f < 1.0);
            panel::view(told, pos, hovered.map(|(u, _)| u), (selected, numbers))
        } else {
            panel::view(told, pos, self.looked_at(), (selected, numbers))
        };
        // A heal aimed at the unit shown: its HP line says it (Nick, 1286).
        let healed = self
            .aimed_heal()
            .filter(|(id, _)| self.looked_at().is_some_and(|u| u.id == *id) && !self.playing());
        match (healed, view.unit.as_mut()) {
            (Some((_, gain)), Some(unit)) => unit.hp_then = Some(unit.hp + gain),
            _ => view.preview = self.panel_preview(words),
        }
        view
    }

    /// The unit a heal or potion being aimed is aimed at, and the HP it
    /// would gain.
    fn aimed_heal(&self) -> Option<(UnitId, StatValue)> {
        match &self.mode {
            Mode::CastTarget(t) => t.heal(&self.state),
            Mode::ItemTarget(t) => t.heal(&self.state),
            _ => None,
        }
    }

    /// What the spell, item or skill being aimed will do, under its name,
    /// for the side panel: the line `message` shows (1286).
    fn panel_preview(&self, words: Words<'_>) -> Option<PanelPreviewView> {
        let state = &self.state;
        let (title, text) = match &self.mode {
            Mode::CastTarget(t) => {
                let def = state.spells().get(t.spell())?;
                (words.spell(def).to_owned(), t.preview(state, words)?)
            }
            Mode::ItemTarget(t) => (t.name(state, words), t.preview(state, words)),
            Mode::SkillTarget(t) => (
                skills::skill_name(state, words, t.skill()),
                t.preview(state, words),
            ),
            _ => return None,
        };
        Some(PanelPreviewView { title, text })
    }

    /// The box over the map: the objective page (the battle notes under
    /// it) or a yes/no question.
    /// What `banner` shows; the objective's with its lines (0481).
    fn banner_view(&self, banner: &Banner, words: Words<'_>, limit: Option<Turn>) -> BannerView {
        let mut view = banner.view(words, limit);
        if banner.kind == BannerKind::Objective {
            view.lines = objective::lines(&self.state, self.place_name.as_deref(), words);
        }
        view
    }

    fn dialog_view(&self, ctx: &Ctx) -> Option<DialogView> {
        let words = ctx.words();
        let question = |question: String| {
            DialogView::Question(QuestionView {
                question,
                answers: ctx.text_styled("battle.yes_no", &[]),
            })
        };
        match &self.mode {
            Mode::Objective => {
                let lines = notes::note_lines(self.state.battle_notes(), words);
                let under = (!lines.is_empty()).then(|| NotesView {
                    title: ctx.text(notes::NOTES_HEADING).to_owned(),
                    lines,
                });
                Some(DialogView::Objective(ObjectiveView {
                    title: ctx.text("battle.objective.title").to_owned(),
                    lines: objective::lines(&self.state, self.place_name.as_deref(), words),
                    village: objective::village_count(&self.state, words),
                    turn: map_menu::turn_text(&self.state, words),
                    notes: under,
                }))
            }
            Mode::EndTurnPrompt { ready } => {
                Some(question(map_menu::end_turn_question(words, *ready)))
            }
            Mode::RestartPrompt => Some(question(ctx.text("battle.restart_question").to_owned())),
            Mode::SuspendPrompt => Some(question(ctx.text("battle.suspend_question").to_owned())),
            _ => None,
        }
    }

    /// Where the units the battle notes are about are drawn, while the
    /// notes are on screen; empty otherwise.
    fn noted(&self) -> Vec<Pos> {
        if !self.notes_shown() {
            return Vec::new();
        }
        let noted = self.state.battle_notes();
        self.state
            .units()
            .iter()
            .filter(|u| notes::is_noted(noted, u.id))
            .map(|u| self.drawn_pos(u))
            .collect()
    }

    /// Ages the heal and burn numbers and the terrain flashes
    /// ([`Effects::tick`]) and the message by `dt` seconds, dropping those
    /// whose time is up.
    fn tick_popups(&mut self, dt: f32) {
        let banner = self.banner().is_some();
        self.effects.tick(dt, banner);
        if let Some((_, left)) = &mut self.toast {
            *left -= dt;
            if *left <= 0.0 {
                self.toast = None;
            }
        }
    }

    /// The attack forecast on screen: while an attack's target is picked,
    /// or a spell's with the cursor on an enemy.
    fn forecast(&self) -> Option<&Targeting> {
        forecast_of(&self.mode)
    }

    /// The line over the help bar: a combat's message, the preview of the
    /// skill, item or spell being aimed, or the toast.
    fn message(&self) -> Option<MessageView> {
        let words = self.language.words();
        let preview = match &self.mode {
            Mode::Combat(pb) => pb.message(words),
            Mode::SkillTarget(t) => Some(t.preview(&self.state, words)),
            Mode::ItemTarget(t) => Some(t.preview(&self.state, words)),
            Mode::CastTarget(t) => t.preview(&self.state, words),
            _ => None,
        };
        let said = |text| MessageView {
            text,
            kind: MessageKind::Said,
        };
        let toast = || self.toast.as_ref().map(|(m, _)| m.clone());
        preview.map(said).or_else(toast)
    }
}

/// Sets the units of `scene` moving (ticket 0440): the unit of `gait`
/// turned the way it walks, between two tiles, its legs going; every other
/// unit that can still act and isn't falling stepping on the spot, all
/// together, by the scene's clock. A unit that has acted stands still.
fn animate(scene: &mut MapScene, gait: Option<(UnitId, Gait)>) {
    let idle = walk::idle_frame(scene.clock_ms);
    for unit in &mut scene.units {
        match gait {
            Some((id, gait)) if id == unit.id => {
                unit.facing = gait.facing;
                unit.frame = gait.frame;
                unit.offset = gait.offset;
            }
            _ if !unit.acted && unit.fade <= 0.0 => unit.frame = idle,
            _ => {}
        }
    }
}

// engine-friction: engine#3206, moves (game 1217 deletes it)
/// The camera for a viewport of `view` tiles on `state`'s map: `camera`,
/// if it was kept for that size, else (the map skin changed) one centred
/// on the `cursor`'s tile.
fn fit_camera(
    camera: Camera,
    kept_for: (i32, i32),
    cursor: Pos,
    state: &BattleState,
    view: (i32, i32),
) -> Camera {
    if view == kept_for {
        return camera;
    }
    let tiles = &state.map().tiles;
    Camera::centred_on(cursor, tiles.width(), tiles.height(), view)
}

/// Whole milliseconds of `ctx`'s clock, which only goes forward: a map
/// scene's clock. 0 when idle motion is off ([`loop_clock_s`]).
fn clock_ms(ctx: &Ctx) -> u64 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let ms = (loop_clock_s(ctx).max(0.0) * 1000.0) as u64;
    ms
}

/// Where and when the ranges opened, now that they would spread `from`
/// (a selected unit's tile; `None` with none) at `now_ms`: what was
/// `kept` while they still spread from the same tile, opened now from a
/// new one, none without (ticket 0923).
fn ranges_opened(from: Option<Pos>, kept: Option<RangesShown>, now_ms: u64) -> Option<RangesShown> {
    match (from, kept) {
        (Some(from), Some(kept)) if kept.from == from => Some(kept),
        (Some(from), _) => Some(RangesShown::new(from, now_ms)),
        (None, _) => None,
    }
}

/// A view of `size` tiles from `origin` showing `state`'s terrain (and
/// the terrain one tile outside the view, for a skin to join pictures to)
/// and nothing else.
fn terrain_scene(state: &BattleState, origin: Pos, size: (i32, i32)) -> MapScene {
    let mut scene = MapScene::new(origin, size);
    let tiles = &state.map().tiles;
    scene.set_terrain(|pos| tiles.get(pos).copied());
    scene
}

impl BattleScreen {
    /// [`Screen::uncovered`]'s answer to what the screen above popped
    /// with, before the guide's lines are let go if it leaves.
    fn answered(&mut self, ctx: &mut Ctx, result: Option<ScreenResult>) -> Transition {
        let Some(result) = result else {
            return Transition::None;
        };
        let result = match result.downcast::<RewindEffect>() {
            Ok(effect) => return self.rewound(ctx, effect),
            Err(other) => other,
        };
        let result = match result.downcast::<ShopVisit>() {
            Ok(visit) => {
                // Nothing chosen: the unit hasn't acted, its menu is back.
                if let Some(cmd) = visit.command() {
                    self.apply(&cmd);
                }
                return Transition::None;
            }
            Err(other) => other,
        };
        match result.take::<GameOverChoice>() {
            Some(GameOverChoice::Rewind) => self.rewind_from_defeat(ctx),
            Some(GameOverChoice::Retry) => Transition::pop_with(BattleEnd::Restart),
            Some(GameOverChoice::Title) => Transition::pop_with(BattleEnd::Title),
            None => Transition::None,
        }
    }

    /// `transition`, once the guide's lines are let go if it takes the
    /// battle off the stack (0496): anything but staying or a screen
    /// pushed over it.
    fn leaving(&mut self, transition: Transition, ctx: &mut Ctx) -> Transition {
        if !matches!(transition, Transition::None | Transition::Push(_)) {
            self.guide.leave(ctx);
        }
        transition
    }

    /// [`Screen::update`]'s frame, before its tip is noted as shown.
    fn update_frame(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        self.begin_frame(ctx, input.dt());
        // A banner already up (a battle decided at its start).
        self.play_outcome_music(ctx);
        let dt = if input.dt().is_finite() {
            input.dt().max(0.0)
        } else {
            0.0
        };
        self.tick_popups(dt);
        self.take_in_tips(ctx);
        let mut actions: VecDeque<Action> = input.actions.iter().copied().collect();
        while let Some(action) = actions.pop_front() {
            self.guide.tick_lessons(ctx);
            if self.overlay_key(ctx, action) {
                if let Some(leaving) = self.leaving {
                    return self.leave(leaving);
                }
                continue;
            }
            self.guide.pressed_on_map();
            // The zoom is the player's view, not a move: in any phase.
            if action == Action::ZoomMap {
                self.toggle_zoom(ctx);
                self.guide.notice_pressed(action, self.ai_phase());
                continue;
            }
            // The AI's phase: only Confirm (hold: faster) and Cancel (skip
            // a fight) do anything else.
            if self.ai_phase() && !matches!(action, Action::Confirm | Action::Cancel) {
                continue;
            }
            if action == Action::ToggleAutoEnd {
                self.toggle_auto_end(ctx);
                self.guide.notice_pressed(action, self.ai_phase());
                continue;
            }
            if self.progress_key(action) {
                continue;
            }
            if self.flight().is_some() {
                if action == Action::Confirm {
                    self.queue.pop_front();
                }
                continue;
            }
            if self.banner().is_some() {
                if action == Action::Confirm {
                    let t = self.close_banner(ctx);
                    if !matches!(t, Transition::None) {
                        return t;
                    }
                }
                continue;
            }
            if self.ai_phase() && !self.playing() {
                continue;
            }
            if let Some(opened) = self.play_key(ctx, action) {
                return self.open(ctx, opened);
            }
            // A map menu shortcut: its key, next.
            if let Some(pressed) = self.pressed.take() {
                actions.push_front(pressed);
            }
            if let Some(leaving) = self.leaving {
                return self.leave(leaving);
            }
        }

        self.guide.tick_lessons(ctx);
        self.tick_notes(dt);
        let was = Marks::of(&self.mode);
        self.animate(ctx, dt, input.is_held(Action::Confirm));
        self.guide.notice_mode(was, &self.mode);
        if matches!(self.mode, Mode::Selected(_)) {
            let threat = self.threat_target().map(|_| self.threat_view().is_some());
            self.guide.notice_threat(threat);
        }
        self.guide.tick_lessons(ctx);
        let waiting = self.shown_tip().is_some() || self.playing();
        if !waiting {
            self.tick_flight(dt);
        }
        if let Some(Queued::Banner(banner)) = self.queue.front_mut()
            && !waiting
        {
            banner.t += dt;
            if banner.expired() {
                // Only phase banners expire, and closing one never leaves.
                self.close_banner(ctx);
            }
        }
        self.play_outcome_music(ctx);
        if let Some(scene) = self.next_scene(ctx) {
            return Transition::Push(Box::new(scene));
        }
        self.take_in_lessons(ctx);
        self.check_auto_end(ctx);
        self.drive_ai(ctx);
        self.guide.tick_lessons(ctx);
        // On screen as this frame ends, if it can still be done: open
        // from now on.
        let can_zoom = ctx.game.map_skin.zoom().is_some();
        self.guide.settle_lessons(&self.state, &self.mode, can_zoom);
        self.guide.mark_shown(self.moment());
        Transition::None
    }
}

impl Screen<Shuyi> for BattleScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::BATTLE.to_vec())
    }

    /// The battle's rules state, in a session record's frame hash (ticket
    /// 1210): a playback that plays the battle differently is found at the
    /// frame it parts.
    fn state_hash(&self) -> Option<shuyi_core::StateHash> {
        shuyi_core::state_hash(&self.state).ok()
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        let transition = self.update_frame(ctx, input);
        self.note_ranges(ctx);
        // Still on screen: its tip, if any, is seen (0467), and the guide
        // says what its box or the lesson's says (0476). A screen going
        // over the battle keeps her set as it was, but for a scene it
        // plays now, whose lines stay in it (1296); the set is sent anew
        // once back on top.
        let stays = matches!(transition, Transition::None);
        let m = self.moment();
        let scenes = self
            .guide
            .keep_due(stays, m, ctx)
            .then(|| self.coming_scene_lines(ctx));
        let battle = self.battle_id.as_deref();
        self.guide.end_frame(stays, m, battle, scenes, ctx);
        self.leaving(transition, ctx)
    }

    /// A screen over the battle (a scene, the rewind screen, …): no tip or
    /// lesson shows until it goes.
    fn covered(&mut self, _ctx: &mut Ctx) {
        self.guide.covered();
    }

    /// On top again, with what the screen above popped with: the rewind
    /// screen's [`RewindEffect`], carried out; or Game Over's choice (0822),
    /// where `Rewind` opens the rewind screen on the lost battle and
    /// `Retry Battle` and `Title` leave. A scene, the Options screen and the
    /// Guides list hand nothing back.
    fn uncovered(&mut self, ctx: &mut Ctx, result: Option<ScreenResult>) -> Transition {
        self.guide.uncovered();
        let transition = self.answered(ctx, result);
        self.leaving(transition, ctx)
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        SKINS.paint(ctx, &self.view(ctx), buf);
    }

    /// The grid kit's `grid` channel of the debug view (engine ADR-0085):
    /// the map's layout as its skin laid it out, while the map shows (a
    /// fight's stage covers it).
    fn debug_draw(&self, ctx: &Ctx, out: &mut shuyi_ui::debug::gizmo::Gizmos) {
        use shuyi_ui::grid::debug::{GRID, layout_gizmos};
        // Asking offers the channel, so the debug menu lists it.
        if !out.channel_on(GRID) || self.stage_view(ctx).is_some() {
            return;
        }
        let scene = self.scene(ctx);
        let grid = ctx.game.map_skin.grid(&scene, map_area(ctx));
        layout_gizmos(&scene, &grid, out);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    fn as_any_mut(&mut self) -> Option<&mut dyn core::any::Any> {
        Some(self)
    }
}

/// Helpers for this module's tests and its submodules'.
#[cfg(test)]
pub(crate) mod testing {
    use std::sync::Arc;

    use trpg_core::{
        BattleMap, BattlePack, BattleSetup, BattleState, Command, Objective, Pos, SkillId,
        StatValue, Stock, Unit, UnitAction, UnitId,
    };

    use core::ops::Deref;

    use super::guides::GuidesScreen;
    use super::layout::MAP_VIEW;
    use super::rewind::RewindScreen;
    use crate::glyph_buffer::GlyphBuffer;
    use crate::input::Action;
    use crate::screen::{Ctx, FrameInput, Screen, ScreenStack};
    use crate::widgets::menu::Menu;

    use super::art_list::ArtLine;
    use super::items::{EquipChoice, PackGroup};
    use super::magic::SpellChoice;
    use super::mode::Line;
    use super::skills::SkillChoice;

    /// What a menu's items stand for, in order.
    pub trait Values<T> {
        /// The items' values.
        fn values(&self) -> Vec<T>;
    }

    impl<T: Clone> Values<T> for Menu<T> {
        fn values(&self) -> Vec<T> {
            self.items().iter().map(|i| i.value.clone()).collect()
        }
    }

    /// A list's lines of each kind, in order.
    pub trait Lines {
        /// Its arts.
        fn arts(&self) -> Vec<ArtLine>;
        /// Its spells.
        fn spells(&self) -> Vec<SpellChoice>;
        /// Its skills.
        fn skills(&self) -> Vec<SkillChoice>;
        /// Its pack's groups.
        fn groups(&self) -> Vec<PackGroup>;
        /// Its weapons and spells to equip.
        fn equips(&self) -> Vec<EquipChoice>;
    }

    impl Lines for Menu<Line> {
        fn arts(&self) -> Vec<ArtLine> {
            let art = |l: &Line| {
                if let Line::Art(a) = l {
                    Some(a.clone())
                } else {
                    None
                }
            };
            self.items().iter().filter_map(|i| art(&i.value)).collect()
        }

        fn spells(&self) -> Vec<SpellChoice> {
            let spell = |l: &Line| {
                if let Line::Spell(c) = l {
                    Some(c.clone())
                } else {
                    None
                }
            };
            self.items()
                .iter()
                .filter_map(|i| spell(&i.value))
                .collect()
        }

        fn skills(&self) -> Vec<SkillChoice> {
            let skill = |l: &Line| {
                if let Line::Skill(c) = l {
                    Some(c.clone())
                } else {
                    None
                }
            };
            self.items()
                .iter()
                .filter_map(|i| skill(&i.value))
                .collect()
        }

        fn groups(&self) -> Vec<PackGroup> {
            let group = |l: &Line| {
                if let Line::Item(g) = l {
                    Some(g.clone())
                } else {
                    None
                }
            };
            self.items()
                .iter()
                .filter_map(|i| group(&i.value))
                .collect()
        }

        fn equips(&self) -> Vec<EquipChoice> {
            let equip = |l: &Line| {
                if let Line::Equip(c) = l {
                    Some(c.clone())
                } else {
                    None
                }
            };
            self.items()
                .iter()
                .filter_map(|i| equip(&i.value))
                .collect()
        }
    }

    /// A battle screen on a screen stack, as the game has it: what it opens
    /// over itself (the rewind screen, the Guides list, a scene) goes on
    /// top, and the battle hears what each pops with. It derefs to the
    /// battle under them; [`battle_mut`](Self::battle_mut) changes it
    /// (never a `DerefMut`, which would let a helper update the battle
    /// past the screens over it).
    pub struct Stacked(ScreenStack);

    impl Stacked {
        /// `battle`, alone on the stack.
        pub fn new(battle: super::BattleScreen) -> Self {
            Self(ScreenStack::new(Box::new(battle)))
        }

        /// One frame of `input` for the screen on top, its transition
        /// applied.
        pub fn update(&mut self, c: &mut Ctx, input: &FrameInput) {
            self.0.update(c, input);
        }

        /// One frame of `dt` seconds with `actions`.
        pub fn frame(&mut self, c: &mut Ctx, actions: &[Action], dt: f32) {
            self.update(c, &FrameInput::new(actions.to_vec(), dt, vec![]));
        }

        /// Each of `actions` in an instant frame of its own.
        pub fn press(&mut self, c: &mut Ctx, actions: &[Action]) {
            for &action in actions {
                self.frame(c, &[action], 0.0);
            }
        }

        /// The battle under the screens, to change.
        pub fn battle_mut(&mut self) -> &mut super::BattleScreen {
            self.0.find_mut().unwrap_or_else(|| panic!("no battle"))
        }

        /// The rewind screen, while it is open.
        pub fn rewind(&self) -> Option<&RewindScreen> {
            self.0.find()
        }

        /// The Guides list, while it is open.
        pub fn guides(&self) -> Option<&GuidesScreen> {
            self.0.find()
        }

        /// The shop screen, while it is open.
        pub fn shop(&self) -> Option<&super::shop::ShopScreen> {
            self.0.find()
        }

        /// Paints the stack as the game does.
        pub fn paint(&self, c: &Ctx, buf: &mut GlyphBuffer) {
            self.0.draw(c, buf);
        }
    }

    impl Deref for Stacked {
        type Target = super::BattleScreen;

        fn deref(&self) -> &super::BattleScreen {
            self.0.find().unwrap_or_else(|| panic!("no battle"))
        }
    }

    /// The transition the battle `s` answers one frame of `action` with,
    /// as it prints (`None`, `Push(rewind)`): whether it opens a screen.
    pub fn answer(s: &mut super::BattleScreen, c: &mut Ctx, action: Action) -> String {
        let input = FrameInput::new(vec![action], 0.0, vec![]);
        format!("{:?}", s.update(c, &input))
    }

    /// The console cell of the left glyph of `tile` on `s`'s map, if it is
    /// in view.
    pub fn tile_cell(s: &super::BattleScreen, c: &Ctx, tile: Pos) -> Option<(i32, i32)> {
        let cells = c.game.map_skin.tile_cells(&s.scene(c), MAP_VIEW, tile)?;
        Some((cells.x, cells.y))
    }

    /// A battle on `map` with `units` and `objective`, using the game's
    /// tables, without rewind charges.
    pub fn battle_with(
        c: &Ctx,
        map: BattleMap,
        units: Vec<Unit>,
        objective: Objective,
    ) -> BattleState {
        battle_charged(c, map, units, objective, 0)
    }

    /// [`battle_with`], with `charges` rewind charges.
    pub fn battle_charged(
        c: &Ctx,
        map: BattleMap,
        units: Vec<Unit>,
        objective: Objective,
        charges: u8,
    ) -> BattleState {
        battle_packed(c, map, units, objective, charges, BattlePack::default())
    }

    /// [`battle_charged`], with the battle pack `pack`.
    pub fn battle_packed(
        c: &Ctx,
        map: BattleMap,
        units: Vec<Unit>,
        objective: Objective,
        charges: u8,
        pack: BattlePack,
    ) -> BattleState {
        BattleState::new(BattleSetup {
            pack,
            rewind_charges: charges,
            ..setup(c, map, units, objective)
        })
        .0
    }

    /// The setup of a battle on `map` with `units` and `objective`, using
    /// the game's tables: no pack, rewind charges or triggers, seed 0,
    /// Classic.
    pub fn setup(c: &Ctx, map: BattleMap, units: Vec<Unit>, objective: Objective) -> BattleSetup {
        BattleSetup {
            map,
            terrain: Arc::new(c.game.content.terrain.rules.clone()),
            classes: Arc::new(c.game.content.classes.clone()),
            items: Arc::new(c.game.content.items.clone()),
            spells: Arc::new(c.game.content.spells.clone()),
            skills: Arc::new(c.game.content.skills.clone()),
            arts: Arc::new(c.game.content.arts.clone()),
            supports: Arc::new(c.game.content.supports.clone()),
            bonds: trpg_core::SupportBook::default(),
            pack: BattlePack::default(),
            gold: 0,
            stock: Stock::default(),
            units,
            reinforcements: vec![],
            objective,
            rewind_charges: 0,
            seed: 0,
            triggers: vec![],
            mode: trpg_core::GameMode::Classic,
            battle_notes: vec![],
            village: None,
        }
    }

    /// The Quick Battle's map and units.
    pub fn quick_units(c: &Ctx) -> (BattleMap, Vec<Unit>) {
        let quick = super::quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
        (quick.map().clone(), quick.units().to_vec())
    }

    /// A rout battle on `map` with `units`.
    pub fn battle(c: &Ctx, map: BattleMap, units: Vec<Unit>) -> BattleState {
        battle_with(c, map, units, Objective::Rout { turn_limit: None })
    }

    /// The Quick Battle with the archer (unit 3) at (8, 4), two tiles below
    /// the first brigand (unit 4), knowing Vault, after a Vault attack on
    /// it: the archer waits to move after its attack.
    pub fn vaulted(c: &Ctx) -> BattleState {
        let quick = super::quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
        let mut units = quick.units().to_vec();
        units[2].pos = Pos::new(8, 4);
        assert!(units[2].learn_skill(&SkillId::new("vault"), &c.game.content.skills));
        let rout = Objective::Rout { turn_limit: None };
        let mut s = battle_with(c, quick.map().clone(), units, rout);
        let attack = Command::Act {
            unit: UnitId(3),
            dest: Pos::new(8, 4),
            action: UnitAction::Attack {
                target: UnitId(4),
                slot: 0,
                active: Some(SkillId::new("vault")),
                art: None,
            },
        };
        if let Err(e) = s.apply(&attack) {
            panic!("{e}");
        }
        assert!(s.pending_move().is_some());
        s
    }

    /// The Quick Battle set for a fight: the lord (unit 1) at (6, 2), one
    /// step left of (7, 2), from where it can hit the raider (unit 6, at
    /// (7, 1)) above and the first brigand (unit 4, at (8, 2), with
    /// `brigand_hp` HP) to its right; the archer (unit 3) at (8, 4), two
    /// tiles below the brigand, which can't counter at that range.
    pub fn skirmish(c: &Ctx, brigand_hp: StatValue) -> BattleState {
        skirmish_charged(c, brigand_hp, 0)
    }

    /// [`skirmish`], with `charges` rewind charges.
    pub fn skirmish_charged(c: &Ctx, brigand_hp: StatValue, charges: u8) -> BattleState {
        let quick = super::quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
        let mut units = quick.units().to_vec();
        units[0].pos = Pos::new(6, 2);
        units[2].pos = Pos::new(8, 4);
        units[3].hp = brigand_hp;
        let rout = Objective::Rout { turn_limit: None };
        battle_charged(c, quick.map().clone(), units, rout, charges)
    }

    /// `unit` of `state` waiting where it stands.
    pub fn wait(state: &mut BattleState, unit: usize) {
        let u = &state.units()[unit];
        let cmd = Command::Act {
            unit: u.id,
            dest: u.pos,
            action: UnitAction::Wait,
        };
        if let Err(e) = state.apply(&cmd) {
            panic!("{e}");
        }
    }

    /// Plays `s` through the AI's phases (0502) in frames of `dt` seconds
    /// with no keys, until the player phase is back or the battle is over.
    /// Returns how many frames that took.
    ///
    /// # Panics
    ///
    /// If that takes more than 1000 frames (e.g. a level-up page waiting
    /// for Confirm).
    pub fn through_ai_phases(s: &mut super::BattleScreen, c: &mut Ctx, dt: f32) -> usize {
        for n in 0..1000 {
            if !s.ai_phase() || s.state().outcome().is_some() {
                return n;
            }
            crate::screen::Screen::update(
                s,
                c,
                &crate::screen::FrameInput::new(vec![], dt, vec![]),
            );
        }
        panic!("the AI phase never ended: {:?}", s.mode());
    }
}

#[cfg(test)]
mod act1_tests;
#[cfg(test)]
mod art_tests;
#[cfg(test)]
mod attack_tests;

#[cfg(test)]
mod heal_aim_tests;
#[cfg(test)]
mod item_tests;

#[cfg(test)]
mod magic_tests;

#[cfg(test)]
mod progress_tests;

#[cfg(test)]
mod rewind_tests;

#[cfg(test)]
mod scene_tests;
#[cfg(test)]
mod scene_voice_tests;
#[cfg(test)]
mod settings_tests;

#[cfg(test)]
mod skill_tests;

#[cfg(test)]
mod sound_tests;

#[cfg(test)]
mod threat_tests;
#[cfg(test)]
mod tip_tests;

#[cfg(test)]
mod lesson_tests;

#[cfg(test)]
mod help_bar_tests;
#[cfg(test)]
mod turn_tests;

#[cfg(test)]
mod ai_phase_tests;

#[cfg(test)]
mod trigger_tests;

#[cfg(test)]
mod notes_tests;

#[cfg(test)]
mod zoom_tests;

#[cfg(test)]
mod tests;
#[cfg(test)]
mod view_tests;

#[cfg(test)]
mod ranges_opened_tests {
    use trpg_core::Pos;

    use super::{RangesShown, ranges_opened};

    /// Ticket 0923: the ranges open when a unit is selected, stay opened
    /// while it is, open again from another unit's tile, and close.
    #[test]
    fn the_ranges_open_on_each_new_tile_and_close_with_none() {
        let (a, b) = (Pos::new(3, 5), Pos::new(4, 5));
        let opened = ranges_opened(Some(a), None, 100);
        assert_eq!(opened, Some(RangesShown::new(a, 100)));
        assert_eq!(ranges_opened(Some(a), opened, 900), opened);
        let other = ranges_opened(Some(b), opened, 900);
        assert_eq!(other, Some(RangesShown::new(b, 900)));
        assert_eq!(ranges_opened(None, other, 950), None);
    }
}
