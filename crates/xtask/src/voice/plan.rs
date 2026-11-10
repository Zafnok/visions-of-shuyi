//! What `cargo xtask voice` has to do, as a pure function of the scripts,
//! the names, the cast, `say.ron`, the manifest and `sent.ron`: the clips
//! to make or make again (and why), the recorded clips a person must record
//! again, and the clips to remove.

use std::collections::{BTreeMap, BTreeSet};

use trpg_content::dialogue::{DialogueTable, LineId, REPLY_SPEAKER};
use trpg_content::names::Names;
use trpg_content::voice::{self, MadeBy, Variant, VoiceClip};
use trpg_core::lead::{LEAD_ID, LeadGender};

use super::files::{Cast, Reader, Say, SentMap, for_reader, respell};

/// Why a clip is to be made.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Why {
    /// The line has no clip.
    Missing,
    /// The clip says other words than the line says today.
    Words,
    /// The line's direction changed.
    Direction,
    /// The speaker's voice or its reader changed.
    Voice,
    /// The words would be sent another way: `say.ron` spells a word of
    /// them another way, or the tool reads their punctuation another way
    /// (ticket 0762).
    Spelling,
}

impl Why {
    /// What `status` says of it.
    pub const fn says(self) -> &'static str {
        match self {
            Self::Missing => "no clip",
            Self::Words => "the words changed",
            Self::Direction => "the direction changed",
            Self::Voice => "the voice changed",
            Self::Spelling => "it would be sent differently (say.ron, punctuation)",
        }
    }
}

/// A clip a line needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wanted {
    /// The line.
    pub line: LineId,
    /// Which of its clips.
    pub variant: Variant,
    /// Who says it (a character id, or `>`).
    pub speaker: String,
    /// The cast entry whose voice says it.
    pub entry: String,
    /// The words it says (the manifest's `spoken`).
    pub spoken: String,
    /// The words sent to the reader: `spoken` respelled by `say.ron`, its
    /// trailing-offs and breaks read as [`for_reader`] says.
    pub sent: String,
    /// How it is said.
    pub direction: Option<String>,
    /// The line before it in the scene: its speaker and words.
    pub before: Option<(String, String)>,
}

impl Wanted {
    /// Its file, relative to the language's folder.
    pub fn file(&self) -> String {
        voice::clip_file(&self.line, self.variant)
    }

    /// The key a reader writes its take under: the file name without
    /// `.ogg`.
    pub fn key(&self) -> String {
        format!("{}{}", self.line, suffix(self.variant))
    }
}

/// What goes between the line id and `.ogg`.
const fn suffix(variant: Variant) -> &'static str {
    match variant {
        Variant::None => "",
        Variant::M => ".m",
        Variant::F => ".f",
    }
}

/// Everything there is to do.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Plan {
    /// Every clip the lines need, by scene id, then in script order.
    pub wanted: Vec<Wanted>,
    /// The clips of [`wanted`](Self::wanted) to make, by index.
    pub make: Vec<(usize, Why)>,
    /// Recorded clips of [`wanted`](Self::wanted) a person must record
    /// again, by index. The tool never makes these.
    pub record: Vec<(usize, Why)>,
    /// Clips to remove: their line is gone, or it needs other clips.
    pub remove: Vec<VoiceClip>,
    /// Lines whose speaker has no voice in the cast: line and cast entry.
    pub uncast: Vec<(LineId, String)>,
}

/// What a plan is made from.
pub struct Inputs<'a> {
    /// The lines as written (`trpg_content::dialogue::written`).
    pub dialogue: &'a DialogueTable,
    /// The scenes to voice.
    pub scenes: &'a BTreeSet<String>,
    /// Today's names.
    pub names: &'a Names,
    /// The cast.
    pub cast: &'a Cast,
    /// How to spell words for the readers.
    pub say: &'a Say,
    /// The manifest's clips.
    pub clips: &'a [VoiceClip],
    /// What was sent to make each clip.
    pub sent: &'a SentMap,
    /// Each reference clip's fingerprint, by clip name.
    pub references: &'a BTreeMap<String, String>,
    /// The tool the manifest names for a reader's take, directed or not.
    pub tool: &'a dyn Fn(Reader, bool) -> String,
}

/// The clips a line said by `speaker` needs: none for a reply, the lead's
/// voices for the lead, a pair for a line whose words change with the
/// lead's gender, else one.
fn variants(speaker: &str, text: &str) -> &'static [Variant] {
    if speaker == REPLY_SPEAKER {
        &[]
    } else if speaker == LEAD_ID || voice::changes_with_gender(text) {
        &[Variant::M, Variant::F]
    } else {
        &[Variant::None]
    }
}

/// The words a line written `text` says as `variant`, with `names`.
fn spoken(text: &str, variant: Variant, names: &Names) -> Option<String> {
    let gender = variant.gender().unwrap_or(LeadGender::Male);
    voice::spoken_text(text, names, gender)
}

/// Makes the plan.
pub fn plan(inputs: &Inputs<'_>) -> Plan {
    let names = inputs.names;
    let mut plan = Plan::default();
    let mut needed = BTreeSet::new();
    let mut voiced = BTreeSet::new();
    let mut all = BTreeSet::new();
    for (id, scene) in &inputs.dialogue.scenes {
        let lines = scene.lines();
        all.extend(lines.iter().map(|l| l.id.clone()));
        if !inputs.scenes.contains(id) {
            continue;
        }
        let mut before = None;
        for line in lines {
            voiced.insert(line.id.clone());
            for &variant in variants(line.speaker, line.text) {
                let Some(words) = spoken(line.text, variant, names) else {
                    continue;
                };
                needed.insert((line.id.clone(), variant));
                let entry = Cast::entry(line.speaker, variant).to_owned();
                if !inputs.cast.voices.contains_key(&entry) {
                    plan.uncast.push((line.id.clone(), entry));
                    continue;
                }
                let respelled = respell(&words, inputs.say, |id| names.get(id).map(str::to_owned));
                let sent = for_reader(&respelled);
                plan.wanted.push(Wanted {
                    line: line.id.clone(),
                    variant,
                    speaker: line.speaker.to_owned(),
                    entry,
                    spoken: words,
                    sent,
                    direction: line.direction.map(str::to_owned),
                    before: before.clone(),
                });
            }
            let shown = spoken(line.text, Variant::M, names)
                .unwrap_or_else(|| names.substitute(line.text).into_owned());
            before = Some((line.speaker.to_owned(), shown));
        }
    }
    let made: BTreeMap<(&LineId, Variant), &VoiceClip> = inputs
        .clips
        .iter()
        .map(|clip| ((&clip.line, clip.variant), clip))
        .collect();
    for (i, wanted) in plan.wanted.iter().enumerate() {
        let Some(&clip) = made.get(&(&wanted.line, wanted.variant)) else {
            plan.make.push((i, Why::Missing));
            continue;
        };
        if let Some(why) = why_remake(clip, wanted, inputs) {
            match clip.made_by {
                MadeBy::Recorded { .. } => plan.record.push((i, why)),
                MadeBy::Generated { .. } => plan.make.push((i, why)),
            }
        }
    }
    plan.remove = inputs
        .clips
        .iter()
        .filter(|clip| {
            let ours = voiced.contains(&clip.line) || !all.contains(&clip.line);
            ours && !needed.contains(&(clip.line.clone(), clip.variant))
        })
        .cloned()
        .collect();
    plan
}

/// Why `clip`, made for `wanted`'s line, must be made again, if it must.
/// A recorded clip is judged by its words and direction only.
fn why_remake(clip: &VoiceClip, wanted: &Wanted, inputs: &Inputs<'_>) -> Option<Why> {
    let (dialogue, names) = (inputs.dialogue, inputs.names);
    if voice::is_stale(clip, dialogue, names) {
        return Some(Why::Words);
    }
    if voice::needs_remake(clip, dialogue, names) {
        return Some(Why::Direction);
    }
    let MadeBy::Generated { tool, .. } = &clip.made_by else {
        return None;
    };
    let cast = &inputs.cast.voices[&wanted.entry];
    let reference = inputs.references.get(&cast.clip);
    let sent = inputs.sent.get(&wanted.file());
    let voice_changed = clip.voice != cast.clip
        || *tool != (inputs.tool)(cast.reader, wanted.direction.is_some())
        || sent.is_some_and(|s| Some(&s.reference) != reference);
    if voice_changed {
        Some(Why::Voice)
    } else if sent.is_some_and(|s| s.text != wanted.sent) {
        Some(Why::Spelling)
    } else {
        None
    }
}
