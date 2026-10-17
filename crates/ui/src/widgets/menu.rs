//! A vertical list menu: Shuyi Engine's (`shuyi_ui::widgets::menu`,
//! ticket 1112), re-exported here under its old name. [`Menu::view`] is
//! what it shows and [`glyph::paint`] draws that view as glyphs
//! (ADR-0054): the screens' own skins call it for their menu.

pub use shuyi_ui::widgets::menu::*;

#[cfg(test)]
mod tests {
    use insta::assert_snapshot;

    use super::*;
    use crate::audio::{AUDIO, AudioQueue};
    use crate::color::UiColor;
    use crate::color::tests::game_palette;
    use crate::glyph_buffer::{Cell, GlyphBuffer};
    use crate::input::Action::{self, Cancel, Confirm, CursorDown, CursorUp};

    /// This game's actions move and choose with this game's sounds.
    #[test]
    fn this_games_keys_drive_it_with_its_sounds() {
        let mut m = Menu::new(vec![
            MenuItem::new("a"),
            MenuItem::disabled("b"),
            MenuItem::new("c"),
        ]);
        let mut audio = AudioQueue::new(AUDIO);
        let mut events = Vec::new();
        for a in [CursorDown, CursorUp, CursorUp, Confirm, Cancel] {
            events.push(m.handle_with_sound(a, &mut audio));
        }
        assert_eq!(
            events,
            [
                None,
                None,
                None,
                Some(MenuEvent::Chosen(2)),
                Some(MenuEvent::Cancelled)
            ]
        );
        let cues: Vec<_> = audio.take().iter().map(|r| r.cue().map(str::to_owned)).collect();
        let cue = |c: &str| Some(c.to_owned());
        assert_eq!(
            cues,
            [
                cue("menu_move"),
                cue("menu_move"),
                cue("menu_move"),
                cue("menu_select"),
                cue("menu_cancel")
            ]
        );
        // Confirm on a greyed-out item: `menu_cancel` until Nick's tone
        // (0427).
        let mut m = Menu::new(vec![MenuItem::disabled("x")]);
        m.handle_with_sound(Action::Confirm, &mut audio);
        assert_eq!(audio.take()[0].cue(), Some("menu_cancel"));
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
}
