//! What the Camp screen shows, as plain data: no cells, glyphs, colours or
//! positions. The screen builds a [`CampView`] each frame
//! ([`CampScreen::view`](super::CampScreen::view)) and a skin paints it
//! ([`super::glyph::paint`] today); see ADR-0054.

use crate::prelude::*;
use crate::widgets::MenuView;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampView {
    /// The screen's name.
    pub heading: String,
    /// The sections (Supports, Camp events, Break camp), the open one
    /// focused, `NEW` beside one with something new.
    pub sections: MenuView,
    /// Whether the focus is in the sections (else in the list).
    pub sections_focused: bool,
    /// The list's heading: what it lists.
    pub list_title: String,
    /// The list: the units with supports, one unit's conversations, or
    /// the camp events, `NEW` beside each new one. `None` when there is
    /// nothing to list (or nothing to list for Break camp).
    pub list: Option<MenuView>,
    /// What is said instead of an empty list.
    pub empty: String,
    /// The help line: what the keys do here.
    pub help: String,
}
