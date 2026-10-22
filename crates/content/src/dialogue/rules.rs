//! What the engine's scene checks ask the game: its cast, portraits, music
//! cues and the lead's rules ([`Rules`]), and the lead's tokens
//! ([`LeadTokens`]: `{lead}`, `{they}`…, and the names a script may not
//! write as a name token or spell out).
use crate::prelude::*;

use shuyi_assets::dialogue::{DialogueLimits, SceneRules, Tokens};
use trpg_core::CharacterId;
use trpg_core::lead::{self, LEAD_ID, PORTRAIT_FEMALE, PORTRAIT_MALE};

use super::{
    LIMITS, MAX_LEAD_LINE_LEN, ParsedScene, STANDARD_EXPRESSIONS, Stage, deaths, openings,
};
use crate::audio::AudioManifest;
use crate::character::CharacterTable;
use crate::error::ContentError;
use crate::names::Names;
use crate::portrait::{Portrait, PortraitTable};

/// The lead's tokens (`{lead}`, `{they}`, `{them}`, `{their}`, `{theirs}`,
/// `{themself}`, capitalised too), filled in by
/// [`LeadProfile::substitute`](trpg_core::LeadProfile::substitute); and the
/// lead's default name, which the names table has as `lead` but a script
/// writes as `{lead}`.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct LeadTokens;

impl Tokens for LeadTokens {
    fn longest(&self, token: &str) -> Option<usize> {
        lead::longest(token)
    }

    fn unknown(&self, token: &str) -> String {
        format!(
            "unknown token \"{{{token}}}\"; use {{lead}}, {{they}}, {{them}}, {{their}}, \
             {{theirs}}, {{themself}} (capitalised: {{They}}...) or a name, {{n:<id>}}"
        )
    }

    fn reserved_name(&self, id: &str, token: &str) -> Option<String> {
        (id == LEAD_ID).then(|| {
            format!(
                "{{{token}}} is only the lead's default name; write {{lead}} for the name the player chose"
            )
        })
    }

    fn unknown_name(&self, id: &str, token: &str) -> String {
        format!(
            "unknown name id \"{id}\" in {{{token}}}; name ids are listed in assets/data/names.ron"
        )
    }

    fn written_name(&self, id: &str, form: &str) -> Option<String> {
        (id == LEAD_ID).then(|| {
            format!(
                "\"{form}\" is the lead's default name; write {{lead}} for the name the player chose"
            )
        })
    }
}

/// The game's tables a scene is checked against, each `None` if its files
/// failed to load (its checks are skipped then).
pub(super) struct Rules<'a> {
    /// Who may be placed and speak.
    pub characters: Option<&'a CharacterTable>,
    /// The expressions of each portrait.
    pub portraits: Option<&'a PortraitTable>,
    /// The names table.
    pub names: Option<&'a Names>,
    /// The music cues.
    pub audio: Option<&'a AudioManifest>,
}

impl Rules<'_> {
    /// The portraits that `character` may show: the lead has one per
    /// gender, anyone else at most their own.
    fn portraits_of(&self, character: &CharacterId) -> Vec<&Portrait> {
        let Some(portraits) = self.portraits else {
            return Vec::new();
        };
        if character.0 == LEAD_ID {
            [PORTRAIT_MALE, PORTRAIT_FEMALE]
                .iter()
                .filter_map(|id| portraits.get(*id))
                .collect()
        } else {
            portraits.get(&character.0).into_iter().collect()
        }
    }
}

impl SceneRules<Stage> for Rules<'_> {
    fn limits(&self) -> &DialogueLimits {
        &LIMITS
    }

    fn tokens(&self) -> &dyn Tokens {
        &LeadTokens
    }

    fn names(&self) -> Option<&Names> {
        self.names
    }

    fn can_speak(&self, character: &CharacterId) -> bool {
        self.characters.is_none_or(|t| t.can_speak(character))
    }

    /// `expression` must be one of the character's portraits' expressions
    /// when they have any (the lead: both portraits'), else one of the
    /// [`STANDARD_EXPRESSIONS`].
    fn expression_problem(&self, character: &CharacterId, expression: &str) -> Option<String> {
        let portraits = self.portraits_of(character);
        if let Some(portrait) = portraits
            .iter()
            .find(|p| p.expression(expression).is_none())
        {
            let names: Vec<&str> = portrait
                .expressions
                .iter()
                .map(|e| e.name.as_str())
                .collect();
            return Some(format!(
                "\"{}\"'s portrait has no expression \"{expression}\"; use one of {}",
                portrait.character,
                names.join(", ")
            ));
        }
        (portraits.is_empty() && !STANDARD_EXPRESSIONS.contains(&expression)).then(|| {
            format!(
                "unknown expression \"{expression}\"; use one of {}",
                STANDARD_EXPRESSIONS.join(", ")
            )
        })
    }

    /// The lead speaks in short lines.
    fn speech_problem(&self, speaker: &CharacterId, cells: usize) -> Option<String> {
        (speaker.0 == LEAD_ID && cells > MAX_LEAD_LINE_LEN).then(|| {
            format!(
                "the lead's line is {cells} characters; the limit is {MAX_LEAD_LINE_LEN}: \
                 the lead speaks in short, neutral lines and otherwise through reply \
                 choices (docs/design/setting-and-tone.md, \"Rules for writing the lead\")"
            )
        })
    }

    fn music_problem(&self, cue: &str) -> Option<String> {
        self.audio.and_then(|audio| music_problem(audio, cue))
    }

    fn scene_problems(&self, parsed: &ParsedScene) -> Vec<ContentError> {
        let mut errors = openings::problems(parsed);
        errors.extend(deaths::problems(parsed, self.characters));
        errors
    }
}

/// What is wrong with `@music <cue>`, if anything: `cue` must be a music
/// cue of the audio manifest (not a sound, nor a music pool).
fn music_problem(audio: &AudioManifest, cue: &str) -> Option<String> {
    if audio.music.contains_key(cue) {
        return None;
    }
    let what = if audio.sounds.contains_key(cue) {
        "a sound, not a music cue"
    } else if audio.pools.contains_key(cue) {
        "a music pool, not a music cue"
    } else {
        "not a music cue"
    };
    Some(format!(
        "\"{cue}\" is {what}; music cues are listed in assets/audio/audio.ron"
    ))
}
