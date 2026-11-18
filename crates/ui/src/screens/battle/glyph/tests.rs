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
    BannerView, MenuHeader, MessageView, ObjectiveLine, ObjectiveView, PanelCellView,
    PanelPreviewView, PanelUnitView, QuestionView, TerrainView, TogglesView, UsesView,
};

const NARROW: shuyi_assets::font::CellWidths = shuyi_assets::font::CellWidths::NARROW;

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

/// A menu that keeps clear of an area skill's reach goes beside the whole
/// reach (ticket 0492): right of it, or left of it at the view's right
/// edge.
#[test]
fn a_menu_goes_beside_the_reach_it_keeps_clear_of() {
    // Inspire's radius-2 diamond around the glyph tile at (x, 10).
    let reach = |x: i32| {
        let near = (-2..=2_i32).flat_map(|dx| (-2..=2_i32).map(move |dy| (dx, dy)));
        near.filter(|&(dx, dy)| dx.abs() + dy.abs() <= 2 && (dx, dy) != (0, 0))
            .map(move |(dx, dy)| Rect::new(x + 2 * dx, 10 + dy, 2, 1))
    };
    let tile = Rect::new(20, 10, 2, 1);
    let anchor = beside_all(tile, reach(20));
    assert_eq!(anchor, Rect::new(16, 8, 10, 5));
    // Right of the reach, its first item level with the reach's top row.
    assert_eq!(menu_origin(anchor, (10, 4)), (27, 7));
    // No reach: beside the tile, as every menu.
    assert_eq!(beside_all(tile, []), tile);
    // At the view's right edge (the map's): left of the reach.
    let edge = beside_all(Rect::new(64, 10, 2, 1), reach(64));
    assert_eq!(edge, Rect::new(60, 8, 10, 5));
    assert_eq!(menu_origin(edge, (10, 4)), (49, 7));
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
            ..ellery_panel(true)
        }),
        gold: "Gold 0".to_owned(),
        preview: None,
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

/// Ellery on Chapter 1's start, as `look-and-feel.md` *Side panel* shows
/// her; `armed`: her Iron Sword in hand, or nothing.
fn ellery_panel(armed: bool) -> PanelUnitView {
    let cells = |pairs: &[(&str, &str)]| -> Vec<PanelCellView> {
        pairs
            .iter()
            .map(|&(name, value)| PanelCellView {
                name: name.to_owned(),
                value: value.to_owned(),
                changed: false,
            })
            .collect()
    };
    let (atk, hit, crit) = if armed {
        ("13", "104", "3")
    } else {
        ("--", "--", "--")
    };
    PanelUnitView {
        name: "Ellery".to_owned(),
        faction: Faction::Player,
        class_level: "Exile  Lv 1".to_owned(),
        hp_name: "HP".to_owned(),
        hp: 19,
        max: 19,
        hp_then: None,
        faction_name: "Player".to_owned(),
        stats: cells(&[
            ("Str", "8"),
            ("Mag", "1"),
            ("Dex", "7"),
            ("Spd", "7"),
            ("Def", "6"),
            ("Res", "5"),
            ("Mov", "5"),
        ]),
        weapon: armed.then(|| MenuHeader {
            name: "Iron Sword".to_owned(),
            uses: Some(UsesView {
                text: "20/20".to_owned(),
                spent: false,
            }),
        }),
        no_weapon: "--".to_owned(),
        combat: cells(&[
            ("Atk", atk),
            ("Hit", hit),
            ("Crit", crit),
            ("Avo", "14"),
            ("AS", "5"),
        ]),
    }
}

/// The side panel painted over `quick` with `unit` under the cursor.
fn panel_with(unit: PanelUnitView) -> GlyphBuffer {
    panel_aimed(unit, None)
}

/// [`panel_with`], with `preview` of what is aimed at the top (1286).
fn panel_aimed(unit: PanelUnitView, preview: Option<PanelPreviewView>) -> GlyphBuffer {
    let c = ctx();
    let mut buf = render(&quick(), &c);
    let panel = PanelView {
        selected: false,
        terrain: Some(TerrainView {
            name: "Road".to_owned(),
            bonuses: "DEF +0  AVO +0".to_owned(),
            heals: None,
        }),
        unit: Some(unit),
        gold: "Gold 0".to_owned(),
        preview,
    };
    paint_panel(&c, &panel, &mut buf);
    buf
}

/// The side panel's rows, from the terrain to the gold, trimmed.
fn panel_text(buf: &GlyphBuffer) -> String {
    (TERRAIN_ROW..=GOLD_ROW)
        .map(|y| panel_row(buf, y))
        .collect::<Vec<_>>()
        .join(
            "
",
        )
        .trim_end()
        .to_owned()
}

#[test]
fn the_panel_paints_the_stats_weapon_and_combat_numbers_as_designed() {
    let buf = panel_with(ellery_panel(true));
    let rows: Vec<_> = (STATS_ROW..STATS_ROW + 9)
        .map(|y| panel_row(&buf, y))
        .collect();
    // `look-and-feel.md` *Side panel*, line for line.
    assert_eq!(
        rows,
        [
            "Str   8     Mag   1",
            "Dex   7     Spd   7",
            "Def   6     Res   5",
            "Mov   5",
            "",
            "Iron Sword        20/20",
            "Atk  13     Hit 104",
            "Crit  3     Avo  14",
            "AS    5",
        ]
    );
    assert_eq!(panel_row(&buf, UNIT_ROW + 4), "");
    // Names dim, numbers lit.
    let p = game_palette();
    let fg = |x: i32, y: i32| buf.get(x, y).unwrap().fg;
    assert_eq!(fg(TEXT_X, STATS_ROW), p.get(UiColor::TextDim));
    assert_eq!(fg(TEXT_X + 6, STATS_ROW), p.get(UiColor::Text));
    assert_eq!(
        fg(TEXT_X + PANEL_CELL_GAP, STATS_ROW + 6),
        p.get(UiColor::TextDim)
    );
    assert_eq!(
        fg(TEXT_X + PANEL_CELL_GAP + 6, STATS_ROW + 6),
        p.get(UiColor::Text)
    );
}

#[test]
fn a_long_weapon_name_is_cut_a_space_before_its_uses() {
    let mut unit = ellery_panel(true);
    if let Some(w) = unit.weapon.as_mut() {
        w.name = "W".repeat(30);
    }
    let buf = panel_with(unit);
    // 23 columns: 17 of the name, a space, `20/20`.
    assert_eq!(
        panel_row(&buf, STATS_ROW + 5),
        format!("{} 20/20", "W".repeat(17))
    );
}

#[test]
fn a_raised_stat_and_spent_uses_have_their_colours() {
    let mut unit = ellery_panel(true);
    unit.stats[4].changed = true;
    if let Some(uses) = unit.weapon.as_mut().and_then(|w| w.uses.as_mut()) {
        uses.text = "0/20".to_owned();
        uses.spent = true;
    }
    let buf = panel_with(unit);
    let p = game_palette();
    let fg = |x: i32, y: i32| buf.get(x, y).unwrap().fg;
    assert_eq!(fg(TEXT_X + 6, STATS_ROW + 2), p.get(UiColor::Effect));
    assert_eq!(fg(TEXT_X + 22, STATS_ROW + 5), p.get(UiColor::HpLow));
}

#[test]
fn panel_snapshots_an_ally_an_enemy_and_a_unit_with_nothing_in_hand() {
    let brigand = PanelUnitView {
        name: "Brigand".to_owned(),
        faction: Faction::Enemy,
        class_level: "Brigand  Lv 1".to_owned(),
        hp: 20,
        max: 20,
        faction_name: "Enemy".to_owned(),
        weapon: Some(MenuHeader {
            name: "Iron Axe".to_owned(),
            uses: Some(UsesView {
                text: "20/20".to_owned(),
                spent: false,
            }),
        }),
        combat: ellery_panel(true)
            .combat
            .into_iter()
            .zip(["14", "79", "1", "8", "-1"])
            .map(|(cell, value)| PanelCellView {
                value: value.to_owned(),
                ..cell
            })
            .collect(),
        ..ellery_panel(true)
    };
    insta::assert_snapshot!("panel_ally", panel_text(&panel_with(ellery_panel(true))));
    insta::assert_snapshot!("panel_enemy", panel_text(&panel_with(brigand)));
    insta::assert_snapshot!(
        "panel_unarmed",
        panel_text(&panel_with(ellery_panel(false)))
    );
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
            lines: Vec::new(),
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
            lines: vec![ObjectiveLine {
                label: "Win:".to_owned(),
                text: "Rout the enemy".to_owned(),
            }],
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
    assert!(rows[0].contains("║ Win: Rout the enemy"), "{}", rows[0]);
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

/// The quick battle's view painted with `toggles`, `message`, the help
/// `the help` and the debug hint.
fn bar(toggles: Option<TogglesView>, message: Option<MessageView>) -> GlyphBuffer {
    painted(&quick(), |v| {
        v.toggles = toggles;
        v.message = message;
        v.help = "the help".into();
        v.debug_hint = Some("F2 debug".to_owned());
    })
}

#[test]
fn the_toggles_row_has_danger_zone_at_the_left_and_auto_end_in_its_column() {
    let c = ctx();
    let toggles = TogglesView::new("zoom · danger".into(), "auto".into(), 30, &NARROW);
    let buf = bar(Some(toggles), None);
    let row = cells(&buf, HELP_BAR.y, 0, HELP_BAR.w);
    assert_eq!(row.find("zoom · danger"), Some(1), "{row}");
    let auto = row.find("auto").map(|at| row[..at].chars().count());
    assert_eq!(auto, Some(31), "{row}");
    let help = cells(&buf, HELP_ROW, 0, HELP_BAR.w);
    assert!(help.starts_with(" the help "), "{help}");
    assert!(help.ends_with(" F2 debug "), "{help}");
    let p = &c.palette;
    assert_eq!(buf.get(1, HELP_BAR.y).unwrap().fg, p.get(UiColor::TextDim));
    assert_eq!(buf.get(31, HELP_BAR.y).unwrap().fg, p.get(UiColor::TextDim));
    assert_eq!(buf.get(1, HELP_ROW).unwrap().fg, p.get(UiColor::TextDim));
}

#[test]
fn a_preview_line_is_at_the_left_of_the_toggles_row() {
    let c = ctx();
    let said = MessageView {
        text: "Forest → Burning".to_owned(),
        kind: MessageKind::Said,
    };
    let buf = bar(None, Some(said));
    let row = cells(&buf, HELP_BAR.y, 0, HELP_BAR.w);
    assert_eq!(row.trim_end(), " Forest → Burning");
    assert_eq!(
        buf.get(1, HELP_BAR.y).unwrap().fg,
        c.palette.get(UiColor::Text)
    );
    // Nothing over the map's bottom row.
    let plain = bar(None, None);
    let above = HELP_BAR.y - 1;
    assert_eq!(cells(&buf, above, 0, 70), cells(&plain, above, 0, 70));
}

#[test]
fn a_note_is_a_strip_over_the_maps_bottom_row() {
    let c = ctx();
    let toggles = TogglesView::new("danger".into(), "auto".into(), 30, &NARROW);
    let note = MessageView::note("Auto-end: ON".to_owned());
    let buf = bar(Some(toggles), Some(note));
    let strip = cells(&buf, HELP_BAR.y - 1, MAP_VIEW.x, MAP_VIEW.w);
    assert_eq!(strip.trim_end(), " Auto-end: ON");
    let p = &c.palette;
    for x in [MAP_VIEW.x, MAP_VIEW.x + MAP_VIEW.w - 1] {
        assert_eq!(
            buf.get(x, HELP_BAR.y - 1).unwrap().bg,
            p.get(UiColor::PanelBg)
        );
    }
    assert_eq!(
        buf.get(1, HELP_BAR.y - 1).unwrap().fg,
        p.get(UiColor::TextHighlight)
    );
    // The toggles stay where they are.
    let row = cells(&buf, HELP_BAR.y, 0, HELP_BAR.w);
    assert!(row.starts_with(" danger "), "{row}");
}

#[test]
fn a_long_warning_wraps_in_a_box_inside_the_map() {
    let text = "Save storage error: localStorage is turned off or blocked in this browser";
    let buf = bar(None, Some(MessageView::warning(text.to_owned())));
    // The box's bottom border is on the map's bottom row, its left on the
    // map's left edge, and nothing passes the map's right edge.
    let bottom = HELP_BAR.y - 1;
    assert_eq!(buf.get(MAP_VIEW.x, bottom).unwrap().glyph, '└');
    let rows: Vec<String> = (bottom - 3..=bottom)
        .map(|y| cells(&buf, y, MAP_VIEW.x, MAP_VIEW.w))
        .collect();
    assert!(rows[0].starts_with('┌'), "{rows:?}");
    let words: String = rows[1..3]
        .iter()
        .map(|r| r.trim_matches(|ch| ch == '│' || ch == ' '))
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(words.split_whitespace().collect::<Vec<_>>().join(" "), text);
    assert!(rows[0].trim_end().ends_with('┐'), "{rows:?}");
    assert!(rows[1].trim_end().ends_with('│'), "{rows:?}");
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

/// The objective banner (0481) with `lines`, each `(label, text)`.
fn objective_banner(lines: &[(&str, &str)]) -> BannerView {
    BannerView {
        kind: BannerKind::Objective,
        title: "OBJECTIVE".to_owned(),
        subtitle: String::new(),
        lines: lines
            .iter()
            .map(|&(label, text)| ObjectiveLine {
                label: label.to_owned(),
                text: text.to_owned(),
            })
            .collect(),
    }
}

/// A battle like Chapter 1's (win, and the lead's fall) and one like
/// Chapter 2's (its village too, and the bonus).
pub(crate) const OBJECTIVES: [&[(&str, &str)]; 2] = [
    &[("Win:", "Defeat Harl Coster"), ("Lose:", "Ellery falls")],
    &[
        ("Win:", "Defeat Silas Hendry"),
        ("Lose:", "3 of 5 houses burn"),
        ("or", "Ellery falls"),
        ("Bonus:", "Save 4 or 5 houses for rewards"),
    ],
];

/// Acceptance (0481): the objective banner, its labels in a column in the
/// highlight colour, its texts lined up after them.
#[test]
fn the_objective_banner_snapshot() {
    let ctx = ctx();
    for (i, lines) in OBJECTIVES.into_iter().enumerate() {
        let buf = painted(&quick(), |v| v.banner = Some(objective_banner(lines)));
        insta::assert_snapshot!(
            format!("objective_banner_{i}"),
            crate::harness::snapshot_of(&buf, MAP_VIEW, &ctx.palette)
        );
        let (width, height) = objective_banner_size("OBJECTIVE", &objective_banner(lines).lines);
        let (left, top) = (
            MAP_VIEW.x + (MAP_VIEW.w - width) / 2,
            MAP_VIEW.y + (MAP_VIEW.h - height) / 2,
        );
        assert_eq!(buf.get(left, top).unwrap().glyph, '╔');
        let palette = &ctx.palette;
        assert_eq!(
            buf.get(left + 3, top + 4).unwrap().fg,
            palette.get(UiColor::TextHighlight)
        );
        assert_eq!(
            buf.get(left + 3, top + 5).unwrap().fg,
            palette.get(UiColor::TextHighlight)
        );
        assert_eq!(
            buf.get(left + 9, top + 4).unwrap().fg,
            palette.get(UiColor::Text)
        );
        assert_eq!(cells(&buf, top + 5, left + 3, 6), "Lose: ");
    }
}

#[test]
fn a_short_warning_is_a_box_of_three_rows() {
    let buf = bar(None, Some(MessageView::warning("No room".to_owned())));
    let bottom = HELP_BAR.y - 1;
    assert_eq!(buf.get(MAP_VIEW.x, bottom - 2).unwrap().glyph, '┌');
    assert_eq!(cells(&buf, bottom - 1, MAP_VIEW.x, 11), "│ No room │");
    assert_eq!(buf.get(MAP_VIEW.x, bottom).unwrap().glyph, '└');
}

#[test]
fn a_note_wider_than_the_map_wraps_onto_a_second_row_above() {
    // 69 characters: one more than the 68 a row of the map holds.
    let text = format!("{} {}", "a".repeat(34), "b".repeat(34));
    let buf = bar(None, Some(MessageView::note(text)));
    let row = |y| cells(&buf, y, MAP_VIEW.x, MAP_VIEW.w);
    assert_eq!(
        row(HELP_BAR.y - 2).trim_end(),
        format!(" {}", "a".repeat(34))
    );
    assert_eq!(
        row(HELP_BAR.y - 1).trim_end(),
        format!(" {}", "b".repeat(34))
    );
    // A note of 68 fits on one row.
    let text = format!("{} {}", "a".repeat(34), "b".repeat(33));
    let buf = bar(None, Some(MessageView::note(text.clone())));
    let one = cells(&buf, HELP_BAR.y - 1, MAP_VIEW.x, MAP_VIEW.w);
    assert_eq!(one.trim_end(), format!(" {text}"));
}

#[test]
fn a_long_aimed_preview_wraps_inside_the_panel_over_the_tile_and_unit() {
    let c = ctx();
    let text = "Shove on Brigand Captain of the Northern Hills (8 → 7 uses, 20 → 19)";
    let preview = PanelPreviewView {
        title: "Shove".to_owned(),
        text: text.to_owned(),
    };
    let buf = panel_aimed(ellery_panel(true), Some(preview));
    let lines = wrap_words(text, TEXT_W);
    assert!(lines.len() > 2, "{lines:?}");
    // Every line inside the text width, the words all there in order.
    let rows: Vec<String> = (0..lines.len())
        .map(|i| panel_row(&buf, TERRAIN_ROW + 1 + to_i32(i)))
        .collect();
    assert!(rows.iter().all(|r| r.chars().count() <= TEXT_W), "{rows:?}");
    assert_eq!(rows.join(" "), text);
    // Nothing reaches the border or the blank column before it.
    let border = c.palette.get(UiColor::PanelBorder);
    for y in SIDE_PANEL.y + 1..SIDE_PANEL.y + SIDE_PANEL.h - 1 {
        let gap = cells(&buf, y, TEXT_X + to_i32(TEXT_W), 1);
        assert_eq!(gap, " ", "row {y}");
        let edge = buf.get(SIDE_PANEL.x + SIDE_PANEL.w - 1, y).unwrap();
        assert_eq!(edge.fg, border, "row {y}");
    }
    insta::assert_snapshot!("panel_long_preview", panel_text(&buf));
}

#[test]
fn a_heal_aimed_at_the_unit_shows_the_change_on_its_hp_line() {
    let p = game_palette();
    let hurt = PanelUnitView {
        hp: 6,
        max: 19,
        hp_then: Some(16),
        ..ellery_panel(true)
    };
    let buf = panel_with(hurt);
    let hp_row = UNIT_ROW + 2;
    assert!(panel_row(&buf, hp_row).starts_with("HP 6 → 16/19"));
    let lit = buf.get(TEXT_X, hp_row).unwrap().fg;
    assert_eq!(lit, p.get(UiColor::TextHighlight));
    // The bar after the line, showing the HP the heal leaves, inside the
    // panel's text width.
    let bar_x = TEXT_X + to_i32("HP 6 → 16/19 ".chars().count());
    // 16 of 19 HP: 8 of its 10 cells full, the rest drained.
    let bar = cells(&buf, hp_row, bar_x, HP_BAR_CELLS);
    assert!(bar.starts_with("████████"), "{bar}");
    assert!(!bar.ends_with('█'), "{bar}");
    let after = cells(&buf, hp_row, bar_x + HP_BAR_CELLS, 3);
    assert_eq!(after, "   ");
    assert!(bar_x + HP_BAR_CELLS <= TEXT_X + to_i32(TEXT_W));
}

#[test]
fn a_long_hp_change_line_keeps_its_bar_inside_the_panel() {
    // `HP 999 → 999/999` leaves 9 cells for the bar, not 10.
    let big = PanelUnitView {
        hp: 999,
        max: 999,
        hp_then: Some(999),
        ..ellery_panel(true)
    };
    let buf = panel_with(big);
    let hp_row = UNIT_ROW + 2;
    let line = "HP 999 → 999/999";
    assert_eq!(panel_row(&buf, hp_row), format!("{line} {}", "█".repeat(9)));
    // Not one cell past the text width; the blank column before the
    // border stays blank.
    let edge = TEXT_X + to_i32(TEXT_W);
    assert_eq!(cells(&buf, hp_row, edge - 9, 9), "█".repeat(9));
    assert_eq!(cells(&buf, hp_row, edge, 1), " ");
    // A name so long no cell is left: no bar, and nothing past the line.
    let full = PanelUnitView {
        hp_name: "Hit Points".to_owned(),
        hp: 999,
        max: 9999,
        hp_then: Some(999),
        ..ellery_panel(true)
    };
    let buf = panel_with(full);
    assert_eq!(panel_row(&buf, hp_row), "Hit Points 999 → 999/9999");
    assert_eq!(cells(&buf, hp_row, edge, 1), " ");
}
