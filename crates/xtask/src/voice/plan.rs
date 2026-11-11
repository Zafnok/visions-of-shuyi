//! The hooks the engine's clip plan takes (`shuyi_assets::voice::plan`,
//! ticket 1242): which clips a line needs, the words each says, and why a
//! generated clip's voice changed (its reference clip, its reader, or the
//! words the reader would be sent).

use std::collections::{BTreeMap, BTreeSet};

use shuyi_assets::voice::plan::{self as engine, VoicePlan, WantedClip};
use shuyi_assets::voice::{CastVoice, ClipVariant};
use trpg_content::dialogue::{DialogueTable, REPLY_SPEAKER};
use trpg_content::names::Names;
use trpg_content::voice::{self, MadeBy, Variant, VoiceClip};
use trpg_core::lead::{LEAD_ID, LeadGender};

use super::files::{Cast, Reader, Say, SentMap, for_reader, respell};

/// Everything there is to do.
pub type Plan = VoicePlan<Variant>;
/// A clip a line needs.
pub type Wanted = WantedClip<Variant>;

/// Why: the reference clip was recorded again.
pub const REFERENCE: &str = "the reference clip changed";
/// Why: another program reads the voice's lines.
pub const READER: &str = "the reader changed";
/// Why: the words would be sent another way: `say.ron` spells a word of
/// them another way, or the tool reads their punctuation another way
/// (ticket 0762).
pub const SPELLING: &str = "it would be sent differently (say.ron, punctuation)";

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

/// The words sent to a reader for a clip that says `spoken`: respelled by
/// `say`, its trailing-offs and breaks read as [`for_reader`] says.
pub fn sent(spoken: &str, say: &Say, names: &Names) -> String {
    for_reader(&respell(spoken, say, |id| names.get(id).map(str::to_owned)))
}

/// The key a reader writes `w`'s take under: its file name without `.ogg`.
pub fn key(w: &Wanted) -> String {
    format!("{}{}", w.line, w.variant.suffix())
}

/// Makes the plan.
pub fn plan(inputs: &Inputs<'_>) -> Plan {
    let names = inputs.names;
    engine::plan(
        inputs.dialogue,
        inputs.scenes,
        inputs.cast,
        inputs.clips,
        variants,
        |text, variant| spoken(text, variant, names),
        |clip, cast| voice_changed(clip, cast, inputs),
    )
}

/// Why generated `clip`, still in `cast`'s voice and saying its line's
/// words, must be made again, if it must.
fn voice_changed(
    clip: &VoiceClip,
    cast: &CastVoice<Reader>,
    inputs: &Inputs<'_>,
) -> Option<String> {
    let MadeBy::Generated { tool, .. } = &clip.made_by else {
        return None;
    };
    let sent = inputs.sent.get(&voice::clip_file(&clip.line, clip.variant));
    let reference = inputs.references.get(&cast.voice);
    let why = if sent.is_some_and(|s| Some(&s.reference) != reference) {
        REFERENCE
    } else if *tool != (inputs.tool)(cast.settings, clip.direction.is_some()) {
        READER
    } else if sent.is_some_and(|s| s.text != self::sent(&clip.spoken, inputs.say, inputs.names)) {
        SPELLING
    } else {
        return None;
    };
    Some(why.to_owned())
}
