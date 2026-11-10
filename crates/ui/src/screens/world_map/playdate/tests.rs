//! The world map's one-bit look (ticket 1066), on the test world map
//! (`assets-test/worldmaps/test_world.ron`, 70 × 30 cells) in the
//! Playdate's palette, and its menu's shortcuts on a platform with no
//! keyboard.

use insta::assert_snapshot;
use trpg_content::{Ink, battle_campaign};
use trpg_core::{Campaign, GameMode, LeadGender, LeadProfile};

use super::super::{MenuEntry, WorldMapScreen};
use super::*;
use crate::glyph_buffer::Overlay;
use crate::input::Action;
use crate::screen::{FrameInput, Keyboard, Screen, ShuyiCtx, Transition};
use crate::screens::playdate::tests::{blank, c, cell, find, pd_ctx, row};
use crate::widgets::{MenuItemView, MenuView};

/// The test battle's army at the test world map's first chapter, on its
/// map, the army in Millbrook (14, 20).
fn army(ctx: &Ctx) -> Campaign {
    let content = &ctx.game.content;
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut c = battle_campaign(content, &content.battles["test"], GameMode::Classic, lead);
    c.chapter = "test_wm1".into();
    c.enter_world(&content.worldmaps["test_world"].map);
    c
}

/// A place with `mark` at `pos`.
fn place(mark: PlaceMark, pos: (i32, i32), locked: bool) -> PlaceView {
    PlaceView {
        pos: Pos::new(pos.0, pos.1),
        mark,
        level: None,
        locked,
    }
}

/// The test world map with `places`, the army at Millbrook and the
/// cursor at `cursor`, nothing open over it.
fn view(places: Vec<PlaceView>, cursor: (i32, i32)) -> WorldMapView {
    WorldMapView {
        map: "test_world".into(),
        places,
        roads: Vec::new(),
        locked_roads: Vec::new(),
        army: Pos::new(14, 20),
        army_label: "Ma".into(),
        cursor: Pos::new(cursor.0, cursor.1),
        cursor_on_army: false,
        cursor_brightness: 0.5,
        panel: PanelView {
            title: "Stone Mill".into(),
            act: "Act 1 · The Mill".into(),
            kind: "Story battle".into(),
            lines: vec!["Enemy level 2".into()],
        },
        list: None,
        help: "help".into(),
    }
}

fn painted(ctx: &Ctx, view: &WorldMapView) -> GlyphBuffer {
    let mut buf = blank();
    paint(ctx, view, &mut buf);
    buf
}

/// The overlays painted exactly at `rect`, by ink.
fn at(buf: &GlyphBuffer, rect: PxRect) -> Vec<Rgb> {
    let overlays = buf.overlays();
    overlays
        .iter()
        .filter(|o: &&Overlay| o.rect == rect)
        .map(|o| o.color)
        .collect()
}

/// A map smaller than the screen is centred; a bigger one keeps the
/// cursor in the middle, and stops at its edges.
#[test]
fn the_map_is_centred_or_scrolls_with_the_cursor() {
    let room = (50, 15);
    assert_eq!(origin((40, 12), room, Pos::new(3, 3)), (5, 1));
    let big = (70, 30);
    assert_eq!(origin(big, room, Pos::new(35, 15)), (-10, -8));
    assert_eq!(origin(big, room, Pos::new(14, 20)), (0, -13));
    assert_eq!(origin(big, room, Pos::new(65, 2)), (-20, 0));
    assert_eq!(origin(big, room, Pos::new(69, 29)), (-20, -15));
}

/// Every kind of place, open or not yet, has its own symbol and fill:
/// no two look the same without colour (Nick, `world-structure.md`:
/// skirmishes "clearly delineated").
#[test]
fn every_kind_of_place_looks_its_own_in_one_bit() {
    let marks = [
        PlaceMark::StoryBattle,
        PlaceMark::FixedSkirmish,
        PlaceMark::RandomSkirmish,
        PlaceMark::SideQuest,
        PlaceMark::Town,
        PlaceMark::Capital,
        PlaceMark::Cleared,
        PlaceMark::Empty,
    ];
    let mut looks = Vec::new();
    for mark in marks {
        for locked in [false, true] {
            let look = (plate(mark).1, fill(&place(mark, (0, 0), locked)));
            if !looks.contains(&look) {
                looks.push(look);
            }
        }
    }
    // Each open kind its own; each kind not open yet its own but for a
    // place to pass through, a dot either way, and a town and a capital,
    // both `⌂` in grey dots (on the PC both grey too).
    assert_eq!(looks.len(), 14);
    assert_eq!(
        marks.map(|m| fill(&place(m, (0, 0), false))),
        [
            Fill::Black,
            Fill::Grey,
            Fill::Grey,
            Fill::Black,
            Fill::Outlined,
            Fill::Black,
            Fill::Outlined,
            Fill::Bare,
        ]
    );
    // The skirmishes and the story battle: their own symbols too.
    assert_eq!(
        [
            PlaceMark::StoryBattle,
            PlaceMark::FixedSkirmish,
            PlaceMark::RandomSkirmish
        ]
        .map(|m| plate(m).1),
        ['‼', '!', '?']
    );
}

/// Each fill is painted in its inks: a black plate under a white symbol,
/// a grey one under a white-edged one, an outline round a black one,
/// grey dots for one not open yet, a bare dot.
#[test]
fn each_plate_is_painted_in_its_inks() {
    let ctx = pd_ctx();
    // The cursor on Stone Mill (34, 13): the map's cell (0, 0) at
    // (-9, -6) on the screen.
    let places = vec![
        place(PlaceMark::StoryBattle, (34, 13), false),
        place(PlaceMark::FixedSkirmish, (30, 10), false),
        place(PlaceMark::Town, (40, 10), false),
        place(PlaceMark::SideQuest, (30, 16), true),
        place(PlaceMark::Empty, (40, 16), false),
    ];
    let buf = painted(&ctx, &view(places, (34, 13)));
    assert_eq!(origin((70, 30), (50, 15), Pos::new(34, 13)), (-9, -6));
    let screen = |x: i32, y: i32| Pos::new(x - 9, y - 6);
    let check = |p: Pos, glyph: char, fg: Ink, fill: &[Ink], top: &[Ink]| {
        let at_cell = cell(&buf, p.x, p.y);
        assert_eq!((at_cell.glyph, at_cell.fg), (glyph, c(fg)), "{p:?}");
        let rect = plate_px(p, PLATE_PX.0);
        assert_eq!(
            at(&buf, rect),
            fill.iter().map(|&i| c(i)).collect::<Vec<_>>()
        );
        let side = PxRect::new(rect.x, rect.y, rect.w, 1);
        assert_eq!(
            at(&buf, side),
            top.iter().map(|&i| c(i)).collect::<Vec<_>>()
        );
    };
    check(screen(34, 13), '‼', Ink::White, &[Ink::Black], &[]);
    check(screen(30, 10), '!', Ink::BlackEdged, &[Ink::Grey], &[]);
    check(screen(40, 10), '⌂', Ink::Black, &[], &[Ink::Black]);
    check(screen(30, 16), '♥', Ink::Grey, &[], &[Ink::Grey]);
    check(screen(40, 16), '●', Ink::Black, &[], &[]);
}

/// The level marker is black above its place; the army is its two
/// letters white on a black plate under a black flag; the cursor's
/// corner marks flip what they cover.
#[test]
fn the_level_marker_army_and_cursor() {
    let ctx = pd_ctx();
    let mut battle = place(PlaceMark::StoryBattle, (20, 20), false);
    battle.level = Some("Lv2".into());
    let buf = painted(&ctx, &view(vec![battle], (20, 20)));
    // The map's cell (0, 0) at (0, -13): row 7 is the map's row 20.
    let (y, black) = (7, c(Ink::Black));
    let lv = find(&buf, y - 1, "Lv2").expect("the level marker");
    assert_eq!(lv, 19);
    assert_eq!(cell(&buf, lv, y - 1).fg, black);
    let letters = (cell(&buf, 14, y), cell(&buf, 15, y));
    assert_eq!((letters.0.glyph, letters.1.glyph), ('M', 'a'));
    assert_eq!(letters.0.fg, c(Ink::White));
    assert_eq!(at(&buf, army_px(Pos::new(14, y))), [black]);
    assert_eq!(cell(&buf, 14, y - 1).glyph, '▼');
    let cursor: Vec<_> = buf
        .overlays()
        .iter()
        .filter(|o| o.layer == Layer::Over)
        .map(|o| o.color)
        .collect();
    assert_eq!(cursor, [c(Ink::Invert); 8]);
}

/// The place under the cursor is a window in the top corner away from
/// it: its name in the frame, its kind and lines, the act at the bottom.
/// With a list open, the list is there and the place's window under it.
#[test]
fn the_places_window_and_a_list_sit_away_from_the_cursor() {
    let ctx = pd_ctx();
    // The cursor in the screen's left half: the window on the right.
    let buf = painted(&ctx, &view(Vec::new(), (14, 20)));
    assert_eq!(find(&buf, 0, "╔═ Stone Mill"), Some(28));
    assert_eq!(find(&buf, 1, "Story battle"), Some(30));
    assert_eq!(find(&buf, 2, "Enemy level 2"), Some(30));
    assert!(row(&buf, 3).ends_with("╚═ Act 1 · The Mill ═╝"));
    // A longer act line is cut in the widest window.
    let mut long = view(Vec::new(), (14, 20));
    long.panel.act = "Act 1 · A chapter with a long title".into();
    let buf = painted(&ctx, &long);
    assert_eq!(find(&buf, 0, "╔═ Stone Mill"), Some(50 - PLACE_W));
    assert!(row(&buf, 3).ends_with("╚═ Act 1 · A chapte…═╝"));
    // A place in the top corner (the map's (40, 14), the screen's (40, 1)):
    // the window takes the bottom one, which hides none.
    let hidden = view(vec![place(PlaceMark::Town, (40, 14), false)], (14, 20));
    let buf = painted(&ctx, &hidden);
    assert_eq!(find(&buf, 11, "╔═ Stone Mill"), Some(28));
    assert_eq!(cell(&buf, 40, 1).glyph, '⌂');
    // In its right half: on the left.
    let buf = painted(&ctx, &view(Vec::new(), (60, 20)));
    assert_eq!(find(&buf, 0, "╔═ Stone Mill"), Some(0));
    // A list over it, then the window.
    let mut open = view(Vec::new(), (14, 20));
    let item = |label: &str, focused| MenuItemView {
        label: label.into(),
        suffix: None,
        note: None,
        enabled: true,
        focused,
    };
    open.list = Some(ListView {
        title: "Millbrook".into(),
        menu: MenuView {
            items: vec![item("Armory", true), item("Vendor", false)],
            first_shown: 0,
            shown: 2,
        },
    });
    let buf = painted(&ctx, &open);
    assert_eq!(find(&buf, 0, "╔═ Millbrook ═╗"), Some(35));
    assert_eq!(find(&buf, 1, "Armory"), Some(37));
    assert_eq!(find(&buf, 4, "╔═ Stone Mill"), Some(28));
}

/// The world map as the Playdate starts it: Millbrook, the open places
/// round it, the window on the right.
#[test]
fn the_test_world_map_at_its_start() {
    let ctx = pd_ctx();
    let s = WorldMapScreen::new(army(&ctx), 0.0);
    let mut buf = blank();
    s.draw(&ctx, &mut buf);
    let shown: Vec<String> = (0..15).map(|y| untrimmed(&buf, y)).collect();
    assert_snapshot!("playdate_world_map_start", shown.join("\n"));
}

/// Row `y` of `buf`, its glyphs, with the spaces at its end trimmed.
fn untrimmed(buf: &GlyphBuffer, y: i32) -> String {
    let line: String = (0..50).map(|x| cell(buf, x, y).glyph).collect();
    line.trim_end().to_owned()
}

/// The context of the Playdate with no keyboard and its own buttons.
fn buttonless() -> Ctx {
    let mut ctx = pd_ctx();
    ctx.platform.keyboard = Keyboard::Absent;
    ctx.use_playdate_buttons();
    ctx
}

fn press(s: &mut WorldMapScreen, ctx: &mut Ctx, action: Action) {
    let input = FrameInput::new(vec![action], 0.0, vec![]);
    s.update(ctx, &input);
}

/// The menu's labels, in order.
fn menu_labels(s: &WorldMapScreen, ctx: &Ctx) -> Vec<String> {
    let list = s.view(ctx).list.expect("a list");
    list.menu
        .items
        .iter()
        .map(|i| ctx.label(&i.label))
        .collect()
}

/// With no keyboard and no button for them, the menu lists Find and Your
/// army first (`playdate.md`: the map menu holds every shortcut the
/// Playdate has no button for); the PC's menu is as it was.
#[test]
fn the_menu_lists_find_and_your_army_where_they_have_no_button() {
    let mut pc = crate::screen::tests::ctx();
    assert_eq!(MenuEntry::entries(&pc), MenuEntry::ALL);
    let mut s = WorldMapScreen::new(army(&pc), 0.0);
    press(&mut s, &mut pc, Action::Cancel);
    assert_eq!(menu_labels(&s, &pc), ["Save", "Camp", "Options"]);

    let mut ctx = buttonless();
    let mut s = WorldMapScreen::new(army(&ctx), 0.0);
    press(&mut s, &mut ctx, Action::Cancel);
    assert_eq!(
        menu_labels(&s, &ctx),
        ["Find", "Your army", "Save", "Camp", "Options"]
    );
    // Find opens the Find list, on the army.
    press(&mut s, &mut ctx, Action::Confirm);
    let title = s.view(&ctx).list.map(|l| l.title);
    assert_eq!(title.as_deref(), Some("Find"));
    // Next place, then Your army from the menu: back on the army.
    press(&mut s, &mut ctx, Action::Cancel);
    press(&mut s, &mut ctx, Action::NextUnit);
    assert_ne!(s.cursor(), "home");
    press(&mut s, &mut ctx, Action::Cancel);
    press(&mut s, &mut ctx, Action::CursorDown);
    press(&mut s, &mut ctx, Action::Confirm);
    assert_eq!(s.cursor(), "home");
    assert!(s.view(&ctx).list.is_none());
}

/// The system menu's Options (ticket 1035) opens Options on the map, as
/// the map's own menu does.
#[test]
fn the_system_menus_options_opens_options_on_the_map() {
    let mut ctx = buttonless();
    let mut s = WorldMapScreen::new(army(&ctx), 0.0);
    let input = FrameInput::new(vec![Action::Options], 0.0, vec![]);
    match s.update(&mut ctx, &input) {
        Transition::Push(screen) => assert_eq!(screen.name(), "options"),
        _ => panic!("Options didn't open"),
    }
}
