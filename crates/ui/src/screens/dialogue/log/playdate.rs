//! The Playdate's look of the dialogue log (ticket 1043): paints a
//! [`LogView`] as one window over the whole 50 × 15 screen, its title in
//! the top of the frame and its help in the bottom. A row per line seen:
//! the speaker's name, then the text under it a little in; narration has
//! no name. The cursor's row is the bar of the one-bit look, a line seen
//! past the one on screen is in grey dots, and the list scrolls with the
//! cursor as the PC's does, a `▲` or `▼` in the frame saying there is
//! more. No faces: a face at the Playdate's 2× would take more than half
//! the screen's rows a line.
//!
//! Nothing here decides what the log shows or does (ADR-0054).

use super::glyph::{Laid, window as shown_rows};
use super::{LogView, RowKind};
use crate::glyph_buffer::GlyphBuffer;
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{bar, footer, inks, more_marks, to_i32, to_usize, window, wrap};

/// Column of the names (and of narration).
const TEXT_X: i32 = 2;
/// How far a line's text sits in from its speaker's name.
const INDENT: &str = "  ";

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &LogView, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    let frame = buf.bounds();
    window(ctx, buf, frame, &view.title);
    footer(ctx, buf, frame, &view.help);

    let text_w = to_usize(frame.w - 2 * TEXT_X) - INDENT.len();
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
            let indent = if lines.is_empty() { "" } else { INDENT };
            let wrapped = wrap(&text, text_w).into_iter();
            lines.extend(wrapped.map(|line| format!("{indent}{line}")));
            let height = to_i32(lines.len());
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
    let (first, last) = shown_rows(&laid, focus, frame.h - 2);

    let mut y = frame.y + 1;
    for (i, row_laid) in laid.iter().enumerate().take(last).skip(first) {
        let row = view.rows.get(i);
        let focused = row.map_or_else(
            || view.back_to_latest.as_ref().is_some_and(|b| b.focused),
            |r| r.focused,
        );
        let fg = if row.is_some_and(|r| r.ahead) {
            ink.off
        } else {
            ink.text
        };
        for (dy, line) in (0..).zip(&row_laid.lines) {
            if focused {
                bar(ctx, buf, TEXT_X - 1, y + dy, frame.w - TEXT_X, line);
            } else {
                buf.print(TEXT_X, y + dy, line, fg, ink.inside);
            }
        }
        y += row_laid.height + 1;
    }
    more_marks(ctx, buf, frame, first > 0, last < laid.len());
}

#[cfg(test)]
mod tests;
