//! The talk shot (ticket 0818): text boxes of a dialogue scene playing by
//! themselves in the dialogue screen's look. Which box is up and how much
//! of its text is typed out come from the time alone ([`talk_at`]).

use crate::prelude::*;
use trpg_content::{Present, Scene, Side};
use trpg_core::{CharacterId, LeadProfile};

use crate::dialogue::{DialoguePlayer, Portrait, View};
use crate::glyph_buffer::GlyphBuffer;
use crate::screen::Ctx;
use crate::screens::dialogue::view::told;
use crate::screens::dialogue::{TEXT_LINES, TEXT_W, glyph};
use crate::widgets::word_wrap;

/// The part of a box's time, at its end, for which all of its text is
/// shown: a box that would still be typing by then types faster
/// (*tunable*).
pub const HOLD_SHARE: f32 = 1.0 / 3.0;

/// A character on one side of a text box: who, with which expression.
type Placed = (CharacterId, String);

/// One text box of a scene as the dialogue screen shows it, tokens filled
/// in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TalkBox {
    left: Option<Placed>,
    right: Option<Placed>,
    speaker: Option<Side>,
    caption: Option<String>,
    narration: bool,
    /// The text, wrapped: its first page (a cinematic turns no pages; the
    /// validator refuses a box that could need a second).
    lines: Vec<String>,
}

impl TalkBox {
    /// The box `view` shows.
    fn of(view: &View) -> Self {
        let placed =
            |p: Option<Portrait>| p.map(|p| (p.character.clone(), p.expression.to_owned()));
        let mut lines = word_wrap(view.text.as_deref().unwrap_or(""), TEXT_W);
        lines.truncate(TEXT_LINES);
        Self {
            left: placed(view.left),
            right: placed(view.right),
            speaker: view.speaker,
            caption: view.caption.as_deref().map(str::to_owned),
            narration: view.narration,
            lines,
        }
    }

    /// The text's lines.
    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    /// How many characters the text has to type out.
    pub fn chars(&self) -> usize {
        self.lines.iter().map(|l| l.chars().count()).sum()
    }

    /// Draws the box over the whole of `buf` with the first `shown`
    /// characters of its text, `lead` being the lead.
    pub fn draw(&self, ctx: &Ctx, lead: &LeadProfile, shown: usize, buf: &mut GlyphBuffer) {
        fn portrait(placed: Option<&Placed>) -> Option<Portrait<'_>> {
            let (character, expression) = placed?;
            Some(Portrait {
                character,
                expression,
            })
        }
        let view = View {
            left: portrait(self.left.as_ref()),
            right: portrait(self.right.as_ref()),
            speaker: self.speaker,
            caption: self.caption.as_deref().map(Into::into),
            narration: self.narration,
            ..View::default()
        };
        glyph::paint(ctx, &told(ctx, lead, &view, &self.lines, shown), buf);
    }
}

/// Text boxes `first .. first + count` of `scene` (counted from 0, as
/// `trpg_content::cinematic::opening_boxes` counts them: everyone is
/// there), in the player's language, with `lead`'s name and pronouns.
/// Stops at a reply choice or the scene's end, so it can give fewer.
pub fn boxes(
    ctx: &Ctx,
    scene: &Scene,
    lead: &LeadProfile,
    first: usize,
    count: usize,
) -> Vec<TalkBox> {
    let words = ctx.words();
    let names = words.names(&ctx.content.names);
    let scene = words.scene(scene, lead.gender);
    let mut player = DialoguePlayer::new(&scene, lead.clone(), names, &Present::Everyone);
    let mut boxes = Vec::new();
    for i in 0..first.saturating_add(count) {
        if player.is_finished() || player.is_choosing() {
            break;
        }
        if i >= first {
            boxes.push(TalkBox::of(&player.current()));
        }
        player.advance();
    }
    boxes
}

/// Which text box is up `progress` (0 to 1) through a talk shot `shot_s`
/// seconds long, and how many of its characters are typed out. `lens` is
/// each box's length in characters; `None` without boxes.
///
/// Each box has an equal share of the shot. Its text types out at
/// `chars_per_s` (`None`: all at once), or faster if that would leave it
/// whole for less than the last [`HOLD_SHARE`] of its share; then it
/// holds.
pub fn talk_at(
    progress: f32,
    shot_s: f32,
    lens: &[usize],
    chars_per_s: Option<f32>,
) -> Option<(usize, usize)> {
    let last = lens.len().checked_sub(1)?;
    let progress = if progress.is_finite() {
        progress.clamp(0.0, 1.0)
    } else {
        0.0
    };
    // A shot has a handful of boxes of a few hundred characters at most.
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    {
        let boxes = lens.len() as f32;
        let along = progress * boxes;
        let index = (along as usize).min(last);
        let len = lens[index];
        let Some(speed) = chars_per_s else {
            return Some((index, len));
        };
        let share_s = shot_s.max(0.0) / boxes;
        let since_s = (along - index as f32) * share_s;
        let typing_s = share_s * (1.0 - HOLD_SHARE);
        if since_s >= typing_s {
            return Some((index, len));
        }
        let speed = speed.max(len as f32 / typing_s);
        Some((index, ((since_s * speed) as usize).min(len)))
    }
}
