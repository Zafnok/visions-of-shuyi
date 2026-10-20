//! What the screens' Playdate skins share (ticket 1045, ADR-0083): the
//! one-bit look of a screen, its windows and its lists, at the Playdate's
//! 50 × 15 cells (`docs/design/playdate.md`).
//!
//! - **The ground is white** and the letters black.
//! - **A window** is white inside a double black frame, its title in the
//!   top of the frame and a line of its own (a message, a hint) in the
//!   bottom of it, as the guide's box (0066) and the battle's windows.
//! - **The focus is a bar**: white letters on black. One bit has no
//!   brighter or dimmer letters, so what the PC lights up is a bar here,
//!   and what it dims (an item that can't be chosen) is grey dots.
//! - **A list too long for its window scrolls** with its focus, the
//!   focused line kept in the middle where it can be; a `▲` or `▼` in the
//!   frame's right end says there is more above or below.
//!
//! Every ink is the palette's ([`Inks`]): the Playdate palette's one-bit
//! inks on the device, the PC's colours in a test. Nothing here decides
//! what a screen shows.

use crate::color::{Palette, Rgb, UiColor};
use crate::glyph_buffer::{BoxStyle, Cell, GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::widgets::{AskView, MenuView, word_wrap};
pub use shuyi_ui::screens::cut;

/// The mark in a list's frame that more lines are above.
pub const MORE_ABOVE: &str = "▲";
/// The mark in a list's frame that more lines are below.
pub const MORE_BELOW: &str = "▼";

/// The inks a Playdate screen paints with.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Inks {
    /// The screen's ground.
    pub ground: Rgb,
    /// Inside a window.
    pub inside: Rgb,
    /// Letters.
    pub text: Rgb,
    /// A window's frame.
    pub frame: Rgb,
    /// The focus bar, and its letters.
    pub bar: (Rgb, Rgb),
    /// The letters of what can't be chosen: the palette's
    /// `window_text_off`, grey dots on the Playdate (dimmed on the PC).
    pub off: Rgb,
}

impl Inks {
    /// The inks of `palette`.
    pub fn of(palette: &Palette) -> Self {
        Self {
            ground: palette.get(UiColor::Black),
            inside: palette.get(UiColor::PanelBg),
            text: palette.get(UiColor::Text),
            frame: palette.get(UiColor::PanelBorder),
            bar: (
                palette.get(UiColor::PanelBorderFocus),
                palette.get(UiColor::PanelBg),
            ),
            off: palette
                .lookup("window_text_off")
                .unwrap_or_else(|| palette.get(UiColor::TextDim)),
        }
    }
}

/// The inks of `ctx`'s palette.
pub fn inks(ctx: &Ctx) -> Inks {
    Inks::of(&ctx.palette)
}

/// Cells `text` takes, as a coordinate.
pub fn cells(text: &str) -> i32 {
    i32::try_from(text.chars().count()).unwrap_or(i32::MAX)
}

/// `n` as a coordinate.
pub fn to_i32(n: usize) -> i32 {
    i32::try_from(n).unwrap_or(i32::MAX)
}

/// `n` as a count, 0 below 0.
pub fn to_usize(n: i32) -> usize {
    usize::try_from(n).unwrap_or(0)
}

/// Paints the white ground over the whole of `buf`.
pub fn clear(ctx: &Ctx, buf: &mut GlyphBuffer) {
    let ink = inks(ctx);
    buf.fill_rect(buf.bounds(), Cell::new(' ', ink.text, ink.ground));
}

/// Prints `text` centred across `buf` on row `y`, on the ground (cut to
/// the buffer's width).
pub fn centred(ctx: &Ctx, buf: &mut GlyphBuffer, y: i32, text: &str) {
    let ink = inks(ctx);
    crate::screens::print_centred(buf, y, text, ink.text, ink.ground);
}

/// A window at `rect`: white inside a double black frame, `title` (if
/// not empty) in the top of the frame from two cells in.
pub fn window(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, title: &str) {
    let ink = inks(ctx);
    buf.fill_rect(rect, Cell::new(' ', ink.text, ink.inside));
    buf.draw_box(rect, BoxStyle::Double, ink.frame, ink.inside);
    if !title.is_empty() {
        let room = to_usize(rect.w - 4);
        let shown = cut(&format!(" {title} "), room);
        buf.print(rect.x + 2, rect.y, &shown, ink.text, ink.inside);
    }
}

/// `text` in the bottom of `rect`'s frame, centred (cut to fit inside the
/// corners, a cell of frame left either side).
pub fn footer(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, text: &str) {
    if text.is_empty() {
        return;
    }
    let ink = inks(ctx);
    let shown = cut(&format!(" {text} "), to_usize(rect.w - 4));
    let x = rect.x + (rect.w - cells(&shown)) / 2;
    buf.print(x, rect.y + rect.h - 1, &shown, ink.text, ink.inside);
}

/// The marks in `rect`'s frame that a list in it goes on above or below
/// what it shows: at the right end of the top and the bottom edge.
pub fn more_marks(ctx: &Ctx, buf: &mut GlyphBuffer, rect: Rect, above: bool, below: bool) {
    let ink = inks(ctx);
    let x = rect.x + rect.w - 3;
    if above {
        buf.print(x, rect.y, MORE_ABOVE, ink.text, ink.inside);
    }
    if below {
        buf.print(x, rect.y + rect.h - 1, MORE_BELOW, ink.text, ink.inside);
    }
}

/// `text` as the focus bar from column `x` on row `y`, `w` cells wide:
/// white letters on black, a cell of bar before them (cut to fit).
pub fn bar(ctx: &Ctx, buf: &mut GlyphBuffer, x: i32, y: i32, w: i32, text: &str) {
    let (bg, fg) = inks(ctx).bar;
    let w = to_usize(w);
    let shown = format!(
        " {:<w$}",
        cut(text, w.saturating_sub(1)),
        w = w.saturating_sub(1)
    );
    buf.print(x, y, &shown, fg, bg);
}

/// `text` at `at` inside a window, `w` cells at most: as the bar if
/// focused (from the cell before it to the cell after its `w`), in grey
/// dots if it can't be chosen.
pub fn item(ctx: &Ctx, buf: &mut GlyphBuffer, at: (i32, i32), w: i32, text: &str, state: Item) {
    let ink = inks(ctx);
    let (x, y) = at;
    match state {
        Item::Focused => bar(ctx, buf, x - 1, y, w + 2, text),
        Item::Plain => {
            buf.print(x, y, &cut(text, to_usize(w)), ink.text, ink.inside);
        }
        Item::Off => {
            buf.print(x, y, &cut(text, to_usize(w)), ink.off, ink.inside);
        }
    }
}

/// How a line of a list is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Item {
    /// The focus: a bar.
    Focused,
    /// Plain black letters.
    Plain,
    /// It can't be chosen: grey dots.
    Off,
}

impl Item {
    /// The state of a line that is `focused`, or can't be chosen when not
    /// `enabled`.
    pub fn of(focused: bool, enabled: bool) -> Self {
        match (focused, enabled) {
            (true, _) => Self::Focused,
            (false, true) => Self::Plain,
            (false, false) => Self::Off,
        }
    }
}

/// `text` wrapped at `width` cells by words, keeping its own line breaks.
pub fn wrap(text: &str, width: usize) -> Vec<String> {
    text.lines()
        .flat_map(|line| word_wrap(line, width))
        .collect()
}

/// The first of `len` lines a list shows in `room` rows to keep line
/// `focus` in sight, in the middle where it can be: 0 when all fit.
pub fn first_shown(len: usize, room: usize, focus: Option<usize>) -> usize {
    if len <= room {
        return 0;
    }
    // A focus past the end shows the end, as the last line does.
    focus.unwrap_or(0).saturating_sub(room / 2).min(len - room)
}

/// The size, in cells, of `menu`'s window: its longest item with a cell
/// either side, inside the frame; a row per item.
pub fn menu_size(ctx: &Ctx, menu: &MenuView) -> (i32, i32) {
    let widest = menu.items.iter().map(|it| cells(&menu_text(ctx, it))).max();
    (widest.unwrap_or(0) + 4, to_i32(menu.items.len()) + 2)
}

/// An item's text: its label and what follows it.
fn menu_text(ctx: &Ctx, item: &crate::widgets::MenuItemView) -> String {
    let label = ctx.label(&item.label);
    match &item.suffix {
        Some((suffix, _)) => format!("{label} {suffix}"),
        None => label,
    }
}

/// `menu` in a window from `(x, y)`, [`menu_size`] big: an item a row, the
/// focused one a bar across the window, one that can't be chosen in grey
/// dots.
pub fn menu(ctx: &Ctx, buf: &mut GlyphBuffer, at: (i32, i32), menu: &MenuView) {
    let (w, h) = menu_size(ctx, menu);
    let rect = Rect::new(at.0, at.1, w, h);
    window(ctx, buf, rect, "");
    for (row, it) in (rect.y + 1..).zip(&menu.items) {
        let state = Item::of(it.focused, it.enabled);
        item(ctx, buf, (rect.x + 2, row), w - 4, &menu_text(ctx, it), state);
    }
}

/// Cells a question's text takes across at most, in its window.
pub const QUESTION_W: usize = 40;

/// What a question's window shows under its text.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Answers<'a> {
    /// How to answer, in words (`A yes · B no`).
    Hint(&'a str),
    /// The answers, on one line, the one in focus the bar.
    Choices(&'a [(&'a str, bool)]),
}

/// The answers' line, each answer with a cell either side and two cells
/// between them, and where each starts in it.
fn answers_line(answers: Answers<'_>) -> (String, Vec<i32>) {
    match answers {
        Answers::Hint(hint) => (hint.to_owned(), Vec::new()),
        Answers::Choices(choices) => {
            let mut line = String::new();
            let mut starts = Vec::new();
            for (i, (label, _)) in choices.iter().enumerate() {
                if i > 0 {
                    line.push_str("  ");
                }
                starts.push(cells(&line));
                line.push(' ');
                line.push_str(label);
                line.push(' ');
            }
            (line, starts)
        }
    }
}

/// A question in a window in the middle of `buf`: its text wrapped to
/// [`QUESTION_W`], then a blank row and its answers. Returns the window.
pub fn ask(ctx: &Ctx, buf: &mut GlyphBuffer, text: &str, answers: Answers<'_>) -> Rect {
    let ink = inks(ctx);
    let lines = wrap(text, QUESTION_W);
    let (line, starts) = answers_line(answers);
    let widest = lines.iter().map(|l| cells(l)).chain([cells(&line)]).max();
    let w = (widest.unwrap_or(0) + 4).min(i32::from(buf.width()));
    let h = to_i32(lines.len()) + 4;
    let rect = Rect::new(
        (i32::from(buf.width()) - w) / 2,
        (i32::from(buf.height()) - h) / 2,
        w,
        h,
    );
    window(ctx, buf, rect, "");
    for (y, text) in (rect.y + 1..).zip(&lines) {
        buf.print(rect.x + 2, y, text, ink.text, ink.inside);
    }
    let y = rect.y + h - 2;
    let x = rect.x + (w - cells(&line)) / 2;
    match answers {
        Answers::Hint(hint) => {
            let shown = cut(hint, to_usize(w - 2));
            buf.print(x.max(rect.x + 1), y, &shown, ink.text, ink.inside);
        }
        Answers::Choices(choices) => {
            for ((label, focused), start) in choices.iter().zip(starts) {
                let at = x + start;
                if *focused {
                    bar(ctx, buf, at, y, cells(label) + 2, label);
                } else {
                    buf.print(at + 1, y, label, ink.text, ink.inside);
                }
            }
        }
    }
    rect
}

/// [`ask`] fed from the question widget's view (`shuyi_ui`'s `Ask`): its
/// answers as the choices, or the line naming its keys as the hint.
pub fn ask_view<T>(ctx: &Ctx, buf: &mut GlyphBuffer, view: &AskView<T>) -> Rect {
    if let Some(keys) = &view.keys {
        return ask(ctx, buf, &view.text, Answers::Hint(keys));
    }
    let choices: Vec<_> = view
        .answers
        .iter()
        .map(|a| (a.label.as_str(), a.focused))
        .collect();
    ask(ctx, buf, &view.text, Answers::Choices(&choices))
}

/// A list of `items` (each its text and whether it is focused) in a
/// window in the middle of `buf`, `title` in its frame: an item a row,
/// the focused one a bar across the window.
pub fn list(ctx: &Ctx, buf: &mut GlyphBuffer, title: &str, items: &[(&str, bool)]) -> Rect {
    let widest = items.iter().map(|(t, _)| cells(t)).max().unwrap_or(0);
    let w = widest.max(cells(title) + 2) + 6;
    let h = to_i32(items.len()) + 2;
    let rect = Rect::new(
        (i32::from(buf.width()) - w) / 2,
        (i32::from(buf.height()) - h) / 2,
        w,
        h,
    );
    window(ctx, buf, rect, title);
    for (y, (text, focused)) in (rect.y + 1..).zip(items) {
        let state = Item::of(*focused, true);
        item(ctx, buf, (rect.x + 3, y), w - 6, text, state);
    }
    rect
}

#[cfg(test)]
pub(crate) mod tests;
