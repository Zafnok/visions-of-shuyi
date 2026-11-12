//! What the title screen shows, as plain data: no cells, glyphs, colours
//! or positions. The screen builds a [`TitleView`] each frame
//! ([`TitleScreen::view`](super::TitleScreen::view)) and a skin paints it
//! ([`super::glyph::paint`] today); see ADR-0054.

use crate::prelude::*;
use alloc::rc::Rc;

use trpg_content::ImageId;

use crate::cinematic::CinematicPlayer;
use crate::tips::view::FaceView;
use crate::widgets::MenuView;

/// As solid or as bright as can be ([`TitleView::menu_opacity`],
/// [`TitleView::picture`], [`CinematicView::brightness`]).
pub const FULL: u8 = 255;

/// The intro cinematic as the title shows it this frame (ticket 0819).
#[derive(Debug, Clone, PartialEq)]
pub struct CinematicView {
    /// What plays; it draws any moment of itself.
    pub player: Rc<CinematicPlayer>,
    /// The moment, in milliseconds from the cinematic's start.
    pub t_ms: u32,
    /// How bright it is, up to [`FULL`]: 0 is black.
    pub brightness: u8,
}

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct TitleView {
    /// The cinematic behind everything; `None` while the title shows its
    /// own picture alone.
    pub cinematic: Option<CinematicView>,
    /// How solid the title's own picture (the backdrop, the sparks, the
    /// logo) is, up to [`FULL`], over the cinematic if one shows, else
    /// over black: it goes to black before the cinematic (ticket 0843).
    pub picture: u8,
    /// How solid the menu, its notice, the prompt and the help line are,
    /// up to [`FULL`]: they go to black with the title's picture before
    /// the cinematic and fade in on its logo.
    pub menu_opacity: u8,
    /// The picture behind the logo this launch; `None` when the game has
    /// none.
    pub backdrop: Option<ImageId>,
    /// The sparks drifting up the screen, as how long the title has been
    /// shown, in milliseconds: that alone places them. `None`: no sparks
    /// ([`Ctx::idle_motion`](crate::screen::Ctx::idle_motion) is off).
    pub sparks: Option<u32>,
    /// The "press any key" prompt, while the screen waits for a key; the
    /// menu is not shown meanwhile.
    pub prompt: Option<String>,
    /// The menu, once the prompt is over.
    pub menu: Option<MenuView>,
    /// Why `Continue` couldn't, until the next key.
    pub notice: Option<String>,
    /// The guide's note beside the menu, shown and faded with the menu:
    /// while the profile in use was never set up (ticket 0842); else, where
    /// saves can be exported, while the cursor is on `Load Game` or
    /// `Profile` (0874). It takes no keys.
    pub note: Option<NoteView>,
    /// The help line: what the keys do here (not shown with the prompt).
    pub help: Option<String>,
    /// The hint at the debug menu's key, in a build with debug tools.
    pub debug_hint: Option<String>,
}

/// The guide's note beside the title's menu (ticket 0842): a small tip box
/// that takes no keys.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoteView {
    /// The box's title: the guide's name.
    pub title: String,
    /// The guide's face, if the game has the guide's portrait.
    pub face: Option<FaceView>,
    /// The first line.
    pub heading: String,
    /// The text, one entry per paragraph; the skin wraps it.
    pub lines: Vec<String>,
}
