//! The glyph look of the Import screen: one panel like Load Game's, with
//! the file and where it goes at its top, a row per save (slot, chapter,
//! playtime, and the verdict in green when it imports, red when it
//! doesn't), and "Import these saves?" with its answers at its foot; while
//! waiting, what to do in its middle. Everything about the look lives
//! here; nothing here decides what the screen shows (ADR-0054).

use super::{ImportRowView, ImportView};
use crate::color::UiColor;
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::print_centred;
use crate::widgets::AskView;

/// The panel, in cells: Load Game's.
const PANEL: Rect = Rect::new(2, 1, 96, 26);
/// Column where the lines start.
const LEFT: i32 = PANEL.x + 3;
/// Row of the file's name; the build and where it goes follow.
const FILE_ROW: i32 = PANEL.y + 2;
/// Row of the first save.
const FIRST_ROW: i32 = PANEL.y + 6;
/// Column of the chapters, and of a verdict with no chapter before it.
const CHAPTER_X: i32 = LEFT + 18;
/// Cells a chapter may take.
const CHAPTER_W: usize = 36;
/// Column where the playtimes end (they are right-aligned).
const TIME_END_X: i32 = LEFT + 62;
/// Column of the verdicts after a playtime.
const VERDICT_X: i32 = LEFT + 64;
/// Row of the line about the options.
const OPTIONS_ROW: i32 = PANEL.y + PANEL.h - 4;
/// Row of the question; its answers are on the next.
const QUESTION_ROW: i32 = PANEL.y + PANEL.h - 3;
/// Row of the message under the panel.
const MESSAGE_ROW: i32 = PANEL.y + PANEL.h + 1;
/// Cells between two answers.
const ANSWER_GAP: i32 = 3;

/// Cells `text` takes, as a coordinate.
fn cells(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(0)
}

/// Paints the screen over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &ImportView, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (black, bg) = (c(UiColor::Black), c(UiColor::PanelBg));
    let (text, dim, high) = (
        c(UiColor::Text),
        c(UiColor::TextDim),
        c(UiColor::TextHighlight),
    );
    buf.fill_rect(buf.bounds(), Cell::new(' ', text, black));
    buf.fill_rect(PANEL, Cell::new(' ', text, bg));
    buf.draw_box(PANEL, BoxStyle::Single, c(UiColor::PanelBorder), bg);
    let title = format!(" {} ", view.title);
    buf.print(LEFT - 1, PANEL.y, &title, high, bg);
    if let Some(waiting) = &view.waiting {
        print_centred(buf, PANEL.y + PANEL.h / 2, waiting, high, bg);
    }
    let lines = [
        (&view.file, dim),
        (&view.made_by, high),
        (&view.to, if view.warning { high } else { text }),
    ];
    for (y, (line, color)) in (FILE_ROW..).zip(lines) {
        if let Some(line) = line {
            buf.print(LEFT, y, line, color, bg);
        }
    }
    for (y, row) in (FIRST_ROW..).zip(&view.rows) {
        paint_row(ctx, row, buf, y);
    }
    if let Some(range) = &view.range {
        let range = format!(" {range} ");
        let x = PANEL.x + PANEL.w - 2 - cells(&range);
        buf.print(x, PANEL.y + PANEL.h - 1, &range, dim, bg);
    }
    if let Some(options) = &view.options {
        buf.print(LEFT, OPTIONS_ROW, options, dim, bg);
    }
    if let Some(question) = &view.question {
        paint_question(ctx, question, buf);
    }
    if let Some(message) = &view.message {
        print_centred(buf, MESSAGE_ROW, message, c(UiColor::HpLow), black);
    }
    let bottom = i32::from(buf.height()) - 1;
    print_centred(buf, bottom, &view.help, dim, black);
}

/// The save `row` on console row `y`.
fn paint_row(ctx: &Ctx, row: &ImportRowView, buf: &mut GlyphBuffer, y: i32) {
    let c = |u| ctx.palette.get(u);
    let (bg, text) = (c(UiColor::PanelBg), c(UiColor::Text));
    let verdict = c(if row.ready {
        UiColor::HpHigh
    } else {
        UiColor::HpLow
    });
    buf.print(LEFT, y, &row.label, text, bg);
    let Some(chapter) = &row.chapter else {
        buf.print(CHAPTER_X, y, &row.verdict, verdict, bg);
        return;
    };
    let chapter: String = chapter.chars().take(CHAPTER_W).collect();
    buf.print(CHAPTER_X, y, &chapter, text, bg);
    if let Some(time) = &row.playtime {
        buf.print(TIME_END_X - cells(time), y, time, text, bg);
    }
    buf.print(VERDICT_X, y, &row.verdict, verdict, bg);
}

/// The question centred on its row, its answers on the next, the one in
/// focus as a bar.
fn paint_question(ctx: &Ctx, question: &AskView<bool>, buf: &mut GlyphBuffer) {
    let c = |u| ctx.palette.get(u);
    let (bg, text) = (c(UiColor::PanelBg), c(UiColor::Text));
    print_centred(
        buf,
        QUESTION_ROW,
        &question.text,
        c(UiColor::TextHighlight),
        bg,
    );
    let labels: Vec<String> = question
        .answers
        .iter()
        .map(|a| format!(" {} ", a.label))
        .collect();
    let gaps = ANSWER_GAP * i32::try_from(labels.len().saturating_sub(1)).unwrap_or(0);
    let width: i32 = labels.iter().map(|l| cells(l)).sum::<i32>() + gaps;
    let mut x = (i32::from(buf.width()) - width) / 2;
    for (answer, label) in question.answers.iter().zip(&labels) {
        let (fg, back) = if answer.focused {
            (bg, c(UiColor::PanelBorderFocus))
        } else {
            (text, bg)
        };
        buf.print(x, QUESTION_ROW + 1, label, fg, back);
        x += cells(label) + ANSWER_GAP;
    }
}
