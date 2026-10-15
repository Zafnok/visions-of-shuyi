//! Tests of pointing at a hurt ally to heal it (ticket 0451,
//! `docs/design/turn-structure.md`, *Pointing at an ally*): the tile the
//! path is aimed at (the same rule as an enemy's), the heal list Confirm
//! opens there, the heal's preview on that ally, and that an ally at full
//! HP is no target.

use insta::assert_snapshot;
use trpg_core::{BattleState, CastTarget, Objective, Pos, SpellId, StatValue, UnitId};

use super::BattleScreen;
use super::attack::aim_tile;
use super::magic::{can_heal, heal_choices, heal_tile};
use super::mode::{MenuEntry, Mode, Selection, step};
use super::testing::{battle_with, quick_units};
use crate::harness::Harness;
use crate::input::Action;
use crate::screen::Ctx;
use crate::screen::tests::ctx;

/// The Quick Battle's mage (Fire, Frost, Heal), at (3, 6).
const MAGE: UnitId = UnitId(8);

/// The Quick Battle's archer.
const ARCHER: UnitId = UnitId(3);

/// The Quick Battle's knight.
const KNIGHT: UnitId = UnitId(2);

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The Quick Battle with the archer moved to (1, 6), two tiles left of the
/// mage, at `archer_hp` HP; with `vigil` the mage also knows Vigil Light
/// (Maud's heal, range 1-2), and with `knight_hurt` the knight stands at
/// (2, 7), 5 HP down.
fn field(c: &Ctx, archer_hp: StatValue, vigil: bool, knight_hurt: bool) -> BattleState {
    let (map, mut units) = quick_units(c);
    units[2].pos = p(1, 6);
    units[2].hp = archer_hp.min(units[2].stats.hp);
    if knight_hurt {
        units[1].pos = p(2, 7);
        units[1].hp -= 5;
    }
    if vigil {
        units[7].learned.insert(SpellId::new("vigil_light"));
    }
    battle_with(c, map, units, Objective::Rout { turn_limit: None })
}

fn archer_max(state: &BattleState) -> StatValue {
    state.unit(ARCHER).unwrap().stats.hp
}

/// The help line of the harness's screen (its first 90 cells, before the
/// debug key's note).
fn help(h: &Harness) -> String {
    let buf = h.game().buffer();
    (0..90)
        .map(|x| {
            buf.get(x, super::layout::HELP_ROW)
                .map_or(' ', |cell| cell.glyph)
        })
        .collect::<String>()
        .trim()
        .to_owned()
}

/// The lines of the open spell list and whether each can be picked.
fn heal_list(h: &Harness) -> Vec<(SpellId, bool)> {
    match h.battle().unwrap().mode() {
        Mode::SpellMenu { choices, menu, .. } => choices
            .iter()
            .zip(menu.items())
            .map(|(c, item)| (c.spell.clone(), item.enabled))
            .collect(),
        m => panic!("{m:?}"),
    }
}

#[test]
fn the_shared_tile_choice_keeps_the_path_or_takes_the_cheapest_tile() {
    let c = ctx();
    let s = field(&c, 5, false, false);
    let mut sel = Selection::new(&s, MAGE).unwrap();
    // Nothing reaches from anywhere: no tile.
    assert_eq!(aim_tile(&sel, |_| false), None);
    // Where the unit stands reaches: it stays.
    assert_eq!(aim_tile(&sel, |_| true), Some(p(3, 6)));
    // Heal reaches 1 tile: the cheapest tile next to the archer is the
    // road at (2, 6), one step.
    assert_eq!(heal_tile(&s, &sel, ARCHER), Some(p(2, 6)));
    // A path the player drew that already ends next to the archer is kept
    // (the forest above it, dearer than the road).
    sel.steer(p(1, 5), &s);
    assert_eq!(sel.dest(), p(1, 5));
    assert_eq!(heal_tile(&s, &sel, ARCHER), Some(p(1, 5)));
    // With Vigil Light (range 1-2) the mage already reaches from where it
    // stands, two tiles away: the closest tile any heal reaches from.
    let v = field(&c, 5, true, false);
    let sel = Selection::new(&v, MAGE).unwrap();
    assert_eq!(heal_tile(&v, &sel, ARCHER), Some(p(3, 6)));
    assert!(can_heal(&v, MAGE, p(3, 6), ARCHER));
    assert!(!can_heal(&s, MAGE, p(3, 6), ARCHER));
}

#[test]
fn only_a_hurt_ally_a_heal_reaches_is_healed_by_pointing() {
    let c = ctx();
    // At full HP: no tile, from anywhere.
    let full = field(&c, 99, true, false);
    let sel = Selection::new(&full, MAGE).unwrap();
    assert_eq!(heal_tile(&full, &sel, ARCHER), None);
    // An enemy is never healed, and the mage can't heal itself.
    let s = field(&c, 5, true, false);
    let sel = Selection::new(&s, MAGE).unwrap();
    assert_eq!(heal_tile(&s, &sel, UnitId(5)), None);
    assert!(!can_heal(&s, MAGE, p(3, 6), MAGE));
    // A unit with no heal spell (the lord) heals nobody by pointing.
    let lord = Selection::new(&s, UnitId(1)).unwrap();
    assert_eq!(heal_tile(&s, &lord, ARCHER), None);
    // No uses left of either heal: nobody.
    let mut c = ctx();
    for id in ["heal", "vigil_light"] {
        let spells = &mut c.content.spells.spells;
        spells.get_mut(&SpellId::new(id)).unwrap().uses = 0;
    }
    let spent = field(&c, 5, true, false);
    let sel = Selection::new(&spent, MAGE).unwrap();
    assert_eq!(heal_tile(&spent, &sel, ARCHER), None);
}

#[test]
fn the_heal_list_has_every_heal_and_starts_each_on_the_ally_pointed_at() {
    let c = ctx();
    // From (2, 6): the archer left and the hurt knight below, both in
    // Heal's and Vigil Light's reach; the aimed one comes first.
    let s = field(&c, 5, true, true);
    let lines = heal_choices(&s, MAGE, p(2, 6), KNIGHT);
    let spells: Vec<&str> = lines.iter().map(|l| l.spell.0.as_str()).collect();
    assert_eq!(spells, ["heal", "vigil_light"]);
    for line in &lines {
        let targets = [CastTarget::Unit(KNIGHT), CastTarget::Unit(ARCHER)];
        assert_eq!(line.targets, targets, "{}", line.spell.0);
    }
    let lines = heal_choices(&s, MAGE, p(2, 6), ARCHER);
    assert_eq!(
        lines[0].targets,
        [CastTarget::Unit(ARCHER), CastTarget::Unit(KNIGHT)]
    );
    // From two tiles away only Vigil Light reaches the archer: Heal is
    // dimmed, even though it reaches the knight from there.
    let lines = heal_choices(&s, MAGE, p(3, 6), ARCHER);
    assert!(lines[0].targets.is_empty());
    assert_eq!(lines[1].targets[0], CastTarget::Unit(ARCHER));
}

/// Acceptance: a healer selected, the cursor on a hurt ally two tiles
/// away: the path ends next to it; Confirm walks there and opens the heal
/// list; Confirm opens the heal's preview on that ally; Confirm heals it
/// and the healer has acted.
#[test]
fn harness_pointing_at_a_hurt_ally_walks_next_to_it_and_heals_it() {
    let c = ctx();
    let state = field(&c, 5, false, false);
    let max = archer_max(&state);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state.clone())));
    // Select the mage, then the cursor onto the archer.
    h.keys("Down f Left Left");
    assert_eq!(h.cursor_tile(), Some(p(1, 6)));
    assert_eq!(h.path(), [p(3, 6), p(2, 6)]);
    assert_eq!(help(&h), "arrows move · f heal · d cancel");
    // Confirm: the walk, then the list of the mage's heals (not Fire or
    // Frost).
    h.keys("f").wait(0.5);
    assert_eq!(heal_list(&h), [(SpellId::new("heal"), true)]);
    assert_eq!(h.battle().unwrap().mode().drawn_pos(MAGE), Some(p(2, 6)));
    // Confirm: Heal's preview on the archer (Heal 10 + Mag 6, capped).
    h.keys("f");
    let healed = (5 + 16).min(max);
    let line = format!("Heal on Test Archer: HP 5 → {healed}");
    let buf = h.game().buffer();
    let shown = (0..i32::from(crate::console::ConsoleSpec::PC.rows)).any(|y| {
        let row: String = (0..i32::from(crate::console::ConsoleSpec::PC.cols))
            .map(|x| buf.get(x, y).map_or(' ', |cell| cell.glyph))
            .collect();
        row.contains(&line)
    });
    assert!(shown, "{line}");
    assert_eq!(h.cursor_tile(), Some(p(1, 6)));
    assert_snapshot!(h.snapshot());
    // Confirm heals.
    h.keys("f").wait(3.0);
    let after = h.battle().unwrap().state();
    assert_eq!(after.unit(ARCHER).unwrap().hp, healed);
    let mage = after.unit(MAGE).unwrap();
    assert_eq!((mage.pos, mage.acted), (p(2, 6), true));
}

#[test]
fn harness_up_and_down_step_to_the_other_hurt_ally_and_cancel_backs_out() {
    let c = ctx();
    let state = field(&c, 5, false, true);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state.clone())));
    h.keys("Down f Left Left f").wait(0.5);
    h.keys("f");
    let target = |h: &Harness| match h.battle().unwrap().mode() {
        Mode::CastTarget(t) => t.target(),
        m => panic!("{m:?}"),
    };
    assert_eq!(target(&h), CastTarget::Unit(ARCHER));
    h.keys("Down");
    assert_eq!(target(&h), CastTarget::Unit(KNIGHT));
    h.keys("Up");
    assert_eq!(target(&h), CastTarget::Unit(ARCHER));
    // Cancel: the heal list, then the action menu on Magic, then the path.
    h.keys("d");
    assert_eq!(heal_list(&h), [(SpellId::new("heal"), true)]);
    h.keys("d");
    let Mode::ActionMenu { menu, entries, .. } = h.battle().unwrap().mode() else {
        panic!("{:?}", h.battle().unwrap().mode());
    };
    assert_eq!(entries[menu.focus()], MenuEntry::Magic);
    h.keys("d");
    assert!(matches!(h.battle().unwrap().mode(), Mode::Selected(_)));
    assert_eq!(h.battle().unwrap().state(), &state);
}

/// Maud's two heals in the list: from next to the ally both can be
/// chosen; from two tiles away only Vigil Light reaches, so Heal is dimmed
/// and the list opens on Vigil Light.
#[test]
fn harness_the_list_offers_heal_and_vigil_light() {
    let c = ctx();
    let state = field(&c, 5, true, false);
    let (heal, vigil) = (SpellId::new("heal"), SpellId::new("vigil_light"));
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state.clone())));
    // The arrow drawn through (2, 6) ends next to the archer: kept.
    h.keys("Down f Left Left");
    assert_eq!(h.path(), [p(3, 6), p(2, 6)]);
    h.keys("f").wait(0.5);
    assert_eq!(heal_list(&h), [(heal.clone(), true), (vigil.clone(), true)]);
    h.keys("Down f");
    let Mode::CastTarget(t) = h.battle().unwrap().mode() else {
        panic!("{:?}", h.battle().unwrap().mode());
    };
    assert_eq!((t.spell(), t.target()), (&vigil, CastTarget::Unit(ARCHER)));
    // Aimed from where the mage stands, two tiles away.
    let mut sel = Selection::new(&state, MAGE).unwrap();
    sel.target = Some(ARCHER);
    let (mode, _) = step(Mode::Selected(sel), Action::Confirm, p(1, 6), &state);
    let Mode::SpellMenu { menu, choices, .. } = mode else {
        panic!("{mode:?}");
    };
    let lines: Vec<(SpellId, bool)> = choices
        .iter()
        .zip(menu.items())
        .map(|(c, item)| (c.spell.clone(), item.enabled))
        .collect();
    assert_eq!(lines, [(heal, false), (vigil, true)]);
    assert_eq!(menu.focus(), 1);
}

/// Acceptance: an ally at full HP is not a target; the cursor behaves as
/// it did before (a tile the mage can't stop on).
#[test]
fn harness_an_ally_at_full_hp_is_not_pointed_at() {
    let c = ctx();
    let state = field(&c, 99, true, false);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state.clone())));
    h.keys("Down f Left Left");
    let Mode::Selected(sel) = h.battle().unwrap().mode() else {
        panic!("{:?}", h.battle().unwrap().mode());
    };
    assert_eq!(sel.target, None);
    let mut steered = Selection::new(&state, MAGE).unwrap();
    steered.steer(p(2, 6), &state);
    steered.steer(p(1, 6), &state);
    assert_eq!(sel.path, steered.path);
    assert_eq!(help(&h), "arrows move · d cancel");
    // Confirm on it does nothing.
    h.keys("f").wait(0.5);
    assert!(matches!(h.battle().unwrap().mode(), Mode::Selected(_)));
}
