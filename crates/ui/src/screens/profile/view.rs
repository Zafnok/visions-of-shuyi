//! What the Profile screen shows, as plain data: no cells, glyphs, colours
//! or positions. The screen builds a [`ProfileView`] each frame
//! ([`ProfileScreen::view`](super::ProfileScreen::view)) and a skin paints
//! it ([`super::glyph::paint`] today); see ADR-0054.

use super::RowKind;
use crate::prelude::*;
pub use crate::widgets::name_entry::{GridView, TypingView};

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProfileView {
    /// The screen's title.
    pub title: String,
    /// The profiles, in the order they were made, then the row that adds
    /// one.
    pub rows: Vec<RowView>,
    /// The words that mark the profile in use.
    pub current_mark: String,
    /// What can be done to the focused profile, while that menu is open.
    pub actions: Option<ActionsView>,
    /// The typing box, while a name is typed on the keyboard.
    pub typing: Option<TypingView>,
    /// The letter grid, while a name is spelled on a controller.
    pub grid: Option<GridView>,
    /// The question open over everything, if any.
    pub question: Option<QuestionView>,
    /// A message for the player (something couldn't be done).
    pub message: Option<String>,
    /// The help line: what the keys do here.
    pub help: String,
}

impl ProfileView {
    /// The focused row.
    pub fn focused(&self) -> Option<&RowView> {
        self.rows.iter().find(|r| r.focused)
    }

    /// The row of the profile in use.
    pub fn current(&self) -> Option<&RowView> {
        self.rows.iter().find(|r| r.current)
    }
}

/// One row: a profile, or the row that adds one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowView {
    /// Which row it is.
    pub kind: RowKind,
    /// The profile's name, or what the row does.
    pub label: String,
    /// Whether it is the profile in use.
    pub current: bool,
    /// Whether the cursor is on it.
    pub focused: bool,
    /// Whether Confirm does something there (no adding past the most
    /// profiles).
    pub enabled: bool,
}

/// The menu of what can be done to a profile.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActionsView {
    /// The profile's name.
    pub title: String,
    /// What can be done, top to bottom.
    pub items: Vec<ChoiceView>,
}

/// A question with its answers on one line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuestionView {
    /// The question.
    pub text: String,
    /// The answers, in order.
    pub choices: Vec<ChoiceView>,
}

impl QuestionView {
    /// The answer in focus.
    pub fn focused(&self) -> Option<&str> {
        self.choices
            .iter()
            .find(|c| c.focused)
            .map(|c| c.label.as_str())
    }
}

/// One item of a menu, or one answer to a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChoiceView {
    /// Its name.
    pub label: String,
    /// Whether the cursor is on it.
    pub focused: bool,
}
