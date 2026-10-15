//! What the credits screen shows, as plain data: no cells, glyphs, colours
//! or positions. The screen builds a [`CreditsView`] each frame
//! ([`CreditsScreen::view`](super::CreditsScreen::view)) and a skin paints
//! it ([`super::glyph::paint`] today); see ADR-0054.

use crate::prelude::*;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditsView {
    /// What the screen is called.
    pub title: String,
    /// The rows of the list on screen, top to bottom. The list is wrapped
    /// to rows already ([`super::CreditsScreen`] scrolls by row), so a
    /// skin lays them out one to a line, at most a screenful.
    pub rows: Vec<CreditRowView>,
    /// Whether the list goes on above these rows.
    pub more_above: bool,
    /// Whether the list goes on below these rows.
    pub more_below: bool,
    /// The help line: what the keys do here.
    pub help: String,
}

/// What a row of the list is, which sets how it stands out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CreditRowKind {
    /// A group heading (or the blank row under it).
    Heading,
    /// An entry's title and author (or the blank row after an entry).
    Entry,
    /// An entry's license and source link.
    Detail,
}

/// One row of the list.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditRowView {
    /// What it is.
    pub kind: CreditRowKind,
    /// Its text; empty for a blank row.
    pub text: String,
}
