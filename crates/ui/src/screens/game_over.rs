//! The screens after a battle's end (ticket 0801): Game Over after a
//! defeat (`Retry Battle` / `Title`, `docs/design/death-and-difficulty.md`) and
//! "To be continued" after the last chapter there is.
//!
//! **The look is a skin** (ADR-0054): this module decides what the screens
//! show and do, and says it as a [`GameOverView`] and a
//! [`ToBeContinuedView`]; [`glyph`] paints them.

pub mod glyph;
pub mod view;

pub use view::{GameOverView, ToBeContinuedView};

use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::widgets::{Menu, MenuEvent, MenuItem};

/// The key of Game Over's heading in the language files ([`Ctx::text`]).
pub const GAME_OVER: &str = "game_over.heading";
/// The keys of Game Over's items: retry, then back to the title.
const ITEMS: [&str; 2] = ["game_over.retry", "game_over.title"];
/// The key of the last chapter's closing words.
pub const TO_BE_CONTINUED: &str = "to_be_continued.heading";

/// What the player picked on Game Over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverChoice {
    /// Start the battle again from its first turn, every rewind charge back.
    Retry,
    /// Back to the title screen.
    Title,
}

/// `GAME OVER` and a `Retry Battle` / `Title` menu. Pops once one is chosen
/// ([`result`](Self::result)); there is nothing to back out of.
#[derive(Debug, Clone)]
pub struct GameOverScreen {
    menu: Menu,
    chosen: Option<GameOverChoice>,
}

impl GameOverScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "game_over";

    /// The screen with `Retry Battle` focused, labelled in `ctx`'s
    /// language.
    pub fn new(ctx: &Ctx) -> Self {
        let items = ITEMS.map(|key| MenuItem::new(ctx.text(key))).to_vec();
        Self {
            menu: Menu::new(items).without_cancel(),
            chosen: None,
        }
    }

    /// The choice, once the screen has popped.
    pub fn result(&self) -> Option<GameOverChoice> {
        self.chosen
    }

    /// The bottom help line.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("game_over.help", &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the heading, the menu and the help line.
    pub fn view(&self, ctx: &Ctx) -> GameOverView {
        GameOverView {
            heading: ctx.text(GAME_OVER).to_owned(),
            menu: self.menu.view(),
            help: Self::help(ctx),
        }
    }
}

impl Screen for GameOverScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        for &action in &input.actions {
            if let Some(MenuEvent::Chosen(i)) = self.menu.handle_with_sound(action, &mut ctx.audio)
            {
                self.chosen = Some(if i == 0 {
                    GameOverChoice::Retry
                } else {
                    GameOverChoice::Title
                });
                return Transition::Pop;
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

/// "To be continued...": Confirm goes back to the title (pops).
#[derive(Debug, Clone, Copy, Default)]
pub struct ToBeContinuedScreen;

impl ToBeContinuedScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "to_be_continued";

    /// The bottom help line.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("to_be_continued.help", &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint_to_be_continued`]): the closing words and the help
    /// line.
    pub fn view(&self, ctx: &Ctx) -> ToBeContinuedView {
        ToBeContinuedView {
            heading: ctx.text(TO_BE_CONTINUED).to_owned(),
            help: Self::help(ctx),
        }
    }
}

impl Screen for ToBeContinuedScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if input.actions.contains(&Action::Confirm) {
            ctx.audio.menu(crate::audio::MenuSound::Select);
            return Transition::Pop;
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint_to_be_continued(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
