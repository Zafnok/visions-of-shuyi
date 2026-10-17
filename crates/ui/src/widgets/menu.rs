//! A vertical list menu: title menu now, action and map menus later.
//!
//! The menu holds its items and its focus. What it shows is
//! [`Menu::view`], plain data, and [`glyph::paint`] draws that view as
//! glyphs (ADR-0054): the screens' own skins call it for their menu.

pub mod glyph;
pub mod view;

use crate::prelude::*;
pub use view::{MenuItemView, MenuView, SuffixKind};

use crate::audio::{AudioQueue, MenuSound};
use crate::input::{ActionSet, UiAction};

/// One menu entry.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MenuItem {
    /// Text shown.
    pub label: String,
    /// Disabled items are drawn dim, skipped by the cursor and can't be chosen.
    pub enabled: bool,
    /// Text shown after the label (e.g. `(broken)`), and what it is.
    pub suffix: Option<(String, SuffixKind)>,
    /// What the item does, shown under the list while it has the focus
    /// (ticket 0429, `look-and-feel.md` → *Descriptions in lists*).
    pub note: Option<String>,
}

impl MenuItem {
    /// An enabled item.
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
            suffix: None,
            note: None,
        }
    }

    /// The same item with `text` of `kind` after its label.
    #[must_use]
    pub fn with_suffix(mut self, text: impl Into<String>, kind: SuffixKind) -> Self {
        self.suffix = Some((text.into(), kind));
        self
    }

    /// The same item with `text` saying what it does, shown under the
    /// list while it has the focus.
    #[must_use]
    pub fn with_note(mut self, text: impl Into<String>) -> Self {
        self.note = Some(text.into());
        self
    }

    /// A disabled item.
    pub fn disabled(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: false,
            suffix: None,
            note: None,
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
    /// Whether the focus also stops on disabled items, so their notes can
    /// be read (they still can't be chosen).
    browse_disabled: bool,
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
            browse_disabled: false,
        }
    }

    /// The same menu where `Cancel` does nothing (and so plays nothing):
    /// for menus with nothing to back out of.
    #[must_use]
    pub fn without_cancel(mut self) -> Self {
        self.cancellable = false;
        self
    }

    /// The same menu where the focus also stops on disabled items, so the
    /// player can read what they do (`look-and-feel.md` → *Descriptions in
    /// lists*). Confirm on one still chooses nothing.
    #[must_use]
    pub fn browsing_disabled(mut self) -> Self {
        self.browse_disabled = true;
        self
    }

    /// The same menu focused on item `index`, if it exists and the focus
    /// may stop on it (it is enabled, or the menu is
    /// [`browsing_disabled`](Self::browsing_disabled)).
    #[must_use]
    pub fn focused(mut self, index: usize) -> Self {
        if self.items.get(index).is_some_and(|i| self.stops_on(i)) {
            self.focus = index;
        }
        self
    }

    /// The items, in order.
    pub fn items(&self) -> &[MenuItem] {
        &self.items
    }

    /// Whether the focus may stop on `item`.
    fn stops_on(&self, item: &MenuItem) -> bool {
        item.enabled || self.browse_disabled
    }

    /// Index of the focused item.
    pub fn focus(&self) -> usize {
        self.focus
    }

    /// Handles one action, by what the game [takes it for](ActionSet::ui).
    /// Returns an event for [`UiAction::Confirm`] on an enabled item and
    /// for [`UiAction::Cancel`]; moves the focus for [`UiAction::Down`] and
    /// [`UiAction::Up`]; ignores everything else.
    pub fn handle<A: ActionSet>(&mut self, action: A) -> Option<MenuEvent> {
        match action.ui() {
            Some(UiAction::Down) => self.step(true),
            Some(UiAction::Up) => self.step(false),
            Some(UiAction::Confirm) => {
                return self
                    .items
                    .get(self.focus)
                    .filter(|i| i.enabled)
                    .map(|_| MenuEvent::Chosen(self.focus));
            }
            Some(UiAction::Cancel) if self.cancellable => return Some(MenuEvent::Cancelled),
            _ => {}
        }
        None
    }

    /// [`handle`](Self::handle), playing the menu sounds on `audio`:
    /// `menu_move` when the focus moves, `menu_select` when an item is
    /// chosen, `menu_cancel` when cancelled, [`MenuSound::Denied`] for
    /// Confirm on a disabled item. Any other action that does nothing (a
    /// move with nowhere to go) plays nothing.
    pub fn handle_with_sound<A: ActionSet>(
        &mut self,
        action: A,
        audio: &mut AudioQueue,
    ) -> Option<MenuEvent> {
        let before = self.focus;
        let event = self.handle(action);
        let sound = match event {
            Some(MenuEvent::Chosen(_)) => Some(MenuSound::Select),
            Some(MenuEvent::Cancelled) => Some(MenuSound::Cancel),
            None if self.focus != before => Some(MenuSound::Move),
            None if action.ui() == Some(UiAction::Confirm) && !self.items.is_empty() => {
                Some(MenuSound::Denied)
            }
            None => None,
        };
        if let Some(sound) = sound {
            audio.menu(sound);
        }
        event
    }

    /// Moves the focus to the next (or previous) item it may stop on
    /// (enabled, unless the menu browses disabled ones too), wrapping.
    /// Stays put if there is none.
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
            .find(|&i| self.stops_on(&self.items[i]));
        if let Some(i) = next {
            self.focus = i;
        }
    }

    /// What the menu shows: every item, and which one has the focus.
    pub fn view(&self) -> MenuView {
        let items = self.items.iter().enumerate().map(|(i, item)| MenuItemView {
            label: item.label.clone(),
            suffix: item.suffix.clone(),
            note: item.note.clone(),
            enabled: item.enabled,
            focused: i == self.focus,
        });
        MenuView {
            items: items.collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;
    use proptest::prelude::*;

    use super::*;
    use crate::color::UiColor;
    use crate::color::tests::game_palette;
    use crate::glyph_buffer::{Cell, GlyphBuffer};
    use crate::input::Action;
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
                    note: None,
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
        assert_eq!(
            crate::audio::AUDIO.cue(MenuSound::Denied),
            "menu_cancel",
            "until 0427"
        );
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
        assert_eq!(glyph::size(&m.view()), (4, 2));
    }

    #[test]
    fn a_browsing_menu_stops_on_disabled_items_but_never_chooses_them() {
        let mut m = menu(&[true, false, true]).browsing_disabled();
        assert_eq!(m.focus(), 0);
        m.handle(CursorDown);
        assert_eq!(m.focus(), 1, "stops on the disabled item");
        assert_eq!(m.handle(Confirm), None);
        let mut audio = AudioQueue::default();
        m.handle_with_sound(Confirm, &mut audio);
        assert_eq!(audio.take().len(), 1, "the denied sound");
        m.handle(CursorDown);
        assert_eq!(m.focus(), 2);
        m.handle(CursorUp);
        assert_eq!(m.focus(), 1);
        // Focusing it by index works too; a plain menu refuses it.
        assert_eq!(
            menu(&[true, false]).browsing_disabled().focused(1).focus(),
            1
        );
        assert_eq!(menu(&[true, false]).focused(1).focus(), 0);
        // It still opens on the first enabled item.
        assert_eq!(menu(&[false, true]).browsing_disabled().focus(), 1);
    }

    #[test]
    fn a_note_goes_to_the_view() {
        let m = Menu::new(vec![
            MenuItem::new("a").with_note("does a"),
            MenuItem::new("b"),
        ]);
        let view = m.view();
        assert_eq!(view.items[0].note.as_deref(), Some("does a"));
        assert_eq!(view.items[1].note, None);
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
                note: None,
            }
        );
        assert!(!MenuItem::disabled("b").enabled);
        let m = Menu::new(vec![MenuItem::new("a"), MenuItem::disabled("bcd")]);
        assert_eq!(m.items()[1].label, "bcd");
        assert_eq!(glyph::size(&m.view()), (7, 4));
    }

    #[test]
    fn the_view_lists_the_items_and_the_focus() {
        let mut m = Menu::new(vec![
            MenuItem::new("Move"),
            MenuItem::disabled("Attack").with_suffix(" (no foe)", SuffixKind::Warning),
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
            Some((" (no foe)".to_owned(), SuffixKind::Warning))
        );
        assert_eq!(view.items[0].suffix, None);
        assert_eq!(view.focused().map(|i| i.label.as_str()), Some("Move"));
        // The focus follows the menu, and an empty menu has none.
        m.handle(CursorDown);
        assert_eq!(m.view().focused().map(|i| i.label.as_str()), Some("Wait"));
        assert_eq!(Menu::new(vec![]).view().focused(), None);
    }

    fn render(m: &Menu) -> String {
        let p = game_palette();
        let view = m.view();
        let (w, h) = glyph::size(&view);
        let mut buf = GlyphBuffer::new(
            u16::try_from(w + 2).unwrap(),
            u16::try_from(h + 2).unwrap(),
            Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black)),
        );
        glyph::paint(&p, &view, &mut buf, 1, 1);
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

    /// A menu works with another game's actions, by what that game takes
    /// them for (ticket 1106): here the engine's sample game.
    #[test]
    fn a_menu_reacts_to_what_another_game_takes_its_actions_for() {
        use shuyi_assets::keymap::sample::Action as Sample;
        let mut menu = Menu::new(vec![MenuItem::new("a"), MenuItem::new("b")]);
        assert_eq!(menu.handle(Sample::CursorDown), None);
        assert_eq!(menu.focus(), 1);
        assert_eq!(menu.handle(Sample::Info), None);
        assert_eq!(menu.handle(Sample::Confirm), Some(MenuEvent::Chosen(1)));
        assert_eq!(menu.handle(Sample::Cancel), Some(MenuEvent::Cancelled));
    }
}
