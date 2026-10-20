//! The title screen: `New Game` starts the game flow ([`FlowScreen`],
//! ticket 0801); `Continue` (shown only while there is a suspend save)
//! carries on the suspended battle and `Load Game` opens the save slots
//! (0802); debug builds add `Quick Battle`, a test battle through the same
//! flow; `Options` opens the [`OptionsScreen`](super::OptionsScreen) (0805); `Profile` the
//! [`ProfileScreen`](profile::ProfileScreen) (0837); `Credits` opens the
//! [`CreditsScreen`] (0808).
//!
//! **The look is a skin** (ADR-0054). This module decides what the screen
//! shows and does, and says it as plain data: a [`TitleView`]
//! ([`TitleScreen::view`]). [`glyph::paint`] draws that view: the logo
//! picture over this launch's backdrop (`docs/design/look-and-feel.md`,
//! *Title screen*; ADR-0059), with the menu under it.
//! Tests of what happened read the view; only [`glyph`]'s tests read cells.
//!
//! **The intro cinematic** (ticket 0819; `docs/design/title-screen.md`,
//! *Intro cinematic*): left alone, the menu fades out and the cinematic
//! [`TITLE_CINEMATIC`] plays in time with the title song ([`song`]); a
//! press skips to its last shot, the logo, where the menu is back.

pub mod glyph;
pub mod song;
pub mod view;

use crate::prelude::*;
pub use view::{CinematicView, TitleView};

use alloc::rc::Rc;

use trpg_content::ImageId;

use song::SongTime;
use view::FULL;

use super::{CreditsScreen, debug_hint, profile};
use crate::audio::MenuSound;
use crate::audio::splitmix64;
use crate::cinematic::{CinematicPlayer, millis};
use crate::flow::FlowScreen;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::save;
use crate::screen::{
    Ctx, FrameInput, IdleMotion, KeyPrompt, Screen, ScreenResult, Shuyi, Transition,
};
use crate::widgets::{Menu, MenuEvent, MenuItem};

/// The title screen's music cue (`audio.md`).
pub const TITLE_MUSIC: &str = "title";

/// The cinematic the title plays (`assets/cinematics/title.ron`).
pub const TITLE_CINEMATIC: &str = "title";

/// The sound of a press that skips the cinematic: the sword hit, until
/// Nick picks one (ticket 0041).
pub const TITLE_SKIP_SOUND: &str = "hit_sword";

/// Seconds the menu waits without a press before the cinematic takes over
/// (Nick's number; *tunable*).
pub const IDLE_SECS: f32 = 15.0;

/// Seconds the menu takes to fade out (*tunable*).
pub const FADE_OUT_SECS: f32 = 1.0;

/// Seconds the menu takes to fade in on the cinematic's logo (*tunable*).
pub const FADE_IN_SECS: f32 = 0.5;

/// What the title is showing once the prompt is over.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Phase {
    /// The logo and the menu.
    Menu {
        /// Seconds since the last press.
        idle: f32,
        /// Whether the menu came with the cinematic's logo (it got there
        /// by itself, or by a skip) and nothing was pressed since: it then
        /// stays until the song loops, however long that is.
        with_logo: bool,
        /// How far the menu has faded in: 0 not at all, 1 and above
        /// fully.
        shown: f32,
    },
    /// The menu fading out over the cinematic; `f` goes from 0 to 1.
    FadingOut {
        /// How far the fade is.
        f: f32,
    },
    /// The cinematic alone.
    Cinematic,
}

impl Phase {
    /// The menu, there at once, after a press.
    const MENU: Self = Self::Menu {
        idle: 0.0,
        with_logo: false,
        shown: 1.0,
    };
}

/// `part` (0 to 1) of [`FULL`].
fn of_full(part: f32) -> u8 {
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped
    let n = (part.clamp(0.0, 1.0) * f32::from(FULL)).round() as u8;
    n
}

/// How much of a fade `whole` seconds long a frame of `dt` seconds is.
fn share(dt: f32, whole: f32) -> f32 {
    dt / whole
}

/// Whether anything was pressed this frame: a key (bound or not, or held
/// long enough to repeat) or a controller button.
pub(crate) fn pressed(input: &FrameInput) -> bool {
    !input.actions.is_empty() || !input.pressed_chords().is_empty() || input.pad_pressed()
}

/// Where the title keeps the backdrop it last showed ([`Storage`] key), so
/// that the next launch shows another.
///
/// [`Storage`]: crate::storage::Storage
pub const BACKDROP_KEY: &str = "title_backdrop";

/// One line of the title menu.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Item {
    /// Carries on the suspended battle.
    Continue,
    /// Starts a new game.
    NewGame,
    /// Opens the save slots.
    LoadGame,
    /// Debug: straight into a test battle.
    QuickBattle,
    /// Opens the options.
    Options,
    /// Opens the player profiles.
    Profile,
    /// Opens the credits.
    Credits,
    /// Quits.
    Quit,
}

impl Item {
    /// The key of its label in the language files ([`Ctx::text`]).
    fn key(self) -> &'static str {
        match self {
            Self::Continue => "title.continue",
            Self::NewGame => "title.new_game",
            Self::LoadGame => "title.load_game",
            Self::QuickBattle => "title.quick_battle",
            Self::Options => "title.options",
            Self::Profile => "title.profile",
            Self::Credits => "title.credits",
            Self::Quit => "title.quit",
        }
    }
}

/// The backdrop for this launch: one of `ctx`'s title backdrops picked by
/// [`Ctx::music_seed`], never the one stored as shown last (unless it is
/// the only one; a storage that can't be read rules none out). `None` if
/// the game has no backdrops.
fn pick_backdrop(ctx: &Ctx) -> Option<ImageId> {
    let all = &ctx.game.content.title.backdrops;
    let last = ctx.storage.computer().read(BACKDROP_KEY).ok().flatten();
    let fresh = |b: &&ImageId| all.len() == 1 || Some(b.path()) != last.as_deref();
    let others: Vec<ImageId> = all.iter().filter(fresh).copied().collect();
    let len = u64::try_from(others.len()).ok().filter(|&n| n > 0)?;
    let index = usize::try_from(splitmix64(ctx.music_seed) % len).ok()?;
    others.get(index).copied()
}

/// The logo and a menu (`Continue` while a battle is suspended,
/// `New Game`, `Load Game`, `Options`, `Profile`, `Credits`, `Quit`), with
/// a help line naming the keys of the active layout.
#[derive(Debug, Clone)]
pub struct TitleScreen {
    /// The menu; each line stands for its [`Item`].
    menu: Menu<Item>,
    /// Why `Continue` couldn't, shown under the menu until the next key.
    notice: Option<String>,
    /// Whether the title music was asked for since this screen was last
    /// shown: starting the game flow (whose battles play their own music)
    /// clears it, and the next update asks again.
    music_on: bool,
    /// Whether the "press any key or button" prompt ([`Ctx::key_prompt`])
    /// is over. It shows once per launch.
    prompt_done: bool,
    /// The picture behind the logo, kept for the whole launch; `None` if
    /// the game has none.
    backdrop: Option<ImageId>,
    /// The backdrop still to be stored as the one shown last (the first
    /// update does: making the screen can't write).
    unstored: Option<ImageId>,
    /// Seconds this screen has been updated for: it places the sparks.
    time: f32,
    /// The cinematic, made each time the title is shown
    /// ([`refresh`](Self::refresh)): it is painted with the colours, map
    /// look and lead of that moment. `None` before that, or if the game
    /// has no title cinematic.
    player: Option<Rc<CinematicPlayer>>,
    /// The cinematic's time.
    song: SongTime,
    /// What is showing.
    phase: Phase,
}

impl TitleScreen {
    /// The title screen with its menu built for the saves in `ctx`'s
    /// storage ([`refresh`](Self::refresh)), `New Game` focused unless a
    /// battle is suspended, labelled in `ctx`'s language.
    pub fn new(ctx: &Ctx) -> Self {
        Self::with_debug(ctx, false)
    }

    /// The title screen with a debug `Quick Battle` item after `Load
    /// Game`, which plays a test battle through the game flow.
    pub fn with_quick_battle(ctx: &Ctx) -> Self {
        Self::with_debug(ctx, true)
    }

    fn with_debug(ctx: &Ctx, quick_battle: bool) -> Self {
        let mut title = Self {
            menu: Self::build(ctx, quick_battle, false, false),
            notice: None,
            music_on: false,
            prompt_done: false,
            backdrop: pick_backdrop(ctx),
            unstored: pick_backdrop(ctx),
            time: 0.0,
            player: None,
            song: SongTime::default(),
            phase: Phase::MENU,
        };
        title.refresh(ctx);
        title
    }

    /// The menu: `Continue` first if a battle is `suspended`; `Load Game`
    /// disabled unless a slot is `saved`. Nothing to back out of on the
    /// title screen.
    fn build(ctx: &Ctx, quick_battle: bool, suspended: bool, saved: bool) -> Menu<Item> {
        let mut items = vec![Item::NewGame, Item::LoadGame];
        if suspended {
            items.insert(0, Item::Continue);
        }
        if quick_battle {
            items.push(Item::QuickBattle);
        }
        items.push(Item::Options);
        items.push(Item::Profile);
        items.push(Item::Credits);
        items.push(Item::Quit);
        let entries = items.into_iter().map(|item| {
            let enabled = item != Item::LoadGame || saved;
            MenuItem::new(ctx.text(item.key()))
                .with_value(item)
                .with_enabled(enabled)
        });
        Menu::new(entries.collect()).without_cancel()
    }

    /// What the menu's lines are, in order.
    fn items(&self) -> Vec<Item> {
        self.menu.items().iter().map(|i| i.value).collect()
    }

    /// Builds the menu for the saves in `ctx`'s storage, in `ctx`'s
    /// language. If its lines come out different (a battle was suspended
    /// or continued, a first save made), the focus goes to its first item;
    /// otherwise it stays.
    ///
    /// Makes the cinematic again too, as `ctx` paints it now, with the
    /// lead of the most recent save ([`save::latest_lead`]).
    pub fn refresh(&mut self, ctx: &Ctx) {
        let lead = save::latest_lead(&ctx.storage);
        let player = CinematicPlayer::new(ctx, TITLE_CINEMATIC, lead);
        self.player = player.map(|p| Rc::new(p.with_backdrop(self.backdrop)));
        let storage = &ctx.storage;
        let suspended = save::has_suspend(storage);
        let saved = save::any_slot(storage);
        let quick_battle = self.items().contains(&Item::QuickBattle);
        let mut menu = Self::build(ctx, quick_battle, suspended, saved);
        let lines = |m: &Menu<Item>| {
            let lines = m.items().iter().map(|i| (i.value, i.enabled));
            lines.collect::<Vec<_>>()
        };
        if lines(&menu) == lines(&self.menu) {
            menu = menu.focused(self.menu.focus());
        }
        self.menu = menu;
    }

    /// The notice under the menu, if any.
    pub fn notice(&self) -> Option<&str> {
        self.notice.as_deref()
    }

    /// Opens the game flow over the title; the music changes under it, so
    /// the title asks for its own again when it is back.
    fn open(&mut self, screen: Box<dyn Screen<Shuyi>>) -> Transition {
        self.music_on = false;
        Transition::Push(screen)
    }

    /// Whether the title is still showing "press any key or button"
    /// instead of its menu.
    fn waiting(&self, ctx: &Ctx) -> bool {
        ctx.key_prompt != KeyPrompt::Off && !self.prompt_done
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the backdrop and the sparks (the logo is always
    /// shown), then either the "press any key" prompt or the menu with
    /// its notice and help line, and the debug hint.
    pub fn view(&self, ctx: &Ctx) -> TitleView {
        let waiting = self.waiting(ctx);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // clamped
        let sparks_ms = (f64::from(self.time) * 1000.0).clamp(0.0, f64::from(u32::MAX)) as u32;
        let cinematic = |brightness: u8| {
            let player = Rc::clone(self.player.as_ref()?);
            let t_ms = player.timeline().wrapped_ms(millis(self.song.t()));
            Some(CinematicView {
                player,
                t_ms,
                brightness,
            })
        };
        // While the prompt waits the phase is the menu's, fully shown.
        let (cinematic, menu_opacity) = match self.phase {
            Phase::Menu { shown, .. } => (None, of_full(shown)),
            Phase::FadingOut { f } => (cinematic(of_full(f)), FULL - of_full(f)),
            Phase::Cinematic => (cinematic(FULL), 0),
        };
        // The cinematic alone: no menu, notice or help line.
        let menu = !waiting && self.phase != Phase::Cinematic;
        TitleView {
            cinematic,
            menu_opacity,
            backdrop: self.backdrop,
            sparks: (ctx.idle_motion == IdleMotion::On).then_some(sparks_ms),
            prompt: waiting.then(|| ctx.text("title.press_any_key").to_owned()),
            menu: menu.then(|| self.menu.view()),
            notice: self.notice.clone().filter(|_| menu),
            help: menu.then(|| Self::help(ctx)),
            debug_hint: debug_hint(ctx),
        }
    }

    /// Seconds into the cinematic: the title song's time while it sounds
    /// ([`song`]).
    pub fn cinematic_time(&self) -> f32 {
        self.song.t()
    }

    /// How far into the title song the music is, if that is what sounds.
    fn song_position(ctx: &Ctx) -> Option<f32> {
        let clock = ctx.music_clock.as_ref()?;
        (clock.cue == TITLE_MUSIC).then_some(clock.position)
    }

    /// Asks for the title song from its start, and starts the title over
    /// with it: the menu, the cinematic's time at 0.
    fn start_song(&mut self, ctx: &mut Ctx) {
        // The music state ignores a request for the track already on.
        ctx.audio.play_music(TITLE_MUSIC);
        self.music_on = true;
        self.song = SongTime::default();
        self.phase = Phase::MENU;
    }

    /// Lets a frame of `dt` seconds pass for the cinematic: moves its time
    /// and the phase. Returns whether the frame's presses are the menu's
    /// to act on (not while the cinematic plays alone: the press that
    /// skips it does nothing else).
    fn play(&mut self, ctx: &mut Ctx, input: &FrameInput) -> bool {
        let Some(player) = self.player.clone() else {
            // No cinematic: the menu, always.
            self.phase = Phase::MENU;
            return true;
        };
        let timeline = player.timeline();
        let dt = if input.dt().is_finite() {
            input.dt().max(0.0)
        } else {
            0.0
        };
        #[allow(clippy::cast_precision_loss)] // far below 2^24 ms
        let secs = |ms: u32| ms as f32 / 1000.0;
        let length = secs(timeline.length_ms());
        if self.song.advance(Self::song_position(ctx), dt, length) {
            // The silence after a skip is over: the song again, from 0.
            ctx.audio.play_music(TITLE_MUSIC);
        }
        let on_logo = timeline.in_last_shot(millis(self.song.t()));
        let pressed = pressed(input);
        match &mut self.phase {
            Phase::Cinematic if pressed => {
                ctx.audio.play_sound(TITLE_SKIP_SOUND);
                ctx.audio.stop_music();
                self.song.skip_to(secs(timeline.last_start_ms()));
                self.phase = Phase::Menu {
                    idle: 0.0,
                    with_logo: true,
                    shown: 1.0,
                };
                return false;
            }
            Phase::Cinematic => {
                if on_logo {
                    // The sparks go on from where the cinematic's were.
                    self.time = self.song.t();
                    self.phase = Phase::Menu {
                        idle: 0.0,
                        with_logo: true,
                        shown: 0.0,
                    };
                }
                return false;
            }
            _ if pressed => self.phase = Phase::MENU,
            Phase::FadingOut { f } => {
                *f += share(dt, FADE_OUT_SECS);
                if *f >= 1.0 {
                    self.phase = Phase::Cinematic;
                }
            }
            Phase::Menu {
                idle,
                with_logo,
                shown,
            } => {
                *idle += dt;
                *shown += share(dt, FADE_IN_SECS);
                // The logo shot always has the menu on it.
                if (*with_logo || *idle >= IDLE_SECS) && !on_logo {
                    self.phase = Phase::FadingOut { f: 0.0 };
                }
            }
        }
        true
    }

    /// The bottom help line: the cursor keys `move`, the Confirm key
    /// `select`, the Cancel key `back`, named from the active keymap.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("title.help", &[])
    }
}

impl Screen<Shuyi> for TitleScreen {
    fn name(&self) -> &'static str {
        "title" // check-text: not player text (the screen's name)
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if let Some(backdrop) = self.unstored.take() {
            // Not being able to remember it must not stop the title.
            let _ = ctx
                .storage
                .computer_mut()
                .write(BACKDROP_KEY, backdrop.path());
        }
        if input.dt().is_finite() {
            self.time += input.dt().max(0.0);
        }
        if self.waiting(ctx) {
            // The key or button that ends the wait does nothing else (a
            // key with no layout chosen yet opens the layout picker over
            // the title first: `Game::step`).
            if ctx.key_prompt == KeyPrompt::Pressed {
                self.prompt_done = true;
                self.start_song(ctx);
            }
            return Transition::None;
        }
        if !self.music_on {
            self.start_song(ctx);
        }
        if !self.play(ctx, input) {
            return Transition::None;
        }
        for &action in &input.actions {
            self.notice = None;
            let chosen = match self.menu.handle_with_sound(action, &mut ctx.audio) {
                Some(MenuEvent::Chosen(item)) => Some(item),
                // The Playdate's system menu (ticket 1035): as the menu's
                // own Options.
                _ if action == Action::Options && self.items().contains(&Item::Options) => {
                    ctx.audio.menu(MenuSound::Select);
                    Some(Item::Options)
                }
                Some(MenuEvent::Cancelled) | None => None,
            };
            match chosen {
                Some(Item::Continue) => match FlowScreen::resume(ctx) {
                    Ok(flow) => return self.open(Box::new(flow)),
                    // The save stays; the title says why it can't go on.
                    Err(e) => {
                        self.notice = Some(e.text(ctx));
                        ctx.audio.menu(MenuSound::Denied);
                    }
                },
                Some(Item::NewGame) => return self.open(Box::new(FlowScreen::new_game(ctx))),
                Some(Item::LoadGame) => return self.open(Box::new(FlowScreen::load_game(ctx))),
                // The test data always builds (tested); should it ever
                // not, the item does nothing.
                Some(Item::QuickBattle) => {
                    if let Some(flow) = FlowScreen::quick_battle(ctx) {
                        return self.open(Box::new(flow));
                    }
                }
                // Not `open`: the options, the profile and the credits
                // leave the music alone (they play the title's).
                Some(Item::Options) => {
                    return Transition::Push(Box::new(super::options::screen()));
                }
                Some(Item::Profile) => {
                    return Transition::Push(Box::new(profile::screen(ctx)));
                }
                Some(Item::Credits) => return Transition::Push(Box::new(CreditsScreen::new(ctx))),
                Some(Item::Quit) => return Transition::Quit,
                _ => {}
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }

    /// Back on top: the screen that was over the title may have changed
    /// the saves (a game played, a profile picked), the options (the
    /// colours the cinematic is in) or the lead the cinematic shows, so it
    /// is built again ([`refresh`](Self::refresh)).
    fn uncovered(&mut self, ctx: &mut Ctx, _result: Option<ScreenResult>) -> Transition {
        self.refresh(ctx);
        Transition::None
    }
}

#[cfg(test)]
mod tests;
