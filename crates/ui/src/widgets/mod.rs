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
    use super::wrap_step;

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
