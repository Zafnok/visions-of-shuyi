//! What the title screen shows, as plain data: no cells, glyphs, colours
//! or positions. The screen builds a [`TitleView`] each frame
//! ([`TitleScreen::view`](super::TitleScreen::view)) and a skin paints it
//! ([`super::glyph::paint`] today); see ADR-0054.

use crate::widgets::MenuView;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TitleView {
    /// The game's name.
    pub title: String,
    /// The line under it.
    pub subtitle: String,
    /// The "press any key" prompt, while the screen waits for a key; the
    /// menu is not shown meanwhile.
    pub prompt: Option<String>,
    /// The menu, once the prompt is over.
    pub menu: Option<MenuView>,
    /// Why `Continue` couldn't, until the next key.
    pub notice: Option<String>,
    /// The help line: what the keys do here (not shown with the prompt).
    pub help: Option<String>,
    /// The hint at the debug menu's key, in a build with debug tools.
    pub debug_hint: Option<String>,
}
