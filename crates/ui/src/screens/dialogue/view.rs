//! What the dialogue screen shows, as plain data: no cells, glyphs, colours
//! or positions (ADR-0054). The screen builds a [`DialogueView`] each frame
//! ([`DialogueScreen::view`](super::DialogueScreen::view)), a cinematic's
//! talk shot builds one for each of its text boxes ([`told`]), and a skin
//! paints it ([`super::glyph::paint`] today). Swapping the look (a bought
//! UI pack) means another painter of the same view.

use crate::prelude::*;
use trpg_content::Side;
use trpg_core::LeadProfile;

use super::log::LogView;
use super::{TEXT_LINES, display_name};
use crate::dialogue::{Portrait, View};
use crate::screen::Ctx;
use crate::widgets::menu::MenuView;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DialogueView {
    /// Whether the scene plays over the screen below (the battle map),
    /// which shows around it; otherwise it fills the screen.
    pub overlay: bool,
    /// The place and time, if the scene has said them.
    pub caption: Option<String>,
    /// Who is on the left, if anyone.
    pub left: Option<FaceView>,
    /// Who is on the right, if anyone.
    pub right: Option<FaceView>,
    /// Which side speaks (`None` for narration).
    pub speaker: Option<Side>,
    /// The text box.
    pub text: TextView,
    /// The dialogue log, while it is open: shown instead of the scene.
    pub log: Option<LogView>,
}

impl DialogueView {
    /// Who is on `side`, if anyone.
    pub fn face(&self, side: Side) -> Option<&FaceView> {
        match side {
            Side::Left => self.left.as_ref(),
            Side::Right => self.right.as_ref(),
        }
    }

    /// Whoever speaks, if they are on screen.
    pub fn speaking(&self) -> Option<&FaceView> {
        self.face(self.speaker?)
    }
}

/// A character on one side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceView {
    /// Their portrait's id (the lead's by their look); a skin shows an
    /// empty frame for one the content hasn't got.
    pub portrait: String,
    /// The portrait's expression.
    pub expression: String,
    /// Their name: the one the player gave the lead, else the story's.
    pub name: String,
}

/// The text box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TextView {
    /// Whether the text is narration (nobody says it).
    pub narration: bool,
    /// The page on screen, word-wrapped, in full: at most [`TEXT_LINES`]
    /// lines. At a choice, the last page of the line being answered.
    pub lines: Vec<String>,
    /// How many of the page's characters are revealed, from its start.
    pub shown: usize,
    /// The lead's replies, while a choice is open.
    pub choices: Option<MenuView>,
    /// The "Skip scene?" question, while it is open: shown instead of the
    /// text.
    pub skip: Option<SkipView>,
    /// The page, all shown, waiting for the key that goes on.
    pub waiting: Option<WaitView>,
}

impl TextView {
    /// The revealed part of each line of the page.
    pub fn revealed(&self) -> Vec<String> {
        let mut budget = self.shown;
        let mut revealed = Vec::new();
        for line in &self.lines {
            revealed.push(line.chars().take(budget).collect());
            budget = budget.saturating_sub(line.chars().count());
        }
        revealed
    }

    /// Whether the whole page is revealed.
    pub fn is_revealed(&self) -> bool {
        let len: usize = self.lines.iter().map(|l| l.chars().count()).sum();
        self.shown >= len
    }
}

/// The question asked before skipping the scene.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkipView {
    /// The question.
    pub question: String,
    /// How to answer, with the keys.
    pub answers: String,
}

/// What shows while a page waits for the key that goes on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WaitView {
    /// The name of the key that goes on.
    pub key: String,
    /// How to open the log, if its action has a key.
    pub log_hint: Option<String>,
    /// Whether the marker that says "press to go on" is lit: it blinks.
    pub marker: bool,
}

/// A character on screen as `lead` and the player's language name them.
fn face(ctx: &Ctx, lead: &LeadProfile, portrait: Portrait) -> FaceView {
    FaceView {
        portrait: lead.portrait_for(&portrait.character.0).to_owned(),
        expression: portrait.expression.to_owned(),
        name: display_name(ctx, lead, portrait).to_owned(),
    }
}

/// The screen showing `view` (who is where, who speaks, the caption) with
/// `text` in its text box, `lead` being the lead. No log.
pub(super) fn scene(
    ctx: &Ctx,
    lead: &LeadProfile,
    view: &View,
    text: TextView,
    overlay: bool,
) -> DialogueView {
    DialogueView {
        overlay,
        caption: view.caption.as_deref().map(str::to_owned),
        left: view.left.map(|p| face(ctx, lead, p)),
        right: view.right.map(|p| face(ctx, lead, p)),
        speaker: view.speaker,
        text,
        log: None,
    }
}

/// A text box of a scene playing by itself (a cinematic's talk shot,
/// ticket 0818): what the full-screen dialogue screen shows of `view`,
/// with the first `shown` characters of the first page of `lines`, and
/// none of the things that wait for a key.
pub fn told(
    ctx: &Ctx,
    lead: &LeadProfile,
    view: &View,
    lines: &[String],
    shown: usize,
) -> DialogueView {
    let text = TextView {
        narration: view.narration,
        lines: lines[..lines.len().min(TEXT_LINES)].to_vec(),
        shown,
        choices: None,
        skip: None,
        waiting: None,
    };
    scene(ctx, lead, view, text, false)
}
