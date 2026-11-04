//! "Play with AI-generated voices?", asked once at `New Game` after the
//! familiarity question (ticket 0826,
//! `docs/design/voices-languages-and-script.md`, *Starting a game, and
//! saying it is AI*), only when voice clips were found. The answer sets
//! [`Settings::voices`](crate::Settings), which Options changes later.
//!
//! **The look is a skin** (ADR-0054): the view is the save prompt's (a
//! question, `Yes` and `No`, a help line) and so is the paint
//! ([`glyph::paint_prompt`]).

use crate::prelude::*;

use crate::glyph_buffer::GlyphBuffer;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};
use crate::screens::pick_one;
use crate::screens::save::{SavePromptView, glyph};
use crate::widgets::{Menu, MenuItem};

/// The key of the question in the language files ([`Ctx::text`]).
pub const QUESTION: &str = "voices_question.question";

/// The voices question, with `Yes` focused. Pops with the answer (a
/// `bool`, `true` for `Yes`); Cancel pops with nothing (back to the
/// familiarity question).
#[derive(Debug, Clone)]
pub struct VoicesQuestionScreen {
    menu: Menu<bool>,
}

impl VoicesQuestionScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "voices_question";

    /// The question with `Yes` focused, labelled in `ctx`'s language.
    pub fn new(ctx: &Ctx) -> Self {
        let items = [("voices_question.yes", true), ("voices_question.no", false)]
            .map(|(key, yes)| MenuItem::new(ctx.text(key)).with_value(yes))
            .to_vec();
        Self {
            menu: Menu::new(items),
        }
    }

    /// The screen as it is now, as plain data for a skin to paint: the
    /// question, `Yes` and `No`, and the help line.
    pub fn view(&self, ctx: &Ctx) -> SavePromptView {
        SavePromptView {
            question: ctx.text(QUESTION).to_owned(),
            menu: self.menu.view(),
            help: ctx.text_with("voices_question.help", &[]),
        }
    }
}

impl Screen<Shuyi> for VoicesQuestionScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<crate::input::Action>> {
        Some(crate::screens::actions::VOICES_QUESTION.to_vec())
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        pick_one(&mut self.menu, ctx, input)
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint_prompt(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
