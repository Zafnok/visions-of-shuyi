//! What the lead screen shows, as plain data: no cells, glyphs, colours or
//! positions. The screen builds a [`LeadSelectView`] each frame
//! ([`LeadSelectScreen::view`](super::LeadSelectScreen::view)) and a skin
//! paints it ([`super::glyph::paint`] today); see ADR-0054.

use crate::prelude::*;
use trpg_core::LeadGender;

use super::Row;
pub use crate::widgets::name_entry::{GridView, TypingView};

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeadSelectView {
    /// What the screen is called.
    pub heading: String,
    /// The two genders, male then female.
    pub genders: Vec<GenderView>,
    /// The row the cursor is on; `None` while the typing box or the letter
    /// grid is open (nothing behind them is lit then).
    pub cursor: Option<Row>,
    /// The name row.
    pub name_row: NameRowView,
    /// The Start button's text.
    pub start: String,
    /// The typing box, while open.
    pub typing: Option<TypingView>,
    /// The letter grid, while open.
    pub grid: Option<GridView>,
    /// The help line: what the keys do here.
    pub help: String,
}

/// One gender, with its portrait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GenderView {
    /// Which gender.
    pub gender: LeadGender,
    /// Its name, e.g. `Male`.
    pub label: String,
    /// The id of its portrait in the content's portraits.
    pub portrait: &'static str,
    /// The portrait's expression shown.
    pub expression: &'static str,
    /// Whether it is the gender chosen.
    pub chosen: bool,
}

/// The name row: its label, the first name and the fixed family name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NameRowView {
    /// The row's label.
    pub label: String,
    /// The first name so far (the typing box's or the grid's, while open).
    pub first: String,
    /// The family name.
    pub family: String,
}
