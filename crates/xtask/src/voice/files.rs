//! The files of a language's voice folder the tool reads and writes
//! (`docs/voice.md`): the cast (`cast.ron`), how to spell words for the
//! readers (`say.ron`), the manifest (`voice.ron`) and what was sent to
//! make each clip (`sent.ron`).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use shuyi_assets::voice;
use trpg_content::dialogue::LineId;
use trpg_content::voice::{MadeBy, Variant, VoiceClip};

/// The cast's file name inside a language's folder. The game never reads it.
pub const CAST_FILE: &str = "cast.ron";
/// How to spell names and words for the readers.
pub const SAY_FILE: &str = "say.ron";
/// What the tool sent to make each clip.
pub const SENT_FILE: &str = "sent.ron";

/// The program that reads a cast entry's lines (ADR-0107 rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Reader {
    /// Qwen3-TTS's Base weights, cloning the reference clip.
    Qwen3,
    /// Chatterbox, reading from the reference clip.
    Chatterbox,
}

/// `cast.ron`: the voice of each speaker, by character id, `>` for
/// narration, and `lead.m` and `lead.f` for the lead's two voices (the
/// engine's cast, with the reader as each voice's settings).
pub type Cast = voice::Cast<Reader>;

/// Reads `cast.ron` from `source` with the engine's reader: every key is
/// narration or a character that `can_speak`, alone or with a variant's
/// suffix.
pub fn cast(source: &str, can_speak: impl Fn(&str) -> bool) -> Result<Cast, String> {
    voice::cast_from_source::<Reader, Variant>(CAST_FILE, source, can_speak).map_err(|errors| {
        let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
        lines.join("\n")
    })
}

/// `say.ron`: a name id or a word, and how to spell it for the readers.
pub type Say = BTreeMap<String, String>;

/// Reads `say.ron` from `source`.
pub fn say(source: &str) -> Result<Say, String> {
    ron::from_str(source).map_err(|e| format!("{SAY_FILE}: {e}"))
}

/// `text` with every whole word of `say` spelled as it says. A key that is
/// a name id stands for that name today.
pub fn respell(text: &str, say: &Say, name: impl Fn(&str) -> Option<String>) -> String {
    let mut out = text.to_owned();
    for (key, spelling) in say {
        let word = name(key).unwrap_or_else(|| key.clone());
        if !word.is_empty() {
            out = replace_word(&out, &word, spelling);
        }
    }
    out
}

/// Whether `text` starts with a mark of a trailing-off or a break: `..`
/// or more dots, `…`, or `--`. Returns the mark's length in bytes.
fn break_mark(text: &str) -> Option<usize> {
    if text.starts_with('\u{2026}') {
        Some('\u{2026}'.len_utf8())
    } else if text.starts_with("..") || text.starts_with("--") {
        let first = text.as_bytes()[0];
        Some(text.bytes().take_while(|&b| b == first).count())
    } else {
        None
    }
}

/// What a run of marks between `before` and `after` is read as (ticket
/// 0762): the readers hold a long silence at every `...` and `--`, and
/// longer at several. A run after a sentence's end or before other
/// punctuation goes; one at the end is a full stop. Any other is an em
/// dash if it has a break (`--`, `broken`): the readers cut off at it as a
/// person does. A trailing-off is a full stop before a new sentence (a
/// capital that isn't "I"), else a comma. The trial that
/// chose these is in ticket 0762's Completion notes.
fn break_as(before: &str, after: &str, broken: bool) -> &'static str {
    let ended = before.trim_end_matches(['"', '\u{201d}', '\'', ' ']);
    let next = after.trim_start_matches(['"', '\u{201d}', ' ']);
    let word: String = next
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '\'')
        .collect();
    let new_sentence =
        word.starts_with(char::is_uppercase) && word != "I" && !word.starts_with("I'");
    let ends = |c: char| matches!(c, '.' | '?' | '!' | ',' | ';' | ':');
    match (ended.chars().next_back(), next.chars().next()) {
        (Some(last), Some(_)) if ends(last) => " ",
        (Some(last), None) if !ends(last) => ".",
        (Some(_), Some(first)) if !ends(first) && broken => "\u{2014} ",
        (Some(_), Some(first)) if !ends(first) && new_sentence => ". ",
        (Some(_), Some(first)) if !ends(first) => ", ",
        _ => "",
    }
}

/// `text` as it is sent to a reader (ticket 0762): every run of
/// trailing-offs (`...`, `…`) and breaks (`--`), with the spaces around
/// it, read as [`break_as`] says. The words and every other mark stay.
pub fn for_reader(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(c) = rest.chars().next() {
        if break_mark(rest).is_none() {
            out.push(c);
            rest = &rest[c.len_utf8()..];
            continue;
        }
        // The run of marks and the spaces between them.
        let mut broken = false;
        loop {
            let spaces = rest.len() - rest.trim_start().len();
            match break_mark(&rest[spaces..]) {
                Some(len) => {
                    broken |= rest[spaces..].starts_with('-');
                    rest = &rest[spaces + len..];
                }
                None => break,
            }
        }
        rest = rest.trim_start();
        out.truncate(out.trim_end().len());
        out.push_str(break_as(&out, rest, broken));
    }
    out
}

/// `text` with each `word` that isn't part of a longer word replaced by
/// `with`.
fn replace_word(text: &str, word: &str, with: &str) -> String {
    let joins = |c: Option<char>| c.is_some_and(|c| c.is_alphanumeric() || c == '\'');
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(word) {
        let end = at + word.len();
        let alone = !joins(rest[..at].chars().next_back()) && !joins(rest[end..].chars().next());
        out.push_str(&rest[..at]);
        out.push_str(if alone { with } else { word });
        rest = &rest[end..];
    }
    out.push_str(rest);
    out
}

/// What the tool sent to make one clip.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sent {
    /// The words, as spelled for the reader.
    pub text: String,
    /// The reference clip's fingerprint ([`fingerprint`]).
    pub reference: String,
}

/// `sent.ron`: what was sent for each clip, by clip file.
pub type SentMap = BTreeMap<String, Sent>;

/// Reads `sent.ron` from `source`.
pub fn sent(source: &str) -> Result<SentMap, String> {
    ron::from_str(source).map_err(|e| format!("{SENT_FILE}: {e}"))
}

/// `sent.ron`'s text, with `\n` line ends on every system (ron's own
/// default is `\r\n` on Windows).
pub fn sent_text(sent: &SentMap) -> String {
    let pretty = ron::ser::PrettyConfig::new()
        .indentor("    ".to_owned())
        .new_line("\n".to_owned());
    let body = ron::ser::to_string_pretty(sent, pretty).unwrap_or_default();
    format!(
        "// What `cargo xtask voice` sent to make each clip (docs/voice.md). The game\n\
         // doesn't read it; the tool makes a clip again when it would send other words\n\
         // or its reference clip changed.\n{body}\n"
    )
}

/// A file's fingerprint: FNV-1a 32 of its bytes, 8 hex digits.
pub fn fingerprint(bytes: &[u8]) -> String {
    let hash = bytes.iter().fold(0x811c_9dc5_u32, |hash, &b| {
        (hash ^ u32::from(b)).wrapping_mul(0x0100_0193)
    });
    format!("{hash:08x}")
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawManifest {
    clips: Vec<RawClip>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawClip {
    line: String,
    #[serde(default)]
    variant: Variant,
    spoken: String,
    #[serde(default)]
    direction: Option<String>,
    voice: String,
    made_by: MadeBy,
}

/// Reads the manifest's clips from `source`, without the game's checks: a
/// clip of a line that is gone is the tool's to remove.
pub fn manifest(source: &str) -> Result<Vec<VoiceClip>, String> {
    let raw: RawManifest = ron::from_str(source).map_err(|e| format!("voice.ron: {e}"))?;
    Ok(raw
        .clips
        .into_iter()
        .map(|c| VoiceClip {
            line: LineId::new(c.line),
            variant: c.variant,
            spoken: c.spoken,
            direction: c.direction,
            voice: c.voice,
            made_by: c.made_by,
        })
        .collect())
}

/// The manifest's text for `clips`, sorted by line id and variant.
pub fn manifest_text(clips: &[VoiceClip]) -> String {
    let quote = |s: &str| ron::ser::to_string(s).unwrap_or_default();
    let mut sorted: Vec<&VoiceClip> = clips.iter().collect();
    sorted.sort_by(|a, b| (a.line.as_str(), a.variant).cmp(&(b.line.as_str(), b.variant)));
    let mut out = String::from(
        "// Every voice clip (docs/voice.md), written by `cargo xtask voice generate`.\n(\n    \
         clips: [\n",
    );
    for clip in sorted {
        out.push_str("        (\n");
        let _ = writeln!(out, "            line: {},", quote(clip.line.as_str()));
        if clip.variant != Variant::None {
            let _ = writeln!(out, "            variant: {:?},", clip.variant);
        }
        let _ = writeln!(out, "            spoken: {},", quote(&clip.spoken));
        if let Some(direction) = &clip.direction {
            let _ = writeln!(out, "            direction: Some({}),", quote(direction));
        }
        let _ = writeln!(out, "            voice: {},", quote(&clip.voice));
        let made_by = match &clip.made_by {
            MadeBy::Generated { tool, model, date } => format!(
                "Generated(tool: {}, model: {}, date: {})",
                quote(tool),
                quote(model),
                quote(date)
            ),
            MadeBy::Recorded { actor } => format!("Recorded(actor: {})", quote(actor)),
        };
        let _ = write!(out, "            made_by: {made_by},\n        ),\n");
    }
    out.push_str("    ],\n)\n");
    out
}
