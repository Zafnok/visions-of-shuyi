//! The Playdate's look of the rewind screen (ticket 1039): the map over the
//! whole screen showing the battle as it was, as on the PC, and the list
//! in a window down its right side where the PC's side panel is: the
//! title in the top of the frame, the charges, then the past actions
//! wrapped to the window (wrapped lines indented), the highlighted one
//! the bar, scrolled into view, a `▲` or `▼` in the frame where there are
//! more. "Use 1 of N charges?" is a question in the middle of the screen
//! with its buttons named inside, as the battle's questions (1038). The
//! lesson box is the PC's until 1058.

use crate::prelude::*;

use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::screen::Ctx;
use crate::screens::battle::glyph::paint_lesson;
use crate::screens::battle::glyph::rewind::{first_shown, wrap_to};
use crate::screens::battle::layout::map_area;
use crate::screens::battle::view::{RewindScreenView, RewindView};
use crate::screens::playdate::{self as pd, Answers, cut, inks, to_i32, to_usize};

/// The list window's width, in cells (as the forecast's).
pub const W: i32 = super::forecast::W;

/// Row of the first entry inside the window, after the charges and a blank
/// row.
const LIST_DY: i32 = 3;

/// Paints the whole rewind screen `view`: the map, the list, the lesson,
/// and the question while it is asked.
pub fn paint(ctx: &Ctx, view: &RewindScreenView, buf: &mut GlyphBuffer) {
    pd::clear(ctx, buf);
    let area = map_area(ctx);
    ctx.game.map_skin.paint(ctx, &view.scene, area, buf);
    paint_list(ctx, &view.list, list_rect(area), buf);
    if let Some(lesson) = &view.lesson {
        paint_lesson(ctx, &view.scene, lesson, buf);
    }
    if let Some(question) = &view.list.prompt {
        pd::ask(ctx, buf, question, Answers::Hint(&view.help));
    }
}

/// The list's window over `area`: down its right side, the full height.
pub fn list_rect(area: Rect) -> Rect {
    let w = W.min(area.w);
    Rect::new(area.x + area.w - w, area.y, w, area.h)
}

/// Paints the list `view` in the window `rect`.
pub fn paint_list(ctx: &Ctx, view: &RewindView, rect: Rect, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    pd::window(ctx, buf, rect, &view.title);
    let x = rect.x + 2;
    let width = to_usize(rect.w - 4);
    buf.print(
        x,
        rect.y + 1,
        &cut(&view.charges, width),
        ink.text,
        ink.inside,
    );
    let top = rect.y + LIST_DY;
    if let Some(nothing) = &view.nothing {
        buf.print(x, top, &cut(nothing, width), ink.text, ink.inside);
        return;
    }
    let bottom = rect.y + rect.h - 1;
    let blocks: Vec<Vec<String>> = view.entries.iter().map(|e| wrap_to(e, width)).collect();
    let first = first_shown(&blocks, view.focus, bottom - top);
    let mut y = top;
    let mut last = first;
    for (i, block) in blocks.iter().enumerate().skip(first) {
        if y + to_i32(block.len()) > bottom {
            break;
        }
        for text in block {
            if i == view.focus {
                pd::bar(ctx, buf, x - 1, y, rect.w - 2, text);
            } else {
                buf.print(x, y, &cut(text, width), ink.text, ink.inside);
            }
            y += 1;
        }
        last = i + 1;
    }
    pd::more_marks(ctx, buf, rect, first > 0, last < blocks.len());
}

#[cfg(test)]
mod tests;
