//! The game's three entries in the Playdate's system menu (ticket 1035,
//! `docs/design/playdate.md`): [`SystemMenuEntry::ALL`], added by the front
//! end with the SDK's `addMenuItem` in that order, each named in the
//! player's language. The SDK calls back when one is chosen, between two
//! updates; [`Chosen`] keeps the choice until the next update hands it to
//! the game (`trpg_ui::game::system_menu::choose`). [`Titles`] says which
//! entries' names changed (the player picked another language), for
//! `setMenuItemTitle`.

use alloc::borrow::ToOwned;
use alloc::string::String;
use alloc::vec::Vec;
use core::sync::atomic::{AtomicU8, Ordering};

use trpg_ui::game::system_menu::SystemMenuEntry;

/// How many entries the game adds: the SDK takes at most three.
pub const ENTRIES: usize = SystemMenuEntry::ALL.len();

/// The entry chosen since the last [`take`](Self::take), if any: its index
/// in [`SystemMenuEntry::ALL`] plus one, 0 for none.
#[derive(Debug, Default)]
pub struct Chosen(AtomicU8);

impl Chosen {
    /// Nothing chosen.
    pub const fn new() -> Self {
        Self(AtomicU8::new(0))
    }

    /// The SDK called back for the entry at `index`; an index the game
    /// never added is ignored. A later choice replaces an earlier one not
    /// yet taken (the menu closes on a choice, so there is only one).
    pub fn set(&self, index: i32) {
        let Some(n) = usize::try_from(index)
            .ok()
            .filter(|&i| i < ENTRIES)
            .and_then(|i| u8::try_from(i + 1).ok())
        else {
            return;
        };
        self.0.store(n, Ordering::Relaxed);
    }

    /// The entry chosen since the last call, if any.
    pub fn take(&self) -> Option<SystemMenuEntry> {
        let n = usize::from(self.0.swap(0, Ordering::Relaxed));
        SystemMenuEntry::ALL.get(n.checked_sub(1)?).copied()
    }
}

/// The entries' names as last given to the SDK.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Titles([String; ENTRIES]);

impl Titles {
    /// Takes the entries' names `now`, in [`SystemMenuEntry::ALL`] order,
    /// and returns the indexes of those that differ from the last ones
    /// (all of them the first time, when the names were empty).
    pub fn update(&mut self, now: [&str; ENTRIES]) -> Vec<usize> {
        let mut changed = Vec::new();
        for (i, (title, new)) in self.0.iter_mut().zip(now).enumerate() {
            if title != new {
                new.clone_into(title);
                changed.push(i);
            }
        }
        changed
    }

    /// The name given to the SDK for the entry at `index`.
    pub fn get(&self, index: usize) -> &str {
        self.0.get(index).map_or("", String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_choice_is_taken_once() {
        let chosen = Chosen::new();
        assert_eq!(chosen.take(), None);
        chosen.set(2);
        assert_eq!(chosen.take(), Some(SystemMenuEntry::Options));
        assert_eq!(chosen.take(), None);
        chosen.set(0);
        chosen.set(1);
        assert_eq!(chosen.take(), Some(SystemMenuEntry::DialogueLog));
    }

    #[test]
    fn an_entry_the_game_never_added_is_ignored() {
        let chosen = Chosen::new();
        for index in [-1, 3, i32::MAX] {
            chosen.set(index);
            assert_eq!(chosen.take(), None, "{index}");
        }
    }

    #[test]
    fn titles_say_which_names_changed() {
        let mut titles = Titles::default();
        assert_eq!(titles.update(["A", "B", "C"]), [0, 1, 2]);
        assert!(titles.update(["A", "B", "C"]).is_empty());
        assert_eq!(titles.update(["A", "b", "C"]), [1]);
        assert_eq!(titles.get(1), "b");
        assert_eq!(titles.get(9), "");
    }
}
