//! What a tip box shows, as plain data (ADR-0054): the guide's name, their
//! face, the tip's heading and lines, and the close hint. A skin paints it
//! ([`super::glyph::paint`]). A lesson's box (0456) is the same box with a
//! checklist instead of the close hint ([`LessonView`],
//! [`super::glyph::paint_lesson`]).

use crate::prelude::*;
use trpg_content::tip::{self, GUIDE_PORTRAIT};
use trpg_content::{Lesson, Tip};
use trpg_core::Pos;

use super::fill_placeholders;
use crate::screen::Ctx;

/// Names-table id of the guide's name, the box's title (0731).
pub const GUIDE_NAME: &str = "guide";
/// Text key of the close hint under the tip.
pub const CLOSE_HINT: &str = "battle.help.close";
/// Text key of the count under a lesson's checklist: `{n} of {m} done`.
pub const LESSON_PROGRESS: &str = "battle.lesson.progress";

/// One tip box.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TipView {
    /// The box's title: the guide's name.
    pub title: String,
    /// The guide's face, if the game has the guide's portrait.
    pub face: Option<FaceView>,
    /// The tip's own title, the first line.
    pub heading: String,
    /// The tip's text, its keys named, one entry per line.
    pub lines: Vec<String>,
    /// The close hint, e.g. `f close`.
    pub close: String,
}

/// A face beside the text: an expression of a portrait.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FaceView {
    /// The portrait's id (`trpg_content::Content::portraits`).
    pub portrait: String,
    /// The expression shown.
    pub expression: String,
}

impl TipView {
    /// The box that shows `tip`, spoken by the guide: with their face when
    /// the guide's portrait has the tip's expression (only the bought art
    /// has the portrait).
    pub fn new(ctx: &Ctx, tip: &Tip) -> Self {
        let words = ctx.words();
        let guide = ctx.content.portraits.get(GUIDE_PORTRAIT);
        let face = guide
            .is_some_and(|p| p.expression(tip::face(tip)).is_some())
            .then(|| FaceView {
                portrait: GUIDE_PORTRAIT.to_owned(),
                expression: tip::face(tip).to_owned(),
            });
        let text = fill_placeholders(words.tip_text(tip), ctx.help_keys());
        Self {
            title: words.name(&ctx.content.names, GUIDE_NAME).to_owned(),
            face,
            heading: words.tip_title(tip).to_owned(),
            lines: text.lines().map(str::to_owned).collect(),
            close: ctx.text_with(CLOSE_HINT, &[]),
        }
    }
}

/// One lesson's box (0456): the tip box, with the lesson's checklist and
/// how much of it is done in place of the close hint (a lesson takes no
/// keys).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonView {
    /// The box's title: the guide's name.
    pub title: String,
    /// The guide's face, if the game has the guide's portrait.
    pub face: Option<FaceView>,
    /// The lesson's own title, the first line.
    pub heading: String,
    /// The lesson's text, its keys named, one entry per line.
    pub lines: Vec<String>,
    /// The checklist, in order.
    pub items: Vec<LessonItemView>,
    /// How much is done, e.g. `2 of 4 done`.
    pub progress: String,
    /// The map tile the box must not hide: the cursor's. The skin puts the
    /// box where it doesn't.
    pub clear_of: Pos,
}

/// One item of a lesson's checklist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LessonItemView {
    /// What to do, its keys named.
    pub label: String,
    /// Whether it is done (ticked).
    pub done: bool,
}

impl LessonView {
    /// The box of `lesson`, spoken by the guide, its items ticked as
    /// `ticked` says (in order; an item past its end isn't), at the bottom
    /// clear of map tile `clear_of`.
    pub fn new(ctx: &Ctx, lesson: &Lesson, ticked: &[bool], clear_of: Pos) -> Self {
        let words = ctx.words();
        let keys = ctx.help_keys();
        let guide = ctx.content.portraits.get(GUIDE_PORTRAIT);
        let face = guide
            .is_some_and(|p| p.expression(lesson.face()).is_some())
            .then(|| FaceView {
                portrait: GUIDE_PORTRAIT.to_owned(),
                expression: lesson.face().to_owned(),
            });
        let text = fill_placeholders(words.lesson_text(lesson), keys);
        let items: Vec<LessonItemView> = (0..lesson.items.len())
            .map(|i| LessonItemView {
                label: fill_placeholders(words.lesson_item(lesson, i), keys),
                done: ticked.get(i).copied().unwrap_or(false),
            })
            .collect();
        let done = items.iter().filter(|i| i.done).count();
        let progress = ctx.text_with(LESSON_PROGRESS, &[("n", &done), ("m", &items.len())]);
        Self {
            title: words.name(&ctx.content.names, GUIDE_NAME).to_owned(),
            face,
            heading: words.lesson_title(lesson).to_owned(),
            lines: text.lines().map(str::to_owned).collect(),
            items,
            progress,
            clear_of,
        }
    }
}
