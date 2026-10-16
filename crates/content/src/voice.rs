//! Voice clips (ADR-0046, `docs/voice.md`): the manifest
//! `voice/<lang>/voice.ron` that lists every clip by dialogue line id, and
//! the cast `voice/<lang>/cast.ron`. The `voice/` folder ships beside the
//! game and is not embedded, so nothing here reads a file: `app` reads the
//! text and these functions parse and validate it.
//!
//! A clip records the words it says (`spoken`). When the line says
//! something else today (a name in `names.ron` changed), the clip is
//! **stale** and is left out of the [`Playable`] set.
//!
//! The manifest, the cast and the staleness rule are the engine's
//! (`shuyi_assets::voice`, ticket 1105); the game adds its folders, its
//! clip [`Variant`]s (one per gender of the lead) and the words a line says
//! of the lead ([`spoken_text`]).
use crate::prelude::*;

use serde::Deserialize;
use shuyi_assets::voice::{self as engine, ClipVariant};
use trpg_core::lead::{self, LeadGender, LeadProfile, Part};

use crate::character::CharacterTable;
use crate::dialogue::{DialogueTable, LineId};
use crate::error::ContentError;
use crate::names::Names;

pub use shuyi_assets::voice::{Cast, MadeBy};

/// Folder, next to the game, that holds one folder of clips per language.
/// Not embedded (ADR-0046 rule 4).
pub const VOICE_DIR: &str = "voice";
/// The manifest's file name inside a language's folder.
pub const MANIFEST_FILE: &str = "voice.ron";
/// The cast's file name inside a language's folder.
pub const CAST_FILE: &str = "cast.ron";
/// The language the script is written in: the only voice folder read until
/// the player can pick a language (ADR-0045 §5).
pub const SOURCE_LANG: &str = "en";

/// Which recording of a line a clip is: lines whose words change with the
/// lead's gender (`{they}`, `{their}`…) have one for each.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Deserialize)]
pub enum Variant {
    /// The line's only clip: `<line id>.ogg`.
    #[default]
    None,
    /// Said of a male lead: `<line id>.m.ogg`.
    M,
    /// Said of a female lead: `<line id>.f.ogg`.
    F,
}

impl Variant {
    /// The variant said of a lead of `gender`.
    pub const fn of(gender: LeadGender) -> Self {
        match gender {
            LeadGender::Male => Self::M,
            LeadGender::Female => Self::F,
        }
    }

    /// The lead's gender this variant is said of.
    pub const fn gender(self) -> Option<LeadGender> {
        match self {
            Self::None => None,
            Self::M => Some(LeadGender::Male),
            Self::F => Some(LeadGender::Female),
        }
    }
}

impl ClipVariant for Variant {
    /// What goes between the line id and `.ogg` in the file name.
    fn suffix(self) -> &'static str {
        match self {
            Self::None => "",
            Self::M => ".m",
            Self::F => ".f",
        }
    }

    fn split() -> &'static [Self] {
        &[Self::M, Self::F]
    }

    fn incomplete(line: &str) -> String {
        format!("line \"{line}\" has an M or an F clip without the other")
    }

    fn mixed(line: &str) -> String {
        format!("line \"{line}\" has a clip for both genders and M or F clips; keep one kind")
    }
}

/// One clip of the manifest.
pub type VoiceClip = engine::VoiceClip<Variant>;
/// The validated voice manifest of one language.
pub type VoiceManifest = engine::VoiceManifest<Variant>;
/// The clips that may be played: those that say what their line says
/// today.
pub type Playable = engine::Playable<Variant>;

/// The words `text` (a dialogue line as written) says of a lead of
/// `gender`, with today's `names`: every name and pronoun token filled in.
/// `None` for a line with `{lead}`: the player types that name, so no clip
/// can say it.
pub fn spoken_text(text: &str, names: &Names, gender: LeadGender) -> Option<String> {
    // Only the pronouns are read: a line with the name has no spoken text.
    let lead = LeadProfile::new("", gender);
    let mut out = String::with_capacity(text.len());
    for part in lead::split_tokens(&names.substitute(text)) {
        match part {
            Part::Token(lead::NAME_TOKEN) => return None,
            Part::Token(t) => {
                if let Some(word) = lead.token(t) {
                    out.push_str(&word);
                } else {
                    out.push('{');
                    out.push_str(t);
                    out.push('}');
                }
            }
            Part::Text(t) | Part::Unclosed(t) => out.push_str(t),
        }
    }
    Some(out)
}

/// Whether the words of `text` change with the lead's gender (it has a
/// pronoun token), so it needs an [`M`](Variant::M) and an
/// [`F`](Variant::F) clip.
pub fn changes_with_gender(text: &str) -> bool {
    lead::split_tokens(text).any(|part| match part {
        Part::Token(t) => t != lead::NAME_TOKEN && lead::is_token(t),
        Part::Text(_) | Part::Unclosed(_) => false,
    })
}

/// A clip's file, relative to its language's folder:
/// `<scene id>/<line id>.ogg`, with `.m` or `.f` before `.ogg` for a
/// gendered variant (ADR-0046 rule 2).
pub fn clip_file(line: &LineId, variant: Variant) -> String {
    engine::clip_file(line, variant)
}

/// Parses and validates manifest `source`, attributing errors to `file`:
/// every clip's line exists in `dialogue`, no line has the same variant
/// twice, a line has either one clip or an `M` and `F` pair, and `spoken`
/// and `voice` aren't empty. Reports every problem found.
///
/// A stale clip is not an error (see [`playable`]).
pub fn from_source(
    file: &str,
    source: &str,
    dialogue: &DialogueTable,
) -> Result<VoiceManifest, Vec<ContentError>> {
    engine::from_source(file, source, dialogue)
}

/// Parses and validates cast `source`, attributing errors to `file`: no
/// voice id is empty, and every speaker is the narration's or, when
/// `characters` is given, a character that can speak.
pub fn cast_from_source(
    file: &str,
    source: &str,
    characters: Option<&CharacterTable>,
) -> Result<Cast, Vec<ContentError>> {
    let can_speak = |speaker: &str| {
        characters.is_none_or(|c| c.can_speak(&trpg_core::CharacterId(speaker.to_owned())))
    };
    engine::cast_from_source(file, source, can_speak)
}

/// The words a clip of `variant` must say for the line written `text`,
/// with today's `names`: [`spoken_text`] of the lead of its gender. `None`
/// when no clip of that variant can say the line: it has `{lead}`, or it
/// changes with the lead's gender and the clip is for both.
fn spoken(text: &str, variant: Variant, names: &Names) -> Option<String> {
    let gender = match variant.gender() {
        Some(gender) => gender,
        None if changes_with_gender(text) => return None,
        None => LeadGender::Male,
    };
    spoken_text(text, names, gender)
}

/// Whether `clip` is stale: its `spoken` is not what its line says
/// today (with `names`, of the lead its variant is for). A clip of a
/// line that is gone, has `{lead}`, or changes with the lead's gender
/// while the clip is for both, is stale too.
pub fn is_stale(clip: &VoiceClip, dialogue: &DialogueTable, names: &Names) -> bool {
    VoiceManifest::is_stale(clip, dialogue, |text, v| spoken(text, v, names))
}

/// The clips of `manifest` that aren't stale ([`is_stale`]).
pub fn playable(manifest: &VoiceManifest, dialogue: &DialogueTable, names: &Names) -> Playable {
    manifest.playable(dialogue, |text, v| spoken(text, v, names))
}

#[cfg(test)]
mod tests;
