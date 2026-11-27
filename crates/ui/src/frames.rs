//! The picture look's window frames (ticket 1380, `docs/design/look-and-feel.md`,
//! *Windows (the picture look)*): a window's skin builds its
//! [`PanelLook`] as before and passes it through [`framed`] with the
//! frame's name. With the *Window look* option on pictures and the
//! frame's picture in this build, the look gets the picture frame, the
//! frame's fill as its background and the title plate; otherwise it comes
//! back as it went in, so the glyph look draws exactly as before.
//!
//! The frames themselves are data (`assets/ui/frames.ron`,
//! [`trpg_content::frames`]); the engine draws them
//! (`shuyi_ui::frame::draw::PanelFrame`, engine#3249).

use crate::prelude::*;

use shuyi_ui::frame::draw::{PanelFrame, TitlePlate};
use trpg_content::frames::{Frame, Plate};

use crate::color::{Rgb, UiColor};
use crate::glyph_buffer::{GlyphBuffer, PanelLook, Rect};
use crate::screen::{Ctx, SkinSet};
use crate::settings::WindowLook;

/// The side panel's and the battle's boxes' frame.
pub const PANEL: &str = "panel";
/// The unit menu's and its sub-menus' frame.
pub const MENU: &str = "menu";
/// The map menu's frame.
pub const MAP_MENU: &str = "map_menu";
/// The plate a window's title sits on.
pub const TITLE: &str = "title";

/// Whether the windows wear pictures: the PC's skins, with *Window look*
/// on pictures. A build without the pictures still draws lines
/// ([`frame`] finds none).
pub fn pictures(ctx: &Ctx) -> bool {
    ctx.game.skin_set == SkinSet::Glyph && ctx.settings().game.window_look == WindowLook::Pictures
}

/// The frame called `name`, when the windows wear pictures and this
/// build has its picture.
pub fn frame(ctx: &Ctx, name: &str) -> Option<Frame> {
    pictures(ctx)
        .then(|| ctx.game.content.frames.frame(name))
        .flatten()
}

/// The title plate, when the windows wear pictures and this build has its
/// picture.
pub fn plate(ctx: &Ctx) -> Option<Plate> {
    pictures(ctx)
        .then(|| ctx.game.content.frames.plate(TITLE))
        .flatten()
}

/// `look` in the frame called `name` ([`framed_with`]).
pub fn framed(ctx: &Ctx, name: &str, look: PanelLook) -> PanelLook {
    framed_with(look, frame(ctx, name), plate(ctx))
}

/// `look` with `frame`'s picture in place of its lines, `frame`'s fill as
/// its background and `plate` behind its title; `look` itself without a
/// frame.
pub fn framed_with(look: PanelLook, frame: Option<Frame>, plate: Option<Plate>) -> PanelLook {
    let Some(f) = frame else {
        return look;
    };
    let panel = PanelFrame::new(f.slice).with_outset(f.outset);
    let plate = plate.map(|p| {
        TitlePlate::new(p.slice)
            .with_pad(p.pad)
            .with_text_top(p.text_top)
    });
    let mut look = look.with_frame(Some(panel)).with_plate(plate);
    let [r, g, b] = f.fill;
    look.bg = Rgb::new(r, g, b);
    look
}

/// The background of a window framed as `name`: its frame's fill, or the
/// panels' colour in the glyph look. For a skin's prints inside the
/// window.
pub fn bg(ctx: &Ctx, name: &str) -> Rgb {
    frame(ctx, name).map_or_else(
        || ctx.palette.get(UiColor::PanelBg),
        |f| {
            let [r, g, b] = f.fill;
            Rgb::new(r, g, b)
        },
    )
}

/// Puts the title plate behind the text on the cells `text` when the
/// windows wear pictures, for a title a skin prints itself (centred, say)
/// rather than through the panel.
pub fn plate_behind(ctx: &Ctx, buf: &mut GlyphBuffer, text: Rect) {
    if let Some(p) = plate(ctx) {
        let plate = TitlePlate::new(p.slice)
            .with_pad(p.pad)
            .with_text_top(p.text_top);
        buf.title_plate(&plate, text);
    }
}

/// Prints `title` with a space each side, centred on the row `row`, in
/// `fg` (the cells keep their background), with the title plate behind
/// it in the picture look: a title a skin sets into a window's top
/// border itself.
pub fn centred_title(ctx: &Ctx, buf: &mut GlyphBuffer, row: Rect, title: &str, fg: Rgb) {
    let shown = format!(" {title} ");
    let cells = buf.text_cells(&shown).min(row.w);
    buf.print_centred(row, &shown, fg);
    let x = row.x + (row.w - cells) / 2;
    plate_behind(ctx, buf, Rect::new(x + 1, row.y, cells - 2, 1));
}

/// Prints `title` with a space each side from `at` in `colors` (its
/// text and background), with the title plate behind it in the picture
/// look. Returns how many cells it took.
pub fn title_at(
    ctx: &Ctx,
    buf: &mut GlyphBuffer,
    (x, y): (i32, i32),
    title: &str,
    (fg, bg): (Rgb, Rgb),
) -> i32 {
    let n = i32::from(buf.print(x, y, &format!(" {title} "), fg, bg));
    plate_behind(ctx, buf, Rect::new(x + 1, y, n - 2, 1));
    n
}

#[cfg(test)]
pub(crate) mod tests;
