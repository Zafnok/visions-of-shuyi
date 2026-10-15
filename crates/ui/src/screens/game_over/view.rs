//! What the Game Over and "To be continued" screens show, as plain data:
//! no cells, glyphs, colours or positions. Each screen builds its view
//! each frame ([`GameOverScreen::view`](super::GameOverScreen::view),
//! [`ToBeContinuedScreen::view`](super::ToBeContinuedScreen::view)) and a
//! skin paints it ([`super::glyph`] today); see ADR-0054.

use crate::prelude::*;
use crate::widgets::MenuView;

pub use crate::screens::title::view::FULL;

/// Game Over, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GameOverView {
    /// The words `GAME OVER`.
    pub heading: String,
    /// How far along the opening animation is, 0 to [`FULL`] (over).
    pub progress: u8,
    /// The ways on: rewind (while a charge is left), retry the battle, or
    /// back to the title. `None` until the animation is over.
    pub menu: Option<MenuView>,
    /// The help line: what the keys do here. `None` until the animation
    /// is over.
    pub help: Option<String>,
}

/// "To be continued", as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToBeContinuedView {
    /// The closing words.
    pub heading: String,
    /// The help line: what the keys do here.
    pub help: String,
}
