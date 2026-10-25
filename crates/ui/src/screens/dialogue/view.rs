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
use crate::map_view::MapScene;
use crate::screen::Ctx;

/// The whole screen, as it is this frame.
#[derive(Debug, Clone, PartialEq)]
pub struct DialogueView {
    /// Whether the scene plays over the screen below (the battle map),
    /// which shows around it, darkened; otherwise it fills the screen.
    pub overlay: bool,
    /// The place behind the talk, if the scene names one: the part of its
    /// map the whole screen shows, for the map skin to paint (darkened)
    /// over the screen below or the black. `None`: black, or the screen
    /// below for an overlay. The characters of an opening played on it
    /// (0734) are its units, where the opening has them.
    pub place: Option<MapScene>,
    /// An opening playing before the text box (0734): the place is shown
    /// without the talk, bright while its characters move, then
    /// darkening.
    pub opening: Option<OpeningView>,
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

/// An opening playing.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OpeningView {
    /// How far the place has darkened: 0 while the characters move, up to
    /// 1 as the talk comes up (as dark as behind the talk).
    pub darkened: f32,
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

/// The text box, the question before skipping and what shows while a page
/// waits: the engine's ([`shuyi_ui::dialogue::text_box`]).
pub use shuyi_ui::dialogue::{SkipView, TextView, WaitView};

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
        place: None,
        opening: None,
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
    let mut text = TextView::new(lines[..lines.len().min(TEXT_LINES)].to_vec(), shown);
    text.narration = view.narration;
    scene(ctx, lead, view, text, false)
}
