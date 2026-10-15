//! The dialogue log (ticket 0729, `docs/design/look-and-feel.md`, *Dialogue
//! log*) as plain data: every line of the scene seen so far, with its
//! speaker's face as it was there. The dialogue screen builds a
//! [`LogView`] from its [`History`] while the log is open
//! ([`DialogueScreen::log_view`](super::DialogueScreen::log_view)) and a
//! skin paints it ([`glyph::paint`] today); see ADR-0054.

pub mod glyph;

use crate::prelude::*;
use trpg_content::{LineId, Side};
use trpg_core::LEAD_ID;

use super::display_name;
use super::history::{History, Seen};
use crate::dialogue::Portrait;
use crate::screen::Ctx;

/// The log, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogView {
    /// Its title.
    pub title: String,
    /// Every line seen, oldest first.
    pub rows: Vec<LogRow>,
    /// The row after them that jumps to the furthest line seen: only after
    /// a jump back.
    pub back_to_latest: Option<BackRow>,
    /// The help line: what the keys do here.
    pub help: String,
}

impl LogView {
    /// The focused line, if the focus is on one (not on
    /// [`back_to_latest`](Self::back_to_latest)).
    pub fn focused(&self) -> Option<&LogRow> {
        self.rows.iter().find(|r| r.focused)
    }
}

/// What kind of line a row is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RowKind {
    /// A character speaks.
    Speech,
    /// Narration: no speaker.
    Narration,
    /// A reply the lead picked.
    Reply,
}

/// One line seen.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRow {
    /// The line's id (its voice clip's, ADR-0046), if it has one.
    pub line: Option<LineId>,
    /// What kind of line it is.
    pub kind: RowKind,
    /// Who says it (none for narration).
    pub name: Option<String>,
    /// The speaker's face as it was on this line, if they have one.
    pub face: Option<Face>,
    /// The text, in full (a reply's as the menu showed it).
    pub text: String,
    /// Whether it was seen past the line on screen (after a jump back).
    pub ahead: bool,
    /// Whether the cursor is on it.
    pub focused: bool,
}

/// A speaker's face: which portrait, with which expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Face {
    /// The portrait's id (the lead's by gender).
    pub portrait: String,
    /// Its expression on that line.
    pub expression: String,
}

/// The *Back to latest* row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackRow {
    /// Its name.
    pub label: String,
    /// Whether the cursor is on it.
    pub focused: bool,
}

/// The rows the cursor can be on: every line seen, then *Back to latest*
/// if it is shown.
pub fn row_count(history: &History) -> usize {
    history.seen().len() + usize::from(has_back_row(history))
}

/// Whether *Back to latest* is shown: the line on screen isn't the
/// furthest one seen.
pub fn has_back_row(history: &History) -> bool {
    history.at() < history.latest()
}

/// The log of `history` with the cursor on row `focus` (of
/// [`row_count`]), in the player's language and naming their keys.
pub fn view(ctx: &Ctx, history: &History, focus: usize) -> LogView {
    let at = history.at();
    let rows = history.seen().iter().enumerate().map(|(i, seen)| LogRow {
        ahead: i > at,
        focused: i == focus,
        ..row(ctx, seen)
    });
    let back_to_latest = has_back_row(history).then(|| BackRow {
        label: ctx.text("dialogue.log.back_to_latest").to_owned(),
        focused: focus == history.seen().len(),
    });
    LogView {
        title: ctx.text("dialogue.log.title").to_owned(),
        rows: rows.collect(),
        back_to_latest,
        help: ctx.text_with("dialogue.log.help", &[]),
    }
}

/// `seen`'s row, not focused, not ahead.
fn row(ctx: &Ctx, seen: &Seen) -> LogRow {
    let player = &seen.player;
    let lead = player.lead();
    let view = player.current();
    let face = |p: Portrait| Face {
        portrait: lead.portrait_for(&p.character.0).to_owned(),
        expression: p.expression.to_owned(),
    };
    if let Some(option) = seen.reply {
        let on_stage = [view.left, view.right].into_iter().flatten();
        let lead_portrait = on_stage.into_iter().find(|p| p.character.0 == LEAD_ID);
        let text = view.choices.unwrap_or_default().get(option).cloned();
        return LogRow {
            line: player.reply_line(option).cloned(),
            kind: RowKind::Reply,
            // check-text: not a data name (the player's own)
            name: Some(lead.name.clone()),
            face: lead_portrait.map(face),
            text: text.unwrap_or_default().into_owned(),
            ahead: false,
            focused: false,
        };
    }
    let speaker = match view.speaker {
        Some(Side::Left) => view.left,
        Some(Side::Right) => view.right,
        None => None,
    };
    LogRow {
        line: player.line().cloned(),
        kind: if view.narration {
            RowKind::Narration
        } else {
            RowKind::Speech
        },
        name: speaker.map(|s| display_name(ctx, lead, s).to_owned()),
        face: speaker.map(face),
        text: view.text.unwrap_or_default().into_owned(),
        ahead: false,
        focused: false,
    }
}
