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
use crate::widgets::help::{HelpKeys, all_key_names, cursor_keys_name, help_line, key_name};
use crate::widgets::{Menu, MenuEvent, MenuItem};

/// Title text.
pub const TITLE: &str = "Pick your layout";

/// The name shown for a layout.
pub fn label(layout: Layout) -> &'static str {
    match layout {
        Layout::RightHanded => "Right-handed",
        Layout::LeftHanded => "Left-handed",
    }
}

/// Shows both layouts and saves the one picked. On first launch it can't
/// be cancelled: the game needs a layout. Pops itself once a layout is
/// picked.
#[derive(Debug, Clone)]
pub struct LayoutPickerScreen {
    menu: Menu,
    /// Whether Cancel backs out without changing anything (opened from
    /// Options).
    cancellable: bool,
}

impl LayoutPickerScreen {
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

    fn menu() -> Menu {
        let items = Layout::ALL.iter().map(|&l| MenuItem::new(label(l)));
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
        let choose = Some(format!(
            "{} {}",
            all_key_names(km, Action::CursorUp),
            all_key_names(km, Action::CursorDown)
        ));
        help_line(&[
            (choose, "choose"),
            (Some(all_key_names(km, Action::Confirm)), "pick"),
        ])
    }

    /// The legend for `km`: the keys and what they do; an action with no
    /// key shows `! not mapped`. Always its keyboard keys: the screen is
    /// about the keyboard's layouts.
    pub fn legend(km: &Keymap) -> Vec<LegendRow> {
        let km = HelpKeys::keyboard(km);
        let row = |keys: String, what: &str, movement: bool| LegendRow {
            keys,
            what: what.to_owned(),
            movement,
        };
        vec![
            row(cursor_keys_name(km), "move", true),
            row(all_key_names(km, Action::Confirm), "select", false),
            row(all_key_names(km, Action::Cancel), "back", false),
            row(key_name(km, Action::PrevUnit), "prev unit", false),
            row(key_name(km, Action::NextUnit), "next unit", false),
            row(key_name(km, Action::Info), "unit info", false),
            row(key_name(km, Action::DangerZone), "danger zone", false),
            row(key_name(km, Action::EndTurn), "end turn", false),
            row(key_name(km, Action::ToggleAutoEnd), "auto-end", false),
            row(key_name(km, Action::Rewind), "rewind", false),
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
                name: label(layout).to_owned(),
                focused: layout == focused,
                legend: Self::legend(&km),
                keys: key_caps(&km),
            }
        });
        LayoutPickerView {
            title: TITLE.to_owned(),
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
        "layout_picker"
    }

    fn update(&mut self, ctx: &mut Ctx, input: &FrameInput) -> Transition {
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
