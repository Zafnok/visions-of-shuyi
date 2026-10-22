//! The Playdate battle skin's look (ADR-0054, ticket 1038): where each
//! window goes on the 50 × 15 screen and how it looks, in cells, and
//! through the one-bit renderer in pixels. What is shown is tested on the
//! view (`battle/view_tests.rs`).

use trpg_content::Ink;
use trpg_core::{Phase, Pos};

use super::*;
use crate::input::Action;
use crate::map_view::playdate::tests::{SCREEN, braille, shown};
use crate::onebit::is_white;
use crate::screen::Screen;
use crate::screens::battle::banner::BannerKind;
use crate::screens::battle::notes::{NOTE_WIDTH, wrap_notes};
use crate::screens::battle::tests::{lord_menu, quick, step};
use crate::screens::battle::view::{
    BoxFighterView, FightBoxView, ObjectiveView, QuestionView, UsesView,
};
use crate::screens::battle::{BattleScreen, HealPopup, PopupKind};
use crate::screens::playdate::tests::{blank, c as ink, cell, find, is_bar, pd_ctx, row, rows};
use crate::widgets::menu::SuffixKind;
use crate::{ConsoleSizes, ConsoleSpec};

/// The test context on the Playdate: its console, palette and skins.
pub(super) fn ctx() -> Ctx {
    let mut ctx = pd_ctx();
    ctx.console = ConsoleSpec::PLAYDATE;
    ctx
}

/// `view` painted on the Playdate's screen.
fn painted(c: &Ctx, view: &BattleView) -> GlyphBuffer {
    let mut buf = blank();
    paint(c, view, &mut buf);
    buf
}

/// `s`'s view changed by `change`, painted.
fn painted_with(s: &BattleScreen, change: impl FnOnce(&mut BattleView)) -> GlyphBuffer {
    let c = ctx();
    let mut view = s.view(&c);
    change(&mut view);
    painted(&c, &view)
}

/// Where the map skin paints the cursor's tile in `s`'s view.
fn cursor_cells(s: &BattleScreen, c: &Ctx) -> Rect {
    let view = s.view(c);
    tile_cells(c, &view.scene, SCREEN, s.cursor().pos).unwrap()
}

fn item(label: &str, note: Option<&str>, enabled: bool, focused: bool) -> MenuItemView {
    MenuItemView {
        label: label.into(),
        suffix: None,
        note: note.map(str::to_owned),
        enabled,
        focused,
    }
}

#[test]
fn the_map_area_is_the_whole_screen_on_the_playdate_and_the_map_view_on_the_pc() {
    assert_eq!(map_area(&ctx()), SCREEN);
    assert_eq!(
        map_area(&crate::screen::tests::ctx()),
        super::super::layout::MAP_VIEW
    );
}

#[test]
fn the_screen_draws_with_the_skin_of_its_skin_set() {
    let c = ctx();
    let s = quick();
    let mut drawn = blank();
    s.draw(&c, &mut drawn);
    assert_eq!(drawn, painted(&c, &s.view(&c)));
    // 24 × 15 tiles: the map fills the screen.
    assert_eq!(s.view(&c).scene.size, (24, 15));
}

#[test]
fn browsing_shows_the_unit_top_left_and_the_terrain_top_right_and_no_help_bar() {
    let c = ctx();
    let s = quick();
    let view = s.view(&c);
    let panel = view.panel.clone().unwrap();
    let (unit, terrain) = (panel.unit.unwrap(), panel.terrain.unwrap());
    let buf = painted(&c, &view);
    // The unit's window in the top-left corner: its name, then its HP.
    let hp = format!("{} {}/{}", unit.hp_name, unit.hp, unit.max);
    assert_eq!(cell(&buf, 0, 0).glyph, '╔');
    assert_eq!(find(&buf, 1, &unit.name), Some(2));
    assert_eq!(find(&buf, 2, &hp), Some(2));
    assert_eq!(cell(&buf, 0, 3).glyph, '╚');
    // The terrain's in the top-right corner: its name, then Def and Avo.
    let w = cells(&terrain.bonuses).max(cells(&terrain.name)) + 4;
    assert_eq!(cell(&buf, 50 - w, 0).glyph, '╔');
    assert_eq!(cell(&buf, 49, 0).glyph, '╗');
    assert_eq!(find(&buf, 1, &terrain.name), Some(52 - w));
    assert_eq!(find(&buf, 2, &terrain.bonuses), Some(52 - w));
    assert_eq!(cell(&buf, 49, 3).glyph, '╝');
    // Black letters on the white inside.
    let name = cell(&buf, 2, 1);
    assert_eq!((name.fg, name.bg), (ink(Ink::Black), ink(Ink::White)));
    // No help bar, no toggles: the map goes to the bottom row.
    let all = rows(&buf).concat();
    assert!(!all.contains(view.help.trim()) && !all.contains(view.status.trim()));
}

#[test]
fn the_pop_ups_move_to_the_bottom_corners_while_the_cursor_is_on_the_top_rows() {
    let mut c = ctx();
    let mut s = quick();
    let (mut top, mut bottom) = (false, false);
    for _ in 0..8 {
        let view = s.view(&c);
        let at_top = cursor_cells(&s, &c).y < TOP_ROWS;
        assert_eq!(corners_at_bottom(&c, &view.scene, SCREEN), at_top);
        let name = view.panel.clone().unwrap().terrain.unwrap().name;
        let buf = painted(&c, &view);
        // The terrain's window is two lines: in the bottom corner its name
        // is on the fourth row from the bottom.
        let (y, other) = if at_top { (12, 1) } else { (1, 12) };
        assert!(find(&buf, y, &name).is_some(), "{:?}", rows(&buf));
        assert_eq!(find(&buf, other, &name), None);
        (top, bottom) = (top || !at_top, bottom || at_top);
        step(&mut s, &mut c, &[Action::CursorUp]);
    }
    assert!(top && bottom, "the cursor never reached both");
}

#[test]
fn with_no_unit_under_the_cursor_only_the_terrain_shows() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::CursorRight]);
    let view = s.view(&c);
    let panel = view.panel.clone().unwrap();
    assert!(panel.unit.is_none());
    let buf = painted(&c, &view);
    assert_ne!(cell(&buf, 0, 0).glyph, '╔');
    let name = panel.terrain.unwrap().name;
    assert!(find(&buf, 1, &name).is_some_and(|x| x > 25));
}

#[test]
fn the_map_menu_opens_beside_the_cursor_with_its_focus_a_bar_and_no_pop_ups() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::CursorRight, Action::Cancel]);
    let view = s.view(&c);
    let menu = view.menu.as_ref().unwrap();
    let buf = painted(&c, &view);
    let size = framed_size(&c, menu, view.map_info.as_ref(), SCREEN);
    let (x, y) = menu_origin_in(SCREEN, cursor_cells(&s, &c), size);
    assert_eq!(cell(&buf, x, y).glyph, '╔');
    assert_eq!(cell(&buf, x + size.0 - 1, y + size.1 - 1).glyph, '╝');
    // Right of the cursor's tile, the first item level with it.
    assert!(x > cursor_cells(&s, &c).x);
    assert_eq!(find(&buf, y + 1, "Units"), Some(x + 2));
    assert!(is_bar(&buf, y + 1, "Units"));
    assert!(!is_bar(&buf, y + 2, "Objective"));
    // No corner pop-ups while it is open.
    let terrain = view.panel.clone().unwrap().terrain.unwrap().name;
    assert!(rows(&buf).iter().all(|r| !r.contains(&terrain)));
}

#[test]
fn the_action_menu_opens_beside_its_unit_on_the_side_with_room() {
    let mut c = ctx();
    let s = lord_menu(&mut c);
    let view = s.view(&c);
    let menu = view.menu.clone().unwrap();
    let MenuPlace::Beside(tile) = menu.place else {
        panic!("not beside its unit");
    };
    let cells = tile_cells(&c, &view.scene, SCREEN, tile).unwrap();
    let size = menu_size(&c, &menu.menu, SCREEN);
    // Moved to the right edge, the unit's menu goes to its left.
    let right = Rect::new(SCREEN.w - 2, cells.y, 2, 1);
    let (left, _) = menu_origin_in(SCREEN, right, size);
    assert_eq!(left, SCREEN.w - 3 - size.0);
    // Where it is, beside it on the right.
    let buf = painted(&c, &view);
    let (x, y) = menu_origin_in(SCREEN, cells, size);
    assert_eq!(x, cells.x + cells.w + 1);
    assert_eq!(cell(&buf, x, y).glyph, '╔');
}

#[test]
fn a_menu_shows_its_header_suffixes_grey_items_and_the_focused_note() {
    let c = ctx();
    let mut cut = item("Flowing Cut", Some("+10 hit; no counter"), true, true);
    cut.suffix = Some(("-2 dur".into(), SuffixKind::Aside));
    let guard = item("Guard Break", Some("ignores Def"), false, false);
    let m = BattleMenuView {
        menu: MenuView::new(vec![cut, guard]),
        place: MenuPlace::Centred,
        header: Some(MenuHeader {
            name: "Iron Sword".into(),
            uses: Some(UsesView {
                text: "20/20".into(),
                spent: false,
            }),
        }),
    };
    // As wide as the longest line or note, with the rule and note rows.
    assert_eq!(menu_size(&c, &m.menu, SCREEN), (23, 6));
    let mut buf = blank();
    paint_menu(&c, &m, (5, 2), SCREEN, &mut buf);
    assert_eq!(row(&buf, 2), "╔═ Iron Sword 20/20 ══╗");
    assert!(is_bar(&buf, 3, "Flowing Cut -2 dur"));
    assert_eq!(row(&buf, 4), "║ Guard Break         ║");
    assert_eq!(cell(&buf, 7, 4).fg, ink(Ink::Grey));
    assert_eq!(row(&buf, 5), "╟─────────────────────╢");
    assert_eq!(row(&buf, 6), "║ +10 hit; no counter ║");
    assert_eq!(row(&buf, 7), "╚═════════════════════╝");
}

#[test]
fn a_menu_too_long_for_the_screen_scrolls_with_its_focus_and_marks_the_rest() {
    let c = ctx();
    let menu = |focus| {
        let items = (0..20)
            .map(|i| item(&format!("Unit {i:02}"), None, true, i == focus))
            .collect();
        BattleMenuView {
            menu: MenuView::new(items),
            place: MenuPlace::Centred,
            header: None,
        }
    };
    // 13 items in the 15 rows: the focus in the middle, the end in view.
    let m = menu(15);
    assert_eq!(menu_size(&c, &m.menu, SCREEN), (11, 15));
    let mut buf = blank();
    paint_menu(&c, &m, (0, 0), SCREEN, &mut buf);
    assert_eq!(row(&buf, 1), "║ Unit 07 ║");
    assert_eq!(row(&buf, 13), "║ Unit 19 ║");
    assert_eq!(row(&buf, 0), "╔═══════▲═╗");
    assert_eq!(row(&buf, 14), "╚═════════╝");
    assert!(is_bar(&buf, 9, "Unit 15"));
    // The focus near the top: the start, and more below.
    let mut buf = blank();
    paint_menu(&c, &menu(3), (0, 0), SCREEN, &mut buf);
    assert_eq!(row(&buf, 1), "║ Unit 00 ║");
    assert_eq!(row(&buf, 0), "╔═════════╗");
    assert_eq!(row(&buf, 14), "╚═══════▼═╝");
}

#[test]
fn a_question_is_a_window_in_the_middle_with_its_answer_keys_inside() {
    let s = quick();
    let buf = painted_with(&s, |v| {
        v.dialog = Some(DialogView::Question(QuestionView {
            question: "End turn with 3 units ready?".into(),
            answers: "A yes / B no".into(),
        }));
    });
    // Its text, a blank row, its keys, in a window in the middle.
    assert_eq!(row(&buf, 5), "╔══════════════════════════════╗");
    assert_eq!(row(&buf, 6), "║ End turn with 3 units ready? ║");
    assert_eq!(row(&buf, 8), "║         A yes / B no         ║");
    assert_eq!(find(&buf, 5, "╔"), Some(9));
}

#[test]
fn the_objective_page_wraps_its_notes_to_the_screen_under_its_title() {
    let s = quick();
    let note = "Burn the forest to cut off the ambush quickly, before the riders \
                reach the ford.";
    let lines = wrap_notes([note], NOTE_WIDTH);
    assert!(lines.iter().any(|l| cells(l) > 46), "{lines:?}");
    let buf = painted_with(&s, |v| {
        v.dialog = Some(DialogView::Objective(ObjectiveView {
            title: "Objective".into(),
            objective: "Rout the enemy".into(),
            turn: "Turn 1".into(),
            notes: Some(NotesView {
                title: "Notes".into(),
                lines,
            }),
        }));
    });
    let shown = rows(&buf);
    let top = shown
        .iter()
        .position(|r| r.contains("═ Objective ═"))
        .unwrap();
    let at = |dy: usize| shown[top + dy].as_str();
    // Wrapped again at the 46 cells inside a window across the screen.
    let line = |text: &str| format!("║ {text:<44} ║");
    assert_eq!(at(1), line("Rout the enemy"));
    assert_eq!(at(2), line("Turn 1"));
    assert_eq!(at(4), line("Notes"));
    assert_eq!(at(5), line("• Burn the forest to cut off the ambush"));
    assert_eq!(at(6), line("  quickly, before the riders reach the ford."));
    assert!(at(7).starts_with('╚'));
}

#[test]
fn the_battle_notes_box_keeps_off_the_rows_of_the_units_they_are_about() {
    let c = ctx();
    let s = quick();
    let lord = s.cursor().pos;
    let notes = NotesView {
        title: "BATTLE NOTES".into(),
        lines: wrap_notes(["The lord must live."], NOTE_WIDTH),
    };
    let window_rows = |noted: Vec<Pos>| {
        let buf = painted_with(&s, |v| {
            v.notes = Some(notes.clone());
            v.noted = noted;
        });
        let top = (0..15).find(|&y| row(&buf, y).contains("BATTLE NOTES"));
        let top = top.unwrap_or_else(|| panic!("no box: {:?}", rows(&buf)));
        top..top + 3
    };
    // Centred; a unit on one of its rows noted, it moves off them.
    let centred = window_rows(vec![]);
    let view = s.view(&c);
    let on_row = |y| Pos::new(lord.x, y);
    let row_of = |pos| tile_cells(&c, &view.scene, SCREEN, pos).map(|t| t.y);
    let unit = (-20..20)
        .map(on_row)
        .find(|&p| row_of(p).is_some_and(|y| centred.contains(&y)))
        .unwrap();
    let unit_row = row_of(unit).unwrap();
    assert!(!window_rows(vec![unit]).contains(&unit_row));
}

#[test]
fn a_banner_is_a_window_in_the_middle_of_the_screen() {
    let s = quick();
    let buf = painted_with(&s, |v| {
        v.banner = Some(BannerView {
            kind: BannerKind::Phase {
                phase: Phase::Enemy,
                turn: 1,
            },
            title: "ENEMY PHASE".into(),
            subtitle: "Turn 1".into(),
        });
    });
    // 24 × 6 in the middle of the 50 × 15 screen.
    let span = |y| -> String { (13..37).map(|x| cell(&buf, x, y).glyph).collect() };
    assert_eq!(span(4), format!("╔{}╗", "═".repeat(22)));
    assert_eq!(span(5), format!("║{}║", " ".repeat(22)));
    assert_eq!(span(6), "║     ENEMY PHASE      ║");
    assert_eq!(span(7), "║        Turn 1        ║");
    assert_eq!(span(9), format!("╚{}╝", "═".repeat(22)));
}

#[test]
fn the_message_goes_on_the_edge_away_from_the_cursor() {
    let mut c = ctx();
    let mut s = quick();
    let message = MessageView {
        text: "Heal Lord +10 HP".into(),
        kind: super::super::view::MessageKind::Said,
    };
    let (mut top, mut bottom) = (false, false);
    for _ in 0..8 {
        let in_top_half = cursor_cells(&s, &c).y < 7;
        let buf = painted_with(&s, |v| v.message = Some(message.clone()));
        let y = if in_top_half { 13 } else { 1 };
        assert_eq!(find(&buf, y, &message.text), Some(17), "{:?}", rows(&buf));
        (top, bottom) = (top || !in_top_half, bottom || in_top_half);
        step(&mut s, &mut c, &[Action::CursorUp]);
    }
    assert!(top && bottom, "the cursor never reached both halves");
}

#[test]
fn heal_numbers_float_a_row_over_their_units() {
    let c = ctx();
    let s = quick();
    let lord = s.cursor().pos;
    let tile = cursor_cells(&s, &c);
    let buf = painted_with(&s, |v| {
        v.popups = vec![HealPopup {
            pos: lord,
            amount: 10,
            kind: PopupKind::Heal,
            t: 0.0,
        }];
    });
    assert_eq!(find(&buf, tile.y - 1, "+10"), Some(tile.x));
}

#[test]
fn the_windows_are_black_frames_round_white_on_the_one_bit_screen() {
    let c = ctx();
    let s = quick();
    let buf = painted(&c, &s.view(&c));
    let frame = shown(&buf);
    // The unit's window: the cell inside its frame on its first line is
    // white, the frame's cell beside it has black.
    let inside = (16..32).all(|y| (8..16).all(|x| is_white(&frame, x, y)));
    let edge = (16..32).any(|y| (0..8).any(|x| !is_white(&frame, x, y)));
    assert!(inside && edge);
    insta::assert_snapshot!(braille(&frame));
}

#[test]
fn the_unit_list_opens_in_the_middle_of_the_screen() {
    let c = ctx();
    let s = quick();
    let items = vec![
        item("Lord", None, true, true),
        item("Knight", None, true, false),
        item("Mage", None, true, false),
    ];
    let menu = BattleMenuView {
        menu: MenuView::new(items),
        place: MenuPlace::Centred,
        header: None,
    };
    let size = menu_size(&c, &menu.menu, SCREEN);
    assert_eq!(size, (10, 5));
    let buf = painted_with(&s, |v| v.menu = Some(menu));
    // (50 - 10) / 2 across, (15 - 5) / 2 down.
    assert_eq!(cell(&buf, 20, 5).glyph, '╔');
    assert_eq!(cell(&buf, 29, 9).glyph, '╝');
    assert!(is_bar(&buf, 6, "Lord"));
}

#[test]
fn a_menu_with_notes_scrolls_sooner_its_note_rows_kept() {
    let c = ctx();
    let items = (0..12)
        .map(|i| item(&format!("Art {i:02}"), Some("does a thing"), true, i == 0))
        .collect();
    let m = BattleMenuView {
        menu: MenuView::new(items),
        place: MenuPlace::Centred,
        header: None,
    };
    // 11 items, the rule and the note fill the 15 rows.
    assert_eq!(menu_size(&c, &m.menu, SCREEN), (16, 15));
    let mut buf = blank();
    paint_menu(&c, &m, (0, 0), SCREEN, &mut buf);
    assert_eq!(row(&buf, 11), "║ Art 10       ║");
    assert_eq!(row(&buf, 12), "╟────────────▼─╢");
    assert_eq!(row(&buf, 13), "║ does a thing ║");
    assert_eq!(row(&buf, 14), "╚══════════════╝");
}

#[test]
fn the_focus_bar_header_note_and_long_lines_stay_inside_the_frame() {
    let c = ctx();
    let long = "x".repeat(60);
    let m = BattleMenuView {
        menu: MenuView::new(vec![item(&long, Some(&"n".repeat(60)), true, true)]),
        place: MenuPlace::Centred,
        header: Some(MenuHeader {
            name: "y".repeat(60),
            uses: None,
        }),
    };
    assert_eq!(menu_size(&c, &m.menu, SCREEN), (50, 5));
    let mut buf = blank();
    paint_menu(&c, &m, (0, 0), SCREEN, &mut buf);
    // The header and the item cut with `…` inside the corners.
    assert_eq!(cell(&buf, 49, 0).glyph, '╗');
    assert_eq!(cell(&buf, 48, 0).glyph, '═');
    assert_eq!(cell(&buf, 47, 0).glyph, '…');
    assert_eq!(cell(&buf, 49, 1).glyph, '║');
    assert_eq!(cell(&buf, 48, 1).glyph, '…');
    // The note too, under its rule.
    assert_eq!(row(&buf, 3), format!("║ {}… ║", "n".repeat(45)));
    // The bar from the cell after the frame to the cell before it.
    let bar = ink(Ink::Black);
    assert_eq!(cell(&buf, 1, 1).bg, bar);
    assert_eq!(cell(&buf, 48, 1).bg, bar);
}

#[test]
fn a_box_is_as_wide_as_its_title_needs_and_centred_across_the_screen() {
    let s = quick();
    let notes = NotesView {
        title: "BATTLE NOTES".into(),
        lines: vec!["• A.".into()],
    };
    let buf = painted_with(&s, |v| v.notes = Some(notes));
    let top = (0..15)
        .find(|&y| row(&buf, y).contains("BATTLE NOTES"))
        .unwrap();
    // The title and a frame cell either side of it with its spaces: 18.
    assert_eq!(cell(&buf, 16, top).glyph, '╔');
    assert_eq!(cell(&buf, 33, top).glyph, '╗');
}

#[test]
fn a_window_shows_no_more_lines_than_fit_and_cuts_wide_ones() {
    let c = ctx();
    let mut lines: Vec<String> = (0..20).map(|i| format!("line {i}")).collect();
    lines[0] = "z".repeat(60);
    let mut buf = blank();
    window_of(&c, &mut buf, Rect::new(0, 0, 20, 6), "", &lines);
    // Four lines inside the frame, the last row its edge.
    assert_eq!(row(&buf, 4), "║ line 3           ║");
    assert_eq!(row(&buf, 5), "╚══════════════════╝");
    assert_eq!(row(&buf, 6), "");
    // The wide one cut with `…` inside the frame.
    assert_eq!(row(&buf, 1), format!("║ {}… ║", "z".repeat(15)));
}

#[test]
fn the_map_menu_has_the_turn_in_its_frame_and_the_rewind_charges_under_it() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::CursorRight, Action::Cancel]);
    let view = s.view(&c);
    let info = view.map_info.clone().expect("the map menu's info");
    let menu = view.menu.as_ref().unwrap();
    let size = framed_size(&c, menu, Some(&info), SCREEN);
    let (x, y) = menu_origin_in(SCREEN, cursor_cells(&s, &c), size);
    let buf = painted(&c, &view);
    assert_eq!(find(&buf, y, &info.turn), Some(x + 3));
    let bottom = row(&buf, y + size.1 - 1);
    assert!(bottom.contains(&format!(" {} ", info.charges)), "{bottom}");
    // Wide enough for both, a frame cell and a space either side.
    assert!(size.0 >= cells(&info.charges) + 6);
}

#[test]
fn the_fight_box_is_a_window_across_the_top_its_striker_the_bar() {
    let c = ctx();
    let fighter = |name: &str| BoxFighterView {
        name: name.into(),
        faction: trpg_core::Faction::Player,
        flashing: false,
        hp: 10,
        max: 20,
        result: None,
    };
    let mut view = FightBoxView {
        banner: Some("Guard Break".into()),
        hp_name: "HP".into(),
        sides: [fighter("Alda"), fighter("Brom")],
    };
    view.sides[0].flashing = true;
    view.sides[1].result = Some("HIT -8".into());
    let mut buf = blank();
    fight_box::paint(&c, &view, SCREEN, &mut buf);
    // 44 wide, centred across the top: from column 3.
    assert_eq!(fight_box::box_rect(SCREEN, 5), Rect::new(3, 0, 44, 5));
    assert_eq!(row(&buf, 0), format!("╔═ Guard Break {}╗", "═".repeat(28)));
    assert_eq!(cell(&buf, 3, 0).glyph, '╔');
    let [left, right] = fight_box::columns(fight_box::box_rect(SCREEN, 5));
    assert_eq!((left, right), (5, 27));
    assert!(is_bar(&buf, 1, "Alda"));
    assert_eq!(find(&buf, 1, "Brom"), Some(right));
    assert!(!is_bar(&buf, 1, "Brom"));
    // HP and a bar: 10 of 20 is half its 10 cells.
    assert_eq!(find(&buf, 2, "HP 10 █████     "), Some(left));
    assert_eq!(find(&buf, 3, "HIT -8"), Some(right));
    assert_eq!(row(&buf, 4), format!("╚{}╝", "═".repeat(42)));
}
