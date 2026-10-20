//! Tests of the Guides list (ticket 0458): every lesson then every tip,
//! the highlighted one's text with the player's keys, opened from the map
//! menu at every tutorial level, closed by Cancel.

use crate::input::LEFT_HANDED;
use crate::screens::battle::testing::Values as _;
use insta::assert_snapshot;
use trpg_core::TutorialLevel;

use super::*;
use crate::color::Rgb;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::{Cell, GlyphBuffer};
use crate::screen::FrameInput;
use crate::screen::tests::ctx;
use crate::screens::battle::map_menu::MapEntry;
use crate::screens::battle::mode::Mode;
use crate::screens::battle::testing::Stacked;
use crate::screens::battle::{BattleScreen, quick_battle};

/// Every topic's title, as listed.
fn titles(c: &Ctx) -> Vec<String> {
    GuidesScreen::new(&c.game.content).list_view(c).topics
}

#[test]
fn lessons_come_first_then_the_tips_in_their_order() {
    let c = ctx();
    let lessons = c
        .game
        .content
        .lessons
        .lessons
        .iter()
        .map(|l| l.title.clone());
    let tips = c.game.content.tips.tips.iter().map(|t| t.title.clone());
    let want: Vec<String> = lessons.chain(tips).collect();
    assert_eq!(titles(&c), want);
    // The story's lessons (0457) first; the new tips are listed too.
    let all = titles(&c);
    assert_eq!(all[0], "Moving the cursor");
    for title in ["Your move", "Skills", "Magic", "Terrain"] {
        assert!(all.iter().any(|t| t == title), "{title}: {all:?}");
    }
}

#[test]
fn a_lessons_checklist_shows_as_plain_lines_with_the_keys() {
    let c = ctx();
    let view = GuidesScreen::new(&c.game.content).list_view(&c);
    assert_eq!(view.title, "Guides");
    assert_eq!(view.focus, 0);
    assert_eq!(view.heading, "Moving the cursor");
    assert_eq!(
        view.lines,
        [
            "That's your cursor. It picks tiles and units.",
            "One tile each way, so you know the feel.",
            "",
            "Move it up: Up",
            "Move it down: Down",
            "Move it left: Left",
            "Move it right: Right",
        ]
    );
}

#[test]
fn up_and_down_move_through_the_topics_and_wrap() {
    let c = ctx();
    let n = titles(&c).len();
    let mut g = GuidesScreen::new(&c.game.content);
    assert_eq!(g.step(Action::CursorUp), GuidesEffect::Moved);
    assert_eq!(g.focus(), n - 1);
    let view = g.list_view(&c);
    assert_eq!(view.heading, "Terrain");
    assert_eq!(view.lines.len(), 3);
    assert_eq!(g.step(Action::CursorDown), GuidesEffect::Moved);
    assert_eq!(g.focus(), 0);
    g.step(Action::CursorDown);
    let view = g.list_view(&c);
    assert_eq!(view.heading, "Played on the keys");
    assert_eq!(view.lines[0], "No mouse here; it's all keys. And every key");
    // Other keys do nothing; Cancel closes.
    for action in [Action::Confirm, Action::CursorLeft, Action::Info] {
        assert_eq!(g.step(action), GuidesEffect::None);
    }
    assert_eq!(g.focus(), 1);
    assert_eq!(g.step(Action::Cancel), GuidesEffect::Close);
}

#[test]
fn an_empty_list_does_not_move() {
    let mut g = GuidesScreen {
        cursor: ListCursor::new(0),
    };
    assert_eq!(g.step(Action::CursorDown), GuidesEffect::None);
    assert_eq!(g.step(Action::CursorUp), GuidesEffect::None);
    let mut one = GuidesScreen {
        cursor: ListCursor::new(1),
    };
    assert_eq!(one.step(Action::CursorDown), GuidesEffect::None);
    assert_eq!(one.step(Action::CursorUp), GuidesEffect::None);
    // Two topics: both ways move.
    let mut two = GuidesScreen {
        cursor: ListCursor::new(2),
    };
    assert_eq!(two.step(Action::CursorUp), GuidesEffect::Moved);
    assert_eq!(two.focus(), 1);
    assert_eq!(two.step(Action::CursorDown), GuidesEffect::Moved);
    assert_eq!(two.focus(), 0);
}

// ---- On the battle screen ------------------------------------------------------

fn press(s: &mut Stacked, c: &mut Ctx, actions: &[Action]) {
    s.update(c, &FrameInput::new(actions.to_vec(), 0.0, vec![]));
}

fn render(s: &Stacked, c: &Ctx) -> GlyphBuffer {
    let blank = Cell::new(' ', Rgb::new(0, 0, 0), Rgb::new(0, 0, 0));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, blank);
    s.paint(c, &mut buf);
    buf
}

/// The Quick Battle with the Guides list opened from the map menu.
fn opened(c: &mut Ctx) -> Stacked {
    let mut s = Stacked::new(BattleScreen::new(quick_battle(&c.game.content).unwrap()));
    // Out of the way first: the tips on screen.
    press(&mut s, c, &[]);
    while s.shown_tip().is_some() {
        press(&mut s, c, &[Action::Confirm]);
    }
    // Cancel with nothing to cancel opens the map menu; Guides is third.
    press(&mut s, c, &[Action::Cancel]);
    assert!(matches!(s.mode(), Mode::MapMenu { .. }), "{:?}", s.mode());
    press(&mut s, c, &[Action::CursorDown, Action::CursorDown]);
    press(&mut s, c, &[Action::Confirm]);
    s
}

#[test]
fn the_map_menu_opens_it_at_every_level_and_cancel_goes_back() {
    for level in [
        None,
        Some(TutorialLevel::Full),
        Some(TutorialLevel::Short),
        Some(TutorialLevel::None),
    ] {
        let mut c = ctx();
        c.game.tips_enabled = true;
        c.game.campaign_tutorial = level;
        let mut s = opened(&mut c);
        let view = s.view(&c);
        let shown = s.guides().expect("the list is open").view(&c);
        let guides = &shown.list;
        assert_eq!(guides.heading, "Moving the cursor", "{level:?}");
        assert_eq!(shown.help, "arrows choose · d close");
        // No tip or lesson shows over it.
        assert_eq!(s.shown_tip(), None, "{level:?}");
        assert_eq!(s.shown_lesson(), None, "{level:?}");
        assert!(view.tip.is_none() && view.lesson.is_none());
        // Stepping through the topics shows each one's text.
        press(&mut s, &mut c, &[Action::CursorDown]);
        let guides = s.guides().unwrap().list_view(&c);
        assert_eq!(guides.focus, 1);
        assert_eq!(guides.heading, "Played on the keys");
        assert!(guides.lines.iter().any(|l| l.contains("arrows")));
        // The map's keys don't reach the map.
        let cursor = s.cursor.pos;
        press(&mut s, &mut c, &[Action::CursorRight, Action::NextUnit]);
        assert_eq!(s.cursor.pos, cursor);
        assert!(s.guides().is_some());
        // Cancel goes back to the map menu, still on Guides.
        press(&mut s, &mut c, &[Action::Cancel]);
        assert!(s.guides().is_none());
        let Mode::MapMenu { menu } = s.mode() else {
            panic!("{:?}", s.mode());
        };
        let entries = menu.values();
        assert_eq!(entries[menu.focus()], MapEntry::Guides);
    }
}

#[test]
fn the_text_names_the_keys_the_player_has_bound() {
    let mut c = ctx();
    c.use_layout(LEFT_HANDED);
    let mut s = opened(&mut c);
    let lines = s.guides().unwrap().list_view(&c).lines;
    assert_eq!(lines[3], "Move it up: w");
    press(&mut s, &mut c, &[Action::CursorDown]);
    let lines = s.guides().unwrap().list_view(&c).lines;
    assert!(
        lines.contains(&"Move the cursor: wasd".to_owned()),
        "{lines:?}"
    );
    assert!(
        lines.contains(&"Open Options from the map menu: k".to_owned()),
        "{lines:?}"
    );
    assert_eq!(s.guides().unwrap().view(&c).help, "wasd choose · k close");
}

#[test]
fn the_list_renders() {
    let mut c = ctx();
    let mut s = opened(&mut c);
    assert_snapshot!("lesson", render(&s, &c).to_snapshot(&c.palette));
    press(&mut s, &mut c, &[Action::CursorUp]);
    assert_snapshot!("tip", render(&s, &c).to_snapshot(&c.palette));
}
