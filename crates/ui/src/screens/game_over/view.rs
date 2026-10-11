//! What the Game Over and "To be continued" screens show, as plain data:
//! no cells, glyphs, colours or positions. Each screen builds its view
//! each frame ([`GameOverScreen::view`](super::GameOverScreen::view),
//! [`ToBeContinuedScreen::view`](super::ToBeContinuedScreen::view)) and a
//! skin paints it ([`super::glyph`] today); see ADR-0054.

use crate::widgets::MenuView;

/// Game Over, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOverView {
    /// The words `GAME OVER`.
    pub heading: String,
    /// The two ways on: retry the battle, or back to the title.
    pub menu: MenuView,
    /// The help line: what the keys do here.
    pub help: String,
}

/// "To be continued", as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToBeContinuedView {
    /// The closing words.
    pub heading: String,
    /// The help line: what the keys do here.
    pub help: String,
}
