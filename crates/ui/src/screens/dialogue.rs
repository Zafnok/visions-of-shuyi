//! The dialogue screen (ticket 0704, ADR-0018): plays a [`DialoguePlayer`]
//! with two portraits, name plates and a typewriter text box. Full-screen,
//! or as an overlay that leaves the battle map visible behind it.
//!
//! What it shows is a [`view::DialogueView`] (ADR-0054); [`glyph`] paints
//! it.
//!
//! The dialogue log (ticket 0729) lists the lines seen so far over the
//! whole screen: [`history`] keeps them, [`log`] is what it shows and
//! [`log::glyph`] paints it.

pub mod glyph;
pub mod history;
pub mod log;
pub mod view;

use crate::prelude::*;
use trpg_content::{MusicLine, Names, Present, Scene};
use trpg_core::{LEAD_ID, LeadProfile};

use crate::audio::MenuSound;
use crate::dialogue::{DialoguePlayer, Portrait};
use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, Button, Chord, Keymap};
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::widgets::help::key_name;
use crate::widgets::menu::{Menu, MenuEvent, MenuItem};
use crate::widgets::word_wrap;
use history::History;
use log::LogView;
use view::{DialogueView, SkipView, TextView, WaitView};

/// Characters per text line: the box less a 3-cell margin each side.
pub const TEXT_W: usize = trpg_content::dialogue::BOX_WIDTH;
/// Text lines per box; longer text pages.
pub const TEXT_LINES: usize = trpg_content::dialogue::BOX_LINES;
/// Reveal speed multiplier while Confirm is held.
pub const FAST_FORWARD: f32 = 6.0;
/// Seconds Confirm must be held on a page before it reveals faster, so the
/// tap that turned the page doesn't speed the next one up.
pub const FAST_FORWARD_DELAY: f32 = 0.3;
/// The marker that says "press to go on" is lit for this long, then out
/// for as long, while a page waits.
const BLINK_S: f32 = 0.5;

/// Plays a scene: Confirm reveals the text at once, or moves on when it's
/// all shown; holding Confirm reveals faster, after a short delay on each
/// page. The End turn key does
/// everything Confirm does. Cancel asks whether to skip
/// the scene. At a reply choice, the cursor keys move through the lead's
/// replies and Confirm picks one; skipping stops at each choice. Pops when
/// the scene ends or is skipped. A `@music` line changes the music when the
/// scene reaches it; skipping asks only for the last one passed.
///
/// The Open the log key opens the dialogue log (`docs/design/look-and-feel.md`,
/// *Dialogue log*): the cursor keys move through the lines seen, Confirm
/// goes to the highlighted one (back, or forward again) and Cancel closes
/// it. If the log's key is also a key the scene is using (Confirm, End
/// turn, Cancel; the cursor keys at a choice), the scene's use wins.
#[derive(Debug, Clone)]
pub struct DialogueScreen {
    player: DialoguePlayer,
    overlay: bool,
    /// The current text box, word-wrapped.
    lines: Vec<String>,
    /// Which page of `lines` is shown ([`TEXT_LINES`] lines each).
    page: usize,
    /// Characters of the page revealed so far (fractional between frames).
    shown: f32,
    /// Seconds an advance key has been held on this page, for the
    /// fast-forward ([`FAST_FORWARD_DELAY`]).
    held_s: f32,
    /// Seconds since the page was fully revealed, for the blinking `▼`.
    waiting: f32,
    /// Whether the "Skip scene?" question is open.
    asking_skip: bool,
    /// The lead's replies, while a choice is open.
    menu: Option<Menu>,
    /// Every line seen, for the log.
    history: History,
    /// The log's cursor (a row of [`log::row_count`]), while it is open.
    log: Option<usize>,
    /// The music the scene set last (`None` before its first `@music`).
    music: Option<MusicLine>,
    /// Music to play: the music of a line the log jumped to.
    jumped_music: Option<MusicLine>,
}

impl DialogueScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "dialogue";

    /// `scene` full-screen, as it plays for those `present` (ADR-0055),
    /// with `lead`'s name and pronouns and the names from `names`.
    pub fn new(scene: &Scene, lead: LeadProfile, names: Names, present: &Present) -> Self {
        let mut screen = Self {
            player: DialoguePlayer::new(scene, lead, names, present),
            overlay: false,
            lines: Vec::new(),
            page: 0,
            shown: 0.0,
            held_s: 0.0,
            waiting: 0.0,
            asking_skip: false,
            menu: None,
            history: History::new(),
            log: None,
            music: None,
            jumped_music: None,
        };
        screen.start_box();
        screen
    }

    /// [`new`](Self::new) in the player's language ([`Ctx::words`]): the
    /// lines, replies and captions its pack translates, with the names of
    /// the story as the pack has them.
    pub fn told(ctx: &Ctx, scene: &Scene, lead: LeadProfile, present: &Present) -> Self {
        let words = ctx.words();
        let names = words.names(&ctx.content.names);
        Self::new(&words.scene(scene, lead.gender), lead, names, present)
    }

    /// [`told`](Self::told), drawn over the screen below (the battle map).
    pub fn told_overlay(ctx: &Ctx, scene: &Scene, lead: LeadProfile, present: &Present) -> Self {
        Self {
            overlay: true,
            ..Self::told(ctx, scene, lead, present)
        }
    }

    /// `scene` drawn over the screen below (the battle map).
    pub fn overlay(scene: &Scene, lead: LeadProfile, names: Names, present: &Present) -> Self {
        Self {
            overlay: true,
            ..Self::new(scene, lead, names, present)
        }
    }

    /// Whether the scene has anything to say for those present. One that
    /// hasn't isn't worth showing: its callers go on to what follows it.
    pub fn has_text(&self) -> bool {
        self.player.scene().has_text()
    }

    /// The scene being played.
    pub fn player(&self) -> &DialoguePlayer {
        &self.player
    }

    /// The lines of the page on screen, in full.
    pub fn page_lines(&self) -> &[String] {
        let start = (self.page * TEXT_LINES).min(self.lines.len());
        let end = (start + TEXT_LINES).min(self.lines.len());
        &self.lines[start..end]
    }

    /// Whether the whole page is revealed.
    pub fn is_revealed(&self) -> bool {
        self.shown >= page_len(self.page_lines())
    }

    /// Whether the "Skip scene?" question is open.
    pub fn is_asking_skip(&self) -> bool {
        self.asking_skip
    }

    /// The replies menu, while a choice is open.
    pub fn menu(&self) -> Option<&Menu> {
        self.menu.as_ref()
    }

    /// The lines seen so far, for the log.
    pub fn history(&self) -> &History {
        &self.history
    }

    /// Whether the dialogue log is open.
    pub fn is_log_open(&self) -> bool {
        self.log.is_some()
    }

    /// What the log shows, while it is open ([`log::glyph::paint`] paints
    /// it).
    pub fn log_view(&self, ctx: &Ctx) -> Option<LogView> {
        self.log.map(|focus| log::view(ctx, &self.history, focus))
    }

    /// The music playing at the line now on screen: the scene's last, or
    /// one it has just reached.
    fn music_here(&self) -> Option<MusicLine> {
        self.player.pending_music().or(self.music.as_ref()).cloned()
    }

    /// A new text box (or a choice) is up: the log keeps a text box, then
    /// [`show_box`](Self::show_box).
    fn start_box(&mut self) {
        let view = self.player.current();
        if view.text.is_some() && view.choices.is_none() {
            self.history.reached(&self.player, self.music_here());
        }
        self.show_box();
    }

    /// Wraps the player's current text box and starts on its first page.
    /// At a choice, the line being answered stays up, fully shown (its last
    /// page), under the replies menu.
    fn show_box(&mut self) {
        let view = self.player.current();
        let lines = word_wrap(view.text.as_deref().unwrap_or(""), TEXT_W);
        let menu = view.choices.map(|choices| {
            Menu::new(choices.into_iter().map(MenuItem::new).collect()).without_cancel()
        });
        self.lines = lines;
        self.menu = menu;
        self.page = 0;
        self.start_page();
        if self.menu.is_some() {
            self.page = self.lines.len().saturating_sub(1) / TEXT_LINES;
            self.shown = page_len(self.page_lines());
        }
    }

    fn start_page(&mut self) {
        self.shown = 0.0;
        self.held_s = 0.0;
        self.waiting = 0.0;
    }

    /// Confirm with everything shown: the next page, else the next box.
    /// Returns `true` once the scene is over.
    fn next(&mut self) -> bool {
        if (self.page + 1) * TEXT_LINES < self.lines.len() {
            self.page += 1;
            self.start_page();
        } else {
            self.player.advance();
            self.start_box();
        }
        self.player.is_finished()
    }
}

/// The actions that advance the dialogue: Confirm, and End turn, so either
/// key reads through a scene.
const ADVANCE_KEYS: [Action; 2] = [Action::Confirm, Action::EndTurn];

/// `action`, with End Turn treated as Confirm.
fn advance_key(action: Action) -> Action {
    if ADVANCE_KEYS.contains(&action) {
        Action::Confirm
    } else {
        action
    }
}

/// Characters on a page.
fn page_len(lines: &[String]) -> f32 {
    let n: usize = lines.iter().map(|l| l.chars().count()).sum();
    // Pages are at most 3 × 92 characters, so this is exact.
    #[expect(clippy::cast_precision_loss, reason = "at most a few hundred")]
    let n = n as f32;
    n
}

impl Screen for DialogueScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        let transition = self.read(ctx, input);
        // The music the scene reached this frame (or, the first frame,
        // before its first text box), or that of a line the log jumped to.
        let music = self
            .jumped_music
            .take()
            .or_else(|| self.player.take_music());
        match &music {
            Some(MusicLine::Cue(cue)) => ctx.audio.play_music(cue),
            Some(MusicLine::Stop) => ctx.audio.stop_music(),
            None => {}
        }
        if music.is_some() {
            self.music = music;
        }
        transition
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn is_overlay(&self) -> bool {
        self.overlay
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

impl DialogueScreen {
    /// One frame of reading: the keys, then the typewriter.
    fn read(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if self.player.is_finished() {
            return Transition::Pop;
        }
        for &action in &input.actions {
            let action = advance_key(action);
            if self.log.is_some() {
                if self.read_log(ctx, action) {
                    // The rest of this frame's keys are for the scene
                    // jumped to; they wait.
                    break;
                }
                continue;
            }
            if action == Action::DialogueLog {
                if !self.asking_skip && !self.history.is_empty() && self.log_key_free(ctx, input) {
                    ctx.audio.menu(MenuSound::Select);
                    self.log = Some(self.history.at());
                }
                continue;
            }
            if let Some(menu) = self.menu.as_mut() {
                if let Some(MenuEvent::Chosen(i)) = menu.handle_with_sound(action, &mut ctx.audio) {
                    let music = self.music_here();
                    self.history.replied(&self.player, i, music);
                    self.player.choose(i);
                    self.start_box();
                    if self.player.is_finished() {
                        return Transition::Pop;
                    }
                }
                continue;
            }
            // Only the skip prompt sounds; reading on plays nothing.
            match (self.asking_skip, action) {
                (true, Action::Confirm) => {
                    ctx.audio.menu(MenuSound::Select);
                    self.asking_skip = false;
                    self.player.skip_to_choice();
                    if self.player.is_finished() {
                        return Transition::Pop;
                    }
                    self.start_box();
                    // The rest of this frame's keys can't pick a reply.
                    break;
                }
                (true, Action::Cancel) => {
                    ctx.audio.menu(MenuSound::Cancel);
                    self.asking_skip = false;
                }
                // Opening the skip prompt sounds like opening a menu.
                (false, Action::Cancel) => {
                    ctx.audio.menu(MenuSound::Select);
                    self.asking_skip = true;
                }
                (false, Action::Confirm) if !self.is_revealed() => {
                    self.shown = page_len(self.page_lines());
                }
                (false, Action::Confirm) if self.next() => return Transition::Pop,
                // A choice just opened: the rest of this frame's keys
                // can't pick a reply.
                (false, Action::Confirm) if self.menu.is_some() => break,
                _ => {}
            }
        }
        if self.log.is_some() {
            // The scene waits while the log is open.
            self.held_s = 0.0;
            return Transition::None;
        }
        if ADVANCE_KEYS.iter().any(|&a| input.is_held(a)) {
            self.held_s += input.dt;
        } else {
            self.held_s = 0.0;
        }
        if !self.asking_skip {
            if self.is_revealed() {
                self.waiting += input.dt;
            } else {
                // The part of this frame past the delay goes faster.
                let fast_dt = (self.held_s - FAST_FORWARD_DELAY).clamp(0.0, input.dt);
                let dt = input.dt + fast_dt * (FAST_FORWARD - 1.0);
                let len = page_len(self.page_lines());
                self.shown = match ctx.settings().text_speed.chars_per_s() {
                    Some(speed) => (self.shown + dt * speed).min(len),
                    // Instant: the whole page at once.
                    None => len,
                };
            }
        }
        Transition::None
    }

    /// One key in the open log: the cursor keys move, Confirm goes to the
    /// line, Cancel closes. Returns whether the log jumped to a line.
    fn read_log(&mut self, ctx: &mut Ctx, action: Action) -> bool {
        let Some(focus) = self.log else {
            return false;
        };
        let rows = log::row_count(&self.history);
        match action {
            Action::CursorUp | Action::CursorDown => {
                let to = if action == Action::CursorUp {
                    focus.saturating_sub(1)
                } else {
                    (focus + 1).min(rows.saturating_sub(1))
                };
                if to != focus {
                    ctx.audio.menu(MenuSound::Move);
                    self.log = Some(to);
                }
                false
            }
            Action::Confirm => {
                ctx.audio.menu(MenuSound::Select);
                self.log = None;
                let line = if focus >= self.history.seen().len() {
                    self.history.latest()
                } else {
                    focus
                };
                self.jump(line);
                true
            }
            Action::Cancel => {
                ctx.audio.menu(MenuSound::Cancel);
                self.log = None;
                false
            }
            _ => false,
        }
    }

    /// Shows line `i` of the log as it was: the scene, its portraits and
    /// caption, its music; the text types out from the start.
    fn jump(&mut self, i: usize) {
        let Some(seen) = self.history.jump(i) else {
            return;
        };
        let mut player = seen.player;
        // The music it had reached then is in `seen.music`.
        player.take_music();
        self.player = player;
        if seen.music.is_some() && seen.music != self.music {
            self.jumped_music = seen.music;
        }
        self.asking_skip = false;
        self.show_box();
    }

    /// The actions the scene uses now: what the log's key gives way to.
    fn scene_actions(&self) -> &'static [Action] {
        if self.menu.is_some() {
            &[
                Action::CursorUp,
                Action::CursorDown,
                Action::Confirm,
                Action::EndTurn,
                Action::Cancel,
            ]
        } else {
            &[Action::Confirm, Action::EndTurn, Action::Cancel]
        }
    }

    /// Whether this frame's Open the log comes from a key or button the
    /// scene isn't using now ([`scene_actions`](Self::scene_actions)): the
    /// log's key may share one with another group's action
    /// (`docs/design/controls.md`, *Dialogue log*). Input made of actions
    /// alone (no keys or buttons) counts as free.
    fn log_key_free(&self, ctx: &Ctx, input: &FrameInput) -> bool {
        let (chords, buttons) = (input.pressed_chords(), input.pressed_buttons());
        if chords.is_empty() && buttons.is_empty() {
            return true;
        }
        let used = self.scene_actions();
        let free = |actions: &[Action]| {
            actions.contains(&Action::DialogueLog) && !actions.iter().any(|a| used.contains(a))
        };
        let keymap: &Keymap = &ctx.keymap;
        let by_key = |&c: &Chord| free(keymap.actions(c));
        let by_button = |&b: &Button| free(keymap.pad_actions(b));
        chords.iter().any(by_key) || buttons.iter().any(by_button)
    }

    /// What the screen shows this frame.
    pub fn view(&self, ctx: &Ctx) -> DialogueView {
        let current = self.player.current();
        let mut view = view::scene(
            ctx,
            self.player.lead(),
            &current,
            self.text_view(ctx, current.narration),
            self.overlay,
        );
        view.log = self.log_view(ctx);
        view
    }

    /// What the text box shows: the page as revealed so far, or the skip
    /// question; the replies while a choice is open; what shows while the
    /// page waits for a key. `narration`: whether the text is narration.
    fn text_view(&self, ctx: &Ctx, narration: bool) -> TextView {
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "shown is between 0 and the page length"
        )]
        let shown = self.shown as usize;
        let skip = self.asking_skip.then(|| SkipView {
            question: ctx.text("dialogue.skip_question").to_owned(),
            answers: ctx.text_with("dialogue.skip_answers", &[]),
        });
        let waits = !self.asking_skip && self.menu.is_none() && self.is_revealed();
        TextView {
            narration,
            lines: self.page_lines().to_vec(),
            shown,
            choices: self.menu.as_ref().map(Menu::view),
            skip,
            waiting: waits.then(|| self.wait_view(ctx)),
        }
    }

    /// What shows while the page waits: the key that goes on, the log's
    /// hint if its action has a key, and the blinking marker.
    fn wait_view(&self, ctx: &Ctx) -> WaitView {
        let km = ctx.help_keys();
        let log_mapped = key_name(km, Action::DialogueLog) != km.not_mapped();
        WaitView {
            key: key_name(km, Action::Confirm),
            log_hint: log_mapped.then(|| ctx.text_with("dialogue.log_hint", &[])),
            marker: self.waiting % (2.0 * BLINK_S) < BLINK_S,
        }
    }
}

/// A character's or speaker's display name: the name the player gave the
/// lead, else their entry in the names table (their id if they have none).
fn display_name<'a>(ctx: &'a Ctx, lead: &'a LeadProfile, portrait: Portrait<'a>) -> &'a str {
    if portrait.character.0 == LEAD_ID {
        // check-text: not a data name (the player's own)
        return &lead.name;
    }
    ctx.words()
        .name(&ctx.content.names, portrait.character.0.as_str())
}

#[cfg(test)]
mod tests;
