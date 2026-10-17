//! The lead screen, after the mode (ticket 0801,
//! `docs/design/setting-and-tone.md`): pick the lead's gender, shown with
//! the `lead_m` and `lead_f` portraits, and first name (default
//! [`DEFAULT_NAME`]; the family name [`FAMILY_NAME`] is fixed).
//!
//! The name (Nick, PR #127: "C"): on a keyboard the player **types** it
//! ([`NameBox`]); when the name is chosen with a controller button, it is
//! spelled on a letter grid instead ([`NameEntry`]). Both are
//! [`crate::widgets::name_entry`]'s, with [`NAME_RULES`]: letters, `-`,
//! `'` and single spaces between words, at most [`MAX_NAME_LEN`]
//! characters, not blank. Cancel leaves the name as it was.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use crate::widgets::name_entry::{GridCell, NameBox, NameEntry, grid};
pub use view::{GenderView, GridView, LeadSelectView, NameRowView, TypingView};

use trpg_core::lead::{DEFAULT_NAME, FAMILY_NAME, MAX_NAME_LEN};
use trpg_core::{LeadGender, LeadProfile};

use crate::audio::MenuSound;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, text_keys_help};
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::widgets::name_entry::{NameEvent, NameRules};

/// The key of the heading in the language files ([`Ctx::text`]).
pub const HEADING: &str = "lead_select.heading";
/// The key of the button that starts the game.
pub const START: &str = "lead_select.start";
/// The key of the name row's label.
const NAME_LABEL: &str = "lead_select.name";
/// The key of the title of the typing box and of the name grid.
const FIRST_NAME: &str = "lead_select.first_name";

/// What the lead's first name may hold.
pub const NAME_RULES: NameRules = NameRules {
    max_chars: MAX_NAME_LEN,
    digits: false,
};

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
            (Some(_), _) => "name_entry.help.grid",
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
                    self.entry = Some(NameEntry::new(&self.name, NAME_RULES));
                } else {
                    self.typing = Some(NameBox::new(&self.name, NAME_RULES));
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

    /// The typing box or the grid closed with `event`: Done takes `name`,
    /// Cancel leaves the name as it was.
    fn close_name(&mut self, ctx: &mut Ctx, event: NameEvent, name: String) {
        if event == NameEvent::Done {
            self.name = name;
        } else {
            ctx.audio.menu(MenuSound::Cancel);
        }
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
            typing: self
                .typing
                .as_ref()
                .map(|t| t.view(ctx, ctx.text(FIRST_NAME))),
            grid: self
                .entry
                .as_ref()
                .map(|e| e.view(ctx, ctx.text(FIRST_NAME))),
            help: self.help(ctx),
        }
    }
}

impl Default for LeadSelectScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen<Shuyi> for LeadSelectScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        // While typing, the game's keys do nothing: they are letters.
        if let Some(typing) = &mut self.typing {
            let event = typing.handle(input, ctx);
            if event != NameEvent::Open {
                let name = typing.name().to_owned();
                self.typing = None;
                self.close_name(ctx, event, name);
            }
            return Transition::None;
        }
        for &action in &input.actions {
            if let Some(entry) = &mut self.entry {
                let event = entry.handle(action, ctx);
                if event != NameEvent::Open {
                    let name = entry.name().to_owned();
                    self.entry = None;
                    self.close_name(ctx, event, name);
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

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
