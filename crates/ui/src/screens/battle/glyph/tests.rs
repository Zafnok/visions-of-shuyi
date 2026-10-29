//! The battle screen skin's tests: where things are painted and in which
//! colours. What is shown is tested on the view (`battle/tests.rs`,
//! `battle/view_tests.rs`).

use trpg_core::{Faction, Outcome, Phase};

use super::*;
use crate::color::tests::game_palette;
use crate::input::Action;
use crate::screen::Screen;
use crate::screen::tests::ctx;
use crate::screens::battle::BattleScreen;
use crate::screens::battle::tests::{knight_in_forest, lord_menu, quick, render, step};
use crate::screens::battle::view::{
    BannerView, MessageView, ObjectiveView, PanelUnitView, QuestionView, TerrainView,
};

/// The glyphs of row `y` from column `from`, `n` of them.
fn cells(buf: &GlyphBuffer, y: i32, from: i32, n: i32) -> String {
    (from..from + n)
        .map(|x| buf.get(x, y).map_or(' ', |c| c.glyph))
        .collect()
}

/// The text of row `y` of the side panel, inside its border, trimmed.
fn panel_row(buf: &GlyphBuffer, y: i32) -> String {
    cells(buf, y, SIDE_PANEL.x + 1, SIDE_PANEL.w - 2)
        .trim()
        .to_owned()
}

/// Row `y`, trimmed.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    cells(buf, y, 0, i32::from(buf.width())).trim().to_owned()
}

/// `view` of `s` changed by `change`, painted.
fn painted(s: &BattleScreen, change: impl FnOnce(&mut BattleView)) -> GlyphBuffer {
    let c = ctx();
    let mut view = s.view(&c);
    change(&mut view);
    let mut buf = render(s, &c);
    paint(&c, &view, &mut buf);
    buf
}

#[test]
fn menus_go_right_of_the_tile_unless_they_would_leave_the_view() {
    // A glyph tile: two cells wide, one high.
    let beside = |(x, y), size| menu_origin(Rect::new(x, y, 2, 1), size);
    // The first item (under the top border) level with the tile.
    assert_eq!(beside((10, 4), (10, 4)), (13, 3));
    // Too far right: left of the tile.
    assert_eq!(beside((60, 4), (10, 4)), (49, 3));
    assert_eq!(beside((57, 4), (10, 4)), (60, 3));
    // Too low or high: moved to fit; never off the left edge.
    assert_eq!(beside((10, 29), (10, 4)), (13, 26));
    // Too wide for either side: below the tile, or above it when there is
    // no room below, never over it.
    assert_eq!(beside((36, 15), (46, 4)), (24, 16));
    assert_eq!(beside((2, 0), (70, 4)), (0, 1));
    let area = MAP_VIEW;
    let bottom = area.y + area.h - 1;
    assert_eq!(beside((36, bottom), (46, 4)), (24, bottom - 4));
    // Room neither below nor above: today's place.
    assert_eq!(beside((2, 3), (70, area.h)), (0, 0));
    // A bigger tile (another skin's): right of all of it, level with
    // its top row.
    assert_eq!(menu_origin(Rect::new(10, 4, 4, 2), (10, 4)), (15, 3));
    assert_eq!(menu_origin(Rect::new(58, 4, 4, 2), (10, 4)), (47, 3));
}

#[test]
fn the_menu_is_painted_one_cell_right_of_its_units_tile() {
    let mut c = ctx();
    let s = lord_menu(&mut c);
    let buf = render(&s, &c);
    // The lord stands at (5, 5); its first item is level with its tile.
    assert_eq!(cells(&buf, 15, 33, 10), "┌────────┐");
    assert_eq!(cells(&buf, 16, 33, 10), "│ Attack │");
    assert_eq!(cells(&buf, 17, 33, 10), "│ Arts   │");
    assert_eq!(cells(&buf, 18, 33, 10), "│ Skill  │");
    assert_eq!(cells(&buf, 19, 33, 10), "│ Item   │");
    assert_eq!(cells(&buf, 20, 33, 10), "│ Equip  │");
    assert_eq!(cells(&buf, 21, 33, 10), "│ Wait   │");
    let p = &c.palette;
    assert_eq!(buf.get(35, 16).unwrap().fg, p.get(UiColor::TextDim));
    assert_eq!(
        buf.get(35, 21).unwrap().bg,
        p.get(UiColor::PanelBorderFocus)
    );
    // The panel shows the lord.
    assert_eq!(panel_row(&buf, 5), "Test Lord");
}

#[test]
fn a_menus_header_is_painted_in_its_top_border() {
    let mut c = ctx();
    let s = lord_menu(&mut c);
    let spent = |spent: bool| {
        painted(&s, |v| {
            let menu = v.menu.as_mut().unwrap();
            menu.header = Some(MenuHeader {
                name: "Iron Sword".to_owned(),
                uses: Some(super::super::view::UsesView {
                    text: "0/20".to_owned(),
                    spent,
                }),
            });
        })
    };
    let buf = spent(false);
    assert_eq!(cells(&buf, 15, 33, 18), "┌─ Iron Sword 0/20");
    let p = &c.palette;
    assert_eq!(buf.get(36, 15).unwrap().fg, p.get(UiColor::TextHighlight));
    assert_eq!(buf.get(47, 15).unwrap().fg, p.get(UiColor::Text));
    assert_eq!(spent(true).get(47, 15).unwrap().fg, p.get(UiColor::HpLow));
}

#[test]
fn the_panel_is_double_bordered_while_a_unit_is_selected() {
    let mut c = ctx();
    let mut s = quick();
    assert_eq!(render(&s, &c).get(SIDE_PANEL.x, 0).unwrap().glyph, '┌');
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(render(&s, &c).get(SIDE_PANEL.x, 0).unwrap().glyph, '╔');
}

#[test]
fn the_panel_paints_the_terrain_then_the_unit() {
    let c = ctx();
    let s = knight_in_forest();
    let buf = render(&s, &c);
    let rows: Vec<String> = (1..10).map(|y| panel_row(&buf, y)).collect();
    assert_eq!(rows[0], "Forest");
    assert_eq!(rows[1], "DEF +1  AVO +20");
    assert_eq!(rows[2], "");
    assert_eq!(rows[4], "Test Knight");
    assert_eq!(rows[5], "Guard  Lv 1");
    assert!(rows[6].starts_with("HP 9/20"), "{}", rows[6]);
    assert_eq!(rows[7], "Player");
    // At 9/20 of max HP: 5 of 10 cells, `hp_mid`.
    let bar = cells(&buf, UNIT_ROW + 2, HP_BAR_X, HP_BAR_CELLS);
    assert_eq!(bar, "█████░░░░░");
    let p = &c.palette;
    let at = |x, y| buf.get(x, y).unwrap().fg;
    assert_eq!(at(HP_BAR_X, UNIT_ROW + 2), p.get(UiColor::HpMid));
    assert_eq!(at(TEXT_X, TERRAIN_ROW), p.get(UiColor::TextHighlight));
    assert_eq!(at(TEXT_X, UNIT_ROW), p.get(UiColor::Player));
    assert_eq!(at(TEXT_X, UNIT_ROW + 3), p.get(UiColor::TextDim));
}

#[test]
fn the_panel_cuts_long_lines_and_names_an_enemy_in_its_colour() {
    let c = ctx();
    let p = game_palette();
    let mut buf = render(&quick(), &c);
    let panel = PanelView {
        selected: false,
        terrain: Some(TerrainView {
            name: "A".repeat(40),
            bonuses: "DEF +0  AVO +0".to_owned(),
            heals: Some("Heals 20% HP".to_owned()),
        }),
        unit: Some(PanelUnitView {
            name: "B".repeat(40),
            faction: Faction::Enemy,
            class_level: "Brigand  Lv 2".to_owned(),
            hp_name: "HP".to_owned(),
            hp: 20,
            max: 20,
            faction_name: "Enemy".to_owned(),
        }),
        gold: "Gold 0".to_owned(),
    };
    paint_panel(&c, &panel, &mut buf);
    assert_eq!(panel_row(&buf, TERRAIN_ROW), "A".repeat(TEXT_W));
    assert_eq!(panel_row(&buf, TERRAIN_ROW + 2), "Heals 20% HP");
    assert_eq!(panel_row(&buf, UNIT_ROW), "B".repeat(TEXT_W));
    assert_eq!(buf.get(TEXT_X, UNIT_ROW).unwrap().fg, p.get(UiColor::Enemy));
    // Full HP: a whole bar.
    let bar = cells(&buf, UNIT_ROW + 2, HP_BAR_X, HP_BAR_CELLS);
    assert_eq!(bar, "██████████");
}

#[test]
fn panel_text_fits_inside_the_border() {
    assert_eq!(
        TEXT_X + i32::try_from(TEXT_W).unwrap(),
        SIDE_PANEL.x + SIDE_PANEL.w - 2
    );
    // The bar ends well inside the border (column 99).
    assert_eq!(HP_BAR_X + HP_BAR_CELLS, 93);
}

#[test]
fn the_notes_box_keeps_off_the_noted_units_rows() {
    // A box of 8 rows is centred on rows 11..19 of the 30-row view.
    assert_eq!(centred_top(8), 11);
    assert_eq!(box_top(8, &[]), 11);
    assert_eq!(box_top(8, &[10, 19]), 11, "just above and below it");
    // Covered: the top (rows 1..9), unless a unit is there too; then
    // the bottom (rows 21..29).
    assert_eq!(box_top(8, &[11]), 1);
    assert_eq!(box_top(8, &[18, 9, 0]), 1);
    assert_eq!(box_top(8, &[18, 1]), 21);
    assert_eq!(box_top(8, &[11, 8]), 21);
    assert_eq!(box_top(8, &[15, 8, 20, 29]), 21);
    // Nowhere is clear: centred.
    assert_eq!(box_top(8, &[15, 8, 21]), 11);
    assert_eq!(box_top(8, &[15, 5, 28]), 11);
}

#[test]
fn banners_take_their_phases_colour_and_are_centred_on_the_map() {
    let kinds = [Phase::Player, Phase::Enemy, Phase::Other]
        .map(|phase| banner_color(BannerKind::Phase { phase, turn: 1 }));
    assert_eq!(kinds, [UiColor::Player, UiColor::Enemy, UiColor::Ally]);
    let won = BannerKind::Outcome(Outcome::Victory);
    assert_eq!(banner_color(won), UiColor::TextHighlight);
    let lost = BannerKind::Outcome(Outcome::Defeat);
    assert_eq!(banner_color(lost), UiColor::HpLow);
    let c = ctx();
    let buf = painted(&quick(), |v| {
        v.banner = Some(BannerView {
            kind: lost,
            title: "DEFEAT".to_owned(),
            subtitle: String::new(),
        });
    });
    // A 24 × 6 box in the middle of the 70 × 30 map view.
    let (x, y) = ((MAP_VIEW.w - BANNER_W) / 2, (MAP_VIEW.h - BANNER_H) / 2);
    assert_eq!((x, y), (23, 12));
    assert_eq!(buf.get(x, y).unwrap().glyph, '╔');
    assert_eq!(cells(&buf, y + 2, x, BANNER_W), "║        DEFEAT        ║");
    assert_eq!(
        buf.get(x + 9, y + 2).unwrap().fg,
        c.palette.get(UiColor::HpLow)
    );
}

#[test]
fn heal_numbers_are_high_and_burns_low() {
    assert_eq!(popup_color(PopupKind::Heal), UiColor::HpHigh);
    assert_eq!(popup_color(PopupKind::Burn), UiColor::HpLow);
}

#[test]
fn a_question_is_a_box_in_the_middle_of_the_map_its_answers_dim() {
    let c = ctx();
    let buf = painted(&quick(), |v| {
        v.dialog = Some(DialogView::Question(QuestionView {
            question: "End the turn?".to_owned(),
            answers: "yes / no".to_owned(),
        }));
    });
    // 13 cells of text plus a blank and a border each side; 2 lines plus
    // two blank rows and the borders: centred on the map view.
    let (width, height) = (17, 6);
    let (left, top) = ((MAP_VIEW.w - width) / 2, (MAP_VIEW.h - height) / 2);
    let border = format!("╔{}╗", "═".repeat(15));
    assert_eq!(cells(&buf, top, left, width), border);
    assert_eq!(cells(&buf, top + 2, left, width), "║ End the turn? ║");
    let fg = |row| buf.get(left + 2, row).unwrap().fg;
    assert_eq!(fg(top + 2), c.palette.get(UiColor::Text));
    assert_eq!(fg(top + 3), c.palette.get(UiColor::TextDim));
}

#[test]
fn the_objective_page_has_its_title_in_the_border_and_a_dim_notes_heading() {
    let c = ctx();
    let buf = painted(&quick(), |v| {
        v.dialog = Some(DialogView::Objective(ObjectiveView {
            title: "OBJECTIVE".to_owned(),
            objective: "Rout the enemy".to_owned(),
            village: None,
            turn: "Turn 1".to_owned(),
            notes: Some(NotesView {
                title: "Notes".to_owned(),
                lines: vec!["• A note.".to_owned()],
            }),
        }));
    });
    let top = (0..MAP_VIEW.h)
        .find(|&y| row(&buf, y).contains(" OBJECTIVE "))
        .unwrap();
    let rows: Vec<String> = (top + 2..top + 7).map(|y| row(&buf, y)).collect();
    assert!(rows[0].contains("║ Rout the enemy"), "{}", rows[0]);
    assert!(rows[1].contains("║ Turn 1"), "{}", rows[1]);
    assert!(rows[3].contains("║ Notes"), "{}", rows[3]);
    assert!(rows[4].contains("║ • A note."), "{}", rows[4]);
    let x = (0..MAP_VIEW.w)
        .find(|&x| buf.get(x, top + 5).unwrap().glyph == 'N')
        .unwrap();
    let p = &c.palette;
    assert_eq!(buf.get(x, top + 5).unwrap().fg, p.get(UiColor::TextDim));
    assert_eq!(buf.get(x, top).unwrap().fg, p.get(UiColor::TextHighlight));
}

#[test]
fn the_bar_has_the_message_left_the_toggles_right_and_the_help_under() {
    let c = ctx();
    let message = |kind| {
        painted(&quick(), |v| {
            v.status = "zoom · danger".to_owned();
            v.message = Some(MessageView {
                text: "Said".to_owned(),
                kind,
            });
            v.help = "the help".to_owned();
            v.debug_hint = Some("F2 debug".to_owned());
        })
    };
    let buf = message(MessageKind::Said);
    let bar = cells(&buf, HELP_BAR.y, 0, HELP_BAR.w);
    assert!(bar.starts_with(" Said "), "{bar}");
    assert!(bar.ends_with(" zoom · danger "), "{bar}");
    let help = cells(&buf, HELP_ROW, 0, HELP_BAR.w);
    assert!(help.starts_with(" the help "), "{help}");
    assert!(help.ends_with(" F2 debug "), "{help}");
    let p = &c.palette;
    assert_eq!(buf.get(1, HELP_BAR.y).unwrap().fg, p.get(UiColor::Text));
    assert_eq!(buf.get(1, HELP_ROW).unwrap().fg, p.get(UiColor::TextDim));
    let toast = message(MessageKind::Toast);
    let lit = p.get(UiColor::TextHighlight);
    assert_eq!(toast.get(1, HELP_BAR.y).unwrap().fg, lit);
}

#[test]
fn the_screens_draw_paints_its_view() {
    let c = ctx();
    let s = knight_in_forest();
    let mut drawn = render(&s, &c);
    s.draw(&c, &mut drawn);
    let mut buf = render(&s, &c);
    paint(&c, &s.view(&c), &mut buf);
    assert_eq!(drawn.to_snapshot(&c.palette), buf.to_snapshot(&c.palette));
}
