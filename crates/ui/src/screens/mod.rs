//! The game's screens. Debug-only screens live in [`crate::debug`].

pub mod battle;
pub mod class_change;
pub mod credits;
pub mod dialogue;
pub mod familiarity;
pub mod game_over;
pub mod key_bindings;
pub mod layout_picker;
pub mod lead_select;
pub mod mode_select;
pub mod options;
pub mod playdate;
pub mod preparations;
pub mod profile;
pub mod results;
pub mod save;
pub mod title;

use crate::prelude::*;
pub use battle::BattleScreen;
pub use class_change::{ChangeKind, ClassChangeScreen};
pub use credits::CreditsScreen;
pub use dialogue::DialogueScreen;
pub use familiarity::FamiliarityScreen;
pub use game_over::{GameOverScreen, ToBeContinuedScreen};
pub use key_bindings::KeyBindingsScreen;
pub use layout_picker::LayoutPickerScreen;
pub use lead_select::LeadSelectScreen;
pub use mode_select::ModeSelectScreen;
pub use options::OptionsScreen;
pub use preparations::PreparationsScreen;
pub use profile::ProfileScreen;
pub use results::ResultsScreen;
pub use save::{SavePromptScreen, SlotPickerScreen};
pub use title::TitleScreen;

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, SkinSet};

/// What paints a view.
pub type Paint<V> = fn(&Ctx, &V, &mut GlyphBuffer);

/// A screen's skins, one for each [`SkinSet`] (ADR-0083): the glyph skin
/// on the PC, the one-bit skin on the Playdate (ticket 1045). A screen's
/// `draw` is `SKINS.paint(ctx, &self.view(ctx), buf);`.
pub struct Skins<V> {
    /// The PC's (`<screen>/glyph.rs`).
    pub glyph: Paint<V>,
    /// The Playdate's (`<screen>/playdate.rs`).
    pub playdate: Paint<V>,
}

impl<V> Skins<V> {
    /// The skin of `ctx`'s skin set.
    pub fn of(&self, ctx: &Ctx) -> Paint<V> {
        match ctx.game.skin_set {
            SkinSet::Glyph => self.glyph,
            SkinSet::Playdate => self.playdate,
        }
    }

    /// Paints `view` over `buf` with the skin of `ctx`'s skin set.
    pub fn paint(&self, ctx: &Ctx, view: &V, buf: &mut GlyphBuffer) {
        (self.of(ctx))(ctx, view, buf);
    }
}

pub(crate) use shuyi_ui::screens::{centre_x, print_centred};

/// The debug-menu hint, e.g. `F2 debug`: `None` unless debug tools are on
/// and the Debug action has a key ([`shuyi_ui::debug::hint`]).
pub(crate) fn debug_hint(ctx: &Ctx) -> Option<String> {
    shuyi_ui::debug::hint(ctx)
}

/// Prints `hint` (a screen's view holds the [`debug_hint`]) right-aligned
/// on row `y`, one cell in from the edge, unless those cells or the one
/// before them already hold text ([`shuyi_ui::debug::glyph::paint_hint`]).
pub(crate) use shuyi_ui::debug::glyph::paint_hint as paint_debug_hint;
