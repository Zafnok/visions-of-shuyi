//! Help text that names keys and controller buttons, for this game's
//! screens: [`HelpKeys`] with this game's actions, and the helpers that
//! name an action's keys (generic, from
//! [`input::help_keys`](crate::input::help_keys)). Names always come from
//! the active [`Keymap`](crate::input::Keymap), never string literals
//! (ADR-0015, ADR-0031), and follow the device the player pressed last.
//!
//! **Key caps** (ticket 0254, `docs/design/look-and-feel.md`, *Key names
//! in hints*): a hint that names a key holds [`StyledText`], each key's
//! name in a key span, which the frame draws as a key cap
//! ([`Shuyi::key_caps`](crate::screen::Shuyi)). Only a keyboard's keys are
//! caps: a controller's buttons (0255) and [`NOT_MAPPED`] read as plain
//! text ([`for_player`]).

use crate::prelude::*;

use shuyi_assets::font::CellWidths;
use shuyi_ui::frame::key_cap::KeyCaps;
use shuyi_ui::input::state::Device;
use shuyi_ui::text::{Args, StyledText, TextRole};

use crate::input::Action;
use crate::screen::Ctx;

pub use crate::input::help_keys::{
    NOT_MAPPED, SEPARATOR, all_key_names, cursor_keys_name, help_line, key_name,
};

/// What help text names keys from, for this game's actions
/// ([`help_keys::HelpKeys`](crate::input::help_keys::HelpKeys)).
pub type HelpKeys<'a> = crate::input::help_keys::HelpKeys<'a, Action>;

/// `text` as this game shows it to the player of `keys`: its key spans as
/// they are on the keyboard; on a controller, and where a key span says
/// [`HelpKeys::not_mapped`], plain text, as hints read before key caps.
pub fn for_player(keys: HelpKeys<'_>, text: &StyledText) -> StyledText {
    let pad = matches!(keys.device(), Device::Pad(_));
    let mut out = StyledText::new();
    for span in text.spans() {
        let key = matches!(span.role, Some(TextRole::Key | TextRole::Arrows));
        // A name filled into text has its spaces as no-break spaces.
        let not_mapped = span.text.replace('\u{a0}', " ") == keys.not_mapped();
        let plain = key && (pad || not_mapped);
        out.push(&span.text, if plain { None } else { span.role });
    }
    out
}

/// A key's name (as [`key_name`] and its siblings give it) as one key
/// span, or plain text where [`for_player`] says.
pub fn key_styled(keys: HelpKeys<'_>, name: &str) -> StyledText {
    let mut key = StyledText::new();
    key.push_key(name);
    for_player(keys, &key)
}

/// The key for `action` ([`key_name`]) as [`key_styled`] gives it.
pub fn action_key(keys: HelpKeys<'_>, action: Action) -> StyledText {
    key_styled(keys, &key_name(keys, action))
}

/// Two actions' keys as `w/s`, each its own key span.
pub fn key_pair(keys: HelpKeys<'_>, first: Action, second: Action) -> StyledText {
    let mut pair = action_key(keys, first);
    pair.push_plain("/").append(&action_key(keys, second));
    pair
}

/// [`help_line`] in spans, as [`for_player`] shows it to the player of
/// `keys`.
pub fn help_line_styled(keys: HelpKeys<'_>, hints: &[(Option<String>, &str)]) -> StyledText {
    for_player(keys, &crate::input::help_keys::help_line_styled(hints))
}

/// `parts` joined with [`SEPARATOR`], as a help line's hints are.
pub fn join_hints(parts: impl IntoIterator<Item = StyledText>) -> StyledText {
    let mut line = StyledText::new();
    for (i, part) in parts.into_iter().enumerate() {
        if i > 0 {
            line.push_plain(SEPARATOR);
        }
        line.append(&part);
    }
    line
}

/// The text `key` filled as [`Ctx::text_styled`] fills it, with each
/// value of `keys_args` (keys built by the caller, such as
/// [`key_pair`]'s) put in as its spans.
pub fn text_with_keys(ctx: &Ctx, key: &str, keys_args: &[(&str, &StyledText)]) -> StyledText {
    use shuyi_ui::GameData;
    let lang = ctx.game.lang();
    debug_assert!(lang.has(key), "text key not in the source text: {key:?}");
    let (text, code) = lang.text_in(ctx.effective_lang(), key);
    let device = ctx.device.message_value();
    let target = ctx.platform.target.as_str();
    let args: &Args<'_> = &[
        (shuyi_assets::message::DEVICE, &device),
        (shuyi_assets::message::TARGET, &target),
    ];
    let keys = ctx.help_keys();
    let lookup = |name: &str| {
        let given = keys_args.iter().find(|(arg, _)| *arg == name);
        given
            .map(|(_, value)| (*value).clone())
            .or_else(|| ctx.game.key_text_styled(name, keys))
    };
    shuyi_ui::text::fill_styled_with(text, code, args, &[], lookup)
}

/// The frame's cell widths: `ctx`'s, with what its key caps take (a
/// cap's extra cells, the arrow cluster's picture's), as the frame
/// measures styled text.
pub fn frame_widths(ctx: &Ctx) -> CellWidths {
    let caps = ctx.key_caps();
    let extra = caps.as_ref().map(KeyCaps::extra_cells);
    let arrows = caps.as_ref().and_then(KeyCaps::arrows_cells);
    ctx.widths()
        .clone()
        .with_key_cap_cells(extra)
        .with_arrows_cells(arrows)
}

/// `text` split at its line breaks, each line keeping its spans' roles.
pub fn styled_lines(text: &StyledText) -> Vec<StyledText> {
    let mut lines = vec![StyledText::new()];
    for span in text.spans() {
        for (i, part) in span.text.split('\n').enumerate() {
            if i > 0 {
                lines.push(StyledText::new());
            }
            if let (false, Some(line)) = (part.is_empty(), lines.last_mut()) {
                line.push(part, span.role);
            }
        }
    }
    lines
}

/// `text` wrapped at the spaces of its plain spans into rows of at most
/// `room` cells as `widths` measure them (a key cap whole, with what
/// follows it up to the next space: `F,` stays together). A word wider
/// than `room` has a row of its own. The engine's
/// [`word_wrap`](crate::widgets::word_wrap) wraps plain text only:
/// workaround until the engine wraps styled text (ticket 0254's notes).
pub fn wrap_styled(text: &StyledText, room: usize, widths: &CellWidths) -> Vec<StyledText> {
    let mut words = vec![StyledText::new()];
    for span in text.spans() {
        let parts: Vec<&str> = match span.role {
            None => span.text.split(' ').collect(),
            Some(_) => vec![span.text.as_str()],
        };
        for (i, part) in parts.into_iter().enumerate() {
            if i > 0 {
                words.push(StyledText::new());
            }
            if let (false, Some(word)) = (part.is_empty(), words.last_mut()) {
                word.push(part, span.role);
            }
        }
    }
    let mut rows = Vec::new();
    let (mut row, mut used) = (StyledText::new(), 0);
    for word in words.into_iter().filter(|w| !w.is_empty()) {
        let w = word.width(widths);
        if !row.is_empty() && used + 1 + w > room {
            rows.push(core::mem::take(&mut row));
            used = 0;
        }
        if !row.is_empty() {
            row.push_plain(" ");
            used += 1;
        }
        row.append(&word);
        used += w;
    }
    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

/// The cell, counted from the line's start as `widths` measure it, where
/// hint `index` of `hints` starts once they are [joined](join_hints)
/// (0445): the cells of the hints before it and of their separators, a
/// key cap's extra cells included. Counted from the hints themselves,
/// never by searching the joined line, so a label that happens to contain
/// another hint's words can't confuse it.
pub fn hint_column(hints: &[StyledText], index: usize, widths: &CellWidths) -> usize {
    let before = hints.iter().take(index);
    let cells: usize = before.map(|h| h.width(widths)).sum();
    cells + index * StyledText::from(SEPARATOR).width(widths)
}

/// `hints` [joined](join_hints), the last one moved right to start in
/// cell `column`, with spaces in place of its separator; where it would
/// start there anyway or later, just joined. Empty `hints` give an empty
/// line.
pub fn pad_last_hint(hints: &[StyledText], column: usize, widths: &CellWidths) -> StyledText {
    let Some((last, rest)) = hints.split_last() else {
        return StyledText::new();
    };
    if rest.is_empty() || hint_column(hints, rest.len(), widths) >= column {
        return join_hints(hints.iter().cloned());
    }
    let mut line = join_hints(rest.iter().cloned());
    let pad = column - line.width(widths);
    line.push_plain(&" ".repeat(pad)).append(last);
    line
}

#[cfg(test)]
mod tests;
