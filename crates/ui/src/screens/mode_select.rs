//! The game mode screen, right after `New Game` (ticket 0801,
//! `docs/design/death-and-difficulty.md`): Classic or Casual, one line
//! explaining each.
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`ModeSelectView`]
//! ([`ModeSelectScreen::view`]); [`glyph::paint`] draws that view, or on
//! the Playdate [`playdate::paint`] ([`SKINS`], ADR-0083).

pub mod glyph;
pub mod playdate;
pub mod view;

use crate::prelude::*;
pub use view::{ModeLineView, ModeSelectView};

use trpg_core::GameMode;

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::{Skins, pick_one};
use crate::widgets::{Menu, MenuItem};

/// The screen's skins: the PC's and the Playdate's (also the familiarity
/// screen's).
pub const SKINS: Skins<ModeSelectView> = Skins {
    glyph: glyph::paint,
    playdate: playdate::paint,
};

/// The key of the heading in the language files ([`Ctx::text`]).
pub const HEADING: &str = "mode_select.heading";

/// Each mode, the key of its name and of the line explaining it.
pub const MODES: [(GameMode, &str, &str); 2] = [
    (
        GameMode::Classic,
        "mode_select.classic",
        "mode_select.classic_line",
    ),
    (
        GameMode::Casual,
        "mode_select.casual",
        "mode_select.casual_line",
    ),
];

/// Classic or Casual: Confirm pops with the focused mode (a
/// [`GameMode`]), Cancel pops with nothing (back).
#[derive(Debug, Clone)]
pub struct ModeSelectScreen {
    menu: Menu<GameMode>,
}

impl ModeSelectScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "mode_select";

    /// The screen with Classic focused, labelled in `ctx`'s language.
    pub fn new(ctx: &Ctx) -> Self {
        let items = MODES
            .iter()
            .map(|&(mode, name, _)| MenuItem::new(ctx.text(name)).with_value(mode));
        Self {
            menu: Menu::new(items.collect()),
        }
    }

    /// The menu.
    pub fn menu(&self) -> &Menu<GameMode> {
        &self.menu
    }

    /// The bottom help line.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("mode_select.help", &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the heading, the menu, the line explaining each
    /// mode (the focused mode's is the one to read) and the help line.
    pub fn view(&self, ctx: &Ctx) -> ModeSelectView {
        let focus = self.menu.focus();
        ModeSelectView {
            heading: ctx.text(HEADING).to_owned(),
            menu: self.menu.view(),
            lines: MODES
                .iter()
                .enumerate()
                .map(|(i, &(_, _, line))| ModeLineView {
                    text: ctx.text(line).to_owned(),
                    focused: i == focus,
                })
                .collect(),
            help: Self::help(ctx),
        }
    }
}

impl Screen<Shuyi> for ModeSelectScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::MODE_SELECT.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        pick_one(&mut self.menu, ctx, input)
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        SKINS.paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
