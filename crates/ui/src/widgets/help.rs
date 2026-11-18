//! Help text that names keys and controller buttons, for this game's
//! screens: [`HelpKeys`] with this game's actions, and the helpers that
//! name an action's keys (generic, from
//! [`input::help_keys`](crate::input::help_keys)). Names always come from
//! the active [`Keymap`](crate::input::Keymap), never string literals
//! (ADR-0015, ADR-0031), and follow the device the player pressed last.

use crate::input::Action;
use crate::prelude::*;

pub use crate::input::help_keys::{
    NOT_MAPPED, SEPARATOR, all_key_names, cursor_keys_name, help_line, key_name,
};

/// What help text names keys from, for this game's actions
/// ([`help_keys::HelpKeys`](crate::input::help_keys::HelpKeys)).
pub type HelpKeys<'a> = crate::input::help_keys::HelpKeys<'a, Action>;

/// The column, in characters from the line's start, where hint `index` of
/// `hints` starts once they are joined with [`SEPARATOR`] (0445): the
/// characters of the hints before it and their separators. Counted from
/// the hints themselves, never by searching the joined line, so a label
/// that happens to contain another hint's words can't confuse it.
pub fn hint_column(hints: &[String], index: usize) -> usize {
    let before = hints.iter().take(index);
    let chars: usize = before.map(|h| h.chars().count()).sum();
    chars + index * SEPARATOR.chars().count()
}

/// `hints` joined with [`SEPARATOR`], the last one moved right to start
/// in `column`, with spaces in place of its separator; where it would
/// start there anyway or later, just joined. Empty `hints` give an empty
/// line.
pub fn pad_last_hint(hints: &[String], column: usize) -> String {
    let Some((last, rest)) = hints.split_last() else {
        return String::new();
    };
    if rest.is_empty() || hint_column(hints, rest.len()) >= column {
        return hints.join(SEPARATOR);
    }
    let start = rest.join(SEPARATOR);
    let pad = column - start.chars().count();
    format!("{start}{:pad$}{last}", "")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hints(words: &[&str]) -> Vec<String> {
        words.iter().map(|w| (*w).to_owned()).collect()
    }

    #[test]
    fn hint_column_counts_the_hints_and_separators_before() {
        let h = hints(&["f select", "e info", "Space end turn"]);
        assert_eq!(hint_column(&h, 0), 0);
        assert_eq!(hint_column(&h, 1), 11);
        let line = h.join(SEPARATOR);
        let at = line.find("Space").unwrap();
        assert_eq!(hint_column(&h, 2), line[..at].chars().count());
    }

    #[test]
    fn hint_column_counts_characters_not_bytes() {
        // `→` and `×` take three and two bytes, one cell each.
        let h = hints(&["q zoom: 2×", "a → b", "end"]);
        assert_eq!(hint_column(&h, 2), 10 + 3 + 5 + 3);
    }

    #[test]
    fn hint_column_is_not_fooled_by_a_label_with_the_same_words() {
        let h = hints(&["x end turn soon", "Space end turn"]);
        assert_eq!(hint_column(&h, 1), 18);
    }

    #[test]
    fn pad_last_hint_moves_the_last_hint_to_the_column() {
        let h = hints(&["f select", "Space end turn"]);
        let line = pad_last_hint(&h, 20);
        assert_eq!(line, format!("f select{}Space end turn", " ".repeat(12)));
        assert_eq!(line.find("Space"), Some(20));
    }

    #[test]
    fn pad_last_hint_never_pulls_a_hint_left() {
        let h = hints(&["f select", "Space end turn"]);
        assert_eq!(pad_last_hint(&h, 5), h.join(SEPARATOR));
        assert_eq!(pad_last_hint(&h, 11), h.join(SEPARATOR));
        assert_eq!(pad_last_hint(&hints(&["one"]), 9), "one");
        assert_eq!(pad_last_hint(&[], 9), "");
    }
}
