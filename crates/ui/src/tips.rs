//! One-time contextual tips (ticket 0406): which have been seen (kept in
//! [`Storage`](crate::storage::Storage) under [`TIPS_SEEN_KEY`], per player
//! profile, ADR-0069; the engine's, ticket 1112), key
//! names in tip text, and the box
//! that shows one. The battle screen decides when a trigger has happened.
//!
//! **The look is a skin** (ADR-0054): a tip box is a [`TipView`], plain
//! data; [`glyph::paint`] draws it. The guide speaks every tip, with their
//! face beside the text when the game has the guide's portrait (ticket
//! 0455).

pub mod glyph;
pub mod screen;
pub mod view;
pub mod voice;

use crate::prelude::*;
pub use screen::TipScreen;
use trpg_content::keymap::ActionSet;
pub use view::{EndAnswer, FaceView, LessonItemView, LessonView, LessonsEndView, TipView};

use crate::words::SOURCE_LANG;
use shuyi_ui::text::Args;

use trpg_content::tip::CURSOR_PLACEHOLDER;

use crate::input::Action;
use crate::screens::battle::split_keys::help_select_action;
use crate::widgets::help::{HelpKeys, cursor_keys_name, key_name};

/// Whether tips show in `ctx`: they are on, and the campaign's tutorial
/// level (if any) shows them (0835). In battle and out of it.
pub fn shown_in(ctx: &crate::screen::Ctx) -> bool {
    ctx.game.tips_enabled
        && ctx
            .game
            .campaign_tutorial
            .is_none_or(trpg_core::TutorialLevel::shows_tips)
}

/// Which tips have been shown and forgetting them: Shuyi Engine's
/// (`shuyi_ui::tips`, ticket 1112).
pub use shuyi_ui::tips::{TIPS_SEEN_KEY, TipsSeen, reset_tips};

/// `text` with each `{Action}` replaced by the key `keys` binds to that
/// action (its primary key, or on a controller its button), `{Cursor}` by
/// the cursor keys, `{Select}` by the key that picks on the map
/// ([`select_action`]: Confirm's while Select has no key), and
/// [`NOT_MAPPED`](crate::widgets::help::NOT_MAPPED) for one with no key.
/// Anything else in braces is left alone.
///
/// [`select_action`]: crate::screens::battle::split_keys::select_action
///
/// For a tip's or a lesson's text: one cursor direction on the arrow keys
/// is its arrow (`guide_key_text`).
pub fn fill_placeholders(text: &str, keys: HelpKeys<'_>) -> String {
    // No values, so no plural to choose by a language's rules.
    shuyi_ui::text::fill(text, SOURCE_LANG, &[], |name| guide_key_text(name, keys))
}

/// [`fill_placeholders`], with each `{name}` that `args` names replaced by
/// its value first (`("count", &3)` fills `{count}`). What is filled in is
/// not looked at again, so a value may hold braces. `lang` is the language
/// `text` is in, whose plural rules a message of it follows.
pub fn fill_text(text: &str, lang: &str, keys: HelpKeys<'_>, args: &Args<'_>) -> String {
    shuyi_ui::text::fill(text, lang, args, |name| key_text(name, keys))
}

/// [`fill_placeholders`] in spans, as the player of `ctx` sees them: each
/// key's name in a key span, drawn as a key cap (ticket 0254), the arrow
/// keys one span ([`Shuyi`](crate::screen::Shuyi)'s `key_text_styled`).
pub fn fill_styled(ctx: &crate::screen::Ctx, text: &str) -> shuyi_ui::text::StyledText {
    use shuyi_ui::GameData;
    let keys = ctx.help_keys();
    let lookup = |name: &str| match arrow_symbol(name, keys) {
        Some(arrow) => {
            let mut styled = shuyi_ui::text::StyledText::new();
            styled.push_key(arrow);
            Some(crate::widgets::help::for_player(keys, &styled))
        }
        None => ctx.game.key_text_styled(name, keys),
    };
    shuyi_ui::text::fill_styled_with(text, SOURCE_LANG, &[], &[], lookup)
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

/// [`key_text`] for the guide's boxes (tips and lessons): a cursor
/// direction on the arrow keys is its arrow ([`arrow_symbol`]).
fn guide_key_text(name: &str, keys: HelpKeys<'_>) -> Option<String> {
    arrow_symbol(name, keys).map_or_else(|| key_text(name, keys), |a| Some(a.to_owned()))
}

/// The arrow a cursor direction is named by while the four directions are
/// on the arrow keys (what the engine names `arrows` together): an arrow
/// symbol pointing that way, not the key's word (ticket 1306, Nick: "some
/// kind of symbol denoting arrow direction"). `None` for any other action,
/// or when the cursor is on other keys or a controller. Only in the
/// guide's boxes: help bars and menus name the keys as before.
fn arrow_symbol(name: &str, keys: HelpKeys<'_>) -> Option<&'static str> {
    let action = Action::from_name(name)?;
    let cursor = crate::input::help_keys::cursor_keys_styled(keys);
    let on_arrows = cursor
        .spans()
        .iter()
        .any(|s| s.role == Some(shuyi_ui::text::TextRole::Arrows));
    match action {
        _ if !on_arrows => None,
        Action::CursorUp => Some("\u{2191}"),
        Action::CursorDown => Some("\u{2193}"),
        Action::CursorLeft => Some("\u{2190}"),
        Action::CursorRight => Some("\u{2192}"),
        _ => None,
    }
}

/// `text` with each `{name}` that `args` names replaced by its value, and
/// no key named: for text built where there are no keys to name (a line
/// of a list, a label). Anything else in braces is left alone. `lang` is
/// the language `text` is in, whose plural rules a message of it follows.
pub fn fill_values(text: &str, lang: &str, args: &Args<'_>) -> String {
    shuyi_ui::text::fill_values(text, lang, args)
}

#[cfg(test)]
mod tests {
    use crate::input::{LEFT_HANDED, RIGHT_HANDED};
    #[test]
    fn values_are_filled_and_keys_left_as_written() {
        let filled = fill_values(
            "{Confirm} {n} of {max} {nope} {",
            SOURCE_LANG,
            &[("n", &3), ("max", &"{n}")],
        );
        assert_eq!(filled, "{Confirm} 3 of {n} {nope} {");
        assert_eq!(fill_values("plain", SOURCE_LANG, &[]), "plain");
    }

    use trpg_content::{Chord, RepeatDef};

    use super::*;
    use crate::input::{Device, Keymap, PadKind};
    use crate::widgets::help::NOT_MAPPED;

    fn keymap(pairs: &[(&str, Action)]) -> Keymap {
        Keymap::new(
            pairs.iter().map(|&(c, a)| (Chord::parse(c).unwrap(), a)),
            RepeatDef::default(),
        )
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
            format!("{} {{Nope}} {{", NOT_MAPPED.replace(' ', "\u{a0}"))
        );
        // A cursor action with no key: the whole cursor placeholder.
        let no_cursor = keymap(&[("f", Action::Confirm)]);
        assert_eq!(
            fill_placeholders("{Cursor} {Confirm}", HelpKeys::keyboard(&no_cursor)),
            "!\u{a0}not\u{a0}mapped f"
        );
    }

    /// Ticket 1306: with the cursor on the arrow keys, each direction is its
    /// arrow; on other keys, or with one of them elsewhere, its key's name.
    #[test]
    fn a_cursor_direction_on_the_arrow_keys_is_its_arrow() {
        let text = "{CursorUp} {CursorDown} {CursorLeft} {CursorRight} {Confirm}";
        let arrows = keymap(&[
            ("f", Action::Confirm),
            ("Up", Action::CursorUp),
            ("Left", Action::CursorLeft),
            ("Down", Action::CursorDown),
            ("Right", Action::CursorRight),
        ]);
        let filled = fill_placeholders(text, HelpKeys::keyboard(&arrows));
        assert_eq!(filled, "\u{2191} \u{2193} \u{2190} \u{2192} f");
        let wasd = keymap(&[
            ("f", Action::Confirm),
            ("w", Action::CursorUp),
            ("a", Action::CursorLeft),
            ("s", Action::CursorDown),
            ("Right", Action::CursorRight),
        ]);
        let filled = fill_placeholders(text, HelpKeys::keyboard(&wasd));
        assert_eq!(filled, "w s a Right f");
    }

    /// Ticket 0233: named values beside key names, each filled once.
    #[test]
    fn named_values_are_filled_with_the_key_names() {
        let km = keymap(&[("f", Action::Confirm)]);
        let keys = HelpKeys::keyboard(&km);
        assert_eq!(
            fill_text(
                "{count} of {total}: {Confirm} {count} {nope}",
                SOURCE_LANG,
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
                SOURCE_LANG,
                keys,
                &[("Confirm", &"{a}"), ("a", &"{Confirm}"), ("b", &"}")]
            ),
            "{a} {Confirm}} {"
        );
        // Text after an unclosed brace is kept as it is.
        assert_eq!(fill_text("a { b", SOURCE_LANG, keys, &[("b", &1)]), "a { b");
        assert_eq!(fill_text("", SOURCE_LANG, keys, &[]), "");
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
            "A/B D-pad/L-stick A RB {Nope} !\u{a0}not\u{a0}mapped"
        );
        assert_eq!(on(PadKind::Generic), on(PadKind::Xbox));
        assert_eq!(
            on(PadKind::PlayStation),
            "✕/◯ D-pad/L-stick ✕ R1 {Nope} !\u{a0}not\u{a0}mapped"
        );
        assert_eq!(
            on(PadKind::Nintendo),
            "A/B D-pad/L-stick A R {Nope} !\u{a0}not\u{a0}mapped"
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
            "!\u{a0}not\u{a0}mapped !\u{a0}not\u{a0}mapped"
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

    /// A test of the game's own tips (ADR-0118): their wording names the
    /// Select key where a tile is picked.
    #[test]
    fn the_map_pick_tips_name_the_select_key() {
        if trpg_content::bundle::skip_without_the_game() {
            return;
        }
        let content = trpg_content::load_embedded().unwrap();
        let tip = |id: &str| {
            let tip = content.tips.tips.iter().find(|t| t.id == id);
            let how = tip.and_then(|t| t.how.as_deref());
            how.map_or_default(|h| h.replace('\n', " "))
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
        // The danger zone's (the start and moving tips are lessons now, 1304).
        let text = tip("danger_zone");
        let with = |key| "One enemy's range: {} on it".replace("{}", key);
        assert!(fill_placeholders(&text, HelpKeys::keyboard(&default)).contains(&with("f")));
        assert!(fill_placeholders(&text, HelpKeys::keyboard(&split)).contains(&with("g")));
        // Menu and forecast wording stays on Confirm.
        let forecast = fill_placeholders(&tip("forecast"), HelpKeys::keyboard(&split));
        assert!(forecast.contains("Attack: f"), "{forecast}");
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
            "j w\u{a0}a\u{a0}s\u{a0}d u"
        );
    }
}
