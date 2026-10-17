//! One-time contextual tips (ticket 0406): which have been seen (kept in
//! [`Storage`] under [`TIPS_SEEN_KEY`], per player profile, ADR-0069), key
//! names in tip text, and the box
//! that shows one. The battle screen decides when a trigger has happened.
//!
//! **The look is a skin** (ADR-0054): a tip box is a [`TipView`], plain
//! data; [`glyph::paint`] draws it. The guide speaks every tip, with their
//! face beside the text when the game has the guide's portrait (ticket
//! 0455).

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{FaceView, LessonItemView, LessonView, TipView};

use alloc::collections::BTreeSet;
use core::fmt::Display;

use trpg_content::tip::CURSOR_PLACEHOLDER;

use crate::input::Action;
use crate::screens::battle::split_keys::help_select_action;
use crate::storage::{Storage, StorageError};
use crate::widgets::help::{HelpKeys, cursor_keys_name, key_name};

/// [`Storage`] key under which the ids of the tips already shown are saved,
/// one per line.
pub const TIPS_SEEN_KEY: &str = "tips_seen";

/// The tips already shown to this profile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TipsSeen {
    ids: BTreeSet<String>,
}

impl TipsSeen {
    /// The ids saved in `storage`; none if nothing is saved or it can't be
    /// read (the tips then show again).
    pub fn load(storage: &dyn Storage) -> Self {
        let saved = storage.read(TIPS_SEEN_KEY).ok().flatten();
        let ids = saved
            .iter()
            .flat_map(|s| s.lines())
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_owned)
            .collect();
        Self { ids }
    }

    /// Whether the tip `id` was shown.
    pub fn contains(&self, id: &str) -> bool {
        self.ids.contains(id)
    }

    /// Records that the tip `id` was shown and saves the list. The tip
    /// counts as seen even if saving fails (it then shows again next
    /// launch).
    pub fn mark(&mut self, id: &str, storage: &mut dyn Storage) {
        if self.ids.insert(id.to_owned()) {
            let text: Vec<&str> = self.ids.iter().map(String::as_str).collect();
            storage.write(TIPS_SEEN_KEY, &text.join("\n")).ok();
        }
    }
}

/// Forgets every tip seen, so they all show again (for the Options menu's
/// "reset tips", 0805).
pub fn reset_tips(storage: &mut dyn Storage) -> Result<(), StorageError> {
    storage.delete(TIPS_SEEN_KEY)
}

/// `text` with each `{Action}` replaced by the key `keys` binds to that
/// action (its primary key, or on a controller its button), `{Cursor}` by
/// the cursor keys, `{Select}` by the key that picks on the map
/// ([`select_action`]: Confirm's while Select has no key), and
/// [`NOT_MAPPED`](crate::widgets::help::NOT_MAPPED) for one with no key.
/// Anything else in braces is left alone.
///
/// [`select_action`]: crate::screens::battle::split_keys::select_action
pub fn fill_placeholders(text: &str, keys: HelpKeys<'_>) -> String {
    fill_text(text, keys, &[])
}

/// [`fill_placeholders`], with each `{name}` that `args` names replaced by
/// its value first (`("count", &3)` fills `{count}`). What is filled in is
/// not looked at again, so a value may hold braces.
pub fn fill_text(text: &str, keys: HelpKeys<'_>, args: &[(&str, &dyn Display)]) -> String {
    shuyi_ui::text::fill(text, args, |name| key_text(name, keys))
}

/// What a `{name}` placeholder that names a key says ([`fill_text`]):
/// `{Cursor}` the cursor keys, `{Select}` the key that picks on the map
/// ([`select_action`]: Confirm's while Select has no key), `{Action}` the
/// key of that action. `None` for any other name.
///
/// [`select_action`]: crate::screens::battle::split_keys::select_action
pub fn key_text(name: &str, keys: HelpKeys<'_>) -> Option<String> {
    if name == CURSOR_PLACEHOLDER {
        Some(cursor_keys_name(keys))
    } else if name == Action::Select.name() {
        Some(key_name(keys, help_select_action(keys)))
    } else {
        Action::from_name(name).map(|action| key_name(keys, action))
    }
}

/// `text` with each `{name}` that `args` names replaced by its value, and
/// no key named: for text built where there are no keys to name (a line
/// of a list, a label). Anything else in braces is left alone.
pub fn fill_values(text: &str, args: &[(&str, &dyn Display)]) -> String {
    shuyi_ui::text::fill_values(text, args)
}

#[cfg(test)]
mod tests {
    use crate::input::{LEFT_HANDED, RIGHT_HANDED};
    #[test]
    fn values_are_filled_and_keys_left_as_written() {
        let filled = fill_values(
            "{Confirm} {n} of {max} {nope} {",
            &[("n", &3), ("max", &"{n}")],
        );
        assert_eq!(filled, "{Confirm} 3 of {n} {nope} {");
        assert_eq!(fill_values("plain", &[]), "plain");
    }

    use trpg_content::{Chord, RepeatDef};

    use super::*;
    use crate::input::{Device, Keymap, PadKind};
    use crate::storage::MemoryStorage;
    use crate::widgets::help::NOT_MAPPED;

    fn keymap(pairs: &[(&str, Action)]) -> Keymap {
        Keymap::new(
            pairs.iter().map(|&(c, a)| (Chord::parse(c).unwrap(), a)),
            RepeatDef::default(),
        )
    }

    #[test]
    fn seen_tips_are_saved_and_loaded() {
        let mut storage = MemoryStorage::new();
        let mut seen = TipsSeen::load(&storage);
        assert!(!seen.contains("a"));
        seen.mark("a", &mut storage);
        seen.mark("b", &mut storage);
        seen.mark("a", &mut storage);
        assert_eq!(
            storage.read(TIPS_SEEN_KEY).unwrap().as_deref(),
            Some("a\nb")
        );
        let again = TipsSeen::load(&storage);
        assert!(again.contains("a") && again.contains("b") && !again.contains("c"));
        assert_eq!(again, seen);
    }

    #[test]
    fn resetting_forgets_every_tip() {
        let mut storage = MemoryStorage::new();
        TipsSeen::default().mark("a", &mut storage);
        let mut seen = TipsSeen::load(&storage);
        seen.mark("b", &mut storage);
        reset_tips(&mut storage).unwrap();
        assert_eq!(TipsSeen::load(&storage), TipsSeen::default());
        // Nothing to reset is fine.
        reset_tips(&mut storage).unwrap();
    }

    #[test]
    fn placeholders_show_the_bound_keys() {
        let km = keymap(&[
            ("f", Action::Confirm),
            ("d", Action::Cancel),
            ("Up", Action::CursorUp),
            ("Left", Action::CursorLeft),
            ("Down", Action::CursorDown),
            ("Right", Action::CursorRight),
        ]);
        assert_eq!(
            fill_placeholders(
                "{Confirm}/{Cancel} {Cursor} {Confirm}",
                HelpKeys::keyboard(&km)
            ),
            "f/d arrows f"
        );
        // Unbound and unknown placeholders.
        assert_eq!(
            fill_placeholders("{Rewind} {Nope} {", HelpKeys::keyboard(&km)),
            format!("{NOT_MAPPED} {{Nope}} {{")
        );
        // A cursor action with no key: the whole cursor placeholder.
        let no_cursor = keymap(&[("f", Action::Confirm)]);
        assert_eq!(
            fill_placeholders("{Cursor} {Confirm}", HelpKeys::keyboard(&no_cursor)),
            "! not mapped f"
        );
    }

    /// Ticket 0233: named values beside key names, each filled once.
    #[test]
    fn named_values_are_filled_with_the_key_names() {
        let km = keymap(&[("f", Action::Confirm)]);
        let keys = HelpKeys::keyboard(&km);
        assert_eq!(
            fill_text(
                "{count} of {total}: {Confirm} {count} {nope}",
                keys,
                &[("count", &3), ("total", &"ten")]
            ),
            "3 of ten: f 3 {nope}"
        );
        // A value wins over a key of the same name, and its own braces
        // are not filled again.
        assert_eq!(
            fill_text(
                "{Confirm} {a}{b} {",
                keys,
                &[("Confirm", &"{a}"), ("a", &"{Confirm}"), ("b", &"}")]
            ),
            "{a} {Confirm}} {"
        );
        // Text after an unclosed brace is kept as it is.
        assert_eq!(fill_text("a { b", keys, &[("b", &1)]), "a { b");
        assert_eq!(fill_text("", keys, &[]), "");
    }

    /// Ticket 0220: on a controller a tip names the pad's buttons.
    #[test]
    fn placeholders_show_the_buttons_on_a_pad() {
        let content = trpg_content::load_embedded().unwrap();
        let km = Keymap::for_layout(&content.keymap, RIGHT_HANDED);
        let text = "{Confirm}/{Cancel} {Cursor} {Select} {NextUnit} {Nope} {Debug}";
        let on = |kind| fill_placeholders(text, HelpKeys::new(&km, Device::Pad(kind)));
        assert_eq!(
            on(PadKind::Xbox),
            "A/B D-pad/L-stick A RB {Nope} ! not mapped"
        );
        assert_eq!(on(PadKind::Generic), on(PadKind::Xbox));
        assert_eq!(
            on(PadKind::PlayStation),
            "✕/◯ D-pad/L-stick ✕ R1 {Nope} ! not mapped"
        );
        assert_eq!(
            on(PadKind::Nintendo),
            "A/B D-pad/L-stick A R {Nope} ! not mapped"
        );
        assert_eq!(
            fill_placeholders(text, HelpKeys::keyboard(&km)),
            "f/d arrows f s {Nope} F2"
        );
        // No buttons at all: every action, and the cursor, is not mapped.
        let keys_only = keymap(&[("f", Action::Confirm)]);
        let pad = HelpKeys::new(&keys_only, Device::Pad(PadKind::Xbox));
        assert_eq!(
            fill_placeholders("{Confirm} {Cursor}", pad),
            format!("{NOT_MAPPED} {NOT_MAPPED}")
        );
    }

    #[test]
    fn select_shows_confirms_key_until_it_has_its_own() {
        let km = keymap(&[("f", Action::Confirm)]);
        assert_eq!(
            fill_placeholders("{Select} {Confirm}", HelpKeys::keyboard(&km)),
            "f f"
        );
        let split = keymap(&[("f", Action::Confirm), ("g", Action::Select)]);
        assert_eq!(
            fill_placeholders("{Select} {Confirm}", HelpKeys::keyboard(&split)),
            "g f"
        );
    }

    #[test]
    fn the_map_pick_tips_name_the_select_key() {
        let content = trpg_content::load_embedded().unwrap();
        let tip = |id: &str| {
            let tip = content.tips.tips.iter().find(|t| t.id == id);
            tip.map(|t| t.text.replace('\n', " ")).unwrap_or_default()
        };
        let default = Keymap::for_layout(&content.keymap, RIGHT_HANDED);
        let mut split = content
            .keymap
            .layouts
            .get(&RIGHT_HANDED)
            .into_iter()
            .flatten()
            .flat_map(|(&a, chords)| chords.iter().map(move |&c| (c, a)))
            .collect::<Vec<_>>();
        split.push((Chord::parse("g").unwrap(), Action::Select));
        let split = Keymap::new(split, RepeatDef::default());
        for (id, phrase) in [
            ("battle_start", "; {} on one of your units"),
            ("unit_selected", "press {} to send it there"),
            ("danger_zone", "Press {} on one enemy for just its range"),
        ] {
            let text = tip(id);
            let with = |key| phrase.replace("{}", key);
            assert!(
                fill_placeholders(&text, HelpKeys::keyboard(&default)).contains(&with("f")),
                "{id}"
            );
            assert!(
                fill_placeholders(&text, HelpKeys::keyboard(&split)).contains(&with("g")),
                "{id}"
            );
        }
        // Menu and forecast wording stays on Confirm.
        let forecast = fill_placeholders(&tip("forecast"), HelpKeys::keyboard(&split));
        assert!(forecast.contains("attack with f."), "{forecast}");
    }

    #[test]
    fn placeholders_follow_the_layout() {
        let content = trpg_content::load_embedded().unwrap();
        let text = "{Confirm} {Cursor} {Rewind}";
        let right = Keymap::for_layout(&content.keymap, RIGHT_HANDED);
        let left = Keymap::for_layout(&content.keymap, LEFT_HANDED);
        assert_eq!(
            fill_placeholders(text, HelpKeys::keyboard(&right)),
            "f arrows r"
        );
        assert_eq!(
            fill_placeholders(text, HelpKeys::keyboard(&left)),
            "j wasd u"
        );
    }
}
