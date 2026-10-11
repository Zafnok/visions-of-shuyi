//! What a [`Menu`](super::Menu) shows, as plain data: its items with their
//! labels and which one is focused, no cells or positions. A screen puts
//! the [`MenuView`] in its own view and its skin paints it
//! ([`super::glyph::paint`] today; ADR-0054).

use crate::color::UiColor;

/// The whole menu, as it is this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuView {
    /// The items, top to bottom.
    pub items: Vec<MenuItemView>,
}

impl MenuView {
    /// The focused item, if there is one.
    pub fn focused(&self) -> Option<&MenuItemView> {
        self.items.iter().find(|i| i.focused)
    }
}

/// One item of a menu.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItemView {
    /// Its text.
    pub label: String,
    /// Text that follows the label (e.g. why it can't be chosen) and the
    /// palette role it is shown in. (A role the screen chose; the screens
    /// that give one move to a meaning when 0241 and 0242 convert them.)
    pub suffix: Option<(String, UiColor)>,
    /// Whether it can be chosen: a disabled item is dimmed and skipped by
    /// the cursor.
    pub enabled: bool,
    /// Whether the cursor is on it.
    pub focused: bool,
}
