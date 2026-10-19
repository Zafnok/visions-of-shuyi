//! The Playdate's look of the name boxes (ticket 1045): the letter grid
//! and the typing box, each a window of the one-bit look
//! ([`crate::screens::playdate`]) with its title in the top of its frame
//! and the name so far with a cursor. The grid has a row of cells per row
//! of letters (the PC's has a blank row between them), so a grid with its
//! row of digits fits the 15-row screen; the focused cell is the bar.

use super::{GridView, TypingView};
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{bar, cells, inks, to_i32, window};

/// The x that centres `text` in `rect`.
fn centred_in(rect: Rect, text: &str) -> i32 {
    rect.x + (rect.w - cells(text)) / 2
}

/// The height of the grid's window for `grid`: the frame, the name, a
/// blank row and a row per row of cells.
pub fn grid_height(grid: &GridView) -> i32 {
    4 + to_i32(grid.rows.len())
}

/// The letter grid's window from `rect`'s top left, `rect.w` wide and as
/// tall as [`grid_height`]: the name so far with a cursor, then the cells,
/// each its label with a cell either side, the focused one the bar.
pub fn paint_grid(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, grid: &GridView) {
    let ink = inks(ctx);
    let rect = Rect::new(rect.x, rect.y, rect.w, grid_height(grid));
    window(ctx, buf, rect, &grid.title);
    let shown = format!("{}_", grid.spelled);
    buf.print(
        centred_in(rect, &shown),
        rect.y + 1,
        &shown,
        ink.text,
        ink.inside,
    );
    for (r, labels) in grid.rows.iter().enumerate() {
        let y = rect.y + 3 + to_i32(r);
        let widths: Vec<i32> = labels.iter().map(|l| cells(l) + 2).collect();
        let mut x = rect.x + (rect.w - widths.iter().sum::<i32>()) / 2;
        for (col, (label, &w)) in labels.iter().zip(&widths).enumerate() {
            if grid.focus == (r, col) {
                bar(ctx, buf, x, y, w, label);
            } else {
                buf.print(x + 1, y, label, ink.text, ink.inside);
            }
            x += w;
        }
    }
}

/// The typing box at `rect` (seven rows tall): the name so far with a
/// cursor, and the hint to type, under it.
pub fn paint_typing(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, typing: &TypingView) {
    let ink = inks(ctx);
    window(ctx, buf, rect, &typing.title);
    let shown = format!("{}_", typing.typed);
    buf.print(
        centred_in(rect, &shown),
        rect.y + 2,
        &shown,
        ink.text,
        ink.inside,
    );
    let hint = &typing.hint;
    buf.print(
        centred_in(rect, hint),
        rect.y + 4,
        hint,
        ink.text,
        ink.inside,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::playdate::tests::{blank, is_bar, pd_ctx, rows};

    fn grid(digits: bool) -> GridView {
        let row = |s: &str| s.chars().map(String::from).collect::<Vec<_>>();
        let mut rows = vec![
            row("ABCDEFGHIJKLM"),
            row("NOPQRSTUVWXYZ"),
            row("abcdefghijklm"),
            row("nopqrstuvwxyz"),
        ];
        if digits {
            rows.push(row("0123456789"));
        }
        rows.push(
            ["-", "'", "Space", "Delete", "Done"]
                .map(String::from)
                .to_vec(),
        );
        GridView {
            title: "First name".to_owned(),
            spelled: "Elle".to_owned(),
            rows,
            focus: (1, 2),
        }
    }

    #[test]
    fn the_grid_is_a_row_per_row_of_cells_its_focus_a_bar() {
        let ctx = pd_ctx();
        let mut buf = blank();
        let g = grid(false);
        assert_eq!(grid_height(&g), 9);
        paint_grid(&ctx, &mut buf, Rect::new(2, 3, 46, 0), &g);
        let shown = rows(&buf);
        assert!(shown[3].starts_with("╔═ First name ═"), "{}", shown[3]);
        assert_eq!(shown[4], format!("║{:^44}║", "Elle_"));
        assert_eq!(shown[5], format!("║{:44}║", ""));
        assert_eq!(shown[6], "║   A  B  C  D  E  F  G  H  I  J  K  L  M    ║");
        assert_eq!(shown[10], "║         -  '  Space  Delete  Done          ║");
        assert!(shown[11].starts_with('╚'));
        // The focus: P, the third cell of the second row.
        assert!(is_bar(&buf, 7, " P "));
        assert!(!is_bar(&buf, 7, " O "));
        // With its row of digits the grid still fits the screen.
        let g = grid(true);
        assert_eq!(grid_height(&g), 10);
        let mut buf = blank();
        paint_grid(&ctx, &mut buf, Rect::new(2, 2, 46, 0), &g);
        assert!(rows(&buf)[11].starts_with('╚'));
    }

    #[test]
    fn the_typing_box_shows_the_name_and_the_hint() {
        let ctx = pd_ctx();
        let mut buf = blank();
        let typing = TypingView {
            title: "First name".to_owned(),
            typed: "Ell".to_owned(),
            hint: "Type a name".to_owned(),
        };
        paint_typing(&ctx, &mut buf, Rect::new(2, 4, 46, 7), &typing);
        let shown = rows(&buf);
        assert!(shown[4].starts_with("╔═ First name ═"));
        assert!(shown[6].contains("Ell_"));
        assert!(shown[8].contains("Type a name"));
        assert!(shown[10].starts_with('╚'));
    }
}
