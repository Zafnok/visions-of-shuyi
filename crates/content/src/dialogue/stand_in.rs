//! What the voice says in place of the player's name (ticket 0754,
//! ADR-0107 rule 4): a line writes `{lead|the exile}`, the text box shows
//! the name the player typed and the voice says "the exile". The stand-in
//! is taken out of the line as the script is read ([`take`]), so the line's
//! text, its id and its translations (ADR-0045) are those of `{lead}`; the
//! lines as written are kept apart ([`StandIns`]) for the voice
//! ([`written`], `voice::spoken_text`).
use crate::prelude::*;

use alloc::collections::BTreeMap;

use trpg_core::lead::{self, Part};

use super::{DialogueTable, LineId, Lines, ParsedScene, Step};
use crate::error::ContentError;

/// The lines written with a stand-in, by id: each line's text as written,
/// `{lead|<words>}` and all.
pub type StandIns = BTreeMap<LineId, String>;

/// The words the voice says for token `token` (the text between the
/// braces) if it is the name with a stand-in, `lead|<words>`.
pub fn stand_in(token: &str) -> Option<&str> {
    let (name, words) = token.split_once('|')?;
    (name == lead::NAME_TOKEN && !words.trim().is_empty() && !words.contains('{')).then_some(words)
}

/// `text` with each `{lead|<words>}` written `{lead}`, or `None` if it has
/// no stand-in.
fn strip(text: &str) -> Option<String> {
    let mut out = String::with_capacity(text.len());
    let mut found = false;
    for part in lead::split_tokens(text) {
        match part {
            Part::Token(t) if stand_in(t).is_some() => {
                found = true;
                out.push('{');
                out.push_str(lead::NAME_TOKEN);
                out.push('}');
            }
            Part::Token(t) => {
                out.push('{');
                out.push_str(t);
                out.push('}');
            }
            Part::Text(t) | Part::Unclosed(t) => out.push_str(t),
        }
    }
    found.then_some(out)
}

/// Whether `text` says the name with no stand-in: a bare `{lead}`.
fn says_bare_name(text: &str) -> bool {
    lead::split_tokens(text).any(|part| matches!(part, Part::Token(lead::NAME_TOKEN)))
}

/// Calls `f` on the text of every line of `steps` (speech, narration,
/// replies and their reactions), in the order of
/// [`Scene::lines`](super::Scene::lines).
fn texts_mut(steps: &mut [Step], f: &mut impl FnMut(&mut String)) {
    for step in steps {
        match step {
            Step::Say { text, .. } | Step::Narrate { text, .. } => f(text),
            Step::Choice { options } => {
                for option in options {
                    f(&mut option.text);
                    texts_mut(&mut option.steps, f);
                }
            }
            Step::If {
                then, otherwise, ..
            } => {
                texts_mut(then, f);
                texts_mut(otherwise, f);
            }
            _ => {}
        }
    }
}

/// Takes the stand-ins out of the lines of `parsed` (each `{lead|<words>}`
/// becomes `{lead}`) and gives the lines their ids again from what is
/// left. Returns the lines that had one, as written.
pub(super) fn take(parsed: &mut ParsedScene) -> StandIns {
    let mut written = Vec::new();
    texts_mut(&mut parsed.scene.steps, &mut |text| {
        written.push(strip(text).map(|shown| core::mem::replace(text, shown)));
    });
    if written.iter().all(Option::is_none) {
        return StandIns::new();
    }
    parsed.scene.assign_line_ids();
    let lines = parsed.scene.lines();
    lines
        .iter()
        .zip(written)
        .filter_map(|(line, written)| Some((line.id.clone(), written?)))
        .collect()
}

/// `dialogue` with the lines of `stand_ins` as written: the words a voice
/// says, which `voice::spoken_text` reads the stand-ins from. Line ids are
/// unchanged.
pub fn written(dialogue: &DialogueTable, stand_ins: &StandIns) -> DialogueTable {
    let mut out = dialogue.clone();
    if stand_ins.is_empty() {
        return out;
    }
    for scene in out.scenes.values_mut() {
        let ids: Vec<LineId> = scene.lines().iter().map(|l| l.id.clone()).collect();
        let mut ids = ids.iter();
        texts_mut(&mut scene.steps, &mut |text| {
            if let Some(as_written) = ids.next().and_then(|id| stand_ins.get(id)) {
                text.clone_from(as_written);
            }
        });
    }
    out
}

/// One error for each speech or narration line of `parsed` that says the
/// player's name with no stand-in: its voice could say nothing there.
/// Replies are silent, so they need none.
pub fn missing(parsed: &ParsedScene, stand_ins: &StandIns) -> Vec<ContentError> {
    let mut errors = Vec::new();
    walk(&parsed.scene.steps, &parsed.lines, &mut |text, id, at| {
        let as_written = stand_ins.get(id).map_or(text, String::as_str);
        if says_bare_name(as_written) {
            errors.push(
                ContentError::new(
                    &parsed.file,
                    "the voice can't say the name the player typed: write the words it says \
                     instead after a bar, {lead|the exile} (docs/voice.md)",
                )
                .at(at, None),
            );
        }
    });
    errors
}

/// Calls `f(text, id, line)` for every speech and narration line of
/// `steps`, written at `lines`.
fn walk(steps: &[Step], lines: &Lines, f: &mut impl FnMut(&str, &LineId, u32)) {
    let mut blocks = lines.blocks.iter();
    for (step, &at) in steps.iter().zip(&lines.steps) {
        let parts: Vec<&[Step]> = match step {
            Step::Say { text, line, .. } | Step::Narrate { text, line, .. } => {
                f(text, line, at);
                continue;
            }
            Step::Choice { options } => options.iter().map(|o| &o.steps[..]).collect(),
            Step::If {
                then, otherwise, ..
            } => vec![then, otherwise],
            _ => continue,
        };
        if let Some(block) = blocks.next() {
            for (part, part_lines) in parts.into_iter().zip(block) {
                walk(part, &part_lines.lines, f);
            }
        }
    }
}

#[cfg(test)]
mod tests;
