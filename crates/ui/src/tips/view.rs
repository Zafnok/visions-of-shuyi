//! What a tip box shows, as plain data (ADR-0054): the guide's name, their
//! face, the tip's heading and lines, and the close hint. A skin paints it
//! ([`super::glyph::paint`]).

use trpg_content::Tip;
use trpg_content::tip::GUIDE_PORTRAIT;

use super::fill_placeholders;
use crate::screen::Ctx;

/// Text key ([`Ctx::text`]) of the guide's name, the box's title.
pub const GUIDE_NAME: &str = "guide.name";
/// Text key of the close hint under the tip.
pub const CLOSE_HINT: &str = "battle.help.close";

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
            .is_some_and(|p| p.expression(tip.face()).is_some())
            .then(|| FaceView {
                portrait: GUIDE_PORTRAIT.to_owned(),
                expression: tip.face().to_owned(),
            });
        let text = fill_placeholders(words.tip_text(tip), ctx.help_keys());
        Self {
            title: ctx.text(GUIDE_NAME).to_owned(),
            face,
            heading: words.tip_title(tip).to_owned(),
            lines: text.lines().map(str::to_owned).collect(),
            close: ctx.text_with(CLOSE_HINT, &[]),
        }
    }
}
