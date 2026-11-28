//! The game's screens. Debug-only screens live in [`crate::debug`].

pub mod actions;
pub mod battle;
pub mod camp;
pub mod chapter_card;
pub mod class_change;
pub mod credits;
pub mod dialogue;
pub mod familiarity;
pub mod files;
pub mod game_over;
pub mod key_bindings;
pub mod language_list;
pub mod layout_picker;
pub mod lead_select;
pub mod mode_select;
pub mod options;
pub mod playdate;
pub mod preparations;
pub mod profile;
pub mod results;
pub mod save;
pub mod save_import;
pub mod title;
pub mod town;
pub mod voices_question;
pub mod world_map;

use crate::prelude::*;
pub use battle::BattleScreen;
pub use camp::CampScreen;
pub use chapter_card::ChapterCardScreen;
pub use class_change::{ChangeKind, ClassChangeScreen};
pub use credits::CreditsScreen;
pub use dialogue::DialogueScreen;
pub use familiarity::FamiliarityScreen;
pub use game_over::{DemoEndScreen, GameOverScreen, ToBeContinuedScreen};
pub use key_bindings::KeyBindingsScreen;
pub use language_list::LanguageListScreen;
pub use layout_picker::LayoutPickerScreen;
pub use lead_select::LeadSelectScreen;
pub use mode_select::ModeSelectScreen;
pub use options::OptionsScreen;
pub use preparations::PreparationsScreen;
pub use profile::ProfileScreen;
pub use results::ResultsScreen;
pub use save::{SavePromptScreen, SlotPickerScreen};
pub use save_import::{ImportDone, ImportScreen};
pub use title::TitleScreen;
pub use town::TownScreen;
pub use voices_question::VoicesQuestionScreen;
pub use world_map::{WorldBattle, WorldMapScreen};

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, FrameInput, SkinSet, Transition};
use crate::widgets::{Menu, MenuEvent};

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

/// The start of `text` that fits in `w` cells as `widths` measure them,
/// cut with no mark (the engine's `cut` ends with `…`): a wide character
/// that would end past `w` is left out whole (ticket 0236).
pub(crate) fn clip(text: &str, w: usize, widths: &shuyi_assets::font::CellWidths) -> String {
    let mut used = 0;
    text.chars()
        .take_while(|&c| {
            used += widths.of_char(c);
            used <= w
        })
        .collect()
}

/// `text` and as many spaces after it as make it `w` cells as `widths`
/// measure them, none if it is wider: a column padded in cells, where
/// `format!`'s width counts characters (ticket 0263).
pub(crate) fn pad_cells(text: &str, w: usize, widths: &shuyi_assets::font::CellWidths) -> String {
    let used = shuyi_assets::lang::text_width(text, widths);
    let mut padded = String::from(text);
    padded.extend(core::iter::repeat_n(' ', w.saturating_sub(used)));
    padded
}

/// The cells of `line` before `marker` as `widths` measure them: where a
/// column built with [`pad_cells`] starts, for its tests (ticket 0263).
#[cfg(test)]
pub(crate) fn cells_before(
    line: &str,
    marker: &str,
    widths: &shuyi_assets::font::CellWidths,
) -> usize {
    let at = line
        .find(marker)
        .unwrap_or_else(|| panic!("{marker:?} in {line:?}"));
    shuyi_assets::lang::text_width(&line[..at], widths)
}

/// A pick-one screen's frame: `input` goes to `menu`, and the item chosen
/// pops the screen with its value for the screen below; Cancel pops it
/// with nothing.
pub(crate) fn pick_one<T: Clone + 'static>(
    menu: &mut Menu<T>,
    ctx: &mut Ctx,
    input: &FrameInput,
) -> Transition {
    for &action in &input.actions {
        match menu.handle_with_sound(action, &mut ctx.audio) {
            Some(MenuEvent::Chosen(v)) => return Transition::pop_with(v),
            Some(MenuEvent::Cancelled) => return Transition::Pop,
            None => {}
        }
    }
    Transition::None
}

/// What `transition` pops with, as a `T`: for the tests of a screen that
/// pops with its answer.
#[cfg(test)]
pub(crate) fn popped_with<T: 'static>(transition: Transition) -> Option<T> {
    match transition {
        Transition::PopWith(result) => result.take(),
        _ => None,
    }
}

/// The debug-menu hint, e.g. `F2 debug`: `None` unless debug tools are on
/// and the Debug action has a key ([`shuyi_ui::debug::hint`]).
pub(crate) fn debug_hint(ctx: &Ctx) -> Option<String> {
    shuyi_ui::debug::hint(ctx)
}

/// Prints `hint` (a screen's view holds the [`debug_hint`]) right-aligned
/// on row `y`, one cell in from the edge, unless those cells or the one
/// before them already hold text ([`shuyi_ui::debug::glyph::paint_hint`]).
pub(crate) use shuyi_ui::debug::glyph::paint_hint as paint_debug_hint;

#[cfg(test)]
mod clip_tests {
    use super::{clip, pad_cells};
    use shuyi_platform::prelude::text::CellWidths;

    /// `clip` keeps what fits in cells: a wide character counts two and is
    /// never cut in half (ticket 0236).
    #[test]
    fn clip_counts_cells_and_keeps_wide_characters_whole() {
        let narrow = CellWidths::NARROW;
        assert_eq!(clip("Steel Sword", 5, &narrow), "Steel");
        assert_eq!(clip("Iron", 10, &narrow), "Iron");
        let wide = CellWidths::wide(['姝', '怡']);
        assert_eq!(clip("姝怡A", 4, &wide), "姝怡");
        assert_eq!(clip("姝怡A", 3, &wide), "姝");
        assert_eq!(clip("A姝怡", 2, &wide), "A");
        assert_eq!(clip("姝", 0, &wide), "");
    }

    /// `pad_cells` pads to a width in cells: a wide character takes two of
    /// them, and text already wider gets no spaces (ticket 0263).
    #[test]
    fn pad_cells_pads_by_cells() {
        let wide = CellWidths::wide(['姝', '怡']);
        assert_eq!(pad_cells("姝怡", 6, &wide), "姝怡  ");
        assert_eq!(pad_cells("姝怡", 6, &CellWidths::NARROW), "姝怡    ");
        assert_eq!(pad_cells("Iron", 6, &wide), "Iron  ");
        assert_eq!(pad_cells("姝怡A", 4, &wide), "姝怡A");
        assert_eq!(pad_cells("", 2, &wide), "  ");
    }
}
