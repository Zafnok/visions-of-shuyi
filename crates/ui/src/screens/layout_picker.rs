//! The first-launch "Pick your layout" screen (`docs/design/controls.md`):
//! one panel per [`Layout`] with a small keyboard diagram and a legend, both
//! read from that layout's bindings. The Options screen opens it again to
//! switch layout ([`LayoutPickerScreen::change`]).
//!
//! **The look is a skin** (ADR-0054). This module decides what the screen
//! shows and does, and says it as plain data: a [`LayoutPickerView`]
//! ([`LayoutPickerScreen::view`]), with each layout's legend and what each
//! key of its keyboard does. [`glyph::paint`] draws that view as glyphs and
//! decides where the keys sit. Tests of what happened read the view; only
//! [`glyph`]'s tests read cells.

pub mod glyph;
pub mod view;

pub use view::{KeyCapView, KeyRole, LayoutPickerView, LayoutView, LegendRow};

use crate::glyph_buffer::GlyphBuffer;
use crate::input::{Action, Chord, Key, Keymap, Layout};
use crate::screen::{Ctx, FrameInput, Screen, Transition};
use crate::widgets::help::{HelpKeys, all_key_names, cursor_keys_name, key_name};
use crate::widgets::{Menu, MenuEvent, MenuItem};

/// The key of the title in the language files ([`Ctx::text`]).
pub const TITLE: &str = "layout_picker.title";
/// The key of the legend's word for the cursor keys: the line that is
/// flagged as movement.
const LEGEND_MOVE: &str = "layout_picker.legend.move";

/// The name shown for a layout, in `ctx`'s language.
pub fn label(ctx: &Ctx, layout: Layout) -> &str {
    ctx.text(match layout {
        Layout::RightHanded => "layout_picker.right_handed",
        Layout::LeftHanded => "layout_picker.left_handed",
    })
}

/// Shows both layouts and saves the one picked. Asked for by a key press
/// while no layout is chosen, it can't be cancelled with a key (the
/// keyboard needs a layout), but any controller button closes it without
/// choosing: that player is on the controller. Pops itself once a layout
/// is picked.
#[derive(Debug, Clone)]
pub struct LayoutPickerScreen {
    menu: Menu,
    /// Whether Cancel backs out without changing anything (opened from
    /// Options).
    cancellable: bool,
}

impl LayoutPickerScreen {
    /// Its [`Screen::name`].
    pub const NAME: &'static str = "layout_picker";

    /// The picker with the first layout (right-handed) focused.
    pub fn new() -> Self {
        Self {
            // A layout must be picked.
            menu: Self::menu().without_cancel(),
            cancellable: false,
        }
    }

    /// The picker as Options opens it, to switch layout: the layout in use
    /// focused, and Cancel backs out without changing anything.
    pub fn change(ctx: &Ctx) -> Self {
        let current = Layout::ALL.iter().position(|&l| Some(l) == ctx.layout());
        Self {
            menu: Self::menu().focused(current.unwrap_or(0)),
            cancellable: true,
        }
    }

    /// One line per layout. The menu only keeps the focus: it is never
    /// drawn, so its lines have no text.
    fn menu() -> Menu {
        let items = Layout::ALL.iter().map(|_| MenuItem::new(String::new()));
        Menu::new(items.collect())
    }

    /// The focused layout.
    pub fn focused(&self) -> Layout {
        Layout::ALL
            .get(self.menu.focus())
            .copied()
            .unwrap_or(Layout::RightHanded)
    }

    /// The bottom help line: every Cursor up and Cursor down key, then every
    /// Confirm key, from the active keymap (before any layout is chosen,
    /// the layout picker's own keys).
    pub fn help(ctx: &Ctx) -> String {
        Self::help_with(ctx, false)
    }

    /// [`help`](Self::help); when the picker can be backed out of
    /// (`cancellable`, from Options), the line that also names the Cancel
    /// key.
    fn help_with(ctx: &Ctx, cancellable: bool) -> String {
        if cancellable {
            return ctx.text_with("layout_picker.help_change", &[]);
        }
        let km = ctx.help_keys();
        let up = all_key_names(km, Action::CursorUp);
        let down = all_key_names(km, Action::CursorDown);
        let confirm = all_key_names(km, Action::Confirm);
        ctx.text_with(
            "layout_picker.help_first",
            &[("up", &up), ("down", &down), ("confirm", &confirm)],
        )
    }

    /// The legend for `km`, in `ctx`'s language: the keys and what they
    /// do; an action with no key shows `! not mapped`. Always its keyboard
    /// keys: the screen is about the keyboard's layouts.
    pub fn legend(ctx: &Ctx, km: &Keymap) -> Vec<LegendRow> {
        let km = HelpKeys::keyboard(km);
        let row = |keys: String, what: &str| LegendRow {
            keys,
            what: ctx.text(what).to_owned(),
            movement: what == LEGEND_MOVE,
        };
        vec![
            row(cursor_keys_name(km), LEGEND_MOVE),
            row(
                all_key_names(km, Action::Confirm),
                "layout_picker.legend.select",
            ),
            row(
                all_key_names(km, Action::Cancel),
                "layout_picker.legend.back",
            ),
            row(
                key_name(km, Action::PrevUnit),
                "layout_picker.legend.prev_unit",
            ),
            row(
                key_name(km, Action::NextUnit),
                "layout_picker.legend.next_unit",
            ),
            row(key_name(km, Action::Info), "layout_picker.legend.unit_info"),
            row(
                key_name(km, Action::DangerZone),
                "layout_picker.legend.danger_zone",
            ),
            row(
                key_name(km, Action::EndTurn),
                "layout_picker.legend.end_turn",
            ),
            row(
                key_name(km, Action::ToggleAutoEnd),
                "layout_picker.legend.auto_end",
            ),
            row(key_name(km, Action::Rewind), "layout_picker.legend.rewind"),
        ]
    }

    /// The screen as it is now, as plain data for a skin to paint
    /// ([`glyph::paint`]): each layout with its legend and what every key
    /// of its keyboard does, the one in focus, and the help line.
    pub fn view(&self, ctx: &Ctx) -> LayoutPickerView {
        let focused = self.focused();
        let layouts = Layout::ALL.iter().map(|&layout| {
            let km = ctx.keymap_for(layout);
            LayoutView {
                layout,
                name: label(ctx, layout).to_owned(),
                focused: layout == focused,
                legend: Self::legend(ctx, &km),
                keys: key_caps(&km),
            }
        });
        LayoutPickerView {
            title: ctx.text(TITLE).to_owned(),
            layouts: layouts.collect(),
            help: Self::help_with(ctx, self.cancellable),
        }
    }
}

/// What `key` does in `km`: moves the cursor (a key that repeats), does
/// something else, or nothing.
// check-keys: keyboard picture
fn key_role(km: &Keymap, key: Key) -> KeyRole {
    match km.action(Chord::plain(key)) {
        Some(action) if action.is_repeatable() => KeyRole::Movement,
        Some(_) => KeyRole::Bound,
        None => KeyRole::Unbound,
    }
}

/// Every key of the keyboard with its role in `km`; a skin draws the ones
/// its picture has.
// check-keys: keyboard picture
fn key_caps(km: &Keymap) -> Vec<KeyCapView> {
    Key::ALL
        .iter()
        .map(|&key| KeyCapView {
            key,
            role: key_role(km, key),
        })
        .collect()
}

impl Default for LayoutPickerScreen {
    fn default() -> Self {
        Self::new()
    }
}

impl Screen for LayoutPickerScreen {
    fn name(&self) -> &'static str {
        Self::NAME
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        // Bound or not; the button does nothing else.
        if !self.cancellable && input.pad_pressed() {
            return Transition::Pop;
        }
        for &action in &input.actions {
            match self.menu.handle_with_sound(action, &mut ctx.audio) {
                Some(MenuEvent::Chosen(i)) => {
                    let layout = Layout::ALL.get(i).copied().unwrap_or(Layout::RightHanded);
                    // If saving fails the layout is still used this
                    // session; the player is just asked again next launch.
                    let _ = ctx.choose_layout(layout);
                    return Transition::Pop;
                }
                Some(MenuEvent::Cancelled) => return Transition::Pop,
                None => {}
            }
        }
        Transition::None
    }

    fn draw(&self, ctx: &Ctx, buf: &mut GlyphBuffer) {
        glyph::paint(ctx, &self.view(ctx), buf);
    }

    fn as_any(&self) -> Option<&dyn std::any::Any> {
        Some(self)
    }
}

#[cfg(test)]
mod tests;
