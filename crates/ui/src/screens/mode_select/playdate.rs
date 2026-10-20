//! The Playdate's look of the game mode screen (ticket 1045): the heading
//! at the top, the modes in a window under it, the focused mode's line
//! under that, wrapped to the screen, and the help on the bottom row, in
//! the one-bit look of [`crate::screens::playdate`]. One bit has no dimmer
//! letters, so only the focused mode's line shows (the PC shows them all,
//! the others dimmed). The familiarity screen (0835) has the same shape
//! and is painted by it too.

use super::view::ModeSelectView;
use crate::glyph_buffer::GlyphBuffer;
use crate::screen::Ctx;
use crate::screens::playdate::{centred, clear, menu, menu_size, wrap};

/// Row of the heading.
const HEADING_ROW: i32 = 1;
/// Top row of the menu's window.
const MENU_ROW: i32 = 3;
/// Cells a line of the explanation takes at most: the screen less two
/// cells either side.
const TEXT_W: usize = 46;

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ModeSelectView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    centred(ctx, buf, HEADING_ROW, &view.heading);
    let (w, h) = menu_size(ctx, &view.menu);
    let x = (i32::from(buf.width()) - w) / 2;
    menu(ctx, buf, (x, MENU_ROW), &view.menu);
    let bottom = i32::from(buf.height()) - 1;
    if let Some(line) = view.lines.iter().find(|l| l.focused) {
        let rows = MENU_ROW + h + 1..bottom - 1;
        for (y, text) in rows.zip(wrap(&line.text, TEXT_W)) {
            centred(ctx, buf, y, &text);
        }
    }
    centred(ctx, buf, bottom, &view.help);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::screens::mode_select::view::ModeLineView;
    use crate::screens::playdate::tests::{blank, is_bar, pd_ctx, rows};
    use crate::widgets::{MenuItemView, MenuView};

    fn view(focus: usize) -> ModeSelectView {
        let item = |label: &str, i| MenuItemView {
            label: label.into(),
            suffix: None,
            note: None,
            enabled: true,
            focused: i == focus,
        };
        let line = |text: &str, i| ModeLineView {
            text: text.to_owned(),
            focused: i == focus,
        };
        ModeSelectView {
            heading: "Choose a game mode".to_owned(),
            menu: MenuView {
                items: vec![item("Classic", 0), item("Casual", 1)],
                first_shown: 0,
                shown: 2,
            },
            lines: vec![
                line(
                    "Classic: a unit that falls in battle dies and is gone for good.",
                    0,
                ),
                line("Casual: a unit that falls retreats.", 1),
            ],
            help: "↑↓ choose · A select · B back".to_owned(),
        }
    }

    #[test]
    fn the_heading_the_modes_the_focused_ones_line_wrapped_and_the_help() {
        let ctx = pd_ctx();
        let mut buf = blank();
        paint(&ctx, &view(0), &mut buf);
        let shown = rows(&buf);
        assert_eq!(shown[1], "Choose a game mode");
        assert_eq!(
            shown[3..7],
            ["╔═════════╗", "║ Classic ║", "║ Casual  ║", "╚═════════╝"]
        );
        assert!(is_bar(&buf, 4, "Classic") && !is_bar(&buf, 5, "Casual"));
        assert_eq!(
            shown[8..10],
            [
                "Classic: a unit that falls in battle dies and",
                "is gone for good."
            ]
        );
        assert!(shown[10..14].iter().all(String::is_empty), "{shown:#?}");
        assert_eq!(shown[14], "↑↓ choose · A select · B back");
        // On the other mode, its line alone.
        paint(&ctx, &view(1), &mut buf);
        let shown = rows(&buf);
        assert_eq!(shown[8], "Casual: a unit that falls retreats.");
        assert!(shown[9].is_empty());
        assert!(is_bar(&buf, 5, "Casual"));
    }

    #[test]
    fn a_long_explanation_stops_a_row_above_the_help() {
        let ctx = pd_ctx();
        let mut buf = blank();
        let mut v = view(0);
        v.lines[0].text = "word ".repeat(60);
        paint(&ctx, &v, &mut buf);
        let shown = rows(&buf);
        assert!(
            shown[8..13].iter().all(|r| r.starts_with("word")),
            "{shown:#?}"
        );
        assert!(shown[13].is_empty());
        assert_eq!(shown[14], v.help);
    }
}
