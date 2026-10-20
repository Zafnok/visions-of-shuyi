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
//!
//! Where the script asks (`@opening`, ticket 0734), a short moving opening
//! plays before a line: the opening's characters walk on the place at
//! full brightness ([`crate::cinematic::opening`]), the place darkens,
//! then the line comes up.

pub mod glyph;
pub mod history;
pub mod log;
pub mod view;

use crate::prelude::*;
use alloc::rc::Rc;
use trpg_content::{MusicLine, Names, OpeningDef, PlaceLine, Present, Scene};
use trpg_core::{LEAD_ID, LeadProfile};

use crate::audio::MenuSound;
use crate::cinematic::opening::{cast_units, darkened, opening_ms, opening_units};
use crate::dialogue::{DialoguePlayer, Portrait};
use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, Button, Chord, Keymap};
use crate::map_view::{MapScene, Place, UnitView, screen_tiles};
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::widgets::Ask;
use crate::widgets::help::key_name;
use crate::widgets::menu::{Menu, MenuEvent, MenuItem};
use history::History;
use log::LogView;
use shuyi_ui::dialogue::{Step, TextBox};
use view::{DialogueView, OpeningView, SkipView, TextView, WaitView};

/// Characters per text line: the box less a 3-cell margin each side.
pub const TEXT_W: usize = trpg_content::dialogue::BOX_WIDTH;
/// Text lines per box; longer text pages.
pub const TEXT_LINES: usize = trpg_content::dialogue::BOX_LINES;
/// Reveal speed multiplier while Confirm is held, and the seconds it must
/// be held on a page before it reveals faster (so the tap that turned the
/// page doesn't speed the next one up): the engine's text box's.
pub use shuyi_ui::dialogue::text_box::{FAST_FORWARD, FAST_FORWARD_DELAY};

/// An opening playing (0734): which, and for how long so far.
#[derive(Debug, Clone, PartialEq)]
struct Playing {
    /// The opening's id (`assets/openings/`).
    id: String,
    /// Seconds since it started.
    elapsed_s: f32,
}

impl Playing {
    /// Milliseconds since it started.
    fn clock_ms(&self) -> u64 {
        crate::cinematic::millis(self.elapsed_s)
    }
}

/// Plays a scene: Confirm reveals the text at once, or moves on when it's
/// all shown; holding Confirm reveals faster, after a short delay on each
/// page. The End turn key does
/// everything Confirm does. Cancel asks whether to skip
/// the scene. At a reply choice, the cursor keys move through the lead's
/// replies and Confirm picks one; skipping stops at each choice. Pops when
/// the scene ends or is skipped. A `@music` line changes the music when the
/// scene reaches it; skipping asks only for the last one passed.
///
/// An `@opening` line plays its opening before the text box after it:
/// Confirm (or End turn) goes straight to the text box, Cancel to the
/// "Skip scene?" question over it. The opening's characters then stay
/// where it left them, darkened with the place, until the place changes.
/// Skipping the scene passes over any opening on the way.
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
    /// The current text box: its pages ([`TEXT_LINES`] lines each) and how
    /// much is revealed.
    text: TextBox,
    /// "Skip scene?", while it is open.
    asking_skip: Option<Ask<bool>>,
    /// The lead's replies, while a choice is open.
    menu: Option<Menu<usize>>,
    /// Every line seen, for the log.
    history: History,
    /// The log's cursor (a row of [`log::row_count`]), while it is open.
    log: Option<usize>,
    /// The music the scene set last (`None` before its first `@music`).
    music: Option<MusicLine>,
    /// Music to play: the music of a line the log jumped to.
    jumped_music: Option<MusicLine>,
    /// The battle's map that `@place battle` shows, if the scene is a
    /// battle's ([`with_battle`](Self::with_battle)).
    battle: Option<Rc<Place>>,
    /// The opening playing before the text box, if any.
    playing: Option<Playing>,
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
            text: TextBox::new("", TEXT_W, TEXT_LINES),
            asking_skip: None,
            menu: None,
            history: History::new(),
            log: None,
            music: None,
            jumped_music: None,
            battle: None,
            playing: None,
        };
        screen.start_box();
        screen
    }

    /// [`new`](Self::new) in the player's language ([`Ctx::words`]): the
    /// lines, replies and captions its pack translates, with the names of
    /// the story as the pack has them.
    pub fn told(ctx: &Ctx, scene: &Scene, lead: LeadProfile, present: &Present) -> Self {
        let words = ctx.words();
        let names = words.names(&ctx.game.content.names);
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

    /// The same screen with `battle` as the map `@place battle` shows: the
    /// field of the chapter's battle for its intro and victory scenes, the
    /// battle as it stands for a scene played during it. Without one, a
    /// `@place battle` line shows no place.
    #[must_use]
    pub fn with_battle(self, battle: Place) -> Self {
        Self {
            battle: Some(Rc::new(battle)),
            ..self
        }
    }

    /// The map `@place battle` shows, if the screen was given one.
    pub fn battle(&self) -> Option<&Place> {
        self.battle.as_deref()
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
        self.text.page_lines()
    }

    /// Whether the whole page is revealed.
    pub fn is_revealed(&self) -> bool {
        self.text.is_revealed()
    }

    /// Whether the "Skip scene?" question is open.
    pub fn is_asking_skip(&self) -> bool {
        self.asking_skip.is_some()
    }

    /// Opens "Skip scene?", with the sound of opening a menu.
    fn ask_skip(&mut self, ctx: &mut Ctx) {
        ctx.audio.menu(MenuSound::Select);
        self.asking_skip = Some(Ask::yes_no(
            "dialogue.skip_question",
            "dialogue.skip_answers",
        ));
    }

    /// The replies menu, while a choice is open.
    pub fn menu(&self) -> Option<&Menu<usize>> {
        self.menu.as_ref()
    }

    /// Whether an opening is playing before the text box (0734).
    pub fn is_opening(&self) -> bool {
        self.playing.is_some()
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

    /// A new text box (or a choice) is up: an opening the scene reached
    /// starts playing before it, the log keeps a text box, then
    /// [`show_box`](Self::show_box).
    fn start_box(&mut self) {
        if let Some(id) = self.player.take_opening() {
            self.playing = Some(Playing { id, elapsed_s: 0.0 });
        }
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
        let text = TextBox::new(view.text.as_deref().unwrap_or(""), TEXT_W, TEXT_LINES);
        let menu = view.choices.map(|choices| {
            let items = choices.into_iter().enumerate();
            let items = items.map(|(i, c)| MenuItem::new(c).with_value(i));
            Menu::new(items.collect()).without_cancel()
        });
        self.text = text;
        self.menu = menu;
        if self.menu.is_some() {
            self.text.to_last_page();
        }
    }

    /// Confirm: reveals the page, else the next page, else the next box.
    /// Returns `true` once the scene is over.
    fn confirm(&mut self) -> bool {
        if self.text.confirm() == Step::Done {
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

impl Screen<Shuyi> for DialogueScreen {
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
        if self.playing.is_some() {
            return self.read_opening(ctx, input);
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
                if !self.is_asking_skip()
                    && !self.history.is_empty()
                    && self.log_key_free(ctx, input)
                {
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
            if let Some(ask) = self.asking_skip.as_mut() {
                let Some(yes) = ask.handle(action, &mut ctx.audio) else {
                    continue;
                };
                self.asking_skip = None;
                if !yes {
                    ctx.audio.menu(MenuSound::Cancel);
                    continue;
                }
                ctx.audio.menu(MenuSound::Select);
                self.player.skip_to_choice();
                if self.player.is_finished() {
                    return Transition::Pop;
                }
                // An opening passed on the way doesn't play.
                self.player.take_opening();
                self.start_box();
                // The rest of this frame's keys can't pick a reply.
                break;
            }
            match action {
                Action::Cancel => self.ask_skip(ctx),
                Action::Confirm if self.confirm() => return Transition::Pop,
                // A choice just opened: the rest of this frame's keys
                // can't pick a reply.
                Action::Confirm if self.menu.is_some() => break,
                // An opening started: the rest of this frame's keys can't
                // skip it.
                Action::Confirm if self.playing.is_some() => break,
                _ => {}
            }
        }
        if self.log.is_some() || self.playing.is_some() {
            // The scene waits while the log is open, and the next text box
            // while the opening a key just reached plays.
            self.text.release();
            return Transition::None;
        }
        let held = ADVANCE_KEYS.iter().any(|&a| input.is_held(a));
        self.text.hold(held, input.dt());
        if !self.is_asking_skip() {
            // Instant text speed: the whole page at once.
            let speed = ctx.settings().text_speed.chars_per_s();
            self.text.type_on(input.dt(), speed);
        }
        Transition::None
    }

    /// One frame of an opening: Confirm (or End turn) ends it, Cancel ends
    /// it and asks whether to skip the scene; otherwise it plays on, and
    /// ends once it has played its length and darkened. The text waits.
    fn read_opening(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            match advance_key(action) {
                Action::Confirm => {
                    self.playing = None;
                    return Transition::None;
                }
                Action::Cancel => {
                    self.playing = None;
                    self.ask_skip(ctx);
                    return Transition::None;
                }
                _ => {}
            }
        }
        let length = self.playing_def(ctx).map_or(0, opening_ms);
        if let Some(playing) = &mut self.playing {
            playing.elapsed_s += input.dt();
            if playing.clock_ms() >= u64::from(length) {
                self.playing = None;
            }
        }
        Transition::None
    }

    /// The opening playing, if the content has it.
    fn playing_def<'c>(&self, ctx: &'c Ctx) -> Option<&'c OpeningDef> {
        ctx.game.content.openings.get(&self.playing.as_ref()?.id)
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
        // The music it had reached then is in `seen.music`; its opening
        // has played.
        player.take_music();
        player.take_opening();
        self.player = player;
        self.playing = None;
        if seen.music.is_some() && seen.music != self.music {
            self.jumped_music = seen.music;
        }
        self.asking_skip = None;
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
        let opening = current
            .opening
            .and_then(|id| ctx.game.content.openings.get(id));
        view.place = current
            .place
            .and_then(|line| self.place_scene(ctx, line, opening));
        if let Some(playing) = &self.playing {
            let def = self.playing_def(ctx);
            view.opening = Some(OpeningView {
                darkened: def.map_or(1.0, |d| darkened(d, playing.clock_ms())),
            });
        }
        view.log = self.log_view(ctx);
        view
    }

    /// The part of the place `line` names that the whole screen shows with
    /// `ctx`'s map skin, centred on the line's tile, with the characters of
    /// the `opening` played on it where it has them now. `None` for no
    /// place, a map the content hasn't got (the validator rules it out)
    /// and the battle when the screen has none.
    fn place_scene(
        &self,
        ctx: &Ctx,
        line: &PlaceLine,
        opening: Option<&OpeningDef>,
    ) -> Option<MapScene> {
        let view = screen_tiles(ctx);
        match line {
            PlaceLine::Map { map, at } => {
                let def = ctx.game.content.maps.get(map)?;
                let mut place = Place::of_map(def);
                if let Some(opening) = opening.filter(|o| o.map == *map) {
                    place.units = self.opening_cast(ctx, opening);
                }
                Some(place.scene(*at, view))
            }
            PlaceLine::Battle { at } => Some(self.battle.as_ref()?.scene(*at, view)),
            PlaceLine::None => None,
        }
    }

    /// The characters of `opening` where it has them now: as far into it
    /// as it has played, or where it left them once it is over.
    fn opening_cast(&self, ctx: &Ctx, opening: &OpeningDef) -> Vec<UnitView> {
        let playing = self.playing.as_ref().filter(|p| p.id == opening.id);
        let clock_ms = playing.map_or(u64::from(opening.length_ms), Playing::clock_ms);
        let start = cast_units(ctx, opening, self.player.lead());
        opening_units(&start, opening, clock_ms)
    }

    /// What the text box shows: the page as revealed so far, or the skip
    /// question; the replies while a choice is open; what shows while the
    /// page waits for a key. `narration`: whether the text is narration.
    fn text_view(&self, ctx: &Ctx, narration: bool) -> TextView {
        let shown = self.text.shown();
        // The text box asks in its own place (the engine's text box skin).
        let skip = self.asking_skip.as_ref().map(|ask| {
            let view = ask.view(ctx);
            SkipView {
                question: view.text,
                answers: view.keys.unwrap_or_default(),
            }
        });
        let waits = !self.is_asking_skip() && self.menu.is_none() && self.is_revealed();
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
            marker: self.text.marker_lit(),
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
        .name(&ctx.game.content.names, portrait.character.0.as_str())
}

#[cfg(test)]
mod tests;
