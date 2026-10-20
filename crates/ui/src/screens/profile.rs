//! The Profile screen (ticket 0837), opened from the title's `Profile`
//! item (`docs/design/options.md`, *Where options are kept*): Shuyi
//! Engine's profiles screen (`shuyi_ui::screens::profiles`, ticket 1164),
//! re-exported here. Adding a profile opens the first-launch layout picker
//! for it ([`layout_picker::first`]). Its look is the engine's glyph skin
//! ([`glyph::paint`], ADR-0054), or on the Playdate [`playdate::paint`]
//! ([`SKINS`], ADR-0083).

pub mod playdate;

use crate::prelude::*;
pub use shuyi_ui::screens::profiles::*;

use super::layout_picker;
use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, Screen, Shuyi};
use crate::screens::Skins;

/// The Profile screen for this game
/// ([`shuyi_ui::screens::profiles::ProfilesScreen`]).
pub type ProfileScreen = ProfilesScreen<Shuyi>;

/// The screen focused on the profile in use, opening the layout picker
/// once a profile is added, painted by the skin of the context's skin set.
pub fn screen(ctx: &Ctx) -> ProfileScreen {
    ProfileScreen::new(ctx)
        .opening_after_add(layout_picker_for_new)
        .painted_by(paint)
}

/// The first-launch layout picker, for the profile just added.
fn layout_picker_for_new(ctx: &mut Ctx) -> Box<dyn Screen<Shuyi>> {
    Box::new(layout_picker::first(ctx))
}

/// The screen's skins: the engine's glyph skin on the PC, and the
/// Playdate's.
pub const SKINS: Skins<ProfilesView> = Skins {
    glyph: glyph::paint::<Shuyi>,
    playdate: playdate::paint,
};

/// Paints `view` with the skin of `ctx`'s skin set ([`SKINS`]).
fn paint(ctx: &Ctx, view: &ProfilesView, buf: &mut GlyphBuffer) {
    SKINS.paint(ctx, view, buf);
}

#[cfg(test)]
mod tests;
