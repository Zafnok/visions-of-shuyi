//! The battle screen's own tests.

use insta::assert_snapshot;

use trpg_core::{BattleMap, Grid, Objective, Phase, TerrainId, Unit};

use super::testing::{battle, vaulted, wait};
use super::view::{MenuPlace, PanelUnitView, PanelView, TerrainView};
use super::*;
use crate::audio::CURSOR_MOVE;
use crate::color::{Rgb, UiColor};
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::glyph_buffer::Cell;
use crate::harness::Harness;
use crate::map_view::CursorStyle;
use crate::screen::tests::ctx;

pub(super) fn quick() -> BattleScreen {
    BattleScreen::new(quick_battle(&ctx().game.content).unwrap())
}

pub(super) fn render(screen: &BattleScreen, c: &Ctx) -> GlyphBuffer {
    let stale = Cell::new('x', Rgb::new(1, 2, 3), Rgb::new(1, 2, 3));
    let mut buf = GlyphBuffer::with_spec(ConsoleSpec::PC, stale);
    screen.draw(c, &mut buf);
    buf
}

#[test]
fn quick_battle_state() {
    let state = quick_battle(&ctx().game.content).unwrap();
    assert_eq!(state.map().name, "Test Field");
    assert_eq!((state.turn(), state.phase()), (1, Phase::Player));
    assert_eq!(state.objective(), Objective::Rout { turn_limit: None });
    assert_eq!(state.outcome(), None);
    let units = state.units();
    let labels: Vec<(&str, Faction)> = units
        .iter()
        .map(|u| (u.map_label.as_str(), u.faction))
        .collect();
    assert_eq!(
        labels,
        [
            ("Lo", Faction::Player),
            ("Kn", Faction::Player),
            ("Ar", Faction::Player),
            ("Br", Faction::Enemy),
            ("Br", Faction::Enemy),
            ("Ra", Faction::Enemy),
            ("Fr", Faction::Enemy),
            ("Ma", Faction::Player),
        ]
    );
    let ids: Vec<u32> = units.iter().map(|u| u.id.0).collect();
    assert_eq!(ids, [1, 2, 3, 4, 5, 6, 7, 8]);
    for u in units {
        assert!(!u.acted, "{}", u.name);
        assert_eq!(u.hp, u.stats.hp, "{}", u.name);
        assert!(state.map().tiles.in_bounds(u.pos));
    }
    // The caster (0410), numbered after the enemies: Fire from its
    // class, Frost and Heal of its own, all at full uses, Fire equipped.
    let mage = state.unit(UnitId(8)).unwrap();
    assert_eq!(
        (mage.name.as_str(), mage.pos),
        ("Test Mage", Pos::new(3, 6))
    );
    let spells: Vec<(&str, u8)> = mage
        .learned
        .iter()
        .map(|s| (s.0.as_str(), mage.spells.uses_left(s)))
        .collect();
    assert_eq!(spells, [("fire", 10), ("frost", 10), ("heal", 8)]);
    let fire = trpg_core::SpellId::new("fire");
    assert_eq!(mage.loadout.equipped, Some(Equipped::Spell(fire)));
    // The elemental holds its tile.
    let elemental = state.unit(UnitId(7)).unwrap();
    assert_eq!(elemental.class.0, "frost_elemental");
    assert_eq!(elemental.ai, trpg_core::AiBehavior::Stationary);
    assert_eq!(elemental.pos, Pos::new(8, 7));
}

#[test]
fn quick_battle_reports_missing_content() {
    let mut content = ctx().game.content;
    content.battles.clear();
    assert_eq!(
        quick_battle(&content),
        Err("no battle \"quick\"".to_owned())
    );
}

#[test]
fn camera_starts_on_the_first_player_unit() {
    let c = ctx();
    let s = quick();
    // test_small is smaller than the viewport: centred.
    assert_eq!(s.camera().origin, Pos::new(-10, -11));
    assert_eq!(s.state().units().len(), 8);
    let big = BattleMap::new("Big", Grid::filled(64, 40, TerrainId(0)));
    let units = s.state().units().to_vec();
    let camera = |units: Vec<Unit>| {
        BattleScreen::new(battle(&c, big.clone(), units))
            .camera()
            .origin
    };
    // Lord at (3, 5): clamped to the top-left.
    assert_eq!(camera(units.clone()), Pos::new(0, 0));
    let mut far = units.clone();
    far[0].pos = Pos::new(60, 30);
    assert_eq!(camera(far), Pos::new(29, 10));
    let mut enemies_only = units;
    enemies_only.retain(|u| u.faction != Faction::Player);
    // Map centre (32, 20).
    assert_eq!(camera(enemies_only), Pos::new(15, 5));
}

#[test]
fn cancel_opens_the_map_menu_and_nothing_pops() {
    let mut s = quick();
    let mut c = ctx();
    let step = |s: &mut BattleScreen, c: &mut Ctx, a: &[Action]| {
        format!(
            "{:?}",
            s.update(c, &FrameInput::new(a.to_vec(), 0.0, vec![]))
        )
    };
    assert_eq!(s.name(), "battle");
    assert!(!s.is_overlay());
    assert_eq!(step(&mut s, &mut c, &[]), "None");
    // Select the lord, then cancel the selection: still here.
    assert_eq!(step(&mut s, &mut c, &[Action::Confirm]), "None");
    assert_eq!(step(&mut s, &mut c, &[Action::Cancel]), "None");
    assert_eq!(s.mode(), &Mode::default());
    // Browsing: Cancel opens the map menu, and Cancel again closes it.
    assert_eq!(step(&mut s, &mut c, &[Action::Cancel]), "None");
    assert!(matches!(s.mode(), Mode::MapMenu { .. }), "{:?}", s.mode());
    assert_eq!(step(&mut s, &mut c, &[Action::Cancel]), "None");
    assert_eq!(s.mode(), &Mode::default());
}

#[test]
fn help_depends_on_what_is_hovered() {
    let mut c = ctx();
    let mut state = quick_battle(&c.game.content).unwrap();
    // The archer waits where it stands.
    wait(&mut state, 2);
    let mut s = BattleScreen::new(state);
    // On the lord, ready to act.
    assert_eq!(
        s.help(&c),
        "f select · e info · s next unit · r rewind · d menu · Space end turn"
    );
    // On the archer, who has acted, and on an enemy.
    s.cursor.jump(Pos::new(2, 4));
    assert_eq!(
        s.help(&c),
        "arrows move · e info · s next unit · r rewind · d menu · Space end turn"
    );
    s.cursor.jump(Pos::new(8, 2));
    assert_eq!(
        s.help(&c),
        "arrows move · f range · e info · s next unit · r rewind · d menu · Space end turn"
    );
    // An enemy's range shown: Cancel hides it.
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(
        s.help(&c),
        "arrows move · f range · e info · s next unit · r rewind · d hide range · Space end turn"
    );
    step(&mut s, &mut c, &[Action::Cancel]);
    // On an empty tile.
    s.cursor.jump(Pos::new(0, 0));
    assert_eq!(
        s.help(&c),
        "arrows move · f menu · s next unit · r rewind · d menu · Space end turn"
    );
    c.use_layout(crate::input::LEFT_HANDED);
    assert_eq!(
        s.help(&c),
        "wasd move · j menu · l next unit · u rewind · k menu · Space end turn"
    );
    s.cursor.jump(Pos::new(3, 5));
    assert_eq!(
        s.help(&c),
        "j select · i info · l next unit · u rewind · k menu · Space end turn"
    );
}

#[test]
fn help_shows_not_mapped_for_an_action_with_no_key() {
    let mut c = ctx();
    let s = quick();
    let mut keys = c.layout_bindings(crate::input::RIGHT_HANDED);
    keys.clear(Action::Info, 0);
    c.set_layout_bindings(crate::input::RIGHT_HANDED, keys)
        .unwrap();
    assert_eq!(
        s.help(&c),
        "f select · ! not mapped info · s next unit · r rewind · d menu · Space end turn"
    );
}

#[test]
fn cursor_starts_on_the_first_player_lord() {
    let c = ctx();
    let s = quick();
    assert_eq!(s.cursor(), Cursor::new(Pos::new(3, 5)));
    // The lord listed after another player unit: still the lord.
    let mut units = s.state().units().to_vec();
    units.swap(0, 1);
    let map = s.state().map().clone();
    let s2 = BattleScreen::new(battle(&c, map.clone(), units.clone()));
    assert_eq!(s2.cursor().pos, Pos::new(3, 5));
    // No lord: the first player unit.
    for u in &mut units {
        u.is_lord = false;
    }
    let s3 = BattleScreen::new(battle(&c, map.clone(), units.clone()));
    assert_eq!(s3.cursor().pos, Pos::new(4, 6));
    // No player unit: the map's centre (test_small is 14 × 8).
    units.retain(|u| u.faction != Faction::Player);
    let s4 = BattleScreen::new(battle(&c, map, units));
    assert_eq!(s4.cursor().pos, Pos::new(7, 4));
}

/// One frame with `actions`.
pub(super) fn step(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action]) {
    s.update(c, &FrameInput::new(actions.to_vec(), 0.0, vec![]));
}

#[test]
fn cursor_moves_one_tile_per_step_and_stays_on_the_map() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::CursorRight; 3]);
    assert_eq!(s.cursor().pos, Pos::new(6, 5));
    step(&mut s, &mut c, &[Action::CursorDown, Action::CursorLeft]);
    assert_eq!(s.cursor().pos, Pos::new(5, 6));
    step(&mut s, &mut c, &[Action::CursorUp; 20]);
    assert_eq!(s.cursor().pos, Pos::new(5, 0));
    step(&mut s, &mut c, &[Action::CursorLeft; 20]);
    assert_eq!(s.cursor().pos, Pos::new(0, 0));
    step(&mut s, &mut c, &[Action::CursorDown; 20]);
    step(&mut s, &mut c, &[Action::CursorRight; 20]);
    assert_eq!(s.cursor().pos, Pos::new(13, 7));
    // Other keys don't move it; the small map stays centred.
    step(
        &mut s,
        &mut c,
        &[Action::Confirm, Action::Info, Action::DangerZone],
    );
    assert_eq!(s.cursor().pos, Pos::new(13, 7));
    assert_eq!(s.camera().origin, Pos::new(-10, -11));
}

#[test]
fn cursor_pulses_with_frame_time() {
    let mut c = ctx();
    let mut s = quick();
    s.update(&mut c, &FrameInput::new(vec![], 0.5, vec![]));
    assert!((s.cursor().brightness() - cursor::BLINK_MIN).abs() < 1e-6);
    // The map shows it on the lord at (3, 5), dimmed; a fresh battle's
    // at full brightness.
    let shown = |s: &BattleScreen| s.scene(&c).cursor.map(|v| (v.pos, v.brightness));
    assert_eq!(shown(&s), Some((Pos::new(3, 5), cursor::BLINK_MIN)));
    assert_eq!(shown(&quick()), Some((Pos::new(3, 5), 1.0)));
}

#[test]
fn next_and_prev_unit_cycle_ready_units_in_reading_order() {
    let mut c = ctx();
    let mut s = quick();
    // Ready, in reading order: the archer (2, 4), the lord (3, 5), the
    // mage (3, 6) and the knight (4, 6).
    assert_eq!(
        s.ready_units(),
        [
            Pos::new(2, 4),
            Pos::new(3, 5),
            Pos::new(3, 6),
            Pos::new(4, 6)
        ]
    );
    let mut visit = |a: Action| {
        step(&mut s, &mut c, &[a]);
        s.cursor().pos
    };
    assert_eq!(visit(Action::NextUnit), Pos::new(3, 6));
    assert_eq!(visit(Action::NextUnit), Pos::new(4, 6));
    assert_eq!(visit(Action::NextUnit), Pos::new(2, 4));
    assert_eq!(visit(Action::NextUnit), Pos::new(3, 5));
    assert_eq!(visit(Action::PrevUnit), Pos::new(2, 4));
    assert_eq!(visit(Action::PrevUnit), Pos::new(4, 6));
    assert_eq!(visit(Action::PrevUnit), Pos::new(3, 6));
    assert_eq!(visit(Action::PrevUnit), Pos::new(3, 5));
    // From a tile between them, in reading order.
    assert_eq!(visit(Action::CursorRight), Pos::new(4, 5));
    assert_eq!(visit(Action::NextUnit), Pos::new(3, 6));
    assert_eq!(visit(Action::NextUnit), Pos::new(4, 6));
    assert_eq!(visit(Action::CursorUp), Pos::new(4, 5));
    assert_eq!(visit(Action::PrevUnit), Pos::new(3, 5));
    // Below every ready unit: next wraps to the first.
    for _ in 0..5 {
        visit(Action::CursorDown);
    }
    assert_eq!(visit(Action::NextUnit), Pos::new(2, 4));
}

#[test]
fn cycling_does_nothing_without_ready_units_and_scrolls_the_camera() {
    let c = ctx();
    let mut ctx_ = ctx();
    let mut units = quick_battle(&c.game.content).unwrap().units().to_vec();
    for u in &mut units {
        u.acted = u.faction == Faction::Player;
    }
    let map = BattleMap::new("Big", Grid::filled(64, 40, TerrainId(0)));
    let mut s = BattleScreen::new(battle(&c, map.clone(), units.clone()));
    step(&mut s, &mut ctx_, &[Action::NextUnit, Action::PrevUnit]);
    assert_eq!(s.cursor().pos, Pos::new(3, 5));
    // A ready unit far away: the camera follows the jump.
    units[1].acted = false;
    units[1].pos = Pos::new(60, 35);
    let mut s = BattleScreen::new(battle(&c, map, units));
    // (Past the mage, the one unit between the lord and it.)
    step(&mut s, &mut ctx_, &[Action::NextUnit, Action::NextUnit]);
    assert_eq!(s.cursor().pos, Pos::new(60, 35));
    // Scrolled just enough to keep it 3 tiles from the edges.
    assert_eq!(s.camera().origin, Pos::new(29, 9));
}

/// The Quick Battle's units on a 64 × 40 plain, in the harness.
fn big_battle_harness() -> Harness {
    let c = ctx();
    let units = quick_battle(&c.game.content).unwrap().units().to_vec();
    let map = BattleMap::new("Big", Grid::filled(64, 40, TerrainId(0)));
    Harness::with_screen(Box::new(BattleScreen::new(battle(&c, map, units))))
}

/// Where the cursor is in the map view, as tiles right of and below
/// its top-left tile.
fn cursor_in_view(h: &Harness) -> (i32, i32) {
    let scene = h.map_scene().expect("no battle");
    let pos = scene.cursor_tile().expect("no cursor on the map");
    scene.offset(pos).expect("the cursor is in the view")
}

/// Holding a cursor key ticks once per tile moved, and not at the
/// map's edge (ticket 0425).
#[test]
fn held_cursor_ticks_once_per_tile() {
    let ticks = |h: &Harness| h.sounds().iter().filter(|c| *c == CURSOR_MOVE).count();
    let mut h = big_battle_harness();
    // The lord at (3, 5); the camera scrolls as the cursor goes right.
    h.hold("Right", 1.0);
    let right = ticks(&h);
    assert!(right > 5, "{right} ticks: the key should repeat");
    assert_eq!(h.sounds().len(), right, "{:?}", h.sounds());
    // Held left for much longer than it takes to reach x = 0: one tick
    // per tile back, none once at the edge.
    h.clear_audio().hold("Left", 6.0);
    assert_eq!(ticks(&h), right + 3);
    assert_eq!(cursor_in_view(&h).0, 0, "at the left edge");
    h.clear_audio().keys("Left Up Up Up Up Up Up");
    assert_eq!(ticks(&h), 5, "only the five steps up to y = 0");
    // Next unit jumps: one tick.
    h.clear_audio().keys("s");
    assert_eq!(h.sounds(), [CURSOR_MOVE]);
}

#[test]
fn harness_keys_move_the_cursor() {
    let mut h = big_battle_harness();
    // The lord at (3, 5), camera at the top-left.
    assert_eq!(cursor_in_view(&h), (3, 5));
    h.keys("Right Right Right");
    assert_eq!(cursor_in_view(&h), (6, 5));
    h.keys("Down Left");
    assert_eq!(cursor_in_view(&h), (5, 6));
}

#[test]
fn harness_hold_repeats_with_the_keymap_timing_and_scrolls() {
    let c = ctx();
    let repeat = c.game.content.keymap.repeat;
    let (delay, interval) = (repeat.delay_ms, repeat.interval_ms);
    let moves = |ms: u32| i32::try_from(1 + 1 + (ms - delay) / interval).unwrap();
    let mut h = big_battle_harness();
    h.hold("Right", 1.2);
    // 1 press + repeats at 300 ms, then every 55 ms: 18 tiles, to x = 21.
    assert_eq!(moves(1200), 18);
    assert_eq!(cursor_in_view(&h), (3 + 18, 5));
    h.hold("Right", 1.2);
    // x = 39: the camera keeps it 3 tiles from the right edge.
    let x = 3 + 2 * moves(1200);
    let view_w = h.map_scene().unwrap().size.0;
    assert_eq!(view_w, 35);
    let origin = x - (view_w - 1 - Camera::MARGIN);
    assert_eq!(cursor_in_view(&h), (x - origin, 5));
    // Far right, then back: the cursor stops at the edge and the
    // camera shows the map's last columns, then scrolls back.
    h.hold("Right", 3.0).hold("Down", 3.0);
    assert_eq!(cursor_in_view(&h), (34, 29));
    h.hold("Left", 4.0).hold("Up", 3.0);
    assert_eq!(cursor_in_view(&h), (0, 0));
}

/// The Quick Battle with the knight wounded (9/20 HP) and moved into
/// the forest at (1, 5).
pub(super) fn knight_in_forest() -> BattleScreen {
    let c = ctx();
    let state = quick_battle(&c.game.content).unwrap();
    let mut units = state.units().to_vec();
    units[1].pos = Pos::new(1, 5);
    units[1].hp = units[1].stats.hp * 9 / 20;
    let mut s = BattleScreen::new(battle(&c, state.map().clone(), units));
    s.cursor.jump(Pos::new(1, 5));
    s
}

/// The side panel `s` shows.
fn panel(s: &BattleScreen, c: &Ctx) -> PanelView {
    s.view(c).panel.expect("the panel")
}

#[test]
fn panel_shows_the_terrain_and_unit_under_the_cursor() {
    let c = ctx();
    let s = knight_in_forest();
    let knight = s.hovered().unwrap().clone();
    let shown = panel(&s, &c);
    assert!(!shown.selected);
    assert_eq!(
        shown.terrain,
        Some(TerrainView {
            name: "Forest".to_owned(),
            bonuses: "DEF +1  AVO +20".to_owned(),
            heals: None,
        })
    );
    assert_eq!(
        shown.unit,
        Some(PanelUnitView {
            name: "Test Knight".to_owned(),
            faction: Faction::Player,
            class_level: "Guard  Lv 1".to_owned(),
            hp_name: "HP".to_owned(),
            hp: knight.hp,
            max: knight.stats.hp,
            faction_name: "Player".to_owned(),
        })
    );
    assert_eq!((knight.hp, knight.stats.hp), (9, 20));
    // An enemy: its faction.
    let mut s = s;
    s.cursor.jump(Pos::new(8, 2));
    let enemy = panel(&s, &c).unit.unwrap();
    assert_eq!(
        (enemy.faction, enemy.faction_name.as_str()),
        (Faction::Enemy, "Enemy")
    );
}

#[test]
fn panel_shows_healing_terrain_and_whole_names() {
    let mut c = ctx();
    let s = knight_in_forest();
    let mut rules = c.game.content.terrain.rules.clone();
    let forest = c.game.content.terrain.display.id_of("forest").unwrap();
    rules.terrains[usize::from(forest.0)].heal_percent = 20;
    rules.terrains[usize::from(forest.0)].name = "A".repeat(40);
    c.game.content.terrain.rules = rules;
    let mut units = s.state().units().to_vec();
    units[1].name = "B".repeat(40);
    let mut s2 = BattleScreen::new(battle(&c, s.state().map().clone(), units));
    s2.cursor.jump(Pos::new(1, 5));
    let shown = panel(&s2, &c);
    let terrain = shown.terrain.unwrap();
    // Whole: the skin cuts what doesn't fit.
    assert_eq!(terrain.name, "A".repeat(40));
    assert_eq!(terrain.heals.as_deref(), Some("Heals 20% HP"));
    assert_eq!(shown.unit.unwrap().name, "B".repeat(40));
}

/// Hovering the wounded knight, standing in a forest.
#[test]
fn hover_unit_on_forest_snapshot() {
    let c = ctx();
    assert_snapshot!(render(&knight_in_forest(), &c).to_snapshot(&c.palette));
}

/// Hovering an empty plain: terrain only.
#[test]
fn hover_empty_plain_snapshot() {
    let c = ctx();
    let mut s = quick();
    s.cursor.jump(Pos::new(6, 5));
    let shown = panel(&s, &c);
    assert_eq!(shown.terrain.map(|t| t.name).as_deref(), Some("Plain"));
    assert_eq!(shown.unit, None);
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

#[test]
fn draw_covers_the_whole_buffer() {
    let c = ctx();
    let buf = render(&quick(), &c);
    let stale = Rgb::new(1, 2, 3);
    for y in 0..i32::from(ConsoleSpec::PC.rows) {
        for x in 0..i32::from(ConsoleSpec::PC.cols) {
            let cell = buf.get(x, y).unwrap();
            assert!(cell.fg != stale && cell.bg != stale, "({x}, {y})");
        }
    }
}

#[test]
fn terrain_tiles_use_their_two_glyphs_and_colours() {
    let c = ctx();
    let s = quick();
    let buf = render(&s, &c);
    let p = &c.palette;
    // Tile (0, 0) is sea, drawn at cell (20, 11).
    let display = &c.game.content.terrain.display;
    let sea = display.get(display.id_of("sea").unwrap()).unwrap();
    let cell = |x| *buf.get(x, 11).unwrap();
    for x in [20, 21] {
        assert_eq!(cell(x).glyph, sea.glyphs[usize::try_from(x - 20).unwrap()]);
        assert_eq!(Some(cell(x).fg), p.lookup(&sea.fg));
        assert_eq!(Some(cell(x).bg), p.lookup(&sea.bg));
    }
    // Left of the map: blank.
    assert_eq!(
        cell(19),
        Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black))
    );
}

/// A 64 × 40 map of stripes, a unit in the bottom-right corner and the
/// cursor and camera moved there.
#[test]
fn large_map_scrolled_to_bottom_right_snapshot() {
    let c = ctx();
    let display = &c.game.content.terrain.display;
    let ids = ["plain", "forest", "road", "water", "mountain"].map(|t| display.id_of(t).unwrap());
    let cells = (0..40)
        .flat_map(|y: usize| (0..64).map(move |x: usize| ids[(x / 3 + y / 2) % ids.len()]))
        .collect();
    let mut units = quick_battle(&c.game.content).unwrap().units().to_vec();
    // The camera starts on the lord at (3, 5), top-left.
    units[5].pos = Pos::new(63, 39);
    let cornered = units[5].id;
    units[3].pos = Pos::new(30, 12);
    units[4].pos = Pos::new(28, 9); // Above the viewport: not drawn.
    let map = BattleMap::new("Stripes", Grid::from_cells(64, 40, cells).unwrap());
    let mut s = BattleScreen::new(battle(&c, map, units));
    assert_eq!(s.camera().origin, Pos::new(0, 0));
    s.cursor.jump(Pos::new(63, 39));
    s.follow(Pos::new(63, 39));
    assert_eq!(s.camera().origin, Pos::new(29, 10));
    // The corner unit sits in the viewport's last tile.
    let scene = s.scene(&c);
    let corner = Pos::new(63, 39);
    assert_eq!(scene.size, (35, 30));
    assert_eq!(scene.offset(corner), Some((34, 29)));
    assert_eq!(scene.unit_at(corner).map(|u| u.id), Some(cornered));
    assert_snapshot!(render(&s, &c).to_snapshot(&c.palette));
}

/// One frame of `dt` seconds with `actions`, Confirm held if `held`.
pub(super) fn frame(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action], dt: f32, held: bool) {
    let held = if held { vec![Action::Confirm] } else { vec![] };
    s.update(c, &FrameInput::new(actions.to_vec(), dt, held));
}

/// The Quick Battle with the lord selected and walked two tiles right to
/// (5, 5), its action menu open.
pub(super) fn lord_menu(c: &mut Ctx) -> BattleScreen {
    let mut s = quick();
    step(
        &mut s,
        c,
        &[Action::Confirm, Action::CursorRight, Action::CursorRight],
    );
    step(&mut s, c, &[Action::Confirm]);
    assert!(matches!(s.mode(), Mode::Moving { .. }), "{:?}", s.mode());
    frame(&mut s, c, &[], 1.0, false);
    assert!(
        matches!(s.mode(), Mode::ActionMenu { .. }),
        "{:?}",
        s.mode()
    );
    s
}

#[test]
fn select_move_and_wait_ends_the_units_action() {
    let mut c = ctx();
    let mut s = lord_menu(&mut c);
    step(&mut s, &mut c, &[Action::Confirm]);
    let lord = &s.state().units()[0];
    assert_eq!((lord.pos, lord.acted), (Pos::new(5, 5), true));
    assert_eq!(s.mode(), &Mode::default());
    assert_eq!(s.cursor().pos, Pos::new(5, 5));
    // No longer ready: not selectable, and cycling skips it.
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.mode(), &Mode::default());
    assert_eq!(
        s.ready_units(),
        [Pos::new(2, 4), Pos::new(3, 6), Pos::new(4, 6)]
    );
}

#[test]
fn cancelling_from_the_menu_and_the_selection_leaves_the_battle_unchanged() {
    let mut c = ctx();
    let before = quick().state().clone();
    let mut s = lord_menu(&mut c);
    // The lord is drawn at (5, 5), but the battle hasn't changed.
    assert_eq!(s.hovered().map(|u| u.id), Some(UnitId(1)));
    assert_eq!(s.state(), &before);
    step(&mut s, &mut c, &[Action::Cancel]);
    let Mode::Selected(sel) = s.mode() else {
        panic!("{:?}", s.mode());
    };
    assert_eq!(sel.path, [Pos::new(3, 5), Pos::new(4, 5), Pos::new(5, 5)]);
    assert_eq!(s.hovered(), None, "drawn back at its own tile");
    assert_eq!(s.state(), &before);
    step(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.mode(), &Mode::default());
    assert_eq!(s.cursor().pos, Pos::new(3, 5));
    assert_eq!(s.state(), &before);
}

#[test]
fn a_held_confirm_skips_the_walk() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm]);
    step(&mut s, &mut c, &[Action::CursorRight; 2]);
    // The press starts the walk; still held after 0.2 s, it skips.
    frame(&mut s, &mut c, &[Action::Confirm], 0.0, true);
    frame(&mut s, &mut c, &[], mode::HOLD_SKIP_S / 2.0, true);
    assert!(matches!(s.mode(), Mode::Moving { .. }));
    frame(&mut s, &mut c, &[], mode::HOLD_SKIP_S / 2.0, true);
    assert!(matches!(s.mode(), Mode::ActionMenu { .. }));
}

#[test]
fn keys_do_only_what_the_mode_allows() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm, Action::CursorRight]);
    // Selected: cycling does nothing.
    step(&mut s, &mut c, &[Action::NextUnit, Action::PrevUnit]);
    assert_eq!(s.cursor().pos, Pos::new(4, 5));
    // In the menu, cursor keys move its focus, not the cursor.
    let mut s = lord_menu(&mut c);
    step(
        &mut s,
        &mut c,
        &[Action::CursorUp, Action::CursorLeft, Action::NextUnit],
    );
    assert_eq!(s.cursor().pos, Pos::new(5, 5));
    // While walking, too.
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm, Action::CursorRight]);
    step(&mut s, &mut c, &[Action::Confirm, Action::CursorDown]);
    assert_eq!(s.cursor().pos, Pos::new(4, 5));
}

#[test]
fn selecting_shows_ranges_and_marks_the_panel() {
    let mut c = ctx();
    let mut s = quick();
    assert!(!panel(&s, &c).selected);
    assert!(s.scene(&c).tints_at(Pos::new(6, 5)).is_empty());
    step(&mut s, &mut c, &[Action::Confirm]);
    let scene = s.scene(&c);
    // The cursor stays on the lord, as corner marks.
    let cursor = scene.cursor.unwrap();
    let lord = Pos::new(3, 5);
    assert_eq!((cursor.pos, cursor.style), (lord, CursorStyle::Corners));
    // (6, 5), reachable, is in the move range.
    assert_eq!(scene.tints_at(Pos::new(6, 5)), [RangeKind::Move]);
    let Mode::Selected(sel) = s.mode() else {
        panic!()
    };
    let attack = sel.attack.iter().next().unwrap();
    assert_eq!(scene.tints_at(attack), [RangeKind::Attack]);
    // The panel is marked while a unit is selected.
    assert!(panel(&s, &c).selected);
    // No path yet: only the lord's own tile.
    assert_eq!(scene.path, [lord]);
}

#[test]
fn the_path_ends_in_an_arrowhead_with_no_cursor_frame_there() {
    let mut c = ctx();
    let mut s = quick();
    step(
        &mut s,
        &mut c,
        &[Action::Confirm, Action::CursorRight, Action::CursorRight],
    );
    // The path from the lord ends on the fort at (5, 5) (each skin
    // draws its end its own way: the glyph skin an arrowhead): no
    // cursor there, and the lord still on its tile.
    let scene = s.scene(&c);
    let (lord, fort) = (Pos::new(3, 5), Pos::new(5, 5));
    assert_eq!(scene.path, [lord, Pos::new(4, 5), fort]);
    assert_eq!(scene.cursor, None);
    assert_eq!(scene.unit_at(lord).map(|u| u.label.as_str()), Some("Lo"));
    assert_eq!(scene.unit_at(fort), None);
    let fort_id = c.game.content.terrain.display.id_of("fort");
    assert_eq!(scene.terrain_at(fort), fort_id);
    // On to the map's right edge, past the lord's reach (Mov 5): the
    // path, through the fort (cost 2), stops at (7, 5), and the cursor
    // shows its corner marks.
    step(&mut s, &mut c, &[Action::CursorRight; 8]);
    assert_eq!(s.cursor().pos, Pos::new(13, 5));
    let Mode::Selected(sel) = s.mode() else {
        panic!("{:?}", s.mode());
    };
    assert_eq!(sel.dest(), Pos::new(7, 5));
    assert_eq!(s.scene(&c).cursor_tile(), Some(Pos::new(13, 5)));
}

#[test]
fn the_menu_opens_beside_the_unit_drawn_at_its_new_tile() {
    let mut c = ctx();
    let s = lord_menu(&mut c);
    let view = s.view(&c);
    // The lord at (5, 5); its old tile is empty.
    let scene = &view.scene;
    let lord = scene.unit_at(Pos::new(5, 5));
    assert_eq!(lord.map(|u| u.label.as_str()), Some("Lo"));
    assert_eq!(scene.unit_at(Pos::new(3, 5)), None);
    // The menu beside the lord's new tile: `Attack`, `Arts` and `Item`
    // disabled, `Skill`, `Equip`, `Wait` (focused).
    let menu = view.menu.unwrap();
    assert_eq!(menu.place, MenuPlace::Beside(Pos::new(5, 5)));
    assert_eq!(menu.header, None);
    let items: Vec<_> = menu
        .menu
        .items
        .iter()
        .map(|i| (i.label.as_str(), i.enabled, i.focused))
        .collect();
    assert_eq!(
        items,
        [
            ("Attack", false, false),
            ("Arts", false, false),
            ("Skill", true, false),
            ("Item", false, false),
            ("Equip", true, false),
            ("Wait", true, true),
        ]
    );
    // No cursor, no ranges, the panel shows the lord.
    assert_eq!(scene.cursor, None);
    assert!(scene.tiles.iter().all(|t| t.tints.is_empty()));
    let unit = view.panel.and_then(|p| p.unit).unwrap();
    assert_eq!(unit.name, "Test Lord");
}

#[test]
fn help_follows_the_mode() {
    let mut c = ctx();
    let mut s = quick();
    step(&mut s, &mut c, &[Action::Confirm]);
    // On the unit: its own tile is a legal end.
    assert_eq!(s.help(&c), "arrows move · f move here · d cancel");
    step(&mut s, &mut c, &[Action::CursorRight, Action::CursorDown]);
    // On the knight: can't stop there.
    assert_eq!(s.help(&c), "arrows move · d cancel");
    step(&mut s, &mut c, &[Action::CursorUp, Action::CursorRight]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.help(&c), "f skip");
    frame(&mut s, &mut c, &[], 1.0, false);
    assert_eq!(s.help(&c), "arrows choose · f confirm · d back");
    c.use_layout(crate::input::LEFT_HANDED);
    assert_eq!(s.help(&c), "wasd choose · j confirm · k back");
}

#[test]
fn an_enemys_threat_area_is_tinted_until_hidden() {
    let mut c = ctx();
    let mut s = quick();
    let brigand = Pos::new(8, 2);
    s.cursor.jump(brigand);
    assert!(s.scene(&c).tints_at(brigand).is_empty());
    step(&mut s, &mut c, &[Action::Confirm]);
    // The brigand's tile and one it can reach.
    let shown = s.scene(&c);
    for pos in [brigand, Pos::new(8, 7)] {
        assert_eq!(shown.tints_at(pos), [RangeKind::Attack], "{pos:?}");
    }
    step(&mut s, &mut c, &[Action::Confirm]);
    assert!(s.scene(&c).tints_at(brigand).is_empty());
}

#[test]
fn a_pending_move_after_an_attack_is_chosen_on_the_map() {
    let mut c = ctx();
    let state = vaulted(&c);
    let tiles = state.move_after_tiles();
    let mut s = BattleScreen::new(state);
    let Mode::MoveAfter { unit, .. } = s.mode() else {
        panic!("{:?}", s.mode());
    };
    assert_eq!(*unit, UnitId(3));
    // Its tiles are in a move range.
    assert_eq!(s.scene(&c).tints_at(tiles[0]), [RangeKind::Move]);
    assert!(quick().scene(&c).tints_at(tiles[0]).is_empty());
    s.cursor.jump(Pos::new(8, 4));
    assert_eq!(s.help(&c), "arrows move · f stay");
    s.cursor.jump(tiles[0]);
    assert_eq!(s.help(&c), "arrows move · f move here");
    s.cursor.jump(Pos::new(0, 0));
    assert_eq!(s.help(&c), "arrows move");
    // Cancel doesn't leave; Confirm on a tile moves there.
    step(&mut s, &mut c, &[Action::Cancel]);
    assert!(matches!(s.mode(), Mode::MoveAfter { .. }));
    s.cursor.jump(tiles[0]);
    step(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.mode(), &Mode::default());
    let archer = s.state().unit(UnitId(3)).unwrap();
    assert_eq!((archer.pos, archer.acted), (tiles[0], true));
}
