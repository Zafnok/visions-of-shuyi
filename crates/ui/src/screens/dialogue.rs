//! The dialogue screen (ticket 0704, ADR-0018): plays a [`DialoguePlayer`]
//! with two portraits, name plates and a typewriter text box. Full-screen,
//! or as an overlay that leaves the battle map visible behind it.
//!
//! Layout (100×32): 32×16-cell portraits in 34×18 frames at `x = 1` and
//! `x = 65` from row 1, name plates on row 19, the text box on rows 21–27.
//! While the lead's replies are up, the text box grows upward to list them
//! under the line being answered (Nick, 0708: like Stardew Valley).
//!
//! The dialogue log (ticket 0729) lists the lines seen so far over the
//! whole screen: [`history`] keeps them, [`log`] is what it shows and
//! [`log::glyph`] paints it.

pub mod history;
pub mod log;

use trpg_content::{MusicLine, Names, Present, Scene, Side};
use trpg_core::{LEAD_ID, LeadProfile};

use crate::audio::MenuSound;
use crate::color::UiColor;
use crate::dialogue::{DialoguePlayer, Portrait, View};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::input::{Action, Button, Chord, Keymap};
use crate::portrait::draw_portrait;
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::screens::print_centred;
use crate::widgets::help::key_name;
use crate::widgets::menu::{Menu, MenuEvent, MenuItem};
use crate::widgets::word_wrap;
use history::History;
use log::LogView;

/// Top row of the portrait frames.
const FRAME_Y: i32 = 1;
/// Portrait frame size: a 32×16-cell portrait plus its border.
pub const FRAME: (i32, i32) = (34, 18);
/// Left column of the left and right frames.
const LEFT_X: i32 = 1;
/// Left column of the right frame.
const RIGHT_X: i32 = 65;
/// The text box.
const TEXT_BOX: Rect = Rect::new(0, 21, 100, 7);
/// Left column of the text.
const TEXT_X: i32 = 4;
/// First text row.
const TEXT_Y: i32 = 23;
/// Characters per text line: the box less a 3-cell margin each side.
pub const TEXT_W: usize = trpg_content::dialogue::BOX_WIDTH;
/// Text lines per box; longer text pages.
pub const TEXT_LINES: usize = trpg_content::dialogue::BOX_LINES;
/// How much the listener (and both portraits during narration) is dimmed.
pub const LISTENER_DIM: f32 = 0.45;
/// Reveal speed multiplier while Confirm is held.
pub const FAST_FORWARD: f32 = 6.0;
/// Seconds Confirm must be held on a page before it reveals faster, so the
/// tap that turned the page doesn't speed the next one up.
pub const FAST_FORWARD_DELAY: f32 = 0.3;
/// The `▼` is shown for this long, then hidden for as long, while waiting.
const BLINK_S: f32 = 0.5;
/// Cells between the log's key hint and the Confirm key's name.
const LOG_HINT_GAP: i32 = 3;

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
        self.draw_scene(ctx, buf);
    }

    fn is_overlay(&self) -> bool {
        self.overlay
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
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

    /// The whole screen: caption, portraits and text box; or the log.
    fn draw_scene(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        if let Some(view) = self.log_view(ctx) {
            log::glyph::paint(ctx, &view, buf);
            return;
        }
        let c = |u| ctx.palette.get(u);
        let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
        if self.overlay {
            // Below the text box is the battle's key help, whose keys do
            // nothing while the scene plays.
            // (Clipped to the buffer.)
            let below = TEXT_BOX.y + TEXT_BOX.h;
            let rest = Rect::new(0, below, i32::from(buf.width()), i32::from(buf.height()));
            buf.fill_rect(rest, blank);
        } else {
            buf.fill_rect(buf.bounds(), blank);
        }
        let view = self.player.current();
        draw_stage(ctx, buf, self.player.lead(), &view);
        self.draw_text_box(ctx, buf, &view);
    }
}

/// A text box of a scene playing by itself (a cinematic's talk shot,
/// ticket 0818): what the dialogue screen shows of it, without the things
/// that wait for a key.
#[derive(Debug, Clone, Copy)]
pub struct Told<'a> {
    /// The lead, for their name and portrait.
    pub lead: &'a LeadProfile,
    /// Who is left and right, who speaks, the caption.
    pub view: &'a View<'a>,
    /// The text's lines, wrapped: one page ([`TEXT_LINES`]) is drawn.
    pub lines: &'a [String],
    /// How many of its characters are revealed.
    pub shown: usize,
}

/// Draws `told` over the whole of `buf` as the full-screen dialogue screen
/// draws that text box: caption, portraits, name plates and the text box
/// with its text revealed so far. No `▼`, no key name.
pub fn draw_told(ctx: &Ctx, buf: &mut GlyphBuffer, told: &Told) {
    let c = |u| ctx.palette.get(u);
    let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
    buf.fill_rect(buf.bounds(), blank);
    draw_stage(ctx, buf, told.lead, told.view);
    draw_box_frame(ctx, buf, told.lead, told.view, TEXT_BOX);
    let page = &told.lines[..told.lines.len().min(TEXT_LINES)];
    draw_lines(
        ctx,
        buf,
        told.view.narration,
        page,
        told.shown,
        TEXT_Y,
        true,
    );
}

/// The caption and both sides' portraits of `view`.
fn draw_stage(ctx: &Ctx, buf: &mut GlyphBuffer, lead: &LeadProfile, view: &View) {
    let c = |u| ctx.palette.get(u);
    if let Some(caption) = &view.caption {
        let text = format!(" {caption} ");
        print_centred(buf, 0, &text, c(UiColor::TextHighlight), c(UiColor::Black));
    }
    for (side, portrait) in [(Side::Left, view.left), (Side::Right, view.right)] {
        if let Some(portrait) = portrait {
            draw_side(ctx, buf, lead, side, portrait, view.speaker == Some(side));
        }
    }
}

/// The text box `text_box`, empty, with `view`'s speaker's name on its
/// border.
fn draw_box_frame(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    lead: &LeadProfile,
    view: &View,
    text_box: Rect,
) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(text_box, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(text_box, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    let speaker = match view.speaker {
        Some(Side::Left) => view.left,
        Some(Side::Right) => view.right,
        None => None,
    };
    if let Some(speaker) = speaker {
        let name = format!(" {} ", display_name(ctx, lead, speaker));
        let fg = c(UiColor::TextHighlight);
        buf.print(text_box.x + 3, text_box.y, &name, fg, bg);
    }
}

/// The first `shown` characters of the page `lines` from row `text_y`.
/// Narration is dim and centred; vertically too when `alone` (nothing
/// follows it in the box): one line goes in the middle of the three rows
/// (two or three fill them from the top).
fn draw_lines(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    narration: bool,
    lines: &[String],
    shown: usize,
    text_y: i32,
    alone: bool,
) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let fg = if narration {
        c(UiColor::TextDim)
    } else {
        c(UiColor::Text)
    };
    let top = text_y + i32::from(narration && lines.len() == 1 && alone);
    let mut budget = shown;
    for (y, line) in (top..).zip(lines) {
        let len = line.chars().count();
        let x = if narration {
            TEXT_BOX.x + (TEXT_BOX.w - i32::try_from(len).unwrap_or(0)) / 2
        } else {
            TEXT_X
        };
        let visible: String = line.chars().take(budget).collect();
        buf.print(x, y, &visible, fg, bg);
        budget = budget.saturating_sub(len);
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

/// One side's frame, portrait (the lead's by gender) and name plate.
fn draw_side(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    lead: &LeadProfile,
    side: Side,
    portrait: Portrait,
    speaking: bool,
) {
    let x = match side {
        Side::Left => LEFT_X,
        Side::Right => RIGHT_X,
    };
    let frame = FaceFrame {
        at: (x, FRAME_Y),
        portrait: lead.portrait_for(&portrait.character.0),
        expression: portrait.expression,
        name: display_name(ctx, lead, portrait),
        speaking,
        mirror: side == Side::Right,
    };
    draw_face(ctx, buf, &frame);
}

/// A portrait in its frame with a name plate under it.
#[derive(Debug, Clone, Copy)]
pub struct FaceFrame<'a> {
    /// The frame's top-left cell.
    pub at: (i32, i32),
    /// The portrait's id (an empty frame if the content has none).
    pub portrait: &'a str,
    /// Its expression.
    pub expression: &'a str,
    /// The name on the plate.
    pub name: &'a str,
    /// Lit, with a double border, as whoever speaks is; else dimmed.
    pub speaking: bool,
    /// Whether the portrait is flipped left to right.
    pub mirror: bool,
}

/// Draws `face`: a [`FRAME`] with its portrait, and the name centred on
/// the row under it. Clipped to the buffer.
pub fn draw_face(ctx: &Ctx, buf: &mut GlyphBuffer, face: &FaceFrame) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let (x, y) = face.at;
    let frame = Rect::new(x, y, FRAME.0, FRAME.1);
    buf.fill_rect(frame, Cell::new(' ', c(UiColor::Text), bg));
    let (style, border, dim, name_fg) = if face.speaking {
        (
            BoxStyle::Double,
            UiColor::PanelBorderFocus,
            0.0,
            UiColor::Text,
        )
    } else {
        (
            BoxStyle::Single,
            UiColor::PanelBorder,
            LISTENER_DIM,
            UiColor::TextDim,
        )
    };
    buf.draw_box(frame, style, c(border), bg);
    if let Some(art) = ctx.content.portraits.get(face.portrait) {
        let at = (x + 1, y + 1);
        draw_portrait(buf, at, art, face.expression, dim, face.mirror);
    }
    let plate = Rect::new(x, y + FRAME.1, FRAME.0, 1);
    buf.fill_rect(plate, Cell::new(' ', c(UiColor::Text), bg));
    // check-text: not a data name (the view's own)
    let name = face.name;
    let w = i32::try_from(name.chars().count()).unwrap_or(0);
    buf.print(x + (FRAME.0 - w) / 2, plate.y, name, c(name_fg), bg);
}

impl DialogueScreen {
    /// The text box and its first text row. While the replies are up it
    /// grows upward (its bottom stays put) to fit the line being answered,
    /// a blank row and one row per reply.
    pub fn text_box(&self) -> (Rect, i32) {
        let Some(menu) = &self.menu else {
            return (TEXT_BOX, TEXT_Y);
        };
        let question = self.page_lines().len();
        let gap = usize::from(question > 0);
        let rows = i32::try_from(question + gap + menu.items().len()).unwrap_or(0);
        // The last content row is the one the ▼ uses, free during a choice.
        let bottom = TEXT_BOX.y + TEXT_BOX.h - 1;
        let text_y = (bottom - rows).min(TEXT_Y);
        let top = text_y - (TEXT_Y - TEXT_BOX.y);
        (
            Rect::new(TEXT_BOX.x, top, TEXT_BOX.w, bottom + 1 - top),
            text_y,
        )
    }

    /// The text box: speaker name on the border, the revealed text (or the
    /// skip question), the replies while a choice is open, and the
    /// blinking `▼` once the page is shown.
    fn draw_text_box(&self, ctx: &Ctx, buf: &mut GlyphBuffer, view: &View) {
        let c = |u| ctx.palette.get(u);
        let bg = c(UiColor::PanelBg);
        let km = ctx.help_keys();
        let (text_box, text_y) = self.text_box();
        draw_box_frame(ctx, buf, self.player.lead(), view, text_box);

        if self.asking_skip {
            let yes_no = ctx.text_with("dialogue.skip_answers", &[]);
            let question = ctx.text("dialogue.skip_question");
            buf.print(TEXT_X, TEXT_Y, question, c(UiColor::Text), bg);
            buf.print(TEXT_X, TEXT_Y + 1, &yes_no, c(UiColor::TextDim), bg);
            return;
        }

        let lines = self.page_lines();
        #[expect(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            reason = "shown is between 0 and the page length"
        )]
        let shown = self.shown as usize;
        // Narration isn't centred vertically above replies, which follow
        // right after.
        let alone = self.menu.is_none();
        draw_lines(ctx, buf, view.narration, lines, shown, text_y, alone);

        if let Some(menu) = &self.menu {
            let gap = i32::from(!lines.is_empty());
            let first = text_y + i32::try_from(lines.len()).unwrap_or(0) + gap;
            for (y, (i, item)) in (first..).zip(menu.items().iter().enumerate()) {
                let (marker, fg) = if i == menu.focus() {
                    ("> ", c(UiColor::TextHighlight))
                } else {
                    ("  ", c(UiColor::Text))
                };
                buf.print(TEXT_X, y, &format!("{marker}{}", item.label), fg, bg);
            }
        }

        if self.is_revealed() && self.menu.is_none() {
            let right = TEXT_BOX.x + TEXT_BOX.w - 4;
            let bottom = TEXT_BOX.y + TEXT_BOX.h - 2;
            let key = key_name(km, Action::Confirm);
            let w = i32::try_from(key.chars().count()).unwrap_or(0);
            buf.print(right - 1 - w, bottom, &key, c(UiColor::TextDim), bg);
            // The log's key, if it has one, before it.
            if key_name(km, Action::DialogueLog) != km.not_mapped() {
                let hint = ctx.text_with("dialogue.log_hint", &[]);
                let hint_w = i32::try_from(hint.chars().count()).unwrap_or(0);
                let x = right - 1 - w - LOG_HINT_GAP - hint_w;
                buf.print(x, bottom, &hint, c(UiColor::TextDim), bg);
            }
            if self.waiting % (2.0 * BLINK_S) < BLINK_S {
                buf.print(right, bottom, "▼", c(UiColor::TextHighlight), bg);
            }
        }
    }
}

#[cfg(test)]
mod tests;
