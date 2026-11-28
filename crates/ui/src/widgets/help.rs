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
use crate::widgets::{breaks_at, word_wrap};

pub use crate::input::help_keys::{
    NOT_MAPPED, SEPARATOR, all_key_names, cursor_keys_name, help_line, key_name,
};

/// What help text names keys from, for this game's actions
/// ([`help_keys::HelpKeys`](crate::input::help_keys::HelpKeys)).
pub type HelpKeys<'a> = crate::input::help_keys::HelpKeys<'a, Action>;

// engine-friction: engine#3251, moves (game 1412 deletes it)
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

// engine-friction: engine#3251, moves (game 1412 deletes it)
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

// engine-friction: engine#3251, moves (game 1412 deletes it)
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

// engine-friction: engine#3251, moves (game 1412 deletes it)
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

// engine-friction: engine#3251, moves (game 1412 deletes it)
/// A help bar as an ordered list of hints (the friction sweep of
/// 2026-11-17, ticket 0257): each hint the text key of one `{Action} verb`
/// (`hint.back`: `{Cancel} back`), and whether the screen's state shows
/// it. A hint the state doesn't allow is left out of the line, never a
/// second text key for the line without it. Each hint names its own keys,
/// so a language may put the key after the verb.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HelpBar(Vec<(&'static str, bool)>);

// engine-friction: engine#3251, moves (game 1412 deletes it)
impl HelpBar {
    /// A bar with no hints.
    pub fn new() -> Self {
        Self::default()
    }

    /// `self`, then the hint of text key `key`, shown.
    #[must_use]
    pub fn hint(self, key: &'static str) -> Self {
        self.hint_if(key, true)
    }

    /// `self`, then the hint of text key `key`, shown only if `shown`.
    #[must_use]
    pub fn hint_if(mut self, key: &'static str, shown: bool) -> Self {
        self.0.push((key, shown));
        self
    }

    /// Every hint in order, with whether it is shown.
    pub fn hints(&self) -> &[(&'static str, bool)] {
        &self.0
    }

    /// The line: each shown hint in `ctx`'s language with the player's keys
    /// in key spans ([`Ctx::text_styled`]), joined with [`SEPARATOR`].
    pub fn line(&self, ctx: &Ctx) -> StyledText {
        let shown = self.0.iter().filter(|(_, shown)| *shown);
        join_hints(shown.map(|(key, _)| ctx.text_styled(key, &[])))
    }
}

// engine-friction: engine#3251, moves (game 1412 deletes it)
/// The keys of `text`, in order, a no-break space read as a space: what a
/// test of a view's key spans compares with [`spans_of`] (ticket 0257).
#[cfg(any(test, feature = "harness"))]
pub fn key_spans(text: &StyledText) -> Vec<String> {
    let key = |s: &&shuyi_ui::text::TextSpan| s.role.is_some();
    let name = |s: &shuyi_ui::text::TextSpan| s.text.replace('\u{a0}', " ");
    text.spans().iter().filter(key).map(name).collect()
}

// engine-friction: engine#3251, moves (game 1412 deletes it)
/// What [`key_spans`] reads from a hint naming the keys of `actions` in
/// order, each as [`key_name`] gives it: an action by its name in the
/// keymap (`"Confirm"`), and `"Cursor"` for the cursor keys (one span for
/// the arrows, four for W A S D).
#[cfg(any(test, feature = "harness"))]
pub fn spans_of(keys: HelpKeys<'_>, actions: &[&str]) -> Vec<String> {
    use trpg_content::keymap::ActionSet;

    let of = |name: &&str| match Action::from_name(name) {
        Some(action) => vec![key_name(keys, action)],
        // check-text: not player text (the cursor keys' placeholder name)
        None if *name == "Cursor" => key_spans(&crate::input::help_keys::cursor_keys_styled(keys)),
        None => panic!("no action {name:?}"),
    };
    actions.iter().flat_map(of).collect()
}

// engine-friction: engine#3251, moves (game 1412 deletes it)
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

// engine-friction: engine#3251, moves (game 1412 deletes it)
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

/// What [`wrap_styled`] measures a span that isn't plain as: a run of
/// these as wide as the span is drawn, which [`word_wrap`] never breaks
/// inside (nor from a letter or a `,` next to it).
const WHOLE: char = 'x';

// engine-friction: engine#3251, moves (game 1412 deletes it)
/// `text` wrapped by the engine's [`word_wrap`] into rows of at most
/// `room` cells as `widths` measure them, its plain spans broken as
/// `word_wrap` breaks text (at spaces; Chinese and Japanese between
/// characters, ticket 0237), a key (any span with a role) whole, with
/// what follows it up to the next break: `F,` stays together. A key
/// wider than `room` has a row of its own. (`word_wrap` wraps plain text
/// only: each key goes in as a run of `x` its width, a cap's extra
/// cells too, and comes back out whole.)
pub fn wrap_styled(text: &StyledText, room: usize, widths: &CellWidths) -> Vec<StyledText> {
    let spans = text.spans();
    let mut measured = String::new();
    // Each character of `measured`, and the span it stands for if a key.
    let mut source: Vec<(char, Option<usize>)> = Vec::new();
    for (i, span) in spans.iter().enumerate() {
        if span.role.is_none() {
            measured.push_str(&span.text);
            source.extend(span.text.chars().map(|c| (c, None)));
        } else {
            let mut key = StyledText::new();
            key.push(&span.text, span.role);
            let cells = key.width(widths).max(1);
            measured.extend(core::iter::repeat_n(WHOLE, cells));
            source.extend(core::iter::repeat_n((WHOLE, Some(i)), cells));
        }
    }
    let mut source = source.into_iter();
    let mut last_key = None;
    let mut rows = Vec::new();
    for line in word_wrap(&measured, room, widths) {
        let mut row = StyledText::new();
        for c in line.chars() {
            // The line's characters are `measured`'s in order, a break's
            // spaces dropped and one space put where it broke.
            let Some((from, key)) = source.find(|&(s, _)| s == c || (c == ' ' && breaks_at(s)))
            else {
                break;
            };
            match key {
                // After the rest of a key cut over rows (dropped).
                None if c == ' ' && row.is_empty() => {}
                None => {
                    let mut buf = [0; 4];
                    row.push(from.encode_utf8(&mut buf), None);
                }
                Some(i) if last_key != Some(i) => {
                    row.push(&spans[i].text, spans[i].role);
                    last_key = Some(i);
                }
                Some(_) => {}
            }
        }
        if !row.is_empty() {
            rows.push(row);
        }
    }
    rows
}

// engine-friction: engine#3251, moves (game 1412 deletes it)
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

// engine-friction: engine#3251, moves (game 1412 deletes it)
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
