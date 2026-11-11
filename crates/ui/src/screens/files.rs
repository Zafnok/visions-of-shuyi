//! What the Profile screen and the Load Game list share about saves
//! outside the game (ticket 0872, `docs/design/options.md`, *Saves outside
//! the game*): their two keys (export, open the folder), the place line's
//! view, and its glyph look ([`glyph`]). What a file holds and how it is
//! offered is [`crate::save_files`].

pub mod glyph;

use crate::prelude::*;

use crate::input::Action;
use crate::save_files::{self, Exported};
use crate::screen::Ctx;

pub use glyph::paint_place;

/// The keys both screens add to their own: the Unit info key exports the
/// row under the cursor (on menus it has no other use), and the world
/// map's Find key opens the folder of the saves (on a desktop). *The
/// agent's starting rule* (`options.md`).
pub const ACTIONS: &[Action] = &[Action::Info, Action::WorldFind];

/// The keys that move the cursor, which put away the last export's notice.
pub const MOVES: &[Action] = &[Action::CursorUp, Action::CursorDown];

/// Where the saves are, as the line under a screen's panel shows it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlaceView {
    /// The words before the path.
    pub label: String,
    /// The folder's whole path; the skin shortens it from the left.
    pub path: String,
}

/// The place line on this platform: on a desktop only (Nick: the web's
/// export is a download, and its storage nothing a player reaches).
pub fn place_view(ctx: &Ctx) -> Option<PlaceView> {
    let path = save_files::place(&ctx.platform)?;
    Some(PlaceView {
        label: ctx.text(save_files::PLACE).to_owned(),
        path: path.to_owned(),
    })
}

/// Opens the folder of the saves in the system's file manager (the
/// exports folder once a file was written there): on a desktop only.
pub fn open_folder(ctx: &mut Ctx, shown: Option<&Exported>) {
    if save_files::place(&ctx.platform).is_some() {
        ctx.open_storage_folder(Exported::folder(shown));
    }
}
