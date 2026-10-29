//! Playing a dialogue [`Scene`] one text box at a time: Shuyi Engine's
//! player (`shuyi_ui::dialogue`, ticket 1112) with this game's lines and
//! tokens ([`Hooks`]): the lead's name and pronouns and the story's names
//! fill the text, and `@place` and `@opening` set the place behind the
//! talk and the opening played on it. [`DialoguePlayer`] is the state
//! machine only (who stands where, what is said, which reply choice is
//! open, which music and which opening the script asked for); the dialogue
//! screen (ticket 0704) draws its [`View`], asks for the music and plays
//! the opening (0734).

use crate::prelude::*;
use alloc::borrow::Cow;

use shuyi_ui::dialogue::DialogueHooks;
use trpg_content::{LineId, MusicLine, Names, PlaceLine, Present, Scene, Side, Stage};
use trpg_core::LeadProfile;

pub use shuyi_ui::dialogue::Portrait;

/// What this game's own lines have set: the place behind the talk and the
/// opening played on it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Setting {
    /// The place the last `@place` line named; `None` before one, or after
    /// `@place none`.
    place: Option<PlaceLine>,
    /// The opening of the place in force: the last `@opening` line since
    /// the last `@place` line.
    opening: Option<String>,
    /// The last `@opening` line reached and not yet taken.
    opening_due: Option<String>,
}

/// This game's part in playing a scene: the lead and the story's names
/// fill the text; `@place` and `@opening` set the [`Setting`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hooks {
    /// Fills in the lead's name and pronouns.
    lead: LeadProfile,
    /// Fills in name tokens (`{n:king}`).
    names: Names,
}

impl DialogueHooks for Hooks {
    type Directive = Stage;
    type Stage = Setting;

    fn fill<'a>(&self, text: &'a str) -> Cow<'a, str> {
        match self.names.substitute(text) {
            Cow::Borrowed(text) => self.lead.substitute(text),
            Cow::Owned(text) => Cow::Owned(self.lead.substitute(&text).into_owned()),
        }
    }

    fn direct(&self, stage: &mut Setting, directive: &Stage) {
        match directive {
            Stage::Setting(place) => {
                stage.place = (*place != PlaceLine::None).then(|| place.clone());
                stage.opening = None;
            }
            Stage::Opening(id) => {
                stage.opening = Some(id.clone());
                stage.opening_due = Some(id.clone());
            }
            // The flow carries the death or the join out as the scene
            // starts.
            Stage::Dies(_) | Stage::Joins(_) => {}
        }
    }
}

/// What the screen shows for the current text box, with the lead's name
/// and pronouns and the story's names filled in.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct View<'a> {
    /// The left portrait, if any.
    pub left: Option<Portrait<'a>>,
    /// The right portrait, if any.
    pub right: Option<Portrait<'a>>,
    /// Which side is speaking (`None` for narration, or when finished).
    pub speaker: Option<Side>,
    /// The text box's text (`None` once the scene is finished). While a
    /// choice is open, the text box before it (the line being answered).
    pub text: Option<Cow<'a, str>>,
    /// The location/time caption, if one has been shown.
    pub caption: Option<Cow<'a, str>>,
    /// The place behind the talk, if the scene has named one (and not gone
    /// back to none since).
    pub place: Option<&'a PlaceLine>,
    /// The opening played on that place, if one was (`@opening`, 0734):
    /// its characters stay where it left them until the place changes.
    pub opening: Option<&'a str>,
    /// Whether the text is narration (no speaker).
    pub narration: bool,
    /// The lead's replies, in menu order, while a choice is open: pick
    /// one with [`DialoguePlayer::choose`].
    pub choices: Option<Vec<Cow<'a, str>>>,
}

/// Plays a scene ([`shuyi_ui::dialogue::DialoguePlayer`]): each
/// [`advance`](Self::advance) moves to the next text box, applying the
/// portrait, caption, place and opening steps in between. At a reply
/// choice it waits for [`choose`](Self::choose), plays that reply's
/// reaction, then rejoins the scene after the choice. It plays the scene
/// as it is for those present when it starts (ADR-0055). Cheap to clone,
/// so a copy can be kept as the scene was at a line (the dialogue log,
/// ticket 0729).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialoguePlayer(shuyi_ui::dialogue::DialoguePlayer<Hooks>);

impl DialoguePlayer {
    /// Starts `scene` as it plays for those `present`, at its first text
    /// box, with `lead`'s name and pronouns and the names from `names` in
    /// its text. A scene with nothing to say for them is finished at once.
    pub fn new(scene: &Scene, lead: LeadProfile, names: Names, present: &Present) -> Self {
        let hooks = Hooks { lead, names };
        Self(shuyi_ui::dialogue::DialoguePlayer::new(
            scene, hooks, present,
        ))
    }

    /// The lead whose name and pronouns fill the text.
    pub fn lead(&self) -> &LeadProfile {
        &self.0.hooks().lead
    }

    /// The id of the scene being played.
    pub fn scene_id(&self) -> &str {
        self.0.scene_id()
    }

    /// The scene being played: the one it was started with, as it is for
    /// those present.
    pub fn scene(&self) -> &Scene {
        self.0.scene()
    }

    /// What to show now.
    pub fn current(&self) -> View<'_> {
        let view = self.0.current();
        View {
            left: view.left,
            right: view.right,
            speaker: view.speaker,
            text: view.text,
            caption: view.caption,
            place: view.stage.place.as_ref(),
            opening: view.stage.opening.as_deref(),
            narration: view.narration,
            choices: view.choices,
        }
    }

    /// The id of the line in the text box (while a choice is open, of the
    /// line being answered); `None` once the scene is finished.
    pub fn line(&self) -> Option<&LineId> {
        self.0.line()
    }

    /// The id of reply `option` of the open choice, if a choice is open and
    /// has it.
    pub fn reply_line(&self, option: usize) -> Option<&LineId> {
        self.0.reply_line(option)
    }

    /// The music the scene has asked for and [`take_music`](Self::take_music)
    /// hasn't taken yet, without taking it.
    pub fn pending_music(&self) -> Option<&MusicLine> {
        self.0.pending_music()
    }

    /// Moves to the next text box. Advancing a finished scene, or one
    /// waiting at a choice, does nothing.
    pub fn advance(&mut self) {
        self.0.advance();
    }

    /// Picks reply `option` of the open choice: plays its reaction, then
    /// the scene after the choice. Does nothing if no choice is open or
    /// there is no such option.
    pub fn choose(&mut self, option: usize) {
        self.0.choose(option);
    }

    /// Advances until a choice is open or the scene is over (skipping a
    /// scene stops at each choice).
    pub fn skip_to_choice(&mut self) {
        self.0.skip_to_choice();
    }

    /// The music the scene has asked for since the last call: the last
    /// `@music` line reached, whether the lines before it were read or
    /// skipped.
    pub fn take_music(&mut self) -> Option<MusicLine> {
        self.0.take_music()
    }

    /// The opening the scene has asked for since the last call, to play
    /// before the text box now up: the last `@opening` line reached,
    /// whether the lines before it were read or skipped. `None` if no
    /// `@opening` line was reached since.
    pub fn take_opening(&mut self) -> Option<String> {
        self.0.stage_mut().opening_due.take()
    }

    /// Whether a reply choice is waiting for [`choose`](Self::choose).
    pub fn is_choosing(&self) -> bool {
        self.0.is_choosing()
    }

    /// Whether every text box has been shown and advanced past.
    pub fn is_finished(&self) -> bool {
        self.0.is_finished()
    }
}

#[cfg(test)]
mod tests;
