//! Tests of [`action_state`]: two units, `A` and `B`, and lists of walks
//! and strikes written by hand.

use proptest::prelude::*;
use trpg_core::{ClassId, Faction};

use super::*;
use crate::map_view::UnitEffects;
use crate::map_view::sprite::units::toward;

const A: UnitId = UnitId(1);
const B: UnitId = UnitId(2);

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

fn unit(id: UnitId, pos: Pos, hp: (StatValue, StatValue)) -> UnitView {
    UnitView {
        id,
        pos,
        faction: Faction::Enemy,
        label: "Br".into(),
        class: ClassId("brigand".into()),
        character: None,
        acted: false,
        hp,
        effects: UnitEffects::default(),
        fade: 0.0,
        highlight: false,
        facing: Facing::Down,
        frame: STANDING_FRAME,
        offset: (0.0, 0.0),
        flash: 0.0,
    }
}

/// `A` at `a` with 20 of 20 HP and `B` at `b` with 30 of 30.
fn pair(a: Pos, b: Pos) -> Vec<UnitView> {
    vec![unit(A, a, (20, 20)), unit(B, b, (30, 30))]
}

fn walk(at_ms: u32, unit: UnitId, path: &[(i32, i32)], ms: u32) -> ActionDef {
    let path = path.iter().map(|&(x, y)| p(x, y)).collect();
    let act = Act::Walk { unit, path, ms };
    ActionDef { at_ms, act }
}

fn strike(at_ms: u32, unit: UnitId, target: UnitId, hp_left: u8) -> ActionDef {
    let act = Act::Strike {
        unit,
        target,
        hp_left,
    };
    ActionDef { at_ms, act }
}

/// The unit `id` among `units`.
fn of(units: &[UnitView], id: UnitId) -> &UnitView {
    units.iter().find(|u| u.id == id).unwrap()
}

fn near(a: (f32, f32), b: (f32, f32)) -> bool {
    (a.0 - b.0).abs() < 1e-4 && (a.1 - b.1).abs() < 1e-4
}

/// Up two tiles, round a corner, right two tiles: four tiles in two
/// seconds, from one second in.
const BEND: [(i32, i32); 5] = [(3, 5), (3, 4), (3, 3), (4, 3), (5, 3)];

#[test]
fn a_unit_walks_its_path_at_an_even_speed_and_stays_at_its_end() {
    let start = pair(p(3, 5), p(8, 3));
    let actions = [walk(1000, A, &BEND, 2000)];
    let at = |clock| action_state(&start, &actions, clock);
    // Before the walk: as they start.
    assert_eq!(at(0), start);
    assert_eq!(at(999), start);
    // (tile, offset, facing, frame) of the walker.
    let walker = |clock| {
        let units = at(clock);
        // The other unit is never touched.
        assert_eq!(of(&units, B), &start[1], "{clock}");
        assert_eq!(units.len(), 2);
        let a = of(&units, A).clone();
        assert_eq!(
            (a.hp, a.fade.to_bits(), a.flash.to_bits()),
            ((20, 20), 0, 0)
        );
        (a.pos, a.offset, a.facing, a.frame)
    };
    let shown = |clock, pos: Pos, offset: (f32, f32), facing: Facing, frame: u8| {
        let got = walker(clock);
        assert_eq!((got.0, got.2, got.3), (pos, facing, frame), "{clock}");
        assert!(near(got.1, offset), "{clock}: {:?}", got.1);
    };
    // It sets off: turned, a foot forward, still on its tile. Five frames
    // show per tile (one every 0.1 s, half a second a tile).
    shown(1000, p(3, 5), (0.0, 0.0), Facing::Up, 0);
    shown(1250, p(3, 5), (0.0, -0.5), Facing::Up, 2);
    shown(1499, p(3, 5), (0.0, -0.998), Facing::Up, 0);
    shown(1500, p(3, 4), (0.0, 0.0), Facing::Up, 1);
    shown(1850, p(3, 4), (0.0, -0.7), Facing::Up, 0);
    // The corner: on its tile, turned the new way.
    shown(2000, p(3, 3), (0.0, 0.0), Facing::Right, 2);
    shown(2150, p(3, 3), (0.3, 0.0), Facing::Right, 1);
    shown(2750, p(4, 3), (0.5, 0.0), Facing::Right, 1);
    shown(2999, p(4, 3), (0.998, 0.0), Facing::Right, 1);
    // Arrived: standing on the last tile, facing the camera, for good.
    for clock in [3000, 3001, 60_000, u64::MAX] {
        shown(clock, p(5, 3), (0.0, 0.0), Facing::Down, STANDING_FRAME);
    }
}

#[test]
fn a_walk_goes_any_way_and_the_next_one_takes_over() {
    let start = pair(p(3, 5), p(8, 3));
    let actions = [
        walk(0, A, &[(3, 5), (2, 5)], 1000),
        walk(1000, A, &[(2, 5), (2, 6), (2, 7)], 1000),
        walk(1500, B, &[(8, 3), (8, 2)], 100),
    ];
    let at = |clock| action_state(&start, &actions, clock);
    let a = |clock| of(&at(clock), A).clone();
    assert_eq!((a(500).pos, a(500).facing), (p(3, 5), Facing::Left));
    assert!(near(a(500).offset, (-0.5, 0.0)));
    // The first walk is over as the second starts, from its end.
    assert_eq!((a(1000).pos, a(1000).facing), (p(2, 5), Facing::Down));
    assert!(near(a(1250).offset, (0.0, 0.5)));
    assert_eq!(a(1250).pos, p(2, 5));
    assert_eq!(a(1750).pos, p(2, 6));
    assert_eq!((a(2000).pos, a(2000).offset), (p(2, 7), (0.0, 0.0)));
    // Each unit walks on its own.
    assert_eq!(of(&at(1499), B).pos, p(8, 3));
    assert_eq!(of(&at(1550), B).pos, p(8, 3));
    assert_eq!(of(&at(1550), B).facing, Facing::Up);
    assert_eq!(of(&at(1600), B).pos, p(8, 2));
    assert_eq!(of(&at(1600), A).pos, p(2, 6));
    // A walk of no tiles, or by a unit that isn't there, changes nothing.
    let odd = [walk(0, A, &[], 1000), walk(0, UnitId(9), &BEND, 1000)];
    assert_eq!(action_state(&start, &odd, 500), start);
    // No time at all: it is at the end at once.
    let instant = [walk(0, A, &BEND, 0)];
    assert_eq!(of(&action_state(&start, &instant, 0), A).pos, p(5, 3));
}

#[test]
fn a_strike_lunges_then_its_target_flashes_with_less_hp() {
    let start = pair(p(7, 3), p(8, 3));
    let actions = [strike(1000, A, B, 40)];
    let at = |clock| action_state(&start, &actions, clock);
    assert_eq!(at(999), start);
    // (offset across, facing) of the striker; (hp, flash) of its target.
    let shown = |clock, lean: f32, facing: Facing, hp: StatValue, flash: f32| {
        let units = at(clock);
        let (a, b) = (of(&units, A), of(&units, B));
        assert!(near(a.offset, (lean, 0.0)), "{clock}: {:?}", a.offset);
        assert_eq!(
            (a.facing, a.pos, a.hp),
            (facing, p(7, 3), (20, 20)),
            "{clock}"
        );
        assert_eq!(
            (b.hp, b.pos, b.offset),
            ((hp, 30), p(8, 3), (0.0, 0.0)),
            "{clock}"
        );
        assert!((b.flash - flash).abs() < 1e-4, "{clock}: {}", b.flash);
        assert_eq!((b.fade.to_bits(), a.flash.to_bits()), (0, 0), "{clock}");
        assert_eq!(units.len(), 2);
    };
    // It turns to its target and leans out; nothing has landed yet.
    shown(1000, 0.0, Facing::Right, 30, 0.0);
    shown(1100, LUNGE_TILES / 2.0, Facing::Right, 30, 0.0);
    shown(1199, LUNGE_TILES * 0.995, Facing::Right, 30, 0.0);
    // Half way: furthest out, and the blow lands: 40% of 30 HP is 12.
    shown(1200, LUNGE_TILES, Facing::Right, 12, 1.0);
    shown(1250, LUNGE_TILES * 0.75, Facing::Right, 12, 0.75);
    shown(1300, LUNGE_TILES / 2.0, Facing::Right, 12, 0.5);
    shown(1399, LUNGE_TILES * 0.005, Facing::Right, 12, 0.005);
    // Over: the striker stands as before, the target keeps its HP.
    for clock in [1400, 1401, 90_000] {
        shown(clock, 0.0, Facing::Down, 12, 0.0);
    }
    assert_eq!(
        (LUNGE_MS, FLASH_MS, FALL_MS),
        (400, 200, 500),
        "the times above"
    );
}

#[test]
fn a_lunge_goes_toward_where_the_target_stands() {
    let leans = |a: Pos, b: Pos| {
        let units = action_state(&pair(a, b), &[strike(0, A, B, 50)], 200);
        let a = of(&units, A);
        (a.offset, a.facing)
    };
    let (far, quarter) = (LUNGE_TILES, LUNGE_TILES);
    assert_eq!(leans(p(5, 5), p(4, 5)), ((-far, 0.0), Facing::Left));
    assert_eq!(leans(p(5, 5), p(5, 4)), ((0.0, -far), Facing::Up));
    assert_eq!(leans(p(5, 5), p(5, 6)), ((0.0, far), Facing::Down));
    // From afar (an archer): the same few pixels, along the line to it.
    let ((x, y), facing) = leans(p(3, 5), p(8, 3));
    assert!(near((x, y), (quarter, -quarter * 0.4)), "{x} {y}");
    assert_eq!(facing, Facing::Right);
    let ((x, y), _) = leans(p(3, 5), p(2, 1));
    assert!(near((x, y), (-quarter * 0.25, -quarter)), "{x} {y}");
    // On its target's tile there is no way to lean.
    assert_eq!(leans(p(5, 5), p(5, 5)), ((0.0, 0.0), Facing::Down));
    // The target has walked: toward where it is now.
    let start = pair(p(5, 5), p(8, 5));
    let actions = [
        walk(0, B, &[(8, 5), (7, 5), (6, 5), (6, 6), (5, 6)], 400),
        strike(400, A, B, 50),
    ];
    let a = of(&action_state(&start, &actions, 600), A).clone();
    assert_eq!((a.offset, a.facing), ((0.0, LUNGE_TILES), Facing::Down));
    // A unit in the middle of a walk keeps walking: it doesn't lean. Its
    // blow still lands.
    let actions = [walk(0, A, &[(5, 5), (5, 4)], 1000), strike(300, A, B, 50)];
    let units = action_state(&start, &actions, 500);
    let a = of(&units, A);
    assert_eq!((a.offset, a.facing), ((0.0, -0.5), Facing::Up));
    assert_eq!(of(&units, B).hp, (15, 30));
    // A strike by or at a unit that isn't there changes nothing.
    let odd = [strike(0, UnitId(9), B, 0), strike(0, A, UnitId(9), 0)];
    assert_eq!(action_state(&start, &odd, 0), start);
    let after = action_state(&start, &odd, 5000);
    assert_eq!(after.len(), 2);
    assert_eq!(of(&after, B), &start[1]);
}

#[test]
fn hp_left_is_a_part_of_full_hp_rounded_up_and_never_more_than_it_had() {
    let left = |hp: (StatValue, StatValue), hp_left: u8| {
        let start = [unit(A, p(0, 0), (20, 20)), unit(B, p(1, 0), hp)];
        let units = action_state(&start, &[strike(0, A, B, hp_left)], 10_000);
        of(&units, B).hp
    };
    assert_eq!(left((30, 30), 100), (30, 30));
    assert_eq!(left((30, 30), 50), (15, 30));
    assert_eq!(left((30, 30), 34), (11, 30)); // 10.2
    assert_eq!(left((30, 30), 33), (10, 30)); // 9.9
    assert_eq!(left((30, 30), 1), (1, 30)); // 0.3: not yet fallen
    assert_eq!(left((7, 7), 99), (7, 7));
    // Already hurt as the battle starts: a strike never heals.
    assert_eq!(left((10, 30), 50), (10, 30));
    assert_eq!(left((10, 30), 20), (6, 30));
    // Each strike that has landed, in turn: the last one's HP.
    let start = pair(p(0, 0), p(1, 0));
    let both = [strike(0, A, B, 60), strike(1000, A, B, 10)];
    let hp = |clock| of(&action_state(&start, &both, clock), B).hp.0;
    assert_eq!([hp(199), hp(200), hp(1199), hp(1200)], [30, 18, 18, 3]);
    // The second blow flashes anew.
    let flash = |clock| of(&action_state(&start, &both, clock), B).flash;
    assert!(flash(400) == 0.0 && flash(1199) == 0.0);
    assert!((flash(1200) - 1.0).abs() < 1e-6);
}

#[test]
fn a_unit_left_with_no_hp_fades_out_and_is_gone() {
    let start = pair(p(7, 3), p(8, 3));
    let actions = [strike(1000, A, B, 60), strike(2000, A, B, 0)];
    let at = |clock| action_state(&start, &actions, clock);
    // (hp, fade, flash) of the struck unit.
    let b = |clock| {
        let units = at(clock);
        let b = of(&units, B);
        (b.hp.0, b.fade, b.flash)
    };
    // The first blow doesn't fell it.
    assert_eq!(b(1999), (18, 0.0, 0.0));
    assert_eq!(b(2199), (18, 0.0, 0.0));
    // The second lands: no HP, a flash, and it starts to fade.
    assert_eq!(b(2200), (0, 0.0, 1.0));
    assert_eq!(b(2300), (0, 0.2, 0.5));
    assert_eq!(b(2450), (0, 0.5, 0.0));
    let (hp, fade, _) = b(2699);
    assert!(hp == 0 && fade > 0.99 && fade < 1.0, "{fade}");
    // Half a second after the blow it is gone, and stays gone.
    for clock in [2700, 2701, 3000, 1_000_000] {
        let units = at(clock);
        assert_eq!(units.len(), 1, "{clock}");
        assert_eq!(units[0].id, A);
    }
    // The striker stands as it started once its lunge is over.
    assert_eq!(at(3000), [start[0].clone()]);
}

/// A path from (20, 20): each leg a way to go and how many tiles.
fn path_of(legs: &[(u8, u8)]) -> Vec<(i32, i32)> {
    let mut at = (20, 20);
    let mut path = vec![at];
    for &(way, tiles) in legs {
        let (dx, dy) = [(1, 0), (-1, 0), (0, 1), (0, -1)][usize::from(way % 4)];
        for _ in 0..tiles {
            at = (at.0 + dx, at.1 + dy);
            path.push(at);
        }
    }
    path
}

proptest! {
    /// At any time a walking unit is on a tile of its path, heading for
    /// another tile of it (or standing), never further than a tile from
    /// the one it is on; before the walk it is at the start, after it at
    /// the end.
    #[test]
    fn a_walker_is_always_on_its_path(
        legs in prop::collection::vec((0u8..4, 1u8..4), 1..5),
        at_ms in 0u32..3000,
        ms in 1u32..5000,
        clock in 0u64..10_000,
    ) {
        let path = path_of(&legs);
        let start = pair(p(20, 20), p(0, 0));
        let units = action_state(&start, &[walk(at_ms, A, &path, ms)], clock);
        let a = of(&units, A);
        let on_path = |pos: Pos| path.contains(&(pos.x, pos.y));
        prop_assert!(on_path(a.pos), "{:?}", a.pos);
        prop_assert!(on_path(toward(a)), "{:?} to {:?}", a.pos, toward(a));
        prop_assert!(a.offset.0.abs() <= 1.0 && a.offset.1.abs() <= 1.0);
        // Along a row or a column, never both.
        prop_assert!(a.offset.0 == 0.0 || a.offset.1 == 0.0);
        let end = path[path.len() - 1];
        if clock < u64::from(at_ms) {
            prop_assert_eq!(a, &start[0]);
        }
        if clock >= u64::from(at_ms) + u64::from(ms) {
            prop_assert_eq!((a.pos, a.offset), (p(end.0, end.1), (0.0, 0.0)));
            prop_assert_eq!((a.facing, a.frame), (Facing::Down, STANDING_FRAME));
        }
        prop_assert_eq!(of(&units, B), &start[1]);
    }

    /// As the clock grows a unit's HP never rises, a unit that is gone
    /// stays gone, and a flash or a fade is a part from 0 to 1.
    #[test]
    fn hp_never_rises_as_the_clock_grows(
        blows in prop::collection::vec((0u32..3000, 0u8..=100), 1..6),
        clocks in (0u64..6000, 0u64..6000),
    ) {
        // In order of time, each leaving no more than the one before; a
        // blow that leaves none is the last.
        let mut blows = blows;
        blows.sort_by_key(|&(at, _)| at);
        let mut left = 100;
        let mut actions = Vec::new();
        for (at, hp_left) in blows {
            left = left.min(hp_left);
            actions.push(strike(at, A, B, left));
            if left == 0 {
                break;
            }
        }
        let start = pair(p(7, 3), p(8, 3));
        let (sooner, later) = (clocks.0.min(clocks.1), clocks.0.max(clocks.1));
        let b = |clock| {
            let units = action_state(&start, &actions, clock);
            units.into_iter().find(|u| u.id == B)
        };
        match (b(sooner), b(later)) {
            (Some(before), Some(after)) => {
                prop_assert!(after.hp.0 <= before.hp.0);
                prop_assert!(after.fade >= before.fade);
                for u in [before, after] {
                    prop_assert!((0..=30).contains(&u.hp.0));
                    prop_assert!((0.0..=1.0).contains(&u.flash));
                    prop_assert!((0.0..1.0).contains(&u.fade));
                    prop_assert_eq!(u.pos, p(8, 3));
                }
            }
            (None, after) => prop_assert!(after.is_none()),
            (Some(_), None) => {}
        }
    }
}
