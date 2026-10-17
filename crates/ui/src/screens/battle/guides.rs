//! The Guides list (ticket 0458, `docs/design/tutorial.md`, *Guides*):
//! opened from the map menu, it lists every lesson (in their order) and
//! then every tip (in theirs) by title, with the highlighted one's text
//! beside it, its keys named from the player's keymap. A lesson's
//! checklist shows as plain lines. Every topic can be read, seen or not,
//! at every tutorial level. Back closes it.
//!
//! Like the rewind screen it lives inside the battle screen, over the
//! battle, which goes on under it untouched. [`GuidesScreen::view`] says
//! what it shows; the battle skin paints it (ADR-0054).

use crate::prelude::*;

use trpg_content::{Content, Lesson, Tip};

use super::view::GuidesView;
use crate::input::Action;
use crate::screen::Ctx;
use crate::tips::fill_placeholders;

/// Text key of the screen's title.
pub const TITLE: &str = "battle.guides.title";

/// Text key of the help line while it is open.
pub const HELP: &str = "battle.guides.help";

/// One topic of the list.
#[derive(Debug, Clone, Copy)]
enum Topic<'a> {
    /// A checklist lesson.
    Lesson(&'a Lesson),
    /// A one-time tip.
    Tip(&'a Tip),
}

/// Every topic, in list order: the lessons, then the tips, each in their
/// file's order.
fn topics(content: &Content) -> Vec<Topic<'_>> {
    let lessons = content.lessons.lessons.iter().map(Topic::Lesson);
    let tips = content.tips.tips.iter().map(Topic::Tip);
    lessons.chain(tips).collect()
}

/// What the battle screen should do after an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GuidesEffect {
    /// Nothing.
    None,
    /// The highlight moved.
    Moved,
    /// Close the list.
    Close,
}

/// The open Guides list: which topic is highlighted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuidesScreen {
    focus: usize,
    count: usize,
}

impl GuidesScreen {
    /// The list of `content`'s topics, on the first.
    pub fn new(content: &Content) -> Self {
        Self {
            focus: 0,
            count: topics(content).len(),
        }
    }

    /// The highlighted topic's place in the list.
    pub fn focus(&self) -> usize {
        self.focus
    }

    /// Handles one action: up and down move the highlight (wrapping),
    /// Cancel closes; anything else does nothing.
    pub fn step(&mut self, action: Action) -> GuidesEffect {
        let n = self.count;
        match action {
            Action::Cancel => GuidesEffect::Close,
            Action::CursorDown if n > 1 => {
                self.focus = (self.focus + 1) % n;
                GuidesEffect::Moved
            }
            Action::CursorUp if n > 1 => {
                self.focus = (self.focus + n - 1) % n;
                GuidesEffect::Moved
            }
            _ => GuidesEffect::None,
        }
    }

    /// What the list shows, in the player's language and with their keys.
    pub fn view(&self, ctx: &Ctx) -> GuidesView {
        let words = ctx.words();
        let keys = ctx.help_keys();
        let all = topics(&ctx.game.content);
        let titles = all
            .iter()
            .map(|t| match t {
                Topic::Lesson(l) => words.lesson_title(l).to_owned(),
                Topic::Tip(t) => words.tip_title(t).to_owned(),
            })
            .collect();
        let (heading, lines) = match all.get(self.focus) {
            Some(Topic::Lesson(l)) => {
                let text = fill_placeholders(words.lesson_text(l), keys);
                let mut lines: Vec<String> = text.lines().map(str::to_owned).collect();
                // The checklist, after a blank line, as plain lines.
                lines.push(String::new());
                lines.extend(
                    (0..l.items.len()).map(|i| fill_placeholders(words.lesson_item(l, i), keys)),
                );
                (words.lesson_title(l).to_owned(), lines)
            }
            Some(Topic::Tip(t)) => {
                let text = fill_placeholders(words.tip_text(t), keys);
                let lines = text.lines().map(str::to_owned).collect();
                (words.tip_title(t).to_owned(), lines)
            }
            None => (String::new(), Vec::new()),
        };
        GuidesView {
            title: ctx.text(TITLE).to_owned(),
            topics: titles,
            focus: self.focus,
            heading,
            lines,
        }
    }
}

#[cfg(test)]
mod tests;
