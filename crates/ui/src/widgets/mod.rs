//! Reusable pieces screens are built from. The menu and word wrap are
//! Shuyi Engine's (`shuyi_ui::widgets`, ticket 1112), re-exported here
//! under their old names. Typing a name is the engine's too
//! (`shuyi_ui::widgets::name_entry`, ticket 1164), with this game's
//! one-bit skin ([`name_entry::playdate`]).

pub mod help;
pub mod menu;

/// Typing a name: Shuyi Engine's name boxes, and their Playdate look.
pub mod name_entry {
    pub use shuyi_ui::widgets::name_entry::*;

    pub mod playdate;
}

pub use menu::{Menu, MenuEvent, MenuItem, MenuItemView, MenuView};
pub use shuyi_ui::widgets::{AnswerView, Ask, AskView, ListCursor, ask, word_wrap};

use crate::prelude::*;
use shuyi_assets::font::CellWidths;
use shuyi_assets::lang::text_width;
use shuyi_assets::text::NO_BREAK_SPACE;

// engine-friction: engine#2291, moves (game 1410 deletes it)
/// `text` wrapped by [`word_wrap`] into lines of at most `width` cells as
/// `widths` measure them, every line after the first starting with
/// `indent` (and wrapped in the room left after it). No text, no lines.
pub fn wrap_indented(text: &str, width: usize, indent: &str, widths: &CellWidths) -> Vec<String> {
    let Some(first) = word_wrap(text, width, widths).into_iter().next() else {
        return Vec::new();
    };
    let rest = after_line(text, &first);
    let room = width.saturating_sub(text_width(indent, widths));
    let rest = word_wrap(rest, room, widths);
    let mut lines = Vec::with_capacity(rest.len() + 1);
    lines.push(first);
    lines.extend(rest.into_iter().map(|line| format!("{indent}{line}")));
    lines
}

/// Whether [`word_wrap`] breaks at `c`, dropping it: whitespace, but not a
/// no-break space.
pub(crate) fn breaks_at(c: char) -> bool {
    c.is_whitespace() && c != NO_BREAK_SPACE
}

// engine-friction: engine#2291, moves (game 1410 deletes it)
/// What is left of `text` after `line`, the first line [`word_wrap`] made
/// of it: past as many of its characters that aren't a break. (A line
/// holds `text`'s characters in order, with one space where it broke.)
fn after_line<'a>(text: &'a str, line: &str) -> &'a str {
    let mut left = line.chars().filter(|&c| !breaks_at(c)).count();
    for (at, c) in text.char_indices() {
        if left == 0 {
            return &text[at..];
        }
        if !breaks_at(c) {
            left -= 1;
        }
    }
    ""
}

/// The row after `index` (`forward`) or before it in a list of `len`,
/// wrapping: a [`ListCursor`] step, for a list that keeps its focus as a
/// plain index. 0 with no rows.
pub fn wrap_step(index: usize, len: usize, forward: bool) -> usize {
    let mut cursor = ListCursor::new(len).at(index);
    cursor.step(forward, true, |_| true);
    cursor.focus()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indented_wrap_indents_every_line_after_the_first() {
        let n = CellWidths::NARROW;
        assert_eq!(
            wrap_indented("one two three four", 9, "  ", &n),
            ["one two", "  three", "  four"]
        );
        // The rest is wrapped in the room the indent leaves.
        assert_eq!(wrap_indented("abc defg", 6, "  ", &n), ["abc", "  defg"]);
        assert_eq!(wrap_indented("", 6, "  ", &n), [] as [String; 0]);
        // Runs of spaces and a no-break space are as word_wrap keeps them.
        assert_eq!(
            wrap_indented("a  b\u{a0}c   d", 5, "  ", &n),
            ["a b\u{a0}c", "  d"]
        );
    }

    #[test]
    fn indented_wrap_breaks_chinese_between_characters_never_before_a_full_stop() {
        let text = "测试一二三四五六七八。";
        let wide = CellWidths::wide(text.chars());
        let lines = wrap_indented(text, 8, "  ", &wide);
        assert_eq!(lines, ["测试一二", "  三四五", "  六七", "  八。"]);
        assert!(lines.iter().all(|l| text_width(l, &wide) <= 8), "{lines:?}");
    }

    #[test]
    fn a_step_wraps_round_the_ends() {
        assert_eq!(wrap_step(0, 3, true), 1);
        assert_eq!(wrap_step(2, 3, true), 0);
        assert_eq!(wrap_step(0, 3, false), 2);
        assert_eq!(wrap_step(1, 3, false), 0);
        assert_eq!(wrap_step(0, 0, true), 0);
        assert_eq!(wrap_step(0, 1, false), 0);
    }
}
