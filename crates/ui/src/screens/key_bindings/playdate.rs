//! The Playdate's look of the Key bindings screen (ticket 1045): the
//! engine's [`KeyBindingsView`] in the one-bit look of
//! [`crate::screens::playdate`]. A window the screen's width, its title in
//! the top of its frame; in it each group's heading and its actions, a
//! name and three slots each, and Restore defaults. The first slot is wide
//! enough for every name the Playdate has (`Crank forward`); the other two
//! cut a longer one, its last cell a `…`, and there is no room for the
//! slots' headings (`Button 2` would be cut too). The focused slot is the bar; an action that just lost its button
//! to another has a `*` before its name. What doesn't fit scrolls with the
//! focus, a mark in the frame saying there is more. A message goes in the
//! bottom of the frame and the help on the bottom row; the choices of a
//! slot open in a window under it, the question in one over the rows.
//!
//! With no keyboard the view has no switch; with one (a test's), the
//! switch is the first line, the shown side named in the bar's letters
//! while it has the focus.

use super::view::{ChoicesView, SlotView, SwitchView};
use super::{KeyBindingsView, RowView, Side};
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::prelude::*;
use crate::screen::Ctx;
use crate::screens::playdate::{
    Answers, Item, ask, bar, cells, centred, clear, cut, first_shown, footer, inks, item,
    more_marks, to_i32, to_usize, window,
};

/// Column of the group headings.
const HEADING_X: i32 = 2;
/// Column of the actions' names, and the cells they take at most.
const LABEL: (i32, i32) = (3, 15);
/// Each slot's column and width, its bar's: the first takes 13 letters
/// (`Crank forward`), the others 7.
const SLOTS: [(i32, i32); 3] = [(19, 14), (33, 8), (41, 8)];
/// What an empty slot shows.
const EMPTY_SLOT: &str = "·";
/// The mark before an action that just lost its button to another.
const LOST: &str = "*";
/// The choices' window, in cells across.
const CHOICES_W: i32 = 12;

/// A line of the window.
enum Line<'a> {
    Switch(&'a SwitchView),
    Gap,
    Heading(&'a str),
    Row(&'a RowView),
    Restore,
}

impl Line<'_> {
    /// Whether the focus is on it.
    fn focused(&self, view: &KeyBindingsView) -> bool {
        match self {
            Line::Switch(switch) => switch.focused,
            Line::Row(row) => row.focused(),
            Line::Restore => view.restore.focused,
            Line::Gap | Line::Heading(_) => false,
        }
    }
}

/// The window's lines, top to bottom.
fn lines(view: &KeyBindingsView) -> Vec<Line<'_>> {
    let mut lines = Vec::new();
    if let Some(switch) = &view.switch {
        lines.extend([Line::Switch(switch), Line::Gap]);
    }
    for (i, group) in view.groups.iter().enumerate() {
        if i > 0 {
            lines.push(Line::Gap);
        }
        lines.push(Line::Heading(&group.heading));
        lines.extend(group.rows.iter().map(Line::Row));
    }
    lines.extend([Line::Gap, Line::Restore]);
    lines
}

/// Paints `view` over the whole of `buf`.
pub fn paint(ctx: &Ctx, view: &KeyBindingsView, buf: &mut GlyphBuffer) {
    clear(ctx, buf);
    let ink = inks(ctx);
    let bottom = i32::from(buf.height()) - 1;
    let rect = Rect::new(0, 0, i32::from(buf.width()), bottom);
    window(ctx, buf, rect, &view.title);
    let lines = lines(view);
    let focus = lines.iter().position(|l| l.focused(view));
    let top = rect.y + 1;
    let room = to_usize(rect.y + rect.h - 1 - top);
    let first = first_shown(lines.len(), room, focus);
    let mut focused_y = None;
    for (y, line) in (top..).zip(lines.iter().skip(first).take(room)) {
        match line {
            Line::Switch(switch) => paint_switch(ctx, buf, switch, y),
            Line::Heading(heading) => {
                buf.print(HEADING_X, y, heading, ink.text, ink.inside);
            }
            Line::Row(row) => {
                paint_row(ctx, buf, view, row, y);
                if row.focused() {
                    focused_y = Some(y);
                }
            }
            Line::Restore => {
                // Across the name's column and the first slot's.
                let state = Item::of(view.restore.focused, true);
                let w = SLOTS[0].0 + SLOTS[0].1 - LABEL.0;
                item(ctx, buf, (LABEL.0, y), w, &view.restore.label, state);
            }
            Line::Gap => {}
        }
    }
    more_marks(ctx, buf, rect, first > 0, first + room < lines.len());
    if let Some(message) = &view.message {
        footer(ctx, buf, rect, message);
    }
    centred(ctx, buf, bottom, &view.help);
    if let (Some(choices), Some(y)) = (&view.choices, focused_y) {
        paint_choices(ctx, buf, choices, rect, y);
    }
    if let Some(question) = &view.question {
        ask(ctx, buf, &question.text, Answers::Hint(&question.answers));
    }
}

/// The keyboard / controller switch on row `y`: the shown side's name,
/// the bar while the switch has the focus, the other side's in grey.
fn paint_switch(ctx: &Ctx, buf: &mut GlyphBuffer, switch: &SwitchView, y: i32) {
    let mut x = LABEL.0;
    for (side, name) in [
        (Side::Keyboard, switch.keyboard.as_str()),
        (Side::Controller, switch.controller.as_str()),
    ] {
        let state = if side != switch.shown {
            Item::Off
        } else if switch.focused {
            Item::Focused
        } else {
            Item::Plain
        };
        item(ctx, buf, (x, y), cells(name), name, state);
        x += cells(name) + 3;
    }
}

/// `row` on row `y`: its name, then its slots (the one with the cursor
/// the bar), or the not-mapped note over the empty slots.
fn paint_row(ctx: &Ctx, buf: &mut GlyphBuffer, view: &KeyBindingsView, row: &RowView, y: i32) {
    let ink = inks(ctx);
    if row.lost_key {
        buf.print(HEADING_X, y, LOST, ink.text, ink.inside);
    }
    let label = cut(&row.label, to_usize(LABEL.1));
    buf.print(LABEL.0, y, &label, ink.text, ink.inside);
    for (j, (slot, &(x, w))) in row.slots.iter().zip(&SLOTS).enumerate() {
        if row.focus == Some(j) {
            let shown = match slot {
                SlotView::Capturing(prompt) => prompt.as_str(),
                SlotView::Bound(name) => name.as_str(),
                SlotView::Empty => "",
            };
            bar(ctx, buf, x, y, w, shown);
        } else if let SlotView::Bound(name) = slot {
            buf.print(x + 1, y, &cut(name, to_usize(w - 1)), ink.text, ink.inside);
        } else if !row.unmapped {
            buf.print(x + 1, y, EMPTY_SLOT, ink.text, ink.inside);
        }
    }
    if row.unmapped {
        // After the bar of the slot with the cursor, if it is on the row.
        let x = match row.focus.and_then(|j| SLOTS.get(j)) {
            Some(&(x, w)) if x == SLOTS[0].0 => x + w + 1,
            _ => SLOTS[0].0 + 1,
        };
        let room = to_usize(i32::from(buf.width()) - 2 - x);
        buf.print(x, y, &cut(&view.not_mapped, room), ink.text, ink.inside);
    }
}

/// The choices of the slot on row `y` in a window under it (over it where
/// it wouldn't fit), the focused one the bar.
fn paint_choices(ctx: &Ctx, buf: &mut GlyphBuffer, choices: &ChoicesView, rect: Rect, y: i32) {
    let h = to_i32(choices.lines.len()) + 2;
    let x = SLOTS.get(choices.slot).map_or(SLOTS[0].0, |&(x, _)| x);
    let x = x.min(rect.x + rect.w - CHOICES_W);
    let below = y + 1;
    let top = if below + h <= rect.y + rect.h {
        below
    } else {
        y - h
    };
    let at = Rect::new(x, top, CHOICES_W, h);
    window(ctx, buf, at, "");
    for (i, (line_y, line)) in (top + 1..).zip(&choices.lines).enumerate() {
        let state = Item::of(i == choices.focused, true);
        item(ctx, buf, (x + 2, line_y), CHOICES_W - 4, line, state);
    }
}

#[cfg(test)]
mod tests;
