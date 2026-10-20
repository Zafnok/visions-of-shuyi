//! The screens after a battle's end (ticket 0801): Game Over after a
//! defeat (`Retry Battle` / `Title`, `docs/design/death-and-difficulty.md`) and
//! "To be continued" after the last chapter there is.
//!
//! Game Over opens with an animation (ticket 0830;
//! `docs/design/look-and-feel.md`, *Game Over screen*): the lost battle
//! fades to black, the logo and the words fade in, and then the menu is
//! there. Any press jumps to its end and does nothing else.
//!
//! **The look is a skin** (ADR-0054): this module decides what the screens
//! show and do, and says it as a [`GameOverView`] and a
//! [`ToBeContinuedView`]; [`glyph`] paints them.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{GameOverView, ToBeContinuedView};

use view::FULL;

use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::pick_one;
use crate::widgets::{Menu, MenuItem};

/// The key of Game Over's heading in the language files ([`Ctx::text`]).
pub const GAME_OVER: &str = "game_over.heading";
/// The key of Game Over's `Rewind` item, first while a charge is left.
const REWIND: &str = "game_over.rewind";
/// The keys of Game Over's other items: retry, then back to the title.
const ITEMS: [&str; 2] = ["game_over.retry", "game_over.title"];
/// The key of the last chapter's closing words.
pub const TO_BE_CONTINUED: &str = "to_be_continued.heading";

/// How long Game Over's opening animation takes, in seconds (Nick,
/// 2026-10-13: the short one).
pub const ANIMATION_SECONDS: f32 = 2.0;

/// What the player picked on Game Over.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameOverChoice {
    /// Back to the lost battle with the rewind screen open (0822).
    Rewind,
    /// Start the battle again from its first turn, every rewind charge back.
    Retry,
    /// Back to the title screen.
    Title,
}

/// `GAME OVER` and a `Rewind` (while a charge is left) / `Retry Battle` /
/// `Title` menu, after the opening animation, over the lost battle. Pops
/// with the [`GameOverChoice`] once one is chosen; there is nothing to back
/// out of.
#[derive(Debug, Clone)]
pub struct GameOverScreen {
    menu: Menu<GameOverChoice>,
    /// Seconds of the opening animation played, up to
    /// [`ANIMATION_SECONDS`].
    elapsed: f32,
}

impl GameOverScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "game_over";

    /// The screen labelled in `ctx`'s language, with `Rewind` first and
    /// focused when `can_rewind` (the lost battle has a charge left), else
    /// `Retry Battle` focused.
    pub fn new(ctx: &Ctx, can_rewind: bool) -> Self {
        let rewind = can_rewind.then_some((REWIND, GameOverChoice::Rewind));
        let [retry, title] = ITEMS;
        let all = [
            (retry, GameOverChoice::Retry),
            (title, GameOverChoice::Title),
        ];
        let items = rewind.into_iter().chain(all);
        let items = items.map(|(k, c)| MenuItem::new(ctx.text(k)).with_value(c));
        Self {
            menu: Menu::new(items.collect()).without_cancel(),
            elapsed: 0.0,
        }
    }

    /// Jumps to the end of the opening animation.
    pub fn skip(&mut self) {
        self.elapsed = ANIMATION_SECONDS;
    }

    /// Whether the opening animation is over: the menu is there.
    pub fn finished(&self) -> bool {
        self.elapsed >= ANIMATION_SECONDS
    }

    /// How far along the opening animation is, 0 to [`FULL`].
    pub fn progress(&self) -> u8 {
        let part = (self.elapsed / ANIMATION_SECONDS).clamp(0.0, 1.0);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)] // 0 to 255
        let progress = (part * f32::from(FULL)).round() as u8;
        progress
    }

    /// Whether the lost battle still shows behind the screen: the screen
    /// is an overlay over it while it does.
    pub fn shows_battle(&self) -> bool {
        glyph::battle_brightness(self.progress()) > 0.0
    }

    /// The bottom help line.
    pub fn help(ctx: &Ctx) -> String {
        ctx.text_with("game_over.help", &[])
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): the heading and how far along the animation
    /// is; once it is over, the menu and the help line too.
    pub fn view(&self, ctx: &Ctx) -> GameOverView {
        let finished = self.finished();
        GameOverView {
            heading: ctx.text(GAME_OVER).to_owned(),
            progress: self.progress(),
            menu: finished.then(|| self.menu.view()),
            help: finished.then(|| Self::help(ctx)),
        }
    }
}

impl Screen<Shuyi> for GameOverScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        if !self.finished() {
            // A press jumps to the end, and is used up by that.
            if super::title::pressed(input) {
                self.skip();
            } else {
                self.elapsed = (self.elapsed + input.dt()).min(ANIMATION_SECONDS);
            }
            return Transition::None;
        }
        pick_one(&mut self.menu, ctx, input)
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn is_overlay(&self) -> bool {
        self.shows_battle()
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
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

impl Screen<Shuyi> for ToBeContinuedScreen {
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

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
