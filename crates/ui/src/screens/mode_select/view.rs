//! What the game mode screen shows, as plain data: no cells, glyphs,
//! colours or positions. The screen builds a [`ModeSelectView`] each frame
//! ([`ModeSelectScreen::view`](super::ModeSelectScreen::view)) and a skin
//! paints it ([`super::glyph::paint`] today); see ADR-0054.

use crate::prelude::*;
use crate::widgets::MenuView;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeSelectView {
    /// The question the screen asks.
    pub heading: String,
    /// The modes, one item each, in the order of [`lines`](Self::lines).
    pub menu: MenuView,
    /// The line explaining each mode, in the menu's order.
    pub lines: Vec<ModeLineView>,
    /// The help line: what the keys do here.
    pub help: String,
}

/// The line explaining one mode.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeLineView {
    /// The explanation.
    pub text: String,
    /// Whether it is the focused mode's (the one to read).
    pub focused: bool,
}
