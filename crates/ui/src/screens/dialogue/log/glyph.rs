//! The glyph look of the dialogue log: paints a [`LogView`] as one panel
//! over the whole screen, a row per line with the speaker's face at the
//! left, their name, and the text under it, the cursor's row as a dark bar.
//! Everything about the look lives here (where things go, the sizes, the
//! colours, how far the list scrolls); nothing here decides what the log
//! shows or does (ADR-0054).

use super::{LogView, RowKind};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::portrait::draw_portrait_in;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::widgets::word_wrap;

/// Column of a face's left edge.
const FACE_X: i32 = 3;
/// A face's size in cells: 64×64 console pixels, a bought face at 1×.
const FACE: (i32, i32) = (8, 4);
/// Column of the names and the text.
const TEXT_X: i32 = FACE_X + FACE.0 + 1;
/// Column of the cursor's marker.
const MARKER_X: i32 = 1;
/// How much a face seen past the line on screen is dimmed.
const AHEAD_DIM: f32 = 0.6;
/// Rows shown past the cursor's, when there are that many, before earlier
/// rows fill the rest.
const AFTER_FOCUS: usize = 2;

/// One row of the list as placed: its lines and how tall it is.
struct Laid {
    /// The name (if any), then the text's lines.
    lines: Vec<String>,
    /// Console rows it takes, not counting the blank row after it.
    height: i32,
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &LogView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    let panel = buf.bounds();
    buf.fill_rect(panel, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(panel, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    let title = format!(" {} ", view.title);
    buf.print(panel.x + 3, panel.y, &title, c(UiColor::TextHighlight), bg);
    let help = format!(" {} ", view.help);
    let help_w = i32::try_from(help.chars().count()).unwrap_or(0);
    let bottom = panel.y + panel.h - 1;
    buf.print(
        panel.x + panel.w - 3 - help_w,
        bottom,
        &help,
        c(UiColor::TextDim),
        bg,
    );

    let text_w = usize::try_from(panel.w - TEXT_X - 3).unwrap_or(1);
    let mut laid: Vec<Laid> = view
        .rows
        .iter()
        .map(|row| {
            let text = match row.kind {
                RowKind::Reply => format!("> {}", row.text),
                RowKind::Speech | RowKind::Narration => row.text.clone(),
            };
            // check-text: not a data name (the view's own)
            let mut lines: Vec<String> = row.name.iter().cloned().collect();
            lines.extend(word_wrap(&text, text_w));
            let lines_h = i32::try_from(lines.len()).unwrap_or(1);
            let height = if row.face.is_some() {
                lines_h.max(FACE.1)
            } else {
                lines_h
            };
            Laid { lines, height }
        })
        .collect();
    if let Some(back) = &view.back_to_latest {
        laid.push(Laid {
            lines: vec![back.label.clone()],
            height: 1,
        });
    }
    let focus = view
        .rows
        .iter()
        .position(|r| r.focused)
        .or_else(|| view.back_to_latest.as_ref().map(|_| view.rows.len()))
        .unwrap_or(0);
    let inner = panel.h - 2;
    let (first, last) = window(&laid, focus, inner);

    let mut y = panel.y + 1;
    for (i, row_laid) in laid.iter().enumerate().take(last).skip(first) {
        let row = view.rows.get(i);
        let focused = row.map_or_else(
            || view.back_to_latest.as_ref().is_some_and(|b| b.focused),
            |r| r.focused,
        );
        let ahead = row.is_some_and(|r| r.ahead);
        let row_bg = if focused { c(UiColor::Black) } else { bg };
        if focused {
            let bar = Rect::new(panel.x + 1, y, panel.w - 2, row_laid.height);
            buf.fill_rect(bar, Cell::new(' ', c(UiColor::Text), row_bg));
            buf.print(MARKER_X + 1, y, "▶", c(UiColor::TextHighlight), row_bg);
        }
        let text_fg = match (focused, ahead, row.map(|r| r.kind)) {
            (true, ..) => c(UiColor::TextHighlight),
            (false, true, _) => c(UiColor::PanelBorder),
            (false, false, Some(RowKind::Narration)) => c(UiColor::TextDim),
            (false, false, _) => c(UiColor::Text),
        };
        let name_fg = if ahead && !focused {
            c(UiColor::PanelBorder)
        } else {
            c(UiColor::TextHighlight)
        };
        if let Some(face) = row.and_then(|r| r.face.as_ref())
            && let Some(art) = ctx.game.content.portraits.get(&face.portrait)
        {
            let dim = if ahead { AHEAD_DIM } else { 0.0 };
            draw_portrait_in(buf, (FACE_X, y), FACE, art, &face.expression, dim, false);
        }
        // check-text: not a data name (the view's own)
        let has_name = row.is_some_and(|r| r.name.is_some());
        for (dy, line) in (0..).zip(&row_laid.lines) {
            let fg = if dy == 0 && has_name {
                name_fg
            } else {
                text_fg
            };
            buf.print(TEXT_X, y + dy, line, fg, row_bg);
        }
        y += row_laid.height + 1;
    }
}

/// The rows `first..last` of `laid` shown in `inner` console rows: the
/// one at `focus` whole, up to [`AFTER_FOCUS`] after it, then as many
/// before it as fit, then more after if there is still room.
fn window(laid: &[Laid], focus: usize, inner: i32) -> (usize, usize) {
    let focus = focus.min(laid.len().saturating_sub(1));
    let Some(row) = laid.get(focus) else {
        return (0, 0);
    };
    let cost = |l: &Laid| l.height + 1;
    let mut used = cost(row);
    let (mut first, mut last) = (focus, focus + 1);
    while last < laid.len() && last <= focus + AFTER_FOCUS && used + cost(&laid[last]) <= inner + 1
    {
        used += cost(&laid[last]);
        last += 1;
    }
    while first > 0 && used + cost(&laid[first - 1]) <= inner + 1 {
        first -= 1;
        used += cost(&laid[first]);
    }
    while last < laid.len() && used + cost(&laid[last]) <= inner + 1 {
        used += cost(&laid[last]);
        last += 1;
    }
    (first, last)
}

#[cfg(test)]
mod tests;
