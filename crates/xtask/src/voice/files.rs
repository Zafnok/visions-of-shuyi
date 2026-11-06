//! The files of a language's voice folder the tool reads and writes
//! (`docs/voice.md`): the cast (`cast.ron`), how to spell words for the
//! readers (`say.ron`), the manifest (`voice.ron`) and what was sent to
//! make each clip (`sent.ron`).

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde::{Deserialize, Serialize};
use trpg_content::dialogue::{LineId, NARRATION_SPEAKER};
use trpg_content::voice::{MadeBy, Variant, VoiceClip};
use trpg_core::lead::LEAD_ID;

/// The cast's file name inside a language's folder. The game never reads it.
pub const CAST_FILE: &str = "cast.ron";
/// How to spell names and words for the readers.
pub const SAY_FILE: &str = "say.ron";
/// What the tool sent to make each clip.
pub const SENT_FILE: &str = "sent.ron";
/// The cast entries of the lead's two voices (a man's and a woman's).
pub const LEAD_VOICES: [&str; 2] = ["lead_m", "lead_f"];

/// The program that reads a cast entry's lines (ADR-0107 rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Deserialize)]
pub enum Reader {
    /// Qwen3-TTS's Base weights, cloning the reference clip.
    Qwen3,
    /// Chatterbox, reading from the reference clip.
    Chatterbox,
}

/// One cast entry: the voice a speaker's lines are said in.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Voice {
    /// Its reference clip: `assets-src/audio/voice-refs/<clip>.flac`. The
    /// manifest names a clip's voice by it.
    pub clip: String,
    /// Who reads its undirected lines.
    pub reader: Reader,
}

/// `cast.ron`: the voice of each speaker, by character id, `>` for
/// narration, and `lead_m` and `lead_f` for the lead.
#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Cast {
    /// Voices by cast entry.
    pub voices: BTreeMap<String, Voice>,
}

impl Cast {
    /// The cast entry that says `speaker`'s line as `variant`: the lead's
    /// by gender, everyone else's by speaker.
    pub fn entry(speaker: &str, variant: Variant) -> &str {
        match (speaker, variant) {
            (LEAD_ID, Variant::F) => LEAD_VOICES[1],
            (LEAD_ID, _) => LEAD_VOICES[0],
            _ => speaker,
        }
    }
}

/// Reads `cast.ron` from `source`: every entry is narration, one of the
/// lead's voices or a character that `can_speak`, with a clip.
pub fn cast(source: &str, can_speak: impl Fn(&str) -> bool) -> Result<Cast, String> {
    let cast: Cast = ron::from_str(source).map_err(|e| format!("{CAST_FILE}: {e}"))?;
    let mut errors = Vec::new();
    for (entry, voice) in &cast.voices {
        let known = entry == NARRATION_SPEAKER || LEAD_VOICES.contains(&entry.as_str());
        if !known && (entry == LEAD_ID || !can_speak(entry)) {
            errors.push(format!(
                "{CAST_FILE}: \"{entry}\" is not a character that speaks (the lead's voices are \
                 \"lead_m\" and \"lead_f\", narration's \">\")"
            ));
        }
        if voice.clip.trim().is_empty() {
            errors.push(format!("{CAST_FILE}: \"{entry}\" has no clip"));
        }
    }
    if errors.is_empty() {
        Ok(cast)
    } else {
        Err(errors.join("\n"))
    }
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
