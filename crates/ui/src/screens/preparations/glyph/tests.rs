//! Tests of where the Preparations screen puts what its view says, on the
//! debug Quick Battle (`preparations::tests`) and on views made by hand.

use super::*;
use crate::console::ConsoleSpec;
use crate::input::Action::{Confirm, CursorDown, CursorUp};
use crate::screen::{Screen, tests::ctx};
use crate::screens::preparations::tests::{screen, update};
use crate::screens::preparations::{Tab, TabView, UnitRowView};

/// A buffer full of a colour the screen doesn't use, to see what is left
/// unpainted.
fn stale(c: &Ctx) -> GlyphBuffer {
    let enemy = c.palette.get(UiColor::Enemy);
    GlyphBuffer::with_spec(ConsoleSpec::PC, Cell::new('x', enemy, enemy))
}

/// `view` as painted.
fn painted(c: &Ctx, view: &PreparationsView) -> GlyphBuffer {
    let mut buf = stale(c);
    paint(c, view, &mut buf);
    buf
}

/// Row `y` of `buf`.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).map_or(' ', |cell| cell.glyph))
        .collect()
}

/// The colours of the cell at (`x`, `y`): text, then background.
fn colours(buf: &GlyphBuffer, x: i32, y: i32) -> (crate::color::Rgb, crate::color::Rgb) {
    let cell = buf.get(x, y).expect("on the console");
    (cell.fg, cell.bg)
}

fn list<Row>(title: &str, rows: Vec<Row>, cursor: usize, state: ListState) -> ListView<Row> {
    ListView {
        title: title.to_owned(),
        rows,
        cursor,
        state,
    }
}

fn unit(name: &str, deployed: bool) -> UnitRowView {
    UnitRowView {
        name: name.to_owned(),
        deployed,
    }
}

fn item(name: &str, detail: &str, usable: bool) -> ItemRowView {
    ItemRowView {
        name: name.to_owned(),
        detail: detail.to_owned(),
        usable,
    }
}

/// A Loadouts tab made by hand: the second of three units chosen, the
/// cursor on its second slot's first stock row.
fn view() -> PreparationsView {
    let units = vec![unit("Ann", true), unit("Bob", true), unit("Cy", false)];
    let slots = vec![
        SlotRowView {
            label: "Weapon".to_owned(),
            item: Some("Iron Sword".to_owned()),
            wear: "20/25".to_owned(),
        },
        SlotRowView {
            label: "Armor".to_owned(),
            item: None,
            wear: String::new(),
        },
    ];
    let stock = vec![
        item("Chain Mail", "×1  Def +3", true),
        item("Iron Plate", "too heavy", false),
    ];
    PreparationsView {
        heading: "Preparations".to_owned(),
        tabs: vec![
            TabView {
                tab: Tab::Loadouts,
                label: "Loadouts".to_owned(),
                shown: true,
            },
            TabView {
                tab: Tab::Fight,
                label: "Fight!".to_owned(),
                shown: false,
            },
        ],
        on_tabs: false,
        body: PrepBody::Loadouts(LoadoutsView {
            units: list("Units", units, 1, ListState::Chosen),
            gear: Some(GearView {
                slots: list("Bob · Guard", slots, 1, ListState::Chosen),
                speed: Some("AS 5".to_owned()),
                benched_note: Some("Not in this battle".to_owned()),
                stock: list("Stock: armor", stock, 0, ListState::Focused),
            }),
        }),
        message: Some("No.".to_owned()),
        leave_question: None,
        help: "help".to_owned(),
    }
}

#[test]
fn every_cell_is_painted_on_every_tab() {
    let c = ctx();
    let mut view = view();
    let consumables = |n| {
        let row = ConsumableRowView {
            name: "Potion".to_owned(),
            count: n,
            effect: "Restore 10 HP".to_owned(),
        };
        list("Stock", vec![row], 0, ListState::Idle)
    };
    let bodies = [
        view.body.clone(),
        PrepBody::Pack(PackView {
            spare: consumables(4),
            pack: consumables(12),
        }),
        PrepBody::Options,
        PrepBody::Fight("Units 4".to_owned()),
    ];
    for body in bodies {
        view.body = body;
        let buf = painted(&c, &view);
        let left = (0..i32::from(ConsoleSpec::PC.rows))
            .flat_map(|y| (0..i32::from(ConsoleSpec::PC.cols)).map(move |x| (x, y)))
            .filter(|&(x, y)| buf.get(x, y).is_some_and(|cell| cell.glyph == 'x'))
            .count();
        assert_eq!(left, 0, "{left} stale cells on {:?}", view.body);
    }
}

#[test]
fn the_lists_sit_side_by_side_under_the_tabs() {
    let c = ctx();
    let buf = painted(&c, &view());
    assert_eq!(
        row(&buf, 1).trim_end(),
        "  Preparations                 Loadouts      Fight!"
    );
    let top = row(&buf, 3);
    assert!(top.starts_with("┌─ Units ──────┐┌─ Bob · Guard ─"), "{top}");
    assert!(top[top.find("┐┌─ Stock").unwrap_or(0)..].starts_with("┐┌─ Stock: armor ─"));
    // An empty slot is a dash; the columns line up.
    let first = row(&buf, 4);
    assert!(
        first.starts_with("│ Ann          ││ Weapon     Iron Sword      20/25 "),
        "{first}"
    );
    assert!(first.contains("││ Chain Mail      ×1  Def +3 "), "{first}");
    assert!(first.ends_with(" │"), "{first}");
    let second = row(&buf, 5);
    assert!(
        second.starts_with("│ Bob          ││ Armor      -          "),
        "{second}"
    );
    assert!(second.contains("││ Iron Plate      too heavy "), "{second}");
    // The speed is one row under the slots, the note two under it.
    assert!(row(&buf, 7).contains("││ AS 5 "));
    assert!(row(&buf, 9).contains("││ Not in this battle "));
    assert_eq!(row(&buf, 29).trim(), "No.");
    assert_eq!(row(&buf, 31).trim(), "help");
}

#[test]
fn a_pack_row_is_the_name_the_count_and_the_effect() {
    let row = ConsumableRowView {
        name: "Potion".to_owned(),
        count: 4,
        effect: "Restore 10 HP".to_owned(),
    };
    let text = "Potion          ×4   Restore 10 HP".to_owned();
    assert_eq!(consumable_row(&row), (text, true));
}

#[test]
fn the_cursors_row_is_a_bar_the_chosen_rows_are_lit_and_the_rest_dim_or_plain() {
    let c = ctx();
    let p = |u| c.palette.get(u);
    let (bg, bar) = (p(UiColor::PanelBg), p(UiColor::PanelBorderFocus));
    let view = view();
    let buf = painted(&c, &view);
    // Units: a plain row, the chosen one, one left out of the battle.
    assert_eq!(colours(&buf, 2, 4), (p(UiColor::Text), bg));
    assert_eq!(colours(&buf, 2, 5), (p(UiColor::TextHighlight), bg));
    assert_eq!(colours(&buf, 2, 6), (p(UiColor::TextDim), bg));
    // Slots: the chosen one.
    assert_eq!(colours(&buf, 18, 4), (p(UiColor::Text), bg));
    assert_eq!(colours(&buf, 18, 5), (p(UiColor::TextHighlight), bg));
    // Stock: the cursor's row is a bar from border to border; the item the
    // unit can't use is dim.
    for x in [53, 54, 98] {
        assert_eq!(colours(&buf, x, 4), (bg, bar), "column {x}");
    }
    assert_eq!(colours(&buf, 54, 5), (p(UiColor::TextDim), bg));
    // Only the list with the cursor has a lit border.
    assert_eq!(colours(&buf, 0, 3).0, p(UiColor::PanelBorder));
    assert_eq!(colours(&buf, 16, 3).0, p(UiColor::PanelBorder));
    assert_eq!(colours(&buf, 52, 3).0, p(UiColor::PanelBorderFocus));
    // The shown tab is lit; it is a bar while the cursor is on the tabs.
    let black = p(UiColor::Black);
    assert_eq!(colours(&buf, 31, 1), (p(UiColor::TextHighlight), black));
    assert_eq!(colours(&buf, 45, 1), (p(UiColor::Text), black));
    let on_tabs = PreparationsView {
        on_tabs: true,
        ..view
    };
    let buf = painted(&c, &on_tabs);
    assert_eq!(colours(&buf, 30, 1), (bg, bar));
    assert_eq!(colours(&buf, 45, 1), (p(UiColor::Text), black));
    // The bar wins over dim: the cursor on a unit left out of the battle.
    let mut benched = on_tabs;
    if let PrepBody::Loadouts(loadouts) = &mut benched.body {
        loadouts.units.cursor = 2;
        loadouts.units.state = ListState::Focused;
    }
    assert_eq!(colours(&painted(&c, &benched), 2, 6), (bg, bar));
}

#[test]
fn the_leave_question_takes_the_messages_place() {
    let c = ctx();
    let mut view = view();
    view.leave_question = Some("Leave?".to_owned());
    let buf = painted(&c, &view);
    assert_eq!(row(&buf, 29).trim(), "Leave?");
    let at = row(&buf, 29).find("Leave?").unwrap_or(0);
    let x = i32::try_from(at).unwrap_or(0);
    let black = c.palette.get(UiColor::Black);
    let lit = c.palette.get(UiColor::TextHighlight);
    assert_eq!(colours(&buf, x, 29), (lit, black));
    view.leave_question = None;
    view.message = None;
    assert_eq!(row(&painted(&c, &view), 29).trim(), "");
}

#[test]
fn a_window_moves_only_once_the_cursor_leaves_it() {
    // A window of 3 rows over 5.
    let starts: Vec<usize> = (0..5).map(|f| window_start(f, 5, 3)).collect();
    assert_eq!(starts, [0, 0, 0, 1, 2]);
    assert_eq!(window_start(1, 2, 3), 0);
    assert_eq!(window_start(0, 0, 3), 0);
}

/// Row `y` of the screen as drawn.
fn drawn_row(s: &crate::screens::PreparationsScreen, c: &Ctx, y: i32) -> String {
    let black = c.palette.get(UiColor::Black);
    let mut buf = GlyphBuffer::new(100, 32, Cell::new(' ', black, black));
    s.draw(c, &mut buf);
    row(&buf, y)
}

/// A list longer than its box scrolls to keep the cursor in view and never
/// draws over the box's border.
#[test]
fn a_long_list_scrolls_inside_its_box() {
    let mut c = ctx();
    let mut s = screen(&c);
    let scout = s.prep.bench[0].clone();
    for i in 0..30 {
        let mut extra = scout.clone();
        extra.name = format!("Extra {i}");
        s.prep.bench.push(extra);
    }
    // 4 deployed units, the scout and 30 more: Up from the first wraps to
    // the last.
    update(&mut s, &mut c, &[Confirm, CursorUp]);
    assert_eq!(s.unit().map(|u| u.name.as_str()), Some("Extra 29"));
    // The box spans rows 3..=27: 23 rows of names between its borders.
    assert!(
        drawn_row(&s, &c, 4).starts_with("│ Extra 7"),
        "{}",
        drawn_row(&s, &c, 4)
    );
    assert!(drawn_row(&s, &c, 26).starts_with("│ Extra 29"));
    assert!(drawn_row(&s, &c, 27).starts_with("└──────────────┘"));
    assert_eq!(drawn_row(&s, &c, 28).trim(), "");
    // Back at the top, the first rows show again.
    update(&mut s, &mut c, &[CursorDown]);
    assert!(drawn_row(&s, &c, 4).starts_with("│ Test Lord"));
    assert!(drawn_row(&s, &c, 26).starts_with("│ Extra 17"));
}
