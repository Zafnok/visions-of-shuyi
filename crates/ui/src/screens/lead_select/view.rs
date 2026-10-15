//! What the lead screen shows, as plain data: no cells, glyphs, colours or
//! positions. The screen builds a [`LeadSelectView`] each frame
//! ([`LeadSelectScreen::view`](super::LeadSelectScreen::view)) and a skin
//! paints it ([`super::glyph::paint`] today); see ADR-0054.

use trpg_core::LeadGender;

use super::Row;

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

/// The typing box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypingView {
    /// The box's title.
    pub title: String,
    /// The name typed so far (without a cursor: that is the skin's).
    pub typed: String,
    /// What to do: type.
    pub hint: String,
}

/// The letter grid (controller players).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GridView {
    /// The grid's title.
    pub title: String,
    /// The name spelled so far (without a cursor: that is the skin's).
    pub spelled: String,
    /// The cells, row by row, as their labels.
    pub rows: Vec<Vec<String>>,
    /// The focused cell: row, column.
    pub focus: (usize, usize),
}
