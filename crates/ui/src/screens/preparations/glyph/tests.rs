//! Tests of where the Preparations screen puts what its view says, on the
//! debug Quick Battle (`preparations::tests`) and on views made by hand.

use super::*;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::input::Action::{Confirm, CursorDown, CursorUp};
use crate::screen::{Screen, tests::ctx};
use crate::screens::preparations::tests::{screen, update};
use crate::screens::preparations::view::HolderView;
use crate::screens::preparations::{Tab, TabView, UnitRowView};
use crate::widgets::AskView;

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
    // A window as a fresh cursor on `cursor` shows it.
    let first = crate::widgets::ListCursor::new(rows.len())
        .at(cursor)
        .window(LIST_ROWS)
        .start;
    ListView {
        title: title.to_owned(),
        rows,
        first,
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
        holder: None,
    }
}

/// A row of `label`'s vest, with the face `portrait` if any.
fn swap(label: &str, portrait: Option<&str>) -> ItemRowView {
    let holder = HolderView {
        character: None,
        portrait: portrait.map(str::to_owned),
        label: label.to_owned(),
    };
    ItemRowView {
        holder: Some(holder),
        ..item("Leather Vest", "from Them", true)
    }
}

/// [`view`] with `rows` in the stock, the cursor on row `cursor`.
fn with_stock(rows: Vec<ItemRowView>, cursor: usize) -> PreparationsView {
    let mut v = view();
    if let PrepBody::Loadouts(LoadoutsView {
        gear: Some(gear), ..
    }) = &mut v.body
    {
        gear.stock = list("Stock: armor", rows, cursor, ListState::Focused);
    }
    v
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
                item_detail: Some("E  Mt 12  Hit 75  Crit 10  Rng 1-2  Wt 12".to_owned()),
                speed: Some("AS 5".to_owned()),
                benched_note: Some("Not in this battle".to_owned()),
                stock: list("Stock: armor", stock, 0, ListState::Focused),
            }),
        }),
        message: Some("No.".to_owned()),
        leave_question: None,
        help: "help".into(),
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
    // The highlighted item's numbers are one row under the slots, cut at
    // a gap to fit the box; the speed under them, the note two under it.
    assert!(row(&buf, 7).contains("││ E  Mt 12  Hit 75  Crit 10 "));
    assert!(row(&buf, 8).contains("││ Rng 1-2  Wt 12 "));
    assert!(row(&buf, 9).contains("││ AS 5 "));
    assert!(row(&buf, 11).contains("││ Not in this battle "));
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
    view.leave_question = Some(AskView::new("Leave?".to_owned(), Vec::new()));
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

/// A swap row with a face is three rows tall: the face at its left, its
/// text on its middle row (0479).
#[test]
fn a_swap_row_starts_with_the_holders_face() {
    let c = ctx();
    let rows = vec![
        item("Chain Mail", "×1  Def +3", true),
        swap("Lo", Some("test_lord")),
        swap("Sc", None),
    ];
    let buf = painted(&c, &with_stock(rows, 0));
    let (cw, ch) = buf.spec().cell_px();
    let at = (STOCK_BOX.x + 2, STOCK_BOX.y + 2);
    let faces: Vec<_> = buf.sprites().iter().map(|s| (s.dest, s.layer)).collect();
    let dest = PxRect::new(at.0 * cw, at.1 * ch, FACE_PX, FACE_PX);
    assert_eq!(faces, [(dest, Layer::Under)]);
    let skip = usize::try_from(STOCK_BOX.x).unwrap();
    let text = |y| row(&buf, y).chars().skip(skip).collect::<String>();
    assert_eq!(text(5).trim_matches('│').trim(), "");
    assert!(text(6).contains("│        Leather Vest"), "{}", text(6));
    // The holder with no face: its map letters on one row, in the
    // player's colour.
    assert!(text(8).starts_with("│ Sc Leather Vest"), "{}", text(8));
    let player = c.palette.get(UiColor::Player);
    assert_eq!(colours(&buf, STOCK_BOX.x + 2, 8).0, player);
    assert_eq!(text(9).trim_matches('│').trim(), "");
}

/// On the cursor's bar the letters take the bar's text colour.
#[test]
fn the_letters_on_the_bar_are_the_bars_colour() {
    let c = ctx();
    let buf = painted(&c, &with_stock(vec![swap("Sc", None)], 0));
    let (fg, bg) = colours(&buf, STOCK_BOX.x + 2, STOCK_BOX.y + 1);
    assert_eq!((fg, bg), colours(&buf, STOCK_BOX.x + 6, STOCK_BOX.y + 1));
    assert_eq!(bg, c.palette.get(UiColor::PanelBorderFocus));
}

/// Rows hidden below show `▼` at the box's bottom right, rows hidden
/// above `▲` at its top right (Nick, 0479).
#[test]
fn arrows_show_rows_hidden_above_and_below() {
    let c = ctx();
    let faces = || (0..10).map(|_| swap("Lo", Some("test_lord"))).collect();
    let mark = |buf: &GlyphBuffer, y| buf.get(STOCK_BOX.x + STOCK_BOX.w - 2, y).map(|c| c.glyph);
    let (top, bottom) = (STOCK_BOX.y + 1, STOCK_BOX.y + STOCK_BOX.h - 2);
    // Ten rows of three in 23: the first seven show.
    let first = painted(&c, &with_stock(faces(), 0));
    assert_eq!(
        (mark(&first, top), mark(&first, bottom)),
        (Some(' '), Some('▼'))
    );
    let middle = painted(&c, &with_stock(faces(), 8));
    assert_eq!(
        (mark(&middle, top), mark(&middle, bottom)),
        (Some('▲'), Some('▼'))
    );
    let last = painted(&c, &with_stock(faces(), 9));
    assert_eq!(
        (mark(&last, top), mark(&last, bottom)),
        (Some('▲'), Some(' '))
    );
    // A short list has neither.
    let short = painted(&c, &with_stock(vec![swap("Sc", None)], 0));
    assert_eq!(
        (mark(&short, top), mark(&short, bottom)),
        (Some(' '), Some(' '))
    );
}

/// Rows that fill the box exactly all show, the last on its bottom row,
/// with no arrow.
#[test]
fn rows_that_just_fit_all_show() {
    let c = ctx();
    let inner = STOCK_BOX.h - 2;
    let rows = (0..inner)
        .map(|i| item(&format!("Row {i}"), "", true))
        .collect();
    let buf = painted(&c, &with_stock(rows, 0));
    let bottom = STOCK_BOX.y + inner;
    let skip = usize::try_from(STOCK_BOX.x).unwrap();
    let last: String = row(&buf, bottom).chars().skip(skip).collect();
    assert!(last.starts_with("│ Row 22 "), "{last}");
    let mark = buf
        .get(STOCK_BOX.x + STOCK_BOX.w - 2, bottom)
        .map(|c| c.glyph);
    assert_eq!(mark, Some(' '));
}

#[test]
fn the_stock_window_moves_only_once_the_cursor_leaves_it() {
    let heights = [1, 3, 3, 3];
    let starts: Vec<usize> = (0..4).map(|f| stock_window_start(&heights, f, 5)).collect();
    assert_eq!(starts, [0, 0, 2, 3]);
    assert_eq!(stock_window_start(&[], 0, 5), 0);
    // A row taller than the box still shows from its top.
    assert_eq!(stock_window_start(&[1, 9], 1, 5), 1);
}

#[test]
fn a_face_is_drawn_48_pixels_keeping_its_shape() {
    assert_eq!(face_size((64, 64)), (48, 48));
    assert_eq!(face_size((32, 32)), (48, 48));
    assert_eq!(face_size((64, 32)), (48, 24));
}
