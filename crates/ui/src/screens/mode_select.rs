//! The game mode screen, right after `New Game` (ticket 0801,
//! `docs/design/death-and-difficulty.md`): Classic or Casual, one line
//! explaining each.
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`ModeSelectView`]
//! ([`ModeSelectScreen::view`]); [`glyph::paint`] draws that view.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{ModeLineView, ModeSelectView};

use trpg_core::GameMode;

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::widgets::{Menu, MenuEvent, MenuItem};

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

/// Classic or Casual: Confirm picks the focused mode, Cancel goes back.
/// Pops either way; [`result`](Self::result) says which.
#[derive(Debug, Clone)]
pub struct ModeSelectScreen {
    menu: Menu,
    chosen: Option<GameMode>,
}

impl ModeSelectScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "mode_select";

    /// The screen with Classic focused, labelled in `ctx`'s language.
    pub fn new(ctx: &Ctx) -> Self {
        let items = MODES
            .iter()
            .map(|&(_, name, _)| MenuItem::new(ctx.text(name)));
        Self {
            menu: Menu::new(items.collect()),
            chosen: None,
        }
    }

    /// The mode picked, once the screen has popped; `None` if the player
    /// went back.
    pub fn result(&self) -> Option<GameMode> {
        self.chosen
    }

    /// The menu.
    pub fn menu(&self) -> &Menu {
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

impl Screen for ModeSelectScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            match self.menu.handle_with_sound(action, &mut ctx.audio) {
                Some(MenuEvent::Chosen(i)) => {
                    self.chosen = MODES.get(i).map(|&(mode, ..)| mode);
                    return Transition::Pop;
                }
                Some(MenuEvent::Cancelled) => return Transition::Pop,
                None => {}
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
