//! The glyph look of the rewind screen: a double box over the side panel,
//! the title lit, the charges (in the warning colour when none are left),
//! the entries word-wrapped to the panel (wrapped lines indented), the
//! highlighted one inverted and scrolled into view, and the question at
//! the bottom.

use super::{TEXT_W, TEXT_X};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::battle::layout::SIDE_PANEL;
use crate::screens::battle::view::RewindView;

/// Row of the title.
pub const TITLE_ROW: i32 = SIDE_PANEL.y + 1;

/// Row of the charges line.
pub const CHARGES_ROW: i32 = TITLE_ROW + 1;

/// First row of the list, after a blank row.
pub const LIST_ROW: i32 = TITLE_ROW + 3;

/// Row of the confirm prompt (the list stops above it).
pub const PROMPT_ROW: i32 = SIDE_PANEL.y + SIDE_PANEL.h - 3;

/// Indent of an entry's wrapped lines.
const WRAP_INDENT: &str = "  ";

/// Paints the rewind screen `view` over the side panel: title, charges,
/// the list (scrolled to keep the highlighted entry in view) and the
/// prompt.
pub fn paint(ctx: &Ctx, view: &RewindView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let bg = c(UiColor::PanelBg);
    buf.fill_rect(SIDE_PANEL, Cell::new(' ', c(UiColor::Text), bg));
    buf.draw_box(SIDE_PANEL, BoxStyle::Double, c(UiColor::PanelBorder), bg);
    let print = |buf: &mut GlyphBuffer, y, text: &str, fg, bg| {
        buf.print(TEXT_X, y, text, fg, bg);
    };
    print(buf, TITLE_ROW, &view.title, c(UiColor::TextHighlight), bg);
    let charges = if view.spent {
        c(UiColor::HpLow)
    } else {
        c(UiColor::Text)
    };
    print(buf, CHARGES_ROW, &view.charges, charges, bg);
    if let Some(nothing) = &view.nothing {
        print(buf, LIST_ROW, nothing, c(UiColor::TextDim), bg);
        return;
    }
    let blocks: Vec<Vec<String>> = view.entries.iter().map(|e| wrap(e)).collect();
    let first = first_shown(&blocks, view.focus, PROMPT_ROW - 1 - LIST_ROW);
    let mut y = LIST_ROW;
    for (i, block) in blocks.iter().enumerate().skip(first) {
        let rows = i32::try_from(block.len()).unwrap_or(i32::MAX);
        if y + rows > PROMPT_ROW - 1 {
            break;
        }
        let (fg, row_bg) = if i == view.focus {
            (bg, c(UiColor::PanelBorderFocus))
        } else {
            (c(UiColor::Text), bg)
        };
        for text in block {
            buf.fill_rect(
                Rect::new(TEXT_X - 1, y, SIDE_PANEL.w - 2, 1),
                Cell::new(' ', fg, row_bg),
            );
            print(buf, y, text, fg, row_bg);
            y += 1;
        }
    }
    if let Some(ask) = &view.prompt {
        print(buf, PROMPT_ROW, ask, c(UiColor::TextHighlight), bg);
    }
}

/// `line` word-wrapped to the panel's text width, wrapped lines indented.
fn wrap(line: &str) -> Vec<String> {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in line.split(' ') {
        let len = current.chars().count();
        if len > 0 && len + 1 + word.chars().count() > TEXT_W {
            lines.push(core::mem::replace(
                &mut current,
                format!("{WRAP_INDENT}{word}"),
            ));
        } else {
            if len > 0 {
                current.push(' ');
            }
            current.push_str(word);
        }
    }
    lines.push(current);
    lines
}

/// The first block to show so that blocks `first..=focus` fit in `rows`.
/// (`focus` itself if even it alone doesn't fit).
fn first_shown(blocks: &[Vec<String>], focus: usize, rows: i32) -> usize {
    let height = |b: &Vec<String>| i32::try_from(b.len()).unwrap_or(i32::MAX);
    (0..focus)
        .find(|&first| blocks[first..=focus].iter().map(height).sum::<i32>() <= rows)
        .unwrap_or(focus)
}

#[cfg(test)]
mod tests;
