//! Harness tests of attacking (ticket 0404): swapping weapons, targeting,
//! the forecast against `core`'s numbers, the combat playback (Cancel skips, fast
//! forward, falls) and the battle it leaves.

use insta::assert_snapshot;
use trpg_core::{
    BattleState, Command, Equipped, Event, Pos, Side, StatValue, UnitAction, UnitId, forecast,
};

use super::BattleScreen;
use super::playback::{Beat, Playback, TIMINGS};
use super::progress::PROGRESS_TIMINGS;
use super::testing::skirmish;
use super::view::{BattleView, ForecastView, ProgressView};
use crate::harness::{FRAME_DT, Harness};
use crate::screen::tests::ctx;
use crate::words::Words;

/// Where the lord attacks from.
const DEST: Pos = Pos::new(7, 2);

/// The brigand's tile.
const BRIGAND_AT: Pos = Pos::new(8, 2);

/// The skirmish in the harness.
fn harness(brigand_hp: StatValue) -> Harness {
    Harness::with_screen(Box::new(BattleScreen::new(skirmish(&ctx(), brigand_hp))))
}

/// The forecast on screen.
fn shown_forecast(h: &Harness) -> ForecastView {
    view(h).forecast.expect("a forecast")
}

/// The forecast's two sides' names: `Test Lord | Brigand`.
fn names(h: &Harness) -> String {
    let [a, b] = shown_forecast(h).sides;
    format!("{} | {}", a.name, b.name)
}

/// What the forecast's two sides fight with: `Iron Sword | Iron Axe`.
fn weapons(h: &Harness) -> String {
    let [a, b] = shown_forecast(h).sides;
    format!("{} | {}", a.weapon, b.weapon)
}

/// What the battle on `h` shows (ADR-0054).
fn view(h: &Harness) -> BattleView {
    h.battle().expect("a battle").view(h.game().ctx())
}

/// The help line.
fn help(h: &Harness) -> String {
    view(h).help
}

/// The message line; empty with none.
fn message(h: &Harness) -> String {
    view(h).message.map(|m| m.text).unwrap_or_default()
}

/// The name of the unit the side panel shows; empty with none.
fn hovered(h: &Harness) -> String {
    let unit = view(h).panel.and_then(|p| p.unit);
    unit.map(|u| u.name).unwrap_or_default()
}

/// The HP the side panel shows of its unit (`HP 9/20`).
fn hovered_hp(h: &Harness) -> String {
    let unit = view(h).panel.and_then(|p| p.unit).expect("a unit");
    format!("{} {}/{}", unit.hp_name, unit.hp, unit.max)
}

/// The lord selected, moved one step right to (7, 2) and its action menu
/// open (focused on `Attack`).
fn lord_menu(brigand_hp: StatValue) -> Harness {
    let mut h = harness(brigand_hp);
    h.keys("f Right f").wait(0.5);
    h
}

/// The lord targeting the brigand with its iron sword (slot 0).
fn lord_on_brigand(brigand_hp: StatValue) -> Harness {
    let mut h = lord_menu(brigand_hp);
    // `Attack` opens the forecast on the raider; down goes to the brigand.
    h.keys("f Down");
    assert_eq!(names(&h), "Test Lord | Brigand");
    h
}

/// The lord's sword attack on the brigand applied to the skirmish directly:
/// the battle and events the screen should end up with.
fn expected(brigand_hp: StatValue) -> (BattleState, BattleState, Vec<Event>) {
    let before = skirmish(&ctx(), brigand_hp);
    let mut after = before.clone();
    let cmd = Command::Act {
        unit: UnitId(1),
        dest: DEST,
        action: UnitAction::Attack {
            target: UnitId(4),
            slot: 0,
            active: None,
            art: None,
        },
    };
    let events = after.apply(&cmd).unwrap_or_else(|e| panic!("{e}"));
    (before, after, events)
}

/// The playback the screen builds for [`expected`].
fn expected_playback(brigand_hp: StatValue) -> Playback {
    let (before, after, events) = expected(brigand_hp);
    Playback::new(
        Words::ENGLISH,
        &events,
        before.units(),
        after.fallen(),
        TIMINGS,
    )
    .expect("a combat")
}

/// Seconds the EXP bar is up after a combat.
const EXP_S: f32 = PROGRESS_TIMINGS.exp_fill + PROGRESS_TIMINGS.exp_hold;

/// Seconds of playback the `f` that confirms an attack plays: its press
/// frame (Confirm held: fast) and its release frame.
fn confirm_frames() -> f32 {
    FRAME_DT * TIMINGS.fast + FRAME_DT
}

#[test]
fn attack_opens_the_forecast_swaps_the_weapons_and_cycles_targets_in_order() {
    let mut h = lord_menu(20);
    assert_eq!(help(&h), "arrows choose · f confirm · d back");
    // Two swords reach: no weapon list (0442), the forecast with the
    // equipped one on the first target in (y, x) order, the raider at
    // (7, 1); right swaps to the other sword, left back.
    h.keys("f");
    assert_eq!(
        help(&h),
        "Left/Right swap · Up/Down target · f attack · d back"
    );
    assert_eq!(names(&h), "Test Lord | Raider");
    assert_eq!(weapons(&h), "Iron Sword | Steel Axe");
    h.keys("Right");
    assert_eq!(weapons(&h), "Steel Sword | Steel Axe");
    assert_eq!(names(&h), "Test Lord | Raider");
    h.keys("Left");
    assert_eq!(weapons(&h), "Iron Sword | Steel Axe");
    // No arts list beside the forecast.
    assert!(!view(&h).shows("Flowing Cut"), "{:?}", view(&h).texts());
    for (keys, target) in [
        ("Down", "Brigand"),
        ("Down", "Raider"),
        ("Up", "Brigand"),
        ("s", "Raider"),
        ("a", "Brigand"),
    ] {
        h.keys(keys);
        assert_eq!(names(&h), format!("Test Lord | {target}"), "{keys}");
    }
    // Back to the menu, then the path.
    h.keys("d");
    assert_eq!(help(&h), "arrows choose · f confirm · d back");
    h.keys("d");
    assert_eq!(help(&h), "arrows move · f move here · d cancel");
}

#[test]
fn the_forecast_shows_cores_numbers() {
    let h = lord_on_brigand(20);
    // The forecast, from the combat maths directly.
    let state = skirmish(&ctx(), 20);
    let lord = state.unit(UnitId(1)).expect("lord");
    let brigand = state.unit(UnitId(4)).expect("brigand");
    let terrain = |pos| {
        let id = *state.map().tiles.get(pos).expect("on the map");
        state.terrain().get(id).expect("terrain")
    };
    let input = |u: &trpg_core::Unit, with: Option<&Equipped>, pos| {
        let class = state.classes().get(&u.class).expect("class");
        u.combat_input(
            class,
            state.classes(),
            state.items(),
            state.spells(),
            with,
            terrain(pos),
        )
    };
    let lord_in = input(lord, Some(&Equipped::Weapon(0)), DEST);
    let brigand_in = input(brigand, None, brigand.pos);
    let rules = state.items().combat_rules();
    let fc = forecast(&rules, &lord_in, &brigand_in, 1).expect("in range");
    let counter = fc.defender.expect("the brigand counters");
    // Hit and crit, both sides.
    let shown = shown_forecast(&h);
    let [lord_side, brigand_side] = &shown.sides;
    assert_eq!(
        (lord_side.hit, lord_side.crit),
        (Some(fc.attacker.hit), Some(fc.attacker.crit))
    );
    assert_eq!(
        (brigand_side.hit, brigand_side.crit),
        (Some(counter.hit), Some(counter.crit))
    );
    // The lord doubles with a sword: 1st strike, the counter, the
    // follow-up with the sword bonus.
    assert_eq!((fc.attacker.strikes, counter.strikes), (2, 1));
    assert!(fc.attacker.followup_damage > fc.attacker.damage);
    let dmg = |d: StatValue| format!("{d} dmg");
    let strikes: Vec<_> = shown
        .strikes
        .iter()
        .map(|s| (s.by, s.text.trim()))
        .collect();
    assert_eq!(
        strikes,
        [
            (Side::Attacker, dmg(fc.attacker.damage).as_str()),
            (Side::Defender, dmg(counter.damage).as_str()),
            (Side::Attacker, dmg(fc.attacker.followup_damage).as_str()),
        ]
    );
    let total = fc.attacker.damage + fc.attacker.followup_damage;
    let totals = |s: &super::view::ForecastSideView| s.total.map(|t| (t.damage, t.strikes));
    assert_eq!(totals(lord_side), Some((total, 2)));
    assert_eq!(totals(brigand_side), Some((counter.damage, 1)));
}

/// The lord targeting the brigand: both sides strike, the lord twice.
#[test]
fn forecast_with_a_double_and_a_counter_snapshot() {
    assert_snapshot!(lord_on_brigand(20).snapshot());
}

/// The archer, two tiles below the brigand: it hits twice and the brigand
/// can't counter. The brigand at 12 HP falls to the second arrow: a skull.
#[test]
fn forecast_with_no_counter_and_a_kill_snapshot() {
    let mut h = harness(12);
    // To the archer at (8, 4); it attacks from where it stands.
    h.keys("Right Right Down Down f f f");
    assert_eq!(names(&h), "Test Archer | Brigand");
    assert_eq!(shown_forecast(&h).no_counter.as_deref(), Some("no counter"));
    // One bow: nothing to swap, every cursor key picks the target.
    assert_eq!(help(&h), "arrows next target · f attack · d back");
    assert_snapshot!(h.snapshot());
}

/// Mid-way through the first strike's result: the lord's name back to
/// normal, `HIT -8` (or `MISS`) under the brigand, its bar draining.
#[test]
fn playback_mid_strike_snapshot() {
    let mut h = lord_on_brigand(20);
    h.keys("f");
    let pb = expected_playback(20);
    let result = pb
        .steps()
        .iter()
        .find(|s| matches!(s.beat, Beat::Result { strike: 0, .. }))
        .expect("a first strike");
    h.wait(result.start + 0.1 - confirm_frames());
    assert_eq!(help(&h), "d skip · hold f fast");
    assert_snapshot!(h.snapshot());
}

#[test]
fn after_the_playback_the_defenders_hp_is_the_battles() {
    let (_, after, events) = expected(20);
    assert!(!events.iter().any(|e| matches!(e, Event::UnitFell { .. })));
    let brigand = after.unit(UnitId(4)).expect("still standing");
    let mut h = lord_on_brigand(20);
    h.keys("f");
    h.wait(expected_playback(20).total());
    // The lord's EXP bar, then browsing again, the cursor on the brigand.
    assert_eq!(help(&h), "f skip · hold f fast");
    let progress = view(&h).progress;
    assert!(
        matches!(&progress, Some(ProgressView::Exp(e)) if e.name == "Test Lord"),
        "{progress:?}"
    );
    h.wait(EXP_S);
    assert_eq!(
        help(&h),
        "arrows move · f range · e info · s next unit · r rewind · d menu · Space end turn"
    );
    let hp = format!("HP {}/{}", brigand.hp, brigand.stats.hp);
    assert_eq!(hovered_hp(&h), hp);
    // The lord has acted, at (7, 2).
    let lord = h.unit_at(DEST).expect("the lord");
    assert_eq!((lord.label.as_str(), lord.acted), ("Lo", true));
    assert_eq!(message(&h), "");
}

#[test]
fn a_kill_fades_the_unit_out_with_a_message_and_removes_it() {
    let (_, after, events) = expected(3);
    assert!(events.contains(&Event::UnitFell { unit: UnitId(4) }));
    assert!(after.unit(UnitId(4)).is_none());
    let pb = expected_playback(3);
    let fall = pb
        .steps()
        .iter()
        .find(|s| matches!(s.beat, Beat::Fall { .. }))
        .expect("a fall");
    let mut h = lord_on_brigand(3);
    h.keys("f");
    // Just before the fall: the brigand at 0 HP, still standing.
    h.wait(fall.start - 0.05 - confirm_frames());
    let brigand = h.unit_at(BRIGAND_AT).expect("the brigand");
    assert_eq!((brigand.label.as_str(), brigand.hp.0), ("Br", 0));
    assert!(brigand.fade.abs() < f32::EPSILON, "{}", brigand.fade);
    assert_eq!(message(&h), "");
    // During it: the message.
    h.wait(0.3);
    assert_eq!(message(&h), "Brigand has fallen.");
    // Faded, while the box stays up: nobody on its tile in the panel.
    h.wait(fall.len);
    assert_eq!(help(&h), "d skip · hold f fast");
    assert_eq!(hovered(&h), "");
    // Afterwards: nobody where it stood, nor in the panel.
    h.wait(pb.total());
    assert_eq!(h.unit_at(BRIGAND_AT), None);
    assert_eq!(hovered(&h), "");
    assert_eq!(message(&h), "");
    assert_eq!(
        help(&h),
        "arrows move · f menu · s next unit · r rewind · d menu · Space end turn"
    );
}

#[test]
fn cancel_skips_the_playback_and_the_next_keys_work() {
    let mut h = lord_on_brigand(20);
    // A Confirm tap doesn't skip (holding it only speeds up).
    h.keys("f f");
    assert_eq!(help(&h), "d skip · hold f fast");
    h.keys("d");
    // The EXP bar: Cancel finishes it too (and it closes by itself).
    assert_eq!(help(&h), "f skip · hold f fast");
    h.keys("d");
    // Browsing already: the keys after the skip move the cursor.
    assert_eq!(
        help(&h),
        "arrows move · f range · e info · s next unit · r rewind · d menu · Space end turn"
    );
    h.keys("Left");
    assert_eq!(hovered(&h), "Test Lord");
    h.keys("s");
    assert_eq!(hovered(&h), "Test Archer");
    h.keys("f");
    assert_eq!(help(&h), "arrows move · f move here · d cancel");
}

#[test]
fn holding_confirm_plays_four_times_as_fast() {
    let total = expected_playback(20).total();
    let quarter = (total + EXP_S) / TIMINGS.fast + 0.1;
    let mut slow = lord_on_brigand(20);
    slow.keys("f").wait(quarter);
    assert_eq!(help(&slow), "d skip · hold f fast");
    let mut fast = lord_on_brigand(20);
    fast.keys("f").hold("f", quarter);
    assert_eq!(
        help(&fast),
        "arrows move · f range · e info · s next unit · r rewind · d menu · Space end turn"
    );
    // The same battle either way: the brigand's HP matches.
    slow.wait(total + EXP_S);
    assert_eq!(hovered_hp(&slow), hovered_hp(&fast));
}

#[test]
fn pointing_at_an_enemy_walks_there_and_opens_its_forecast() {
    let mut h = harness(20);
    // Select the lord, steer to (7, 2), point at the raider above it.
    h.keys("f Right Up");
    assert_eq!(help(&h), "arrows move · f attack · d cancel");
    // Confirm walks, then the forecast on the raider with the equipped
    // sword; left and right swap to the other sword (0430).
    h.keys("f").wait(0.5);
    assert_eq!(
        help(&h),
        "Left/Right swap · Up/Down target · f attack · d back"
    );
    assert_eq!(names(&h), "Test Lord | Raider");
    assert_eq!(weapons(&h), "Iron Sword | Steel Axe");
    h.keys("Right");
    assert_eq!(weapons(&h), "Steel Sword | Steel Axe");
    assert_eq!(names(&h), "Test Lord | Raider");
    // Cancel goes back through the menu to the path, the cursor on the
    // lord's tile.
    h.keys("d d");
    assert_eq!(help(&h), "arrows move · f move here · d cancel");
}
