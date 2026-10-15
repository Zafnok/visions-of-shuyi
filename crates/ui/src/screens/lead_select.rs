//! The lead screen, after the mode (ticket 0801,
//! `docs/design/setting-and-tone.md`): pick the lead's gender, shown with
//! the `lead_m` and `lead_f` portraits, and first name (default
//! [`DEFAULT_NAME`]; the family name [`FAMILY_NAME`] is fixed).
//!
//! The name (Nick, PR #127: "C"): on a keyboard the player **types** it
//! ([`NameBox`]). While the box is open the game's keys do nothing (Select
//! and the rest are letters then), and a hint says to type; the text box's
//! own fixed keys finish, delete and cancel ([`crate::input::text_key`]).
//! When the name is chosen with a controller button, it is spelled on a
//! letter grid instead ([`NameEntry`]) with the cursor and Confirm, like a
//! console naming screen.
//!
//! Name rules for both: letters (A-Z, a-z), `-`, `'` and single spaces
//! between words, at most [`MAX_NAME_LEN`] characters, not blank.

pub mod glyph;
pub mod view;

pub use view::{GenderView, GridView, LeadSelectView, NameRowView, TypingView};

use trpg_core::lead::{DEFAULT_NAME, FAMILY_NAME, MAX_NAME_LEN};
use trpg_core::{LeadGender, LeadProfile};

use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, TextKey, text_key, text_keys_help};
use crate::screen::{Ctx, FrameInput, Screen, Transition};

/// The key of the heading in the language files ([`Ctx::text`]).
pub const HEADING: &str = "lead_select.heading";
/// The key of the button that starts the game.
pub const START: &str = "lead_select.start";
/// The key of the name row's label.
const NAME_LABEL: &str = "lead_select.name";
/// The key of the name grid's cell that types a space.
pub const SPACE: &str = "lead_select.grid.blank";
/// The key of the cell that deletes the last letter.
pub const DELETE: &str = "lead_select.grid.delete";
/// The key of the cell that closes the name grid.
pub const DONE: &str = "lead_select.grid.done";
/// The key of the title of the typing box and of the name grid.
const FIRST_NAME: &str = "lead_select.first_name";
/// The key of the typing box's hint.
pub const TYPE_HINT: &str = "lead_select.type_hint";

/// Adds `c` to `name` if the name rules allow it (see the module docs).
fn add_char(name: &mut String, c: char) -> bool {
    let full = name.chars().count() >= MAX_NAME_LEN;
    let ok = match c {
        _ if full => false,
        ' ' => !name.is_empty() && !name.ends_with(' '),
        c => c.is_ascii_alphabetic() || c == '-' || c == '\'',
    };
    if ok {
        name.push(c);
    }
    ok
}

/// `name` finished: trimmed, or `None` if blank.
fn finished(name: &str) -> Option<String> {
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_owned())
}

/// The name being typed on the keyboard.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameBox {
    /// The name typed so far.
    name: String,
    /// The name before the box opened (Cancel brings it back).
    before: String,
}

impl NameBox {
    /// The box for `name`, which the player types on from.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            before: name.to_owned(),
        }
    }

    /// The name typed so far.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// One frame: the characters typed, then the text box's keys. Actions
    /// are ignored. A character the rules refuse sounds denied; typing
    /// makes no sound.
    fn handle(&mut self, input: &FrameInput, ctx: &mut Ctx) -> EntryEvent {
        for &c in input.text() {
            if !add_char(&mut self.name, c) {
                ctx.audio.menu(MenuSound::Denied);
            }
        }
        for &chord in input.pressed_chords() {
            match text_key(chord) {
                Some(TextKey::Done) => match finished(&self.name) {
                    Some(name) => {
                        self.name = name;
                        ctx.audio.menu(MenuSound::Select);
                        return EntryEvent::Closed;
                    }
                    None => ctx.audio.menu(MenuSound::Denied),
                },
                Some(TextKey::Delete) => {
                    if self.name.pop().is_some() {
                        ctx.audio.menu(MenuSound::Cancel);
                    }
                }
                Some(TextKey::Cancel) => {
                    self.name.clone_from(&self.before);
                    ctx.audio.menu(MenuSound::Cancel);
                    return EntryEvent::Closed;
                }
                None => {}
            }
        }
        EntryEvent::Open
    }
}

/// The letters of the name grid, one row each.
// check-text: not player text
const LETTER_ROWS: [&str; 4] = [
    "ABCDEFGHIJKLM",
    "NOPQRSTUVWXYZ",
    "abcdefghijklm",
    "nopqrstuvwxyz",
];

/// One cell of the name grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GridCell {
    /// Types this character.
    Char(char),
    /// Types a space (between two words).
    Space,
    /// Deletes the last character.
    Delete,
    /// Closes the grid, keeping the name.
    Done,
}

impl GridCell {
    /// Text shown for the cell, in `ctx`'s language.
    pub fn label(self, ctx: &Ctx) -> String {
        let key = match self {
            GridCell::Char(c) => return c.to_string(),
            GridCell::Space => SPACE,
            GridCell::Delete => DELETE,
            GridCell::Done => DONE,
        };
        ctx.text(key).to_owned()
    }
}

/// The name grid's rows: four rows of letters, then `-`, `'`, Blank,
/// Delete and Done.
pub fn grid() -> Vec<Vec<GridCell>> {
    let mut rows: Vec<Vec<GridCell>> = LETTER_ROWS
        .iter()
        .map(|r| r.chars().map(GridCell::Char).collect())
        .collect();
    rows.push(vec![
        GridCell::Char('-'),
        GridCell::Char('\''),
        GridCell::Space,
        GridCell::Delete,
        GridCell::Done,
    ]);
    rows
}

/// The name grid, open over the lead screen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameEntry {
    /// The name being spelled.
    name: String,
    /// The name before the grid opened (Cancel on an empty name brings it
    /// back).
    before: String,
    /// The focused cell: row, column.
    at: (usize, usize),
}

/// What a key did in the name grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EntryEvent {
    /// Still spelling.
    Open,
    /// Closed; the name is its result.
    Closed,
}

impl NameEntry {
    /// The grid for `name`, on its first letter.
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            before: name.to_owned(),
            at: (0, 0),
        }
    }

    /// The name spelled so far.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// The focused cell.
    pub fn focus(&self) -> (usize, usize) {
        self.at
    }

    /// Handles `action`: the cursor keys move over the grid (wrapping along
    /// a row, keeping to the last cell of a shorter row), Confirm uses the
    /// focused cell, Cancel deletes the last character (on an empty name it
    /// closes the grid, putting the old name back). Plays the menu sounds.
    fn handle(&mut self, action: Action, ctx: &mut Ctx) -> EntryEvent {
        let rows = grid();
        let (row, col) = self.at;
        let width = |r: usize| rows[r].len();
        let moved = match action {
            Action::CursorLeft => (row, (col + width(row) - 1) % width(row)),
            Action::CursorRight => (row, (col + 1) % width(row)),
            Action::CursorUp => {
                let r = (row + rows.len() - 1) % rows.len();
                (r, col.min(width(r) - 1))
            }
            Action::CursorDown => {
                let r = (row + 1) % rows.len();
                (r, col.min(width(r) - 1))
            }
            Action::Confirm => return self.confirm(rows[row][col], ctx),
            Action::Cancel => {
                if self.name.pop().is_some() {
                    ctx.audio.menu(MenuSound::Cancel);
                    return EntryEvent::Open;
                }
                self.name.clone_from(&self.before);
                ctx.audio.menu(MenuSound::Cancel);
                return EntryEvent::Closed;
            }
            _ => return EntryEvent::Open,
        };
        if moved != self.at {
            self.at = moved;
            ctx.audio.menu(MenuSound::Move);
        }
        EntryEvent::Open
    }

    /// Confirm on `cell`. A character past [`MAX_NAME_LEN`], a space at
    /// the start or after another, and Done on a blank name are refused.
    fn confirm(&mut self, cell: GridCell, ctx: &mut Ctx) -> EntryEvent {
        let ok = match cell {
            GridCell::Char(c) => add_char(&mut self.name, c),
            GridCell::Space => add_char(&mut self.name, ' '),
            GridCell::Delete => self.name.pop().is_some(),
            GridCell::Done => {
                if let Some(name) = finished(&self.name) {
                    self.name = name;
                    ctx.audio.menu(MenuSound::Select);
                    return EntryEvent::Closed;
                }
                false
            }
        };
        ctx.audio.menu(if ok {
            MenuSound::Select
        } else {
            MenuSound::Denied
        });
        EntryEvent::Open
    }
}

/// Which row of the lead screen has the focus.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Row {
    /// The two portraits: Left and Right pick the gender.
    Gender,
    /// The first name: Confirm opens the typing box.
    Name,
    /// Confirm starts the game.
    Start,
}

impl Row {
    const ALL: [Row; 3] = [Row::Gender, Row::Name, Row::Start];
}

/// Gender and first name of the lead. Confirm on `Start` pops with the
/// profile ([`result`](Self::result)); Cancel pops without one (back to the
/// mode).
#[derive(Debug, Clone)]
pub struct LeadSelectScreen {
    gender: LeadGender,
    name: String,
    row: Row,
    /// The typing box, while open.
    typing: Option<NameBox>,
    /// The letter grid, while open (controller players).
    entry: Option<NameEntry>,
    result: Option<LeadProfile>,
}

impl LeadSelectScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "lead_select";

    /// The screen with the male lead and the default name, focus on the
    /// portraits.
    pub fn new() -> Self {
        Self {
            gender: LeadGender::Male,
            name: DEFAULT_NAME.to_owned(),
            row: Row::Gender,
            typing: None,
            entry: None,
            result: None,
        }
    }

    /// The lead the player made, once the screen has popped; `None` if they
    /// went back.
    pub fn result(&self) -> Option<&LeadProfile> {
        self.result.as_ref()
    }

    /// The gender picked so far.
    pub fn gender(&self) -> LeadGender {
        self.gender
    }

    /// The first name so far (the typing box's or the grid's, while open).
    pub fn first_name(&self) -> &str {
        match (&self.typing, &self.entry) {
            (Some(t), _) => t.name(),
            (None, Some(e)) => e.name(),
            (None, None) => &self.name,
        }
    }

    /// The typing box, while open.
    pub fn typing(&self) -> Option<&NameBox> {
        self.typing.as_ref()
    }

    /// The focused row.
    pub fn row(&self) -> Row {
        self.row
    }

    /// The name grid, while open.
    pub fn entry(&self) -> Option<&NameEntry> {
        self.entry.as_ref()
    }

    /// The bottom help line.
    pub fn help(&self, ctx: &Ctx) -> String {
        if self.typing.is_some() {
            return text_keys_help(ctx);
        }
        let key = match (&self.entry, self.row) {
            (Some(_), _) => "lead_select.help.grid",
            (None, Row::Gender) => "lead_select.help.gender",
            (None, Row::Name) => "lead_select.help.name",
            (None, Row::Start) => "lead_select.help.start",
        };
        ctx.text_with(key, &[])
    }

    /// Moves the focus one row down (`down`) or up, wrapping.
    fn step_row(&mut self, down: bool, ctx: &mut Ctx) {
        let n = Row::ALL.len();
        let i = Row::ALL.iter().position(|&r| r == self.row).unwrap_or(0);
        let next = if down { (i + 1) % n } else { (i + n - 1) % n };
        self.row = Row::ALL[next];
        ctx.audio.menu(MenuSound::Move);
    }

    /// Handles one action outside the grid. `pad`: it came from a
    /// controller, so the name opens the letter grid, not the typing box.
    /// Returns whether the screen is done (chose Start or went back).
    fn step(&mut self, action: Action, pad: bool, ctx: &mut Ctx) -> bool {
        match (self.row, action) {
            (_, Action::CursorDown) => self.step_row(true, ctx),
            (_, Action::CursorUp) => self.step_row(false, ctx),
            (Row::Gender, Action::CursorLeft | Action::CursorRight) => {
                self.gender = match self.gender {
                    LeadGender::Male => LeadGender::Female,
                    LeadGender::Female => LeadGender::Male,
                };
                ctx.audio.menu(MenuSound::Move);
            }
            (Row::Gender, Action::Confirm) => {
                self.row = Row::Name;
                ctx.audio.menu(MenuSound::Select);
            }
            (Row::Name, Action::Confirm) => {
                if pad {
                    self.entry = Some(NameEntry::new(&self.name));
                } else {
                    self.typing = Some(NameBox::new(&self.name));
                }
                ctx.audio.menu(MenuSound::Select);
            }
            (Row::Start, Action::Confirm) => {
                ctx.audio.menu(MenuSound::Select);
                self.result = Some(LeadProfile::new(self.name.clone(), self.gender));
                return true;
            }
            (_, Action::Cancel) => {
                ctx.audio.menu(MenuSound::Cancel);
                return true;
            }
            _ => {}
        }
        false
    }

    /// What the screen shows this frame, for a skin to paint
    /// ([`glyph::paint`]): the two genders with their portraits, the
    /// cursor's row, the name and Start, the typing box or letter grid
    /// while open, and the help line.
    pub fn view(&self, ctx: &Ctx) -> LeadSelectView {
        let boxed = self.typing.is_some() || self.entry.is_some();
        let genders = LeadGender::ALL
            .into_iter()
            .map(|gender| GenderView {
                gender,
                label: ctx
                    .text(match gender {
                        LeadGender::Male => "lead_select.male",
                        LeadGender::Female => "lead_select.female",
                    })
                    .to_owned(),
                portrait: LeadProfile::new(self.name.clone(), gender).portrait_id(),
                // check-text: not player text
                expression: "neutral",
                chosen: gender == self.gender,
            })
            .collect();
        LeadSelectView {
            heading: ctx.text(HEADING).to_owned(),
            genders,
            cursor: (!boxed).then_some(self.row),
            name_row: NameRowView {
                label: ctx.text(NAME_LABEL).to_owned(),
                first: self.first_name().to_owned(),
                family: FAMILY_NAME.to_owned(),
            },
            start: ctx.text(START).to_owned(),
            typing: self.typing.as_ref().map(|t| TypingView {
                title: ctx.text(FIRST_NAME).to_owned(),
                typed: t.name().to_owned(),
                hint: ctx.text(TYPE_HINT).to_owned(),
            }),
            grid: self.entry.as_ref().map(|e| GridView {
                title: ctx.text(FIRST_NAME).to_owned(),
                spelled: e.name().to_owned(),
                rows: grid()
                    .iter()
                    .map(|row| row.iter().map(|g| g.label(ctx)).collect())
                    .collect(),
                focus: e.focus(),
            }),
            help: self.help(ctx),
        }
    }
}

impl Default for LeadSelectScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for LeadSelectScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        // While typing, the game's keys do nothing: they are letters.
        if let Some(typing) = &mut self.typing {
            if typing.handle(input, ctx) == EntryEvent::Closed {
                self.name = typing.name().to_owned();
                self.typing = None;
            }
            return Transition::None;
        }
        for &action in &input.actions {
            if let Some(entry) = &mut self.entry {
                if entry.handle(action, ctx) == EntryEvent::Closed {
                    self.name = entry.name().to_owned();
                    self.entry = None;
                }
                continue;
            }
            if self.step(action, input.pad_pressed(), ctx) {
                return Transition::Pop;
            }
            // The key that opened the typing box types nothing, and the
            // rest of this frame's keys wait for the box.
            if self.typing.is_some() {
                break;
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
