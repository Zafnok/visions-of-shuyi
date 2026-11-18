//! The chapter title card (ticket 0812, `docs/design/look-and-feel.md`,
//! *The chapter title card*): before a chapter's first scene, `Chapter 1`
//! and the chapter's name between two rules on black, the lines in Cinzel
//! ([`trpg_content::lettering`]). It closes by itself after [`CARD_S`]
//! seconds, or on Confirm or Cancel. The flow pushes it ([`crate::flow`]).
//!
//! **The look is a skin** (ADR-0054, ADR-0083): the view is the two lines;
//! the PC's skin draws them in Cinzel ([`glyph`]), the Playdate's in the
//! cell font ([`playdate`]).

use crate::prelude::*;

use trpg_content::ChapterDef;

use super::Skins;
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;
use crate::screen::{Ctx, FrameInput, Screen, Shuyi, Transition};

pub mod glyph;
pub mod playdate;

/// How long the card stays up unless Confirm or Cancel closes it, in
/// seconds. *Tunable* (the agent's starting rule, 0812).
pub const CARD_S: f32 = 3.0;

/// What splits a chapter's title into its number and its name.
pub const SPLIT: &str = ": ";

/// The screen's skins: the PC's and the Playdate's.
pub const SKINS: Skins<ChapterCardView> = Skins {
    glyph: glyph::paint,
    playdate: playdate::paint,
};

/// The card as plain data, for a skin to paint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ChapterCardView {
    /// The line above the name (`Chapter 1`); empty for a title with no
    /// [`SPLIT`].
    pub line: String,
    /// The chapter's name (`The Letter`), or its whole title.
    pub heading: String,
}

impl ChapterCardView {
    /// The view of chapter title `title`: what comes before its first
    /// [`SPLIT`] above, the rest below; a title without one is the name
    /// alone.
    pub fn of_title(title: &str) -> Self {
        let (line, heading) = title.split_once(SPLIT).unwrap_or(("", title));
        Self {
            line: line.to_owned(),
            heading: heading.to_owned(),
        }
    }
}

/// The chapter title card. Pops with nothing when it closes.
#[derive(Debug, Clone)]
pub struct ChapterCardScreen {
    view: ChapterCardView,
    /// Seconds it has been up.
    shown_s: f32,
}

impl ChapterCardScreen {
    /// Name reported by [`Screen::name`].
    pub const NAME: &'static str = "chapter_card";

    /// The card of `chapter`, its title in `ctx`'s language.
    pub fn new(ctx: &Ctx, chapter: &ChapterDef) -> Self {
        Self {
            view: ChapterCardView::of_title(ctx.words().chapter_title(chapter)),
            shown_s: 0.0,
        }
    }

    /// The screen as plain data.
    pub fn view(&self) -> &ChapterCardView {
        &self.view
    }
}

impl Screen<Shuyi> for ChapterCardScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn actions(&self) -> Option<Vec<Action>> {
        Some(crate::screens::actions::CHAPTER_CARD.to_vec())
    }

    fn update(&mut self, _ctx: &mut Ctx, input: &FrameInput) -> Transition {
        self.shown_s += input.dt();
        let closed = input
            .actions
            .iter()
            .any(|a| matches!(a, Action::Confirm | Action::Cancel));
        if closed || self.shown_s >= CARD_S {
            return Transition::Pop;
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        SKINS.paint(ctx, &self.view, buf);
    }

    fn as_any(&self) -> Option<&dyn core::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
