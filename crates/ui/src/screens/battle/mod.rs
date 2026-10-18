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
//! (0307). One-time [`tips`] pop up over it the first time something new
//! happens (0406), and checklist [`lessons`] (0456) wait, the turn with
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
//! boxes and the help bar are this screen's own cells.

pub mod ai_phase;
pub mod art_list;
pub mod attack;
pub mod banner;
pub mod camera;
pub mod cursor;
pub mod doable;
pub mod event_sounds;
pub mod forecast;
pub mod glyph;
pub mod guides;
pub mod info;
pub mod items;
pub mod layout;
pub mod lessons;
pub mod magic;
pub mod map_menu;
pub mod mode;
pub mod notes;
pub mod panel;
pub mod path;
pub mod playback;
pub mod progress;
pub mod rewind;
pub mod skills;
mod sounds;
pub mod split_keys;
pub mod stage;
pub mod tips;
pub mod units;
pub mod view;
pub mod walk;

use crate::prelude::*;
use alloc::collections::VecDeque;
use core::cell::RefCell;
use core::fmt::Display;
use trpg_content::{Content, DoneWhen, MapLook, Present, TipTrigger, battle_campaign};

use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{
    BattleHistory, BattleState, CastTarget, Command, Equipped, Event, Faction, GameMode, ItemId,
    LeadGender, LeadProfile, Outcome, Phase, Pos, StatValue, TerrainRules, TileSet, Unit, UnitId,
    danger_zone, next_command, threats_at,
};

use self::ai_phase::{AiAction, PACING};
use self::banner::{Banner, BannerKind};

use self::attack::{Targeting, aimed_first, aimed_options};
use self::camera::Camera;
use self::cursor::Cursor;
use self::doable::Doable;
use self::event_sounds::CueQueue;
use self::guides::{GuidesEffect, GuidesScreen};
use self::layout::MAP_VIEW;
use self::lessons::LessonState;
use self::mode::{Effect, MenuEntry, Mode, Selection};
use self::playback::{Playback, SceneCue, TIMINGS};
use self::progress::{PROGRESS_TIMINGS, Progress};
use self::rewind::{RewindEffect, RewindScreen};
use self::tips::TipState;
use self::view::{
    BattleMenuView, BattleView, DialogView, MenuHeader, MenuPlace, MessageKind, MessageView,
    NotesView, ObjectiveView, PanelView, QuestionView,
};
use self::walk::Gait;
use super::dialogue::DialogueScreen;
use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::map_view::{
    CursorView, GlyphSkin, MapScene, MapSkin, Place, RangeKind, ThreatView, UnitView,
};
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::settings::{AnimSpeed, HELD_SPEED, MapZoom};
use crate::tips::{LessonView, TipView};
use crate::widgets::help::{SEPARATOR, key_name};
use crate::words::{Language, Words};

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
    /// Asked for, then a rewind undid the ending: the battle's own music
    /// is due again ([`BattleScreen::take_music_back`]).
    Undone,
}

/// The music pool a won battle picks its sting from (`audio.md`, 0022).
pub const VICTORY_MUSIC_POOL: &str = "victory";

/// The music cue of a lost battle, from its banner on (`audio.md`, 0023).
pub const DEFEAT_MUSIC: &str = "defeat";

/// How long a tile flashes when its terrain changes, in seconds. *Tunable.*
pub const TERRAIN_FLASH_S: f32 = 0.4;

/// A tile whose terrain just changed (a tile cast, a fire burning out): it
/// flashes for [`TERRAIN_FLASH_S`] seconds.
#[derive(Debug, Clone, PartialEq)]
pub struct TerrainFlash {
    /// The tile.
    pub pos: Pos,
    /// Seconds it has flashed.
    pub t: f32,
}

impl TerrainFlash {
    /// How strong the flash is now: `1` at the change, fading to none.
    pub fn strength(&self) -> f32 {
        (1.0 - self.t / TERRAIN_FLASH_S).clamp(0.0, 1.0)
    }
}

/// What a [`HealPopup`] tells of.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PopupKind {
    /// HP restored: `+10`, in `hp_high`.
    Heal,
    /// HP lost to a fire burning out under the unit: `-5`, in `hp_low`.
    Burn,
}

/// A `+10` shown over a unit that was healed, or a `-5` over one a fire
/// burnt, for [`Timings::heal_popup`](playback::Timings::heal_popup)
/// seconds (a burn's start once no banner is on screen).
#[derive(Debug, Clone, PartialEq)]
pub struct HealPopup {
    /// The tile it floats over.
    pub pos: Pos,
    /// HP restored or lost.
    pub amount: StatValue,
    /// Whether the HP was restored or lost.
    pub kind: PopupKind,
    /// Seconds it has been up.
    pub t: f32,
}

impl HealPopup {
    /// What it reads: `+10` for a heal, `-5` for a burn.
    pub fn text(&self) -> String {
        match self.kind {
            PopupKind::Heal => format!("+{}", self.amount),
            PopupKind::Burn => format!("-{}", self.amount),
        }
    }
}

/// The tiles units hostile to the player could attack this turn
/// ([`danger_zone`] with each unit's attack ranges); empty if it can't be
/// worked out.
pub fn danger_tiles(state: &BattleState) -> TileSet {
    let tiles = &state.map().tiles;
    danger_zone(
        state.map(),
        state.terrain(),
        state.classes(),
        state.units(),
        Faction::Player,
        |u| u.attack_ranges(state.classes(), state.items(), state.spells()),
    )
    .unwrap_or_else(|_| TileSet::new(tiles.width(), tiles.height()))
}

/// Where the enemies stand that could attack unit `unit` in their next
/// phase if it stood on `tile` ([`threats_at`] with each unit's attack
/// ranges), in the battle's order; none if it can't be worked out.
pub fn threat_enemies(state: &BattleState, unit: UnitId, tile: Pos) -> Vec<Pos> {
    let ids = threats_at(
        state.map(),
        state.terrain(),
        state.classes(),
        state.units(),
        unit,
        tile,
        |u| u.attack_ranges(state.classes(), state.items(), state.spells()),
    )
    .unwrap_or_default();
    let at = ids.into_iter().filter_map(|id| state.unit(id));
    at.map(|u| u.pos).collect()
}

/// The answer of [`threat_enemies`] for a unit on a tile, kept by the
/// screen so that it isn't worked out again every frame.
#[derive(Debug, Clone, PartialEq, Eq)]
struct KeptThreat {
    unit: UnitId,
    tile: Pos,
    enemies: Vec<Pos>,
}

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

/// Where the rewind screen was opened from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RewindFrom {
    /// The player's Rewind key, mid-battle.
    Browsing,
    /// Game Over's `Rewind`, on a lost battle: closing the screen without
    /// rewinding pops the battle screen again.
    Defeat,
}

/// The battle screen: the player browses the map with the cursor, selects
/// and moves units ([`Mode`]), opens the map menu, ends the turn; phase and
/// outcome banners show as the battle goes on, and the outcome's closes the
/// screen. Rewind while browsing in the player phase opens the
/// [`RewindScreen`]. In the Enemy and Other phases, once their banner has
/// closed, the AI's actions play one by one ([`AiAction`]), then the phase
/// ends.
#[derive(Debug, Clone)]
pub struct BattleScreen {
    state: BattleState,
    history: BattleHistory,
    camera: Camera,
    /// The viewport's size in tiles, across and down, that the camera is
    /// kept for: the map skin's, taken at the start of every frame.
    view: (i32, i32),
    cursor: Cursor,
    mode: Mode,
    /// The danger zone, while shown (recomputed after every command).
    danger: Option<TileSet>,
    /// The threat arrows last worked out (0448): the unit and the tile
    /// they were for, and where the enemies stand
    /// ([`threat_enemies`]). Kept until the scene asks about another unit
    /// or tile, and dropped whenever the battle changes.
    threats: RefCell<Option<KeptThreat>>,
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
    /// A short message and the seconds it has left.
    toast: Option<(String, f32)>,
    /// Banners and scenes waiting to be shown, in the order of their
    /// events, the first on screen (after a combat's playback).
    queue: VecDeque<Queued>,
    rewind: Option<RewindScreen>,
    /// The Guides list, while it is open (0458).
    guides: Option<GuidesScreen>,
    /// A map menu shortcut just chosen (ticket 1035): its action, handled
    /// next as if its key had been pressed.
    pressed: Option<Action>,
    /// Where the rewind screen was opened from (0822).
    rewind_from: RewindFrom,
    /// Where the outcome's music (0809) is.
    outcome_music: OutcomeMusic,
    /// One-time tips waiting to show (0406).
    tips: TipState,
    /// Checklist lessons waiting or open (0456).
    lessons: LessonState,
    /// What the units left to act could do, for the lessons (0464): worked
    /// out when one asks, started anew whenever the battle changes.
    doable: Doable,
    /// What the player did since the lessons were last ticked
    /// ([`tick_lessons`](Self::tick_lessons)).
    noticed: Vec<DoneWhen>,
    /// The id of the battle (its file's), which the game flow gives
    /// ([`in_battle`](Self::in_battle)): a lesson may start only in some
    /// battles. `None` for a battle made any other way.
    battle_id: Option<String>,
    /// Heal numbers floating over units.
    popups: Vec<HealPopup>,
    /// Tiles flashing after their terrain changed.
    flashes: Vec<TerrainFlash>,
    /// The EXP bar and level-up pages of the last command (0602), shown
    /// once its combat has played.
    progress: Option<Progress>,
    /// Sounds of events that no playback or walk plays (0424), waiting
    /// for their moment.
    cues: CueQueue,
    /// The unit whose walk was shown since the last command: its move's
    /// steps have been heard.
    walked: Option<UnitId>,
    /// Walking speed, in tiles per second, and an AI unit's while Confirm
    /// is held: the map skin's ([`MapSkin::walk_tiles_per_s`], or
    /// [`MapSkin::fast_walk_tiles_per_s`] with Fast animations, and
    /// [`MapSkin::held_walk_tiles_per_s`]), as of the frame's start.
    pace: (f32, f32),
    /// How much faster the rest of an AI unit's action plays (the camera's
    /// pan, the mark on the unit): the player's speeds for the AI's phases
    /// ([`Settings::battle_speed`]), as of the frame's start.
    ///
    /// [`Settings::battle_speed`]: crate::settings::Settings::battle_speed
    ai_speed: f32,
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
    /// The language of the last frame: what a command applied between
    /// frames names things in (a combat's fighters, a level up's skills).
    /// English until the first frame.
    language: Language,
}

/// The attack forecast `mode` shows, if any.
fn forecast_of(mode: &Mode) -> Option<&Targeting> {
    match mode {
        Mode::Targeting(t) => Some(t),
        Mode::CastTarget(t) => t.forecast(),
        _ => None,
    }
}

/// What the mode shows that the lessons watch for (0456, 0457): a unit
/// selected, the attack forecast open, an action menu or arts menu open, a
/// spell aimed at a tile, or none of these.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Marks {
    None,
    Selected,
    Forecast,
    ActionMenu,
    Arts,
    TileAimed,
}

impl Marks {
    fn of(mode: &Mode) -> Self {
        match mode {
            Mode::Selected(_) => Self::Selected,
            _ if forecast_of(mode).is_some() => Self::Forecast,
            Mode::ActionMenu { .. } => Self::ActionMenu,
            Mode::ArtsMenu { .. } => Self::Arts,
            Mode::CastTarget(t) if matches!(t.target(), CastTarget::Tile(_)) => Self::TileAimed,
            _ => Self::None,
        }
    }

    /// What it coming up shows the player did.
    fn done(self) -> Option<DoneWhen> {
        match self {
            Self::None => None,
            Self::Selected => Some(DoneWhen::UnitSelected),
            Self::Forecast => Some(DoneWhen::ForecastOpened),
            Self::ActionMenu => Some(DoneWhen::ActionMenuOpened),
            Self::Arts => Some(DoneWhen::ArtsOpened),
            Self::TileAimed => Some(DoneWhen::TileAimed),
        }
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
        let view = GlyphSkin.view_tiles(MAP_VIEW);
        Self {
            camera: Camera::centred_on(start, w, h, view),
            view,
            cursor: Cursor::new(start),
            mode: Mode::after_command(&state),
            history: BattleHistory::new(state.clone()),
            rewind: None,
            guides: None,
            pressed: None,
            rewind_from: RewindFrom::Browsing,
            outcome_music: OutcomeMusic::Unheard,
            state,
            danger: None,
            threats: RefCell::new(None),
            auto_end: false,
            show_fights: true,
            end_armed: false,
            toast: None,
            queue: VecDeque::new(),
            tips: TipState::default(),
            lessons: LessonState::default(),
            doable: Doable::default(),
            noticed: Vec::new(),
            battle_id: None,
            popups: vec![],
            flashes: vec![],
            progress: None,
            cues: CueQueue::default(),
            walked: None,
            pace: (walk::WALK_TILES_PER_S, walk::HELD_WALK_TILES_PER_S),
            ai_speed: 1.0,
            leaving: None,
            player_view: None,
            notes_t: 0.0,
            look: MapLook::default(),
            language: Language::default(),
        }
    }

    /// This screen with its map looking as `look` says.
    #[must_use]
    pub fn with_look(mut self, look: MapLook) -> Self {
        self.look = look;
        self
    }

    /// This screen as battle `id`'s (its file's id), for the lessons that
    /// start only in some battles.
    #[must_use]
    pub fn in_battle(mut self, id: &str) -> Self {
        self.battle_id = Some(id.to_owned());
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
        &self.popups
    }

    /// The tiles flashing after their terrain changed.
    pub fn flashes(&self) -> &[TerrainFlash] {
        &self.flashes
    }

    /// Whether auto-end is on.
    pub fn auto_end(&self) -> bool {
        self.auto_end
    }

    /// The danger zone, while shown.
    pub fn danger(&self) -> Option<&TileSet> {
        self.danger.as_ref()
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

    /// Whether a rewind undid an ending whose music was asked for, so the
    /// battle's own music is due again. Asked once: the next call says no.
    pub fn take_music_back(&mut self) -> bool {
        let back = self.outcome_music == OutcomeMusic::Undone;
        if back {
            self.outcome_music = OutcomeMusic::Unheard;
        }
        back
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

    /// Gives `action` to the tip or the battle notes box on screen, if any:
    /// each takes the keys until it is closed (a tip by Confirm or Cancel,
    /// once it has been drawn, 0467; the notes by Confirm). Returns whether
    /// one took it.
    fn overlay_key(&mut self, action: Action) -> bool {
        if let Some(tip) = self.shown_tip() {
            if matches!(action, Action::Confirm | Action::Cancel) && self.tips.on_screen(tip) {
                self.tips.dismiss(tip);
            }
            return true;
        }
        if self.notes_open() {
            if action == Action::Confirm {
                self.queue.pop_front();
            }
            return true;
        }
        false
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
        self.toast.as_ref().map(|(t, _)| t.as_str())
    }

    /// Shows or hides the danger zone.
    fn toggle_danger(&mut self) {
        self.danger = match self.danger {
            Some(_) => None,
            None => Some(danger_tiles(&self.state)),
        };
    }

    /// Switches the map between its two zooms and saves the choice with
    /// the other options, if the map skin can zoom and the map is what
    /// the player is looking at ([`Mode::shows_map`], no rewind open). The
    /// next frame fits the cameras to the tiles the skin now shows
    /// ([`refit`](Self::refit)): the cursor's tile stays in view. Never a
    /// `Command`: the zoom isn't game state.
    fn toggle_zoom(&self, ctx: &mut Ctx) {
        if self.rewind.is_some() || !self.mode.shows_map() || ctx.game.map_skin.zoom().is_none() {
            return;
        }
        // If saving fails it still holds for this session.
        let _ = ctx.change_settings(|s| s.game.map_zoom = s.game.map_zoom.toggled());
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
        self.toast = Some((text.to_owned(), TOAST_S));
    }

    /// A player unit's level-up in `events` fires [`TipTrigger::FirstLevelUp`].
    fn note_level_ups(&mut self, events: &[Event]) {
        let player_level_up = events.iter().any(|e| {
            matches!(e, Event::LeveledUp { unit, .. }
                if self.state.unit(*unit).is_some_and(|u| u.faction == Faction::Player))
        });
        if player_level_up {
            self.fire(TipTrigger::FirstLevelUp);
        }
    }

    /// Notes that `trigger` happened, for the tips and the lessons.
    fn fire(&mut self, trigger: TipTrigger) {
        self.tips.fire(trigger);
        self.lessons.fire(trigger);
    }

    /// Fires the triggers that show in the battle's current state (the
    /// ones that come from a command's events are fired as it applies).
    fn detect_tips(&mut self) {
        for trigger in self.triggers_now() {
            self.fire(trigger);
        }
    }

    /// The tip and lesson triggers the battle's current state shows.
    fn triggers_now(&self) -> Vec<TipTrigger> {
        let mut fired = Vec::new();
        let state = &self.state;
        if state.turn() == 1 && state.phase() == Phase::Player {
            fired.push(TipTrigger::FirstBattleStart);
        }
        if state.phase() == Phase::Enemy {
            fired.push(TipTrigger::FirstEnemyPhase);
        }
        let low = |u: &Unit| u.faction == Faction::Player && u.hp > 0 && u.hp * 2 <= u.stats.hp;
        if state.units().iter().any(low) {
            fired.push(TipTrigger::FirstLowHp);
        }
        match &self.mode {
            Mode::Selected(sel) => {
                fired.push(TipTrigger::FirstUnitSelected);
                if self.has_skill(sel.unit) {
                    fired.push(TipTrigger::FirstSkillUnit);
                }
                if doable::reaches_enemy(state, sel) {
                    fired.push(TipTrigger::FirstEnemyInRange);
                }
            }
            Mode::Targeting(_) => fired.push(TipTrigger::FirstForecast),
            Mode::Idle { .. } => {
                let on_enemy = self.hovered().is_some_and(|u| u.faction == Faction::Enemy);
                if on_enemy && self.danger.is_none() {
                    fired.push(TipTrigger::DangerZoneAvailable);
                }
                if self.special_terrain(self.cursor.pos) {
                    fired.push(TipTrigger::FirstTerrain);
                }
            }
            Mode::ActionMenu { sel, entries, .. } => {
                if entries.contains(&MenuEntry::Magic) {
                    fired.push(TipTrigger::FirstCaster);
                }
                if art_list::can_use_art(&art_list::art_lines(state, sel.unit, sel.dest())) {
                    fired.push(TipTrigger::ArtsAvailable);
                }
            }
            Mode::SpellMenu { choices, .. } => {
                let on_tile = |t: &CastTarget| matches!(t, CastTarget::Tile(_));
                if choices.iter().any(|c| c.targets.iter().any(on_tile)) {
                    fired.push(TipTrigger::TileSpellAvailable);
                }
            }
            _ => {}
        }
        fired.extend(self.lesson_triggers_now());
        fired
    }

    /// The triggers only lessons use (0457) that the battle's current
    /// state shows, besides the menus': every unit has acted, turn 2
    /// began, the threat arrows show.
    fn lesson_triggers_now(&self) -> Vec<TipTrigger> {
        let mut fired = Vec::new();
        let state = &self.state;
        if state.phase() != Phase::Player || state.outcome().is_some() {
            return fired;
        }
        if self.no_unit_left() && matches!(self.mode, Mode::Idle { .. }) {
            fired.push(TipTrigger::NoUnitLeft);
        }
        if state.turn() == 2 {
            fired.push(TipTrigger::SecondTurn);
        }
        if self.threat_view().is_some() {
            fired.push(TipTrigger::ThreatArrowsShown);
        }
        fired
    }

    /// Whether, in the player's phase of a battle still on, every unit of
    /// the player's has acted.
    fn no_unit_left(&self) -> bool {
        self.state.phase() == Phase::Player
            && self.state.outcome().is_none()
            && map_menu::ready_players(&self.state) == 0
    }

    /// Whether `unit` has a skill it can use now (a passive or an active).
    fn has_skill(&self, unit: UnitId) -> bool {
        let state = &self.state;
        state
            .unit(unit)
            .is_some_and(|u| !u.usable_skills(state.classes(), state.skills()).is_empty())
    }

    /// Whether the terrain at `pos` is not plain ground: it gives a Def,
    /// Avoid or healing bonus, or costs a movement type other than 1 to
    /// enter (more, or impassable).
    fn special_terrain(&self, pos: Pos) -> bool {
        let state = &self.state;
        let rules = state
            .map()
            .tiles
            .get(pos)
            .and_then(|&id| state.terrain().get(id));
        rules.is_some_and(not_plain)
    }

    /// The tip on screen, if any: the first queued whose moment has come
    /// (`due_tip`), unless an open lesson is on screen:
    /// tips wait for it (0456).
    pub fn shown_tip(&self) -> Option<TipTrigger> {
        let lesson_up = self.lessons.is_open() && self.lesson_moment();
        self.due_tip().filter(|_| !lesson_up)
    }

    /// The first queued tip whose moment has come, if any. Tips wait out a
    /// combat's playback, a scene (queued or playing over the battle), the
    /// battle notes and the rewind screen; the enemy-phase tip shows as
    /// that phase begins (over its banner), the others once a player phase
    /// is browsing without a banner.
    fn due_tip(&self) -> Option<TipTrigger> {
        if self.rewind.is_some()
            || self.guides.is_some()
            || self.playing()
            || self.progress.is_some()
            || self.queued_scene().is_some()
            || self.tips.under_scene()
            || self.notes_open()
        {
            return None;
        }
        self.tips.queued().find(|&t| {
            t == TipTrigger::FirstEnemyPhase
                || (self.state.phase() == Phase::Player && self.banner().is_none())
        })
    }

    /// The box of the tip on screen ([`Self::shown_tip`]), if any.
    pub fn tip_view(&self, ctx: &Ctx) -> Option<TipView> {
        if !TipState::shown_in(ctx) {
            return None;
        }
        let tip = ctx.game.content.tips.for_trigger(self.shown_tip()?)?;
        Some(TipView::new(ctx, tip))
    }

    /// Whether a lesson may be on screen now: the player's phase of a
    /// battle still on, with no combat or AI action playing, no EXP or
    /// level up, scene, banner or battle notes over the map, and no rewind
    /// screen unless the lesson needs it. Lessons wait as tips do.
    fn lesson_moment(&self) -> bool {
        (self.rewind.is_none() || self.lessons.needs_rewind())
            && self.guides.is_none()
            && !self.playing()
            && self.progress.is_none()
            && self.queued_scene().is_none()
            && !self.tips.under_scene()
            && !self.notes_open()
            && self.state.phase() == Phase::Player
            && self.state.outcome().is_none()
            && self.banner().is_none()
    }

    /// The id of the lesson on screen, if any: the one in play, once its
    /// moment has come. One not yet open also waits for a tip on screen,
    /// so it comes after the tip of the same moment.
    pub fn shown_lesson(&self) -> Option<&str> {
        let id = self.lessons.current()?;
        let clear = self.lessons.is_open() || self.due_tip().is_none();
        (self.lesson_moment() && clear).then_some(id)
    }

    /// Whether the lesson in play is open: shown once and not done yet.
    /// While it is, the turn waits for it unless an item ends it.
    pub fn lesson_open(&self) -> bool {
        self.lessons.is_open()
    }

    /// Each item of the lesson in play, ticked yet, in order.
    pub fn lesson_ticks(&self) -> &[bool] {
        self.lessons.ticked()
    }

    /// The box of the lesson on screen ([`Self::shown_lesson`]), if any,
    /// which never hides the cursor's tile.
    fn lesson_view(&self, ctx: &Ctx) -> Option<LessonView> {
        let lesson = ctx.game.content.lessons.get(self.shown_lesson()?)?;
        if !LessonState::shows(ctx, lesson.level) {
            return None;
        }
        let ticked = self.lessons.ticked();
        Some(LessonView::new(ctx, lesson, ticked, self.cursor.pos))
    }

    /// Queues the lessons the fired triggers start, then
    /// [settles](Self::settle_lessons) them.
    fn absorb_lessons(&mut self, ctx: &Ctx) {
        let turn = self.state.turn();
        self.lessons.absorb(ctx, self.battle_id.as_deref(), turn);
        self.settle_lessons();
    }

    /// In the player's phase of a battle still on, takes out of play the
    /// lesson in front while no unit left to act can do an item it has
    /// left ([`LessonState::settle`], 0464): one open comes back in a
    /// later turn, one not open yet with its trigger.
    fn settle_lessons(&mut self) {
        if self.state.phase() != Phase::Player || self.state.outcome().is_some() {
            return;
        }
        let state = &self.state;
        let doable = &mut self.doable;
        self.lessons
            .settle(state.turn(), |done| doable.can(state, done));
    }

    /// Notes that the player did `done`, for the lesson in play
    /// ([`tick_lessons`](Self::tick_lessons) ticks it).
    fn notice(&mut self, done: DoneWhen) {
        self.noticed.push(done);
    }

    /// Notes that the battle used `action`, in the player's phase.
    fn notice_pressed(&mut self, action: Action) {
        if !self.ai_phase() {
            self.notice(DoneWhen::Pressed(action));
        }
    }

    /// Notes what the player did that changed the mode from what `was`
    /// marks: selected a unit, picked where it goes, opened the forecast.
    fn notice_mode(&mut self, was: Marks) {
        let now = Marks::of(&self.mode);
        let gone = matches!(
            self.mode,
            Mode::Moving { .. }
                | Mode::ActionMenu { .. }
                | Mode::Targeting(_)
                | Mode::CastTarget(_)
        );
        if was == Marks::Selected && gone {
            self.notice(DoneWhen::UnitMoved);
        }
        if let Some(done) = now.done().filter(|_| now != was) {
            self.notice(done);
        }
    }

    /// Notes where a selected unit's path points, for the threat arrows'
    /// lesson (0457): at a tile an enemy could attack it on (the arrows
    /// show), or at one none could.
    fn notice_threat(&mut self) {
        if !matches!(self.mode, Mode::Selected(_)) || self.threat_target().is_none() {
            return;
        }
        if self.threat_view().is_some() {
            self.notice(DoneWhen::ThreatPointed);
        } else {
            self.notice(DoneWhen::SafePointed);
        }
    }

    /// Notes what the player did that player command `cmd` with `events`
    /// shows: a unit of theirs attacked (in the player's phase every combat
    /// a command fights is the player's attack), moved after its action,
    /// or they ended their phase.
    fn notice_events(&mut self, cmd: &Command, events: &[Event]) {
        match cmd {
            Command::EndPhase => self.notice(DoneWhen::TurnEnded),
            Command::MoveAfter { .. } => self.notice(DoneWhen::UnitMoved),
            _ => {}
        }
        if events
            .iter()
            .any(|e| matches!(e, Event::CombatResolved { .. }))
        {
            self.notice(DoneWhen::Attacked);
        }
    }

    /// Ticks the lesson in play's items for what was noticed since last
    /// time; one that is done closes, and the campaign keeps it.
    fn tick_lessons(&mut self, ctx: &mut Ctx) {
        for done in core::mem::take(&mut self.noticed) {
            self.lessons.tick(ctx, done);
        }
    }

    /// Closes the banner on screen. An outcome's leaves the battle.
    fn close_banner(&mut self) -> Transition {
        let Some(&banner) = self.banner() else {
            return Transition::None;
        };
        self.queue.pop_front();
        match banner.kind {
            BannerKind::Outcome(_) => Transition::Pop,
            BannerKind::Phase { .. } => Transition::None,
        }
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
        self.walked = unit;
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
        let pacing = PACING.at_speed(self.ai_speed, self.pace);
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
        screen.has_text().then(|| screen.with_battle(self.place()))
    }

    /// The battle as it stands, for a scene played over it to show
    /// (`@place battle`): the whole map in its look, with the units as the
    /// map shows them now.
    pub fn place(&self) -> Place {
        let tiles = &self.state.map().tiles;
        let size = (i32::from(tiles.width()), i32::from(tiles.height()));
        let mut scene = terrain_scene(&self.state, Pos::new(0, 0), size);
        self.add_units(&mut scene);
        Place {
            tiles: tiles.clone(),
            look: self.look.clone(),
            units: scene.units,
        }
    }

    /// With auto-end on, ends the player phase once the screen is back to
    /// browsing after the command that left no player unit ready.
    fn check_auto_end(&mut self) {
        // A lesson in play holds the turn (0456), and one about ending it
        // (0457) is the player's to do: auto-end waits for it.
        if !self.end_armed
            || !matches!(self.mode, Mode::Idle { .. })
            || !self.queue.is_empty()
            || self.progress.is_some()
            || self.lessons.in_play()
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

    /// Whether the player chose `Restart Battle` (the screen then pops,
    /// and the game flow starts the battle again).
    pub fn restart_requested(&self) -> bool {
        self.leaving == Some(Leaving::Restart)
    }

    /// Whether the player chose `Suspend` (the screen then pops, and the
    /// game flow saves the battle and goes back to the title).
    pub fn suspend_requested(&self) -> bool {
        self.leaving == Some(Leaving::Suspend)
    }

    /// The suspend save couldn't be written: the battle goes on, showing
    /// `message`.
    pub fn suspend_failed(&mut self, message: String) {
        self.leaving = None;
        self.toast = Some((message, TOAST_S));
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

    /// The rewind screen, if open.
    pub fn rewind(&self) -> Option<&RewindScreen> {
        self.rewind.as_ref()
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

    /// Scrolls the camera to keep `target` [`Camera::MARGIN`] tiles from the
    /// viewport's edges.
    pub fn follow(&mut self, target: Pos) {
        let tiles = &self.state.map().tiles;
        let (w, h) = (tiles.width(), tiles.height());
        self.camera.follow(target, w, h, self.view, Camera::MARGIN);
    }

    /// The camera for a viewport of `view` tiles: the screen's own, or, if
    /// that was kept for another size (the map skin changed), one centred on
    /// the cursor.
    fn camera_in(&self, view: (i32, i32)) -> Camera {
        if view == self.view {
            return self.camera;
        }
        let tiles = &self.state.map().tiles;
        Camera::centred_on(self.cursor.pos, tiles.width(), tiles.height(), view)
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
        let view = ctx.game.map_skin.view_tiles(MAP_VIEW);
        if view != self.view {
            self.refit(view);
        }
        let (skin, settings) = (&ctx.game.map_skin, ctx.settings());
        let walk = match settings.game.anim_speed {
            AnimSpeed::Normal => skin.walk_tiles_per_s(),
            AnimSpeed::Fast => skin.fast_walk_tiles_per_s(),
        };
        self.pace = (walk, skin.held_walk_tiles_per_s());
        self.ai_speed = settings.game.battle_speed(true);
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
        let walked = self.walked.take();
        // Refused: the battle is unchanged and the player browses again.
        let events = self.state.apply(cmd).ok();
        let playback = events.as_ref().and_then(|events| {
            let words = self.language.words();
            Playback::new(words, events, &before, self.state.fallen(), TIMINGS)
        });
        if let Some(events) = &events {
            self.history.push(cmd.clone());
            self.note_level_ups(events);
            if was_player {
                self.notice_events(cmd, events);
            }
            let in_playback = playback.is_some();
            self.queue.extend(events.iter().filter_map(|e| match e {
                Event::SceneTriggered { .. } if !in_playback => SceneCue::of(e).map(Queued::Scene),
                _ => Banner::for_event(e).map(Queued::Banner),
            }));
            self.popups.extend(events.iter().filter_map(|e| match *e {
                Event::Healed { target, amount } if amount > 0 => {
                    let pos = self.state.unit(target)?.pos;
                    Some(HealPopup {
                        pos,
                        amount,
                        kind: PopupKind::Heal,
                        t: 0.0,
                    })
                }
                Event::BurnDamage { pos, amount, .. } if amount > 0 => Some(HealPopup {
                    pos,
                    amount,
                    kind: PopupKind::Burn,
                    t: 0.0,
                }),
                _ => None,
            }));
            self.flashes.extend(events.iter().filter_map(|e| match *e {
                Event::TerrainChanged { pos, .. } => Some(TerrainFlash { pos, t: 0.0 }),
                _ => None,
            }));
            if self.danger.is_some() {
                self.danger = Some(danger_tiles(&self.state));
            }
            self.threats.take();
            self.doable = Doable::default();
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
            self.cues.extend(cues);
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

    /// Plays this frame's sounds (0424): the combat playback's and the
    /// walk's that the coming tick of `dt` seconds reaches (`held`:
    /// Confirm is down), and the queued events' that are due.
    fn play_sounds(&mut self, ctx: &mut Ctx, dt: f32, held: bool) {
        let mut due = match &self.mode {
            Mode::Combat(playback) => playback.sounds(dt, held),
            _ => Vec::new(),
        };
        if let Some((id, tiles)) = self.mode.tiles_entered(dt, held) {
            self.walked = Some(id);
            let step = self
                .state
                .unit(id)
                .and_then(|u| event_sounds::unit_step_sound(&self.state, u));
            due.extend(step.into_iter().cycle().take(tiles));
        }
        due.extend(self.cues.tick(dt));
        for cue in due {
            ctx.audio.play_sound(cue);
        }
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
    /// battle's units, except during a combat's playback, where its
    /// fighters show the HP it has reached (the attacker not dimmed yet)
    /// and the units that fell stay until they have faded; and during an
    /// AI action, before its combat, the units as they were before it.
    pub fn shown_units(&self) -> Vec<(Unit, f32)> {
        if let Mode::AiAction(a) = &self.mode {
            return a.units().into_iter().map(|u| (u, 0.0)).collect();
        }
        let Mode::Combat(pb) = &self.mode else {
            return self
                .state
                .units()
                .iter()
                .map(|u| (u.clone(), 0.0))
                .collect();
        };
        let attacker = pb.bouts().first().map(|b| b.attacker.unit);
        self.state
            .units()
            .iter()
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

    /// Opens the rewind screen on a battle that was lost (0822), clearing
    /// what the defeat left pending. Rewinding plays on from the point
    /// picked; closing the screen pops this one again.
    pub fn rewind_from_defeat(&mut self, ctx: &Ctx) {
        self.queue.clear();
        self.progress = None;
        self.mode = Mode::default();
        self.end_armed = false;
        self.rewind_from = RewindFrom::Defeat;
        let charges = self.state.rewind_charges();
        self.rewind = Some(RewindScreen::new(&self.history, charges, ctx.words()));
    }

    /// Handles one action on the open rewind screen.
    fn rewind_step(&mut self, ctx: &mut Ctx, action: Action) {
        let Some(screen) = self.rewind.as_mut() else {
            return;
        };
        let before = (screen.focused().map(|e| e.point), screen.is_confirming());
        let effect = screen.step(action);
        let after = (screen.focused().map(|e| e.point), screen.is_confirming());
        let sound = match effect {
            RewindEffect::None if before == after => None,
            // Up and down move the highlight; Confirm and Cancel open and
            // close the prompt.
            RewindEffect::None if before.1 == after.1 => Some(MenuSound::Move),
            RewindEffect::None if after.1 => Some(MenuSound::Select),
            RewindEffect::None | RewindEffect::Close => Some(MenuSound::Cancel),
            RewindEffect::Rewind(_) => Some(MenuSound::Select),
        };
        if let Some(sound) = sound {
            ctx.audio.menu(sound);
        }
        match effect {
            RewindEffect::None => {}
            RewindEffect::Close => {
                self.rewind = None;
                self.notice(DoneWhen::RewindClosed);
            }
            RewindEffect::Rewind(point) => {
                // The screen only confirms a listed point with a charge left.
                if let Ok(state) = self.history.rewind_to(point) {
                    self.state = state;
                    self.threats.take();
                    self.doable = Doable::default();
                    self.mode = Mode::after_command(&self.state);
                    self.queue.clear();
                    self.progress = None;
                    self.end_armed = false;
                    if self.outcome_music == OutcomeMusic::Heard {
                        self.outcome_music = OutcomeMusic::Undone;
                    }
                    if self.danger.is_some() {
                        self.danger = Some(danger_tiles(&self.state));
                    }
                }
                self.rewind = None;
            }
        }
    }

    /// Gives `action` to the mode ([`mode::step`], after [`Mode::route`]
    /// applies the optional split keys), plays its menu sound and carries
    /// out its effect. Returns whether the map menu's `Options` was chosen:
    /// the frame then opens the Options screen.
    fn step_mode(&mut self, ctx: &mut Ctx, action: Action) -> bool {
        // One device per frame: a key and a button pressed in the same frame
        // are both routed by the device pressed last (ADR-0036).
        let Some(action) = self.mode.route(action, &ctx.keymap, ctx.device) else {
            return false;
        };
        let before = self.mode.clone();
        let mode = core::mem::take(&mut self.mode);
        let shortcuts = map_menu::shortcuts(ctx.keyboard, &ctx.keymap);
        let (mode, effect) =
            mode::step_with(mode, action, self.cursor.pos, &self.state, &shortcuts);
        if self.lessons.holds_turn() && ends_turn(&before, &mode, &effect) {
            // An open lesson holds the turn (0456): nothing happens.
            self.mode = before;
            ctx.audio.menu(MenuSound::Denied);
            return false;
        }
        let mode = mode.at_pace(self.pace.0);
        if let Some(sound) = sounds::step_sound(action, &before, &mode, &effect) {
            ctx.audio.menu(sound);
        }
        self.mode = mode;
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
            Effect::Options => return true,
            Effect::Guides => self.guides = Some(GuidesScreen::new(&ctx.game.content)),
            Effect::Press(action) => self.pressed = Some(action),
        }
        false
    }

    /// The Guides list, if open (0458).
    pub fn guides(&self) -> Option<&GuidesScreen> {
        self.guides.as_ref()
    }

    /// Handles one action on the open Guides list.
    fn guides_step(&mut self, ctx: &mut Ctx, action: Action) {
        let Some(screen) = self.guides.as_mut() else {
            return;
        };
        match screen.step(action) {
            GuidesEffect::None => {}
            GuidesEffect::Moved => ctx.audio.menu(MenuSound::Move),
            GuidesEffect::Close => {
                ctx.audio.menu(MenuSound::Cancel);
                self.guides = None;
            }
        }
    }

    /// Gives `action` to the rewind screen if open, else to the map: opens
    /// the rewind screen, toggles the danger zone, cycles the units, moves
    /// the cursor, or steps the mode ([`step_mode`](Self::step_mode)).
    /// Notes what the player did for the lessons (0456). Returns whether
    /// the map menu's `Options` was chosen.
    fn play_key(&mut self, ctx: &mut Ctx, action: Action) -> bool {
        if self.rewind.is_some() {
            // On the rewind screen a key is the list's, not the map's.
            self.rewind_step(ctx, action);
            return false;
        }
        if self.guides.is_some() {
            // So on the Guides list.
            self.guides_step(ctx, action);
            return false;
        }
        let was = Marks::of(&self.mode);
        let mut options = false;
        match action {
            // The crank wound back (ticket 1035) opens it on the newest
            // action; its next notches step the open screen.
            Action::Rewind | Action::RewindBack if self.can_open_rewind() => {
                let charges = self.state.rewind_charges();
                self.rewind = Some(RewindScreen::new(&self.history, charges, ctx.words()));
                ctx.audio.menu(MenuSound::Select);
                self.notice(DoneWhen::RewindOpened);
            }
            Action::DangerZone if self.mode.cursor_free() => self.toggle_danger(),
            // The Playdate's system menu (ticket 1035): where the map menu
            // and its Options are a press away.
            Action::Options if matches!(self.mode, Mode::Idle { .. } | Mode::MapMenu { .. }) => {
                ctx.audio.menu(MenuSound::Select);
                options = true;
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
                    self.notice(DoneWhen::CursorMoved);
                }
            }
            _ => options = self.step_mode(ctx, action),
        }
        self.notice_mode(was);
        self.notice_pressed(action);
        options
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
        self.play_sounds(ctx, dt, held);
        let mode = core::mem::take(&mut self.mode);
        self.mode = mode.tick(dt, held, &self.state);
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
    fn overlay_help(&self, ctx: &Ctx) -> Option<String> {
        if self.shown_tip().is_some() || self.notes_open() {
            return Some(ctx.text_with("battle.help.close", &[]));
        }
        if let Some(p) = self.progress() {
            return Some(progress::help(ctx, p));
        }
        let key = match self.banner()?.kind {
            BannerKind::Outcome(_) => "battle.help.continue",
            BannerKind::Phase { .. } => "battle.help.skip",
        };
        Some(ctx.text_with(key, &[]))
    }

    /// The help line for the mode and what is under the cursor, e.g. `f
    /// select · e info · s next unit · r rewind · d back` over a ready unit while
    /// browsing. Key names come from the keymap.
    pub fn help(&self, ctx: &Ctx) -> String {
        if let Some(r) = &self.rewind {
            return rewind_help(r, ctx);
        }
        if self.guides.is_some() {
            return ctx.text_with(guides::HELP, &[]);
        }
        if let Some(line) = self.overlay_help(ctx) {
            return line;
        }
        let key = match &self.mode {
            Mode::Idle { threat } => {
                let back = if threat.is_some() {
                    "battle.hint.hide_range"
                } else {
                    "battle.hint.menu"
                };
                return self.help_idle(ctx, back);
            }
            Mode::Objective => "battle.help.objective",
            Mode::RestartPrompt => "battle.help.restart",
            Mode::SuspendPrompt => "battle.help.suspend",
            Mode::EndTurnPrompt { .. } => {
                let km = ctx.help_keys();
                let [accept, also] = split_keys::help_end_turn_accept_actions(km);
                let (accept, also) = (key_name(km, accept), key_name(km, also));
                let keys = [("accept", &accept as &dyn Display), ("also", &also)];
                return ctx.text_with("battle.help.end_turn", &keys);
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
            Mode::ActionMenu { .. }
            | Mode::ArtsMenu { .. }
            | Mode::SkillMenu { .. }
            | Mode::ItemMenu { .. }
            | Mode::MapMenu { .. }
            | Mode::UnitList { .. } => "battle.help.menu",
            Mode::SpellMenu { .. } => "battle.help.spells",
            Mode::CastTarget(t) => match t.forecast() {
                Some(forecast) => return Self::help_targeting(ctx, forecast),
                None => "battle.help.cast_target",
            },
            Mode::EquipMenu { .. } => "battle.help.equip",
            Mode::ItemTarget(_) | Mode::SkillTarget(_) => "battle.help.use_target",
            Mode::TalkTarget { .. } => "battle.help.talk_target",
            Mode::Targeting(t) => return Self::help_targeting(ctx, t),
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
        ctx.text_with(key, &[])
    }

    /// The help line while picking an attack's target: the cursor keys
    /// pick the target, or left/right swap the weapon or spell and up/down
    /// pick the target when several can make the attack (0430, 0442).
    /// Confirm reads `cast` for a spell.
    fn help_targeting(ctx: &Ctx, t: &Targeting) -> String {
        let km = ctx.help_keys();
        let plain = |key| ctx.text_with(key, &[]);
        // A hint for two keys that are not one action: `←/→ swap`.
        let pair = |key, a, b| {
            let keys = format!("{}/{}", key_name(km, a), key_name(km, b));
            ctx.text_with(key, &[("keys", &keys)])
        };
        let mut hints = if t.can_swap() {
            vec![
                pair("battle.hint.swap", Action::CursorLeft, Action::CursorRight),
                pair("battle.hint.target", Action::CursorUp, Action::CursorDown),
            ]
        } else {
            vec![plain("battle.hint.next_target")]
        };
        hints.push(plain(match t.with {
            Equipped::Weapon(_) => "battle.hint.attack",
            Equipped::Spell(_) => "battle.hint.cast",
        }));
        hints.push(plain("battle.hint.back"));
        hints.join(SEPARATOR)
    }

    /// The help line while browsing, over what is under the cursor; `back`
    /// is the hint of the Cancel key.
    fn help_idle(&self, ctx: &Ctx, back: &str) -> String {
        let (moves, info) = (Some("battle.hint.move"), Some("battle.hint.info"));
        let (next, end) = (Some("battle.hint.next_unit"), Some("battle.hint.end_turn"));
        let rewind = self.can_open_rewind().then_some("battle.hint.rewind");
        let back = Some(back);
        let hints = match self.hovered() {
            Some(u) if self.is_ready(u) && u.faction == Faction::Player => {
                let select = Some("battle.hint.select");
                [None, select, info, next, rewind, back, end]
            }
            Some(u) if u.faction != Faction::Player => {
                let range = Some("battle.hint.range");
                [moves, range, info, next, rewind, back, end]
            }
            Some(_) => [moves, None, info, next, rewind, back, end],
            None => {
                let menu = Some("battle.hint.open_menu");
                [moves, menu, None, next, rewind, back, end]
            }
        };
        let hints = hints.into_iter().flatten();
        let hints: Vec<String> = hints.map(|key| ctx.text_with(key, &[])).collect();
        hints.join(SEPARATOR)
    }

    /// The toggles' state for the message row: the Zoom map key and
    /// `zoom: 2×` (only under a map skin that can zoom), the Danger zone
    /// key and `danger zone: OFF`, then the Auto-end key and
    /// `auto-end: ON`, each key named from the active keymap.
    pub fn status(&self, ctx: &Ctx) -> String {
        let zoom = ctx.game.map_skin.zoom().map(|zoom| match zoom {
            MapZoom::X1 => "battle.status.zoom_x1",
            MapZoom::X2 => "battle.status.zoom_x2",
        });
        let danger = if self.danger.is_some() {
            "battle.status.danger_on"
        } else {
            "battle.status.danger_off"
        };
        let auto = if self.auto_end {
            "battle.status.auto_end_on"
        } else {
            "battle.status.auto_end_off"
        };
        let hints: Vec<String> = zoom
            .into_iter()
            .chain([danger, auto])
            .map(|key| ctx.text_with(key, &[]))
            .collect();
        hints.join(SEPARATOR)
    }

    /// What is on the visible map (ADR-0038), for the map skin to paint and
    /// for tests to read: the terrain, the tiles flashing after their
    /// terrain changed, the ranges the mode shows, the units as drawn, the
    /// cursor and a selected unit's path. On the rewind screen: the map as
    /// it was just before the highlighted action (as it is now with nothing
    /// listed), with no flashes, ranges, cursor or path.
    pub fn scene(&self, ctx: &Ctx) -> MapScene {
        self.scene_of_size(ctx, ctx.game.map_skin.view_tiles(MAP_VIEW))
    }

    /// [`scene`](Self::scene) for a view of `size` tiles instead of the
    /// map area's: the screen's camera if it is that size, else one
    /// centred on the cursor. For a picture of the map alone
    /// (`cargo xtask frame-png --map`).
    pub fn scene_of_size(&self, ctx: &Ctx, size: (i32, i32)) -> MapScene {
        let origin = self.camera_in(size).origin;
        // Whole milliseconds of a clock that only goes forward.
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let clock_ms = (ctx.clock_s.max(0.0) * 1000.0) as u64;
        if let Some(r) = &self.rewind {
            let shown = r.focused().map_or(&self.state, |e| &e.before);
            let mut scene = terrain_scene(shown, origin, size);
            scene.clock_ms = clock_ms;
            scene.look.clone_from(&self.look);
            for unit in shown.units() {
                scene.push_unit(UnitView::of(unit));
            }
            animate(&mut scene, None);
            return scene;
        }
        let mut scene = terrain_scene(&self.state, origin, size);
        scene.clock_ms = clock_ms;
        scene.look.clone_from(&self.look);
        for flash in &self.flashes {
            if let Some(tile) = scene.tile_mut(flash.pos) {
                tile.flashes.push(flash.strength());
            }
        }
        self.add_ranges(&mut scene);
        self.add_units(&mut scene);
        animate(&mut scene, self.mode.gait());
        scene.cursor = self.cursor_view(ctx).filter(|c| scene.contains(c.pos));
        if let Mode::Selected(sel) = &self.mode {
            scene.path.clone_from(&sel.path);
        }
        scene.threat = self.threat_view();
        scene
    }

    /// The unit of the player's and the tile the threat arrows are for
    /// (`docs/design/turn-structure.md`, *Threat arrows*), in the player's
    /// phase of a battle still running:
    ///
    /// - browsing with the cursor on one of the player's units (one that
    ///   has acted too): that unit where it stands;
    /// - a unit selected and the cursor on a tile it can stop on, or on an
    ///   enemy the path is aimed at: the unit at its path's end;
    /// - the unit walking there, and then its action menu and the lists
    ///   that opens: the same. Not while a target is picked (the forecast).
    fn threat_target(&self) -> Option<(UnitId, Pos)> {
        if self.state.phase() != Phase::Player || self.state.outcome().is_some() {
            return None;
        }
        match &self.mode {
            Mode::Idle { .. } => {
                let unit = self.hovered()?;
                (unit.faction == Faction::Player).then_some((unit.id, unit.pos))
            }
            Mode::Selected(sel) => {
                let dest = sel.dest();
                let pointed = self.cursor.pos == dest || sel.target.is_some();
                (pointed && sel.reach.is_stoppable(dest)).then_some((sel.unit, dest))
            }
            Mode::Moving { sel, .. }
            | Mode::ActionMenu { sel, .. }
            | Mode::ArtsMenu { sel, .. }
            | Mode::SpellMenu { sel, .. }
            | Mode::SkillMenu { sel, .. }
            | Mode::ItemMenu { sel, .. }
            | Mode::EquipMenu { sel, .. } => Some((sel.unit, sel.dest())),
            _ => None,
        }
    }

    /// The threat arrows to show: the tile [`threat_target`] gives and
    /// the enemies that could attack its unit there, if any can. Worked out
    /// once for a unit and a tile, then kept until either changes or the
    /// battle does.
    ///
    /// [`threat_target`]: Self::threat_target
    fn threat_view(&self) -> Option<ThreatView> {
        let (unit, tile) = self.threat_target()?;
        let mut kept = self.threats.borrow_mut();
        let enemies = match &*kept {
            Some(k) if (k.unit, k.tile) == (unit, tile) => k.enemies.clone(),
            _ => {
                let enemies = threat_enemies(&self.state, unit, tile);
                *kept = Some(KeptThreat {
                    unit,
                    tile,
                    enemies: enemies.clone(),
                });
                enemies
            }
        };
        (!enemies.is_empty()).then_some(ThreatView { tile, enemies })
    }

    /// Lays the danger zone on `scene`, if shown, then the mode's ranges
    /// over it: a selected unit's move and attack ranges, a shown threat
    /// area, where a unit may move after its attack, or the targets being
    /// chosen from. While a spell's target is picked (0410): the enemies it
    /// can hit as an attack range or the allies it can heal as a heal
    /// range, and each tile it can change marked with the terrain it would
    /// become (Nick: units and terrain must look different).
    fn add_ranges(&self, scene: &mut MapScene) {
        let all = |set: &TileSet| -> Vec<Pos> { set.iter().collect() };
        if let Some(zone) = &self.danger {
            scene.tint(all(zone), RangeKind::Danger);
        }
        match &self.mode {
            Mode::Selected(sel) => {
                scene.tint(all(&sel.moves), RangeKind::Move);
                scene.tint(all(&sel.attack), RangeKind::Attack);
            }
            Mode::Idle {
                threat: Some(threat),
            } => scene.tint(all(&threat.area), RangeKind::Attack),
            Mode::MoveAfter { tiles, .. } => scene.tint(tiles.clone(), RangeKind::Move),
            Mode::Targeting(t) => {
                let at = t.targets.iter().filter_map(|&id| self.state.unit(id));
                scene.tint(at.map(|u| u.pos), RangeKind::Attack);
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
                let kind = if attack {
                    RangeKind::Attack
                } else {
                    RangeKind::Heal
                };
                scene.tint(units, kind);
                for (pos, becomes) in t.tile_changes(&self.state) {
                    if let Some(tile) = scene.tile_mut(pos) {
                        tile.becomes = Some(becomes);
                    }
                }
            }
            Mode::SkillTarget(t) => {
                let at = t.targets().iter().filter_map(|&id| self.state.unit(id));
                scene.tint(at.map(|u| u.pos), RangeKind::Attack);
            }
            Mode::ItemTarget(t) => {
                let at = t.targets().iter().filter_map(|&id| self.state.unit(id));
                let dest = t.sel.dest();
                let tiles = at.map(|u| if u.id == t.sel.unit { dest } else { u.pos });
                scene.tint(tiles, RangeKind::Heal);
            }
            _ => {}
        }
    }

    /// Adds the units to `scene`, each where it is drawn; during a combat's
    /// playback or an AI action, as [`shown_units`](Self::shown_units) has
    /// them. While the battle notes are up, the units they are about blink:
    /// they are highlighted in the on half of the blink.
    fn add_units(&self, scene: &mut MapScene) {
        if !self.playing() {
            let blink = self.notes_shown() && notes::blink_on(self.notes_t);
            for unit in self.state.units() {
                let noted = blink && notes::is_noted(self.state.battle_notes(), unit.id);
                let view = UnitView::of(unit).at(self.drawn_pos(unit));
                scene.push_unit(view.highlighted(noted));
            }
            return;
        }
        for (unit, fade) in self.shown_units() {
            scene.push_unit(UnitView::of(&unit).fading(fade));
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
            | Mode::ArtsMenu { .. }
            | Mode::SpellMenu { .. }
            | Mode::SkillMenu { .. }
            | Mode::ItemMenu { .. }
            | Mode::EquipMenu { .. }
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
            | Mode::TalkTarget { .. }
            | Mode::AiAction(_)
            | Mode::MapMenu { .. } => {}
        }
        Some(CursorView {
            pos,
            brightness: self.cursor.brightness(),
            style: ctx.settings().game.cursor_style,
        })
    }

    /// What the screen shows this frame (ADR-0054): the map's
    /// [`scene`](Self::scene) and everything around and over it, for the
    /// skin to paint and for tests to read. On the rewind screen: the map,
    /// the rewind list, the help and the debug hint; on the Guides list
    /// (0458), the same with the list.
    pub fn view(&self, ctx: &Ctx) -> BattleView {
        let scene = self.scene(ctx);
        let help = self.help(ctx);
        let debug_hint = super::debug_hint(ctx);
        let words = ctx.words();
        let bare = || BattleView {
            lesson: None,
            scene: scene.clone(),
            menu: None,
            popups: Vec::new(),
            panel: None,
            stage: None,
            dialog: None,
            notes: None,
            noted: Vec::new(),
            banner: None,
            tip: None,
            message: None,
            status: String::new(),
            help: help.clone(),
            debug_hint: debug_hint.clone(),
            forecast: None,
            info: None,
            fight_box: None,
            progress: None,
            rewind: None,
            guides: None,
        };
        if let Some(rewind) = &self.rewind {
            return BattleView {
                lesson: self.lesson_view(ctx),
                rewind: Some(rewind.view(words)),
                ..bare()
            };
        }
        if let Some(guides) = &self.guides {
            return BattleView {
                guides: Some(guides.view(ctx)),
                ..bare()
            };
        }
        let told = (&self.state, words);
        let stage = self.stage_view(ctx);
        let fight_box = match (&self.mode, &stage) {
            (Mode::Combat(pb), None) => playback::box_view(words, pb),
            _ => None,
        };
        let info = match self.mode {
            Mode::Info { unit } => self.state.unit(unit).map(|u| info::view(told, u)),
            _ => None,
        };
        let forecast = self.forecast().and_then(|t| forecast::view(told, t));
        let faces = (&ctx.game.content.portraits, &ctx.game.lead);
        let progress = self
            .progress()
            .and_then(|p| progress::view(words, faces, p));
        let notes = self.notes_open().then(|| NotesView {
            title: ctx.text(notes::NOTES_TITLE).to_owned(),
            lines: notes::note_lines(self.state.battle_notes(), words),
        });
        let limit = map_menu::shown_limit(&self.state);
        BattleView {
            menu: self.menu_view(words),
            popups: self.popups.clone(),
            panel: forecast.is_none().then(|| self.panel_view(words)),
            stage,
            dialog: self.dialog_view(ctx),
            notes,
            noted: self.noted(),
            banner: self.banner().map(|b| b.view(words, limit)),
            tip: self.tip_view(ctx),
            lesson: self.lesson_view(ctx),
            message: self.message(),
            status: self.status(ctx),
            scene,
            help,
            debug_hint,
            forecast,
            info,
            fight_box,
            progress,
            rewind: None,
            guides: None,
        }
    }

    /// The menu open, if any: the action menu or a list it opened, beside
    /// its unit (where it will stand); the map menu beside the cursor; the
    /// unit list in the middle. Written in `words`.
    fn menu_view(&self, words: Words<'_>) -> Option<BattleMenuView> {
        let mode = self.mode.told(&self.state, words);
        let (place, menu) = match &*mode {
            Mode::ActionMenu { sel, menu, .. }
            | Mode::ArtsMenu { sel, menu, .. }
            | Mode::SpellMenu { sel, menu, .. }
            | Mode::SkillMenu { sel, menu, .. }
            | Mode::ItemMenu { sel, menu, .. }
            | Mode::EquipMenu { sel, menu, .. } => (MenuPlace::Beside(sel.dest()), menu),
            Mode::MapMenu { menu, .. } => (MenuPlace::Beside(self.cursor.pos), menu),
            Mode::UnitList { menu, .. } => (MenuPlace::Centred, menu),
            _ => return None,
        };
        let header = match &*mode {
            // The weapon the focused art would be used with, and its
            // durability.
            Mode::ArtsMenu { sel, lines, .. } => lines.get(menu.focus()).and_then(|line| {
                let weapon = (sel.unit, &Equipped::Weapon(line.slot));
                art_list::header((&self.state, words), weapon)
            }),
            // The pack's size.
            Mode::ItemMenu { .. } => Some(MenuHeader {
                name: items::pack_header(&self.state, words),
                uses: None,
            }),
            _ => None,
        };
        Some(BattleMenuView {
            menu: menu.view(),
            place,
            header,
        })
    }

    /// The side panel: the terrain and the unit under the cursor (as
    /// drawn, during a playback).
    fn panel_view(&self, words: Words<'_>) -> PanelView {
        let pos = self.cursor.pos;
        let told = (&self.state, words);
        let selected = self.mode.selection().is_some();
        if self.playing() {
            let shown = self.shown_units();
            let hovered = shown.iter().find(|(u, f)| u.pos == pos && *f < 1.0);
            panel::view(told, pos, hovered.map(|(u, _)| u), selected)
        } else {
            panel::view(told, pos, self.hovered(), selected)
        }
    }

    /// The box over the map: the objective page (the battle notes under
    /// it) or a yes/no question.
    fn dialog_view(&self, ctx: &Ctx) -> Option<DialogView> {
        let words = ctx.words();
        let question = |question: String| {
            DialogView::Question(QuestionView {
                question,
                answers: ctx.text_with("battle.yes_no", &[]),
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
                    objective: map_menu::objective_text(&self.state, words),
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

    /// Ages the heal and burn numbers, the terrain flashes and the message
    /// by `dt` seconds, dropping those whose time is up. A burn number
    /// waits while a banner is on screen: the fire burns out as the phase
    /// banner comes up, which would cover it for longer than it lasts.
    fn tick_popups(&mut self, dt: f32) {
        let banner = self.banner().is_some();
        for p in &mut self.popups {
            if !(banner && p.kind == PopupKind::Burn) {
                p.t += dt;
            }
        }
        self.popups.retain(|p| p.t < TIMINGS.heal_popup);
        for f in &mut self.flashes {
            f.t += dt;
        }
        self.flashes.retain(|f| f.t < TERRAIN_FLASH_S);
        if let Some((_, left)) = &mut self.toast {
            *left -= dt;
            if *left <= 0.0 {
                self.toast = None;
            }
        }
    }

    /// The attack forecast on screen: while an attack's target is picked,    /// The attack forecast on screen: while an attack's target is picked,
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
        let toast = || {
            Some(MessageView {
                text: self.toast()?.to_owned(),
                kind: MessageKind::Toast,
            })
        };
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

/// A view of `size` tiles from `origin` showing `state`'s terrain (and
/// the terrain one tile outside the view, for a skin to join pictures to)
/// and nothing else.
fn terrain_scene(state: &BattleState, origin: Pos, size: (i32, i32)) -> MapScene {
    let mut scene = MapScene::new(origin, size);
    let tiles = &state.map().tiles;
    scene.set_terrain(|pos| tiles.get(pos).copied());
    scene
}

/// The help line on the rewind screen `r`.
/// Whether terrain `t` is not plain ground: it gives a Def, Avoid or
/// healing bonus, or costs a movement type other than 1 to enter (more, or
/// impassable).
fn not_plain(t: &TerrainRules) -> bool {
    t.defense != 0
        || t.avoid != 0
        || t.heal_percent != 0
        || t.move_cost.iter().any(|&c| c != Some(1))
}

fn rewind_help(r: &RewindScreen, ctx: &Ctx) -> String {
    let key = if r.is_confirming() {
        "battle.rewind.help.confirm"
    } else if r.can_rewind() {
        "battle.rewind.help.pick"
    } else if r.entries().is_empty() {
        "battle.rewind.help.empty"
    } else {
        "battle.rewind.help.browse"
    };
    ctx.text_with(key, &[])
}

impl BattleScreen {
    /// [`Screen::update`]'s frame, before its tip is noted as shown.
    fn update_frame(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        // Updated, so on top again: any scene it started is closed.

        self.tips.set_under_scene(false);
        self.begin_frame(ctx, input.dt);
        // A banner already up (a battle decided at its start).
        self.play_outcome_music(ctx);
        let dt = if input.dt.is_finite() {
            input.dt.max(0.0)
        } else {
            0.0
        };
        self.tick_popups(dt);
        self.detect_tips();
        self.tips.absorb(ctx);
        self.absorb_lessons(ctx);
        let mut actions: VecDeque<Action> = input.actions.iter().copied().collect();
        while let Some(action) = actions.pop_front() {
            self.tick_lessons(ctx);
            if self.overlay_key(action) {
                continue;
            }
            // The zoom is the player's view, not a move: in any phase.
            if action == Action::ZoomMap {
                self.toggle_zoom(ctx);
                self.notice_pressed(action);
                continue;
            }
            // The AI's phase: only Confirm (hold: faster) and Cancel (skip
            // a fight) do anything else.
            if self.ai_phase() && !matches!(action, Action::Confirm | Action::Cancel) {
                continue;
            }
            if action == Action::ToggleAutoEnd {
                self.toggle_auto_end(ctx);
                self.notice_pressed(action);
                continue;
            }
            if self.progress_key(action) {
                continue;
            }
            if self.banner().is_some() {
                if action == Action::Confirm {
                    let t = self.close_banner();
                    if !matches!(t, Transition::None) {
                        return t;
                    }
                }
                continue;
            }
            if self.ai_phase() && !self.playing() {
                continue;
            }
            if self.play_key(ctx, action) {
                // The map menu's Options: over the battle.
                self.notice(DoneWhen::OptionsOpened);
                self.tick_lessons(ctx);
                return Transition::Push(Box::new(crate::screens::options::screen()));
            }
            // A map menu shortcut: its key, next.
            if let Some(pressed) = self.pressed.take() {
                actions.push_front(pressed);
            }
            if self.leaving.is_some() {
                return Transition::Pop;
            }
            if self.rewind_from == RewindFrom::Defeat && self.rewind.is_none() {
                if self.state.outcome().is_some() {
                    return Transition::Pop;
                }
                self.rewind_from = RewindFrom::Browsing;
            }
        }

        self.tick_lessons(ctx);
        self.tick_notes(dt);
        let was = Marks::of(&self.mode);
        self.animate(ctx, dt, input.is_held(Action::Confirm));
        self.notice_mode(was);
        self.notice_threat();
        self.tick_lessons(ctx);
        let waiting = self.shown_tip().is_some() || self.playing();
        if let Some(Queued::Banner(banner)) = self.queue.front_mut()
            && !waiting
        {
            banner.t += dt;
            if banner.expired() {
                // Only phase banners expire, and closing one never leaves.
                self.close_banner();
            }
        }
        self.play_outcome_music(ctx);
        if let Some(scene) = self.next_scene(ctx) {
            self.tips.set_under_scene(true);
            return Transition::Push(Box::new(scene));
        }
        // What the frame did may start a lesson auto-end must wait for
        // (every unit has acted: ending the turn), or leave one that can't
        // be finished this turn.
        for trigger in self.triggers_now() {
            self.lessons.fire(trigger);
        }
        self.absorb_lessons(ctx);
        self.check_auto_end();
        self.drive_ai(ctx);
        self.tick_lessons(ctx);
        // On screen as this frame ends, if it can still be done: open
        // from now on.
        self.settle_lessons();
        if self.shown_lesson().is_some() {
            self.lessons.mark_open();
        }
        Transition::None
    }
}

impl Screen<Shuyi> for BattleScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        let transition = self.update_frame(ctx, input);
        // Still on screen: its tip, if any, is seen (0467).
        let shown = matches!(transition, Transition::None).then(|| self.shown_tip());
        self.tips.show(shown.flatten(), ctx);
        transition
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
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

    use super::layout::MAP_VIEW;
    use crate::screen::Ctx;

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
