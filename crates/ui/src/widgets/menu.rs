//! A vertical list menu: title menu now, action and map menus later.
//!
//! The menu holds its items and its focus. What it shows is
//! [`Menu::view`], plain data, and [`glyph::paint`] draws that view as
//! glyphs (ADR-0054): the screens' own skins call it for their menu.

pub mod glyph;
pub mod view;

pub use view::{MenuItemView, MenuView};

use crate::audio::{AudioQueue, MenuSound};
use crate::color::{Palette, UiColor};
use crate::glyph_buffer::GlyphBuffer;
use crate::input::Action;

/// One menu entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// Text shown.
    pub label: String,
    /// Disabled items are drawn dim, skipped by the cursor and can't be chosen.
    pub enabled: bool,
    /// Text drawn after the label in its own colour (e.g. `(broken)`).
    pub suffix: Option<(String, UiColor)>,
}

impl MenuItem {
    /// An enabled item.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            suffix: None,
        }
    }

    /// The same item with `text` after its label, drawn in `color`.
    #[must_use]
    pub fn with_suffix(mut self, text: impl Into<String>, color: UiColor) -> Self {
        self.suffix = Some((text.into(), color));
        self
    }

    /// A disabled item.
    pub fn disabled(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: false,
            suffix: None,
        }
    }
}

/// What a menu reports back to its screen.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuEvent {
    /// Confirm on the enabled item at this index.
    Chosen(usize),
    /// Cancel was pressed.
    Cancelled,
}

/// A vertical list of items in a single-line box. `CursorDown`/`CursorUp`
/// move the focus (wrapping, skipping disabled items), `Confirm` chooses the
/// focused item, `Cancel` cancels (unless the menu is
/// [`without_cancel`](Self::without_cancel)).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Menu {
    items: Vec<MenuItem>,
    focus: usize,
    /// Whether `Cancel` reports [`MenuEvent::Cancelled`].
    cancellable: bool,
}

impl Menu {
    /// A menu focused on its first enabled item (or the first item, if none
    /// is enabled).
    pub fn new(items: Vec<MenuItem>) -> Self {
        let focus = items.iter().position(|i| i.enabled).unwrap_or(0);
        Self {
            items,
            focus,
            cancellable: true,
        }
    }

    /// The same menu where `Cancel` does nothing (and so plays nothing):
    /// for menus with nothing to back out of.
    #[must_use]
    pub fn without_cancel(mut self) -> Self {
        self.cancellable = false;
        self
    }

    /// The same menu focused on item `index`, if it exists and is enabled.
    #[must_use]
    pub fn focused(mut self, index: usize) -> Self {
        if self.items.get(index).is_some_and(|i| i.enabled) {
            self.focus = index;
        }
        self
    }

    /// The items, in order.
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Index of the focused item.
    pub fn focus(&self) -> usize {
        self.focus
    }

    /// Handles one action. Returns an event for `Confirm` on an enabled item
    /// and for `Cancel`; moves the focus for `CursorDown`/`CursorUp`;
    /// ignores everything else.
    pub fn handle(&mut self, action: Action) -> Option<MenuEvent> {
        match action {
            Action::CursorDown => self.step(true),
            Action::CursorUp => self.step(false),
            Action::Confirm => {
                return self
                    .items
                    .get(self.focus)
                    .filter(|i| i.enabled)
                    .map(|_| MenuEvent::Chosen(self.focus));
            }
            Action::Cancel if self.cancellable => return Some(MenuEvent::Cancelled),
            _ => {}
        }
        None
    }

    /// [`handle`](Self::handle), playing the menu sounds on `audio`:
    /// `menu_move` when the focus moves, `menu_select` when an item is
    /// chosen, `menu_cancel` when cancelled, [`MenuSound::Denied`] for
    /// Confirm on a disabled item. Any other action that does nothing (a
    /// move with nowhere to go) plays nothing.
    pub fn handle_with_sound(
        &mut self,
        action: Action,
        audio: &mut AudioQueue,
    ) -> Option<MenuEvent> {
        let before = self.focus;
        let event = self.handle(action);
        let sound = match event {
            Some(MenuEvent::Chosen(_)) => Some(MenuSound::Select),
            Some(MenuEvent::Cancelled) => Some(MenuSound::Cancel),
            None if self.focus != before => Some(MenuSound::Move),
            None if action == Action::Confirm && !self.items.is_empty() => Some(MenuSound::Denied),
            None => None,
        };
        if let Some(sound) = sound {
            audio.menu(sound);
        }
        event
    }

    /// Moves the focus to the next (or previous) enabled item, wrapping.
    /// Stays put if no other item is enabled.
    fn step(&mut self, down: bool) {
        let n = self.items.len();
        let next = (1..n)
            .map(|k| {
                if down {
                    (self.focus + k) % n
                } else {
                    (self.focus + n - k) % n
                }
            })
            .find(|&i| self.items[i].enabled);
        if let Some(i) = next {
            self.focus = i;
        }
    }

    /// What the menu shows: every item, and which one has the focus.
    pub fn view(&self) -> MenuView {
        let items = self.items.iter().enumerate().map(|(i, item)| MenuItemView {
            label: item.label.clone(),
            suffix: item.suffix.clone(),
            enabled: item.enabled,
            focused: i == self.focus,
        });
        MenuView {
            items: items.collect(),
        }
    }

    /// Box size in cells ([`glyph::size`] of the [`view`](Self::view)).
    /// Kept for the screens that haven't moved to a view and a skin yet
    /// (ADR-0054; tickets 0241 and 0242 convert them, then this goes).
    pub fn size(&self) -> (i32, i32) {
        glyph::size(&self.view())
    }

    /// Draws the menu with its top-left corner at `(x, y)`
    /// ([`glyph::paint`] of the [`view`](Self::view)). Kept for the screens
    /// that haven't moved to a view and a skin yet (ADR-0054; tickets 0241
    /// and 0242 convert them, then this goes).
    pub fn draw(&self, palette: &Palette, buf: &mut GlyphBuffer, x: i32, y: i32) {
        glyph::paint(palette, &self.view(), buf, x, y);
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use proptest::prelude::*;

    use super::*;
    use crate::color::tests::game_palette;
    use crate::glyph_buffer::Cell;
    use Action::{Cancel, Confirm, CursorDown, CursorLeft, CursorUp, Info};

    fn menu(enabled: &[bool]) -> Menu {
        Menu::new(
            enabled
                .iter()
                .enumerate()
                .map(|(i, &on)| MenuItem {
                    label: format!("item {i}"),
                    enabled: on,
                    suffix: None,
                })
                .collect(),
        )
    }

    /// The sound cues `actions` play on `m`, in order.
    fn sounds(m: &mut Menu, actions: &[Action]) -> Vec<String> {
        let mut audio = AudioQueue::default();
        for &a in actions {
            m.handle_with_sound(a, &mut audio);
        }
        audio
            .take()
            .iter()
            .filter_map(|r| r.cue().map(str::to_owned))
            .collect()
    }

    #[test]
    fn move_select_and_cancel_sound() {
        let mut m = menu(&[true, true, true]);
        assert_eq!(sounds(&mut m, &[CursorDown]), ["menu_move"]);
        assert_eq!(sounds(&mut m, &[CursorUp, CursorUp]), ["menu_move"; 2]);
        assert_eq!(m.focus(), 2, "wrapped");
        assert_eq!(sounds(&mut m, &[Confirm]), ["menu_select"]);
        assert_eq!(sounds(&mut m, &[Cancel]), ["menu_cancel"]);
        assert_eq!(sounds(&mut m, &[CursorLeft, Info]), Vec::<String>::new());
    }

    #[test]
    fn handle_with_sound_reports_the_same_events() {
        let mut m = menu(&[true, true]);
        let mut audio = AudioQueue::default();
        assert_eq!(m.handle_with_sound(CursorDown, &mut audio), None);
        assert_eq!(
            m.handle_with_sound(Confirm, &mut audio),
            Some(MenuEvent::Chosen(1))
        );
        assert_eq!(
            m.handle_with_sound(Cancel, &mut audio),
            Some(MenuEvent::Cancelled)
        );
    }

    #[test]
    fn a_disabled_item_is_denied() {
        // Nothing is enabled, so a disabled item is focused.
        let mut m = menu(&[false, false]);
        assert_eq!(sounds(&mut m, &[Confirm]), ["menu_cancel"]);
        assert_eq!(MenuSound::Denied.cue(), "menu_cancel", "until 0427");
        // An empty menu has nothing to deny.
        assert!(sounds(&mut Menu::new(vec![]), &[Confirm]).is_empty());
    }

    #[test]
    fn doing_nothing_is_silent() {
        let mut m = menu(&[false, false]);
        assert!(sounds(&mut m, &[CursorDown, CursorUp]).is_empty());
        // A single enabled item: the focus has nowhere to go.
        let mut m = menu(&[false, true, false]);
        assert_eq!(
            sounds(&mut m, &[CursorDown, CursorUp]),
            Vec::<String>::new()
        );
        assert_eq!(sounds(&mut m, &[Confirm]), ["menu_select"]);
    }

    #[test]
    fn without_cancel_ignores_cancel() {
        let mut m = menu(&[true, true]).without_cancel();
        assert_eq!(m.handle(Cancel), None);
        assert_eq!(sounds(&mut m, &[Cancel]), Vec::<String>::new());
        assert_eq!(m.handle(Confirm), Some(MenuEvent::Chosen(0)));
    }

    fn focus_after(m: &mut Menu, actions: &[Action]) -> usize {
        for &a in actions {
            m.handle(a);
        }
        m.focus()
    }

    #[test]
    fn down_and_up_wrap() {
        let mut m = menu(&[true, true, true]);
        assert_eq!(m.focus(), 0);
        assert_eq!(focus_after(&mut m, &[CursorDown]), 1);
        assert_eq!(focus_after(&mut m, &[CursorDown]), 2);
        assert_eq!(focus_after(&mut m, &[CursorDown]), 0);
        assert_eq!(focus_after(&mut m, &[CursorUp]), 2);
        assert_eq!(focus_after(&mut m, &[CursorUp]), 1);
    }

    #[test]
    fn disabled_items_are_skipped() {
        let mut m = menu(&[false, true, false, true, false]);
        assert_eq!(m.focus(), 1, "starts on the first enabled item");
        assert_eq!(focus_after(&mut m, &[CursorDown]), 3);
        assert_eq!(focus_after(&mut m, &[CursorDown]), 1);
        assert_eq!(focus_after(&mut m, &[CursorUp]), 3);
        assert_eq!(focus_after(&mut m, &[CursorUp]), 1);
    }

    #[test]
    fn focused_moves_to_an_enabled_item_only() {
        let m = menu(&[true, false, true]);
        assert_eq!(m.clone().focused(2).focus(), 2);
        assert_eq!(m.clone().focused(1).focus(), 0, "disabled");
        assert_eq!(m.focused(9).focus(), 0, "no such item");
    }

    #[test]
    fn a_single_enabled_item_keeps_focus() {
        let mut m = menu(&[false, true, false]);
        assert_eq!(focus_after(&mut m, &[CursorDown, CursorUp, CursorDown]), 1);
    }

    #[test]
    fn nothing_enabled_nothing_chosen() {
        let mut m = menu(&[false, false]);
        assert_eq!(m.focus(), 0);
        assert_eq!(m.handle(CursorDown), None);
        assert_eq!(m.focus(), 0);
        assert_eq!(m.handle(Confirm), None);
        assert_eq!(m.handle(Cancel), Some(MenuEvent::Cancelled));
    }

    #[test]
    fn empty_menu_is_harmless() {
        let mut m = Menu::new(vec![]);
        for a in Action::ALL {
            m.handle(a);
        }
        assert_eq!(m.handle(Confirm), None);
        assert_eq!(m.size(), (4, 2));
    }

    #[test]
    fn events() {
        let mut m = menu(&[true, true]);
        assert_eq!(m.handle(Confirm), Some(MenuEvent::Chosen(0)));
        assert_eq!(m.handle(CursorDown), None);
        assert_eq!(m.handle(Confirm), Some(MenuEvent::Chosen(1)));
        assert_eq!(m.handle(Cancel), Some(MenuEvent::Cancelled));
        assert_eq!(m.handle(CursorLeft), None);
        assert_eq!(m.handle(Info), None);
        assert_eq!(m.focus(), 1, "other actions don't move the focus");
    }

    #[test]
    fn item_constructors() {
        assert_eq!(
            MenuItem::new("a"),
            MenuItem {
                label: "a".into(),
                enabled: true,
                suffix: None,
            }
        );
        assert!(!MenuItem::disabled("b").enabled);
        let m = Menu::new(vec![MenuItem::new("a"), MenuItem::disabled("bcd")]);
        assert_eq!(m.items()[1].label, "bcd");
        assert_eq!(m.size(), (7, 4));
    }

    #[test]
    fn the_view_lists_the_items_and_the_focus() {
        let mut m = Menu::new(vec![
            MenuItem::new("Move"),
            MenuItem::disabled("Attack").with_suffix(" (no foe)", UiColor::HpLow),
            MenuItem::new("Wait"),
        ]);
        let view = m.view();
        let shown: Vec<_> = view
            .items
            .iter()
            .map(|i| (i.label.as_str(), i.enabled, i.focused))
            .collect();
        assert_eq!(
            shown,
            [
                ("Move", true, true),
                ("Attack", false, false),
                ("Wait", true, false)
            ]
        );
        assert_eq!(
            view.items[1].suffix,
            Some((" (no foe)".to_owned(), UiColor::HpLow))
        );
        assert_eq!(view.items[0].suffix, None);
        assert_eq!(view.focused().map(|i| i.label.as_str()), Some("Move"));
        // The focus follows the menu, and an empty menu has none.
        m.handle(CursorDown);
        assert_eq!(m.view().focused().map(|i| i.label.as_str()), Some("Wait"));
        assert_eq!(Menu::new(vec![]).view().focused(), None);
    }

    #[test]
    fn size_and_draw_are_the_skins() {
        let m = Menu::new(vec![MenuItem::new("a"), MenuItem::disabled("bcd")]);
        assert_eq!(m.size(), glyph::size(&m.view()));
        let p = game_palette();
        let blank = Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black));
        let mut drawn = GlyphBuffer::new(12, 8, blank);
        let mut painted = drawn.clone();
        m.draw(&p, &mut drawn, 2, 1);
        glyph::paint(&p, &m.view(), &mut painted, 2, 1);
        assert_eq!(drawn.to_snapshot(&p), painted.to_snapshot(&p));
    }

    fn render(m: &Menu) -> String {
        let p = game_palette();
        let (w, h) = m.size();
        let mut buf = GlyphBuffer::new(
            u16::try_from(w + 2).unwrap(),
            u16::try_from(h + 2).unwrap(),
            Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black)),
        );
        m.draw(&p, &mut buf, 1, 1);
        buf.to_snapshot(&p)
    }

    #[test]
    fn snapshot_with_disabled_item() {
        let mut m = Menu::new(vec![
            MenuItem::new("Move"),
            MenuItem::disabled("Attack"),
            MenuItem::new("Items"),
            MenuItem::new("Wait"),
        ]);
        m.handle(CursorDown);
        assert_eq!(m.focus(), 2);
        assert_snapshot!(render(&m));
    }

    proptest! {
        #[test]
        fn focus_always_lands_on_an_enabled_item(
            enabled in prop::collection::vec(any::<bool>(), 1..8),
            moves in prop::collection::vec(any::<bool>(), 0..20),
        ) {
            let mut m = menu(&enabled);
            for down in moves {
                m.handle(if down { CursorDown } else { CursorUp });
                prop_assert!(m.focus() < enabled.len());
                if enabled.iter().any(|&e| e) {
                    prop_assert!(enabled[m.focus()]);
                    prop_assert_eq!(m.handle(Confirm), Some(MenuEvent::Chosen(m.focus())));
                }
            }
        }

        #[test]
        fn down_then_up_returns(enabled in prop::collection::vec(any::<bool>(), 1..8)) {
            let mut m = menu(&enabled);
            let start = m.focus();
            m.handle(CursorDown);
            m.handle(CursorUp);
            prop_assert_eq!(m.focus(), start);
        }
    }
}
