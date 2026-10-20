//! The familiarity screen, right after the game mode at `New Game`
//! (ticket 0835, `docs/design/tutorial.md`, *How familiar are you?*): not
//! familiar, familiar with other strategy RPGs, or familiar with this game,
//! one line explaining each. The answer is the campaign's
//! [`TutorialLevel`].
//!
//! **The look is a skin** (ADR-0054): this module decides what the screen
//! shows and does, and says it as a [`FamiliarityView`]
//! ([`FamiliarityScreen::view`]); the screen has the mode screen's shape (a
//! heading, a menu and a line explaining each choice), so the mode screen's
//! skins ([`SKINS`]) draw it.

use crate::prelude::*;
use trpg_core::TutorialLevel;

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::mode_select::SKINS;
use crate::screens::mode_select::view::{ModeLineView, ModeSelectView};
use crate::screens::pick_one;
use crate::widgets::{Menu, MenuItem};

/// What the familiarity screen shows: the mode screen's view (a heading,
/// the menu, a line explaining each choice and the help line).
pub type FamiliarityView = ModeSelectView;

/// The key of the heading in the language files ([`Ctx::text`]).
pub const HEADING: &str = "familiarity.heading";

/// Each answer, the level it picks, and the key of its name and of the
/// line explaining it.
pub const LEVELS: [(TutorialLevel, &str, &str); 3] = [
    (
        TutorialLevel::Full,
        "familiarity.full",
        "familiarity.full_line",
    ),
    (
        TutorialLevel::Short,
        "familiarity.short",
        "familiarity.short_line",
    ),
    (
        TutorialLevel::None,
        "familiarity.none",
        "familiarity.none_line",
    ),
];

/// How familiar the player is: Confirm pops with the focused answer's
/// [`TutorialLevel`], Cancel pops with nothing (back).
#[derive(Debug, Clone)]
pub struct FamiliarityScreen {
    menu: Menu<TutorialLevel>,
}

impl FamiliarityScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "familiarity";

    /// The screen with "Not familiar" focused, labelled in `ctx`'s
    /// language.
    pub fn new(ctx: &Ctx) -> Self {
        let items = LEVELS
            .iter()
            .map(|&(level, name, _)| MenuItem::new(ctx.text(name)).with_value(level));
        Self {
            menu: Menu::new(items.collect()),
        }
    }

    /// The bottom help line.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("familiarity.help", &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`SKINS`]): the heading, the menu, the line explaining each
    /// answer (the focused answer's is the one to read) and the help line.
    pub fn view(&self, ctx: &Ctx) -> FamiliarityView {
        let focus = self.menu.focus();
        FamiliarityView {
            heading: ctx.text(HEADING).to_owned(),
            menu: self.menu.view(),
            lines: LEVELS
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

impl Screen<Shuyi> for FamiliarityScreen {
    fn name(&self) -> &'static str {
        Self::NAME
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
