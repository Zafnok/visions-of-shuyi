//! Tests of the threat arrows' list on the battle screen (ticket 0448): it
//! is worked out once for a unit and a tile and kept, and dropped when the
//! battle changes. What the scene shows while playing is in
//! `crates/ui/tests/it/threat.rs`.

use std::collections::BTreeMap;

use trpg_core::{
    AiBehavior, ClassId, Command, ItemId, Objective, Pos, Role, StatKind, Unit, UnitAction, UnitId,
    WeaponInstance, WeaponKind, WeaponRank,
};

use super::testing::{Stacked, battle_with, quick_units, skirmish_charged};
use super::{BattleScreen, KeptThreat, quick_battle, threat_enemies};
use crate::FrameInput;
use crate::harness::FRAME_DT;
use crate::input::Action;
use crate::map_view::{ThreatView, range};
use crate::screen::tests::ctx;
use crate::screen::{Ctx, Screen};

const LORD: UnitId = UnitId(1);

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The arrows the scene shows: the tile and the enemies' tiles.
fn shown(s: &BattleScreen, c: &Ctx) -> Option<(Pos, Vec<Pos>)> {
    let threat = s.scene(c).threat;
    threat.map(|ThreatView { tile, enemies }| (tile, enemies))
}

/// Makes `s` remember `enemies` as the answer for `unit` on `tile`.
fn plant(s: &BattleScreen, unit: UnitId, tile: Pos, enemies: &[Pos]) {
    *s.threats.borrow_mut() = Some(KeptThreat {
        unit,
        tile,
        enemies: enemies.to_vec(),
    });
}

/// The Quick Battle as it starts: the cursor on the lord at (3, 5), whom
/// the brigand at (7, 4) can attack.
fn quick(c: &Ctx) -> BattleScreen {
    let state = quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
    BattleScreen::new(state)
}

#[test]
fn the_enemies_that_could_attack_a_unit_on_a_tile_are_listed_by_their_tiles() {
    let c = ctx();
    let s = quick(&c);
    let state = s.state();
    assert_eq!(threat_enemies(state, LORD, p(3, 5)), [p(7, 4)]);
    // Further east: both brigands and the raider, in the battle's order.
    let all = [p(8, 2), p(7, 4), p(7, 1)];
    assert_eq!(threat_enemies(state, LORD, p(6, 5)), all);
    // The mage, where it stands: nobody.
    assert_eq!(
        threat_enemies(state, UnitId(8), p(3, 6)),
        [] as [trpg_core::Pos; 0]
    );
    // A unit the battle doesn't have: nothing, not a panic.
    assert_eq!(
        threat_enemies(state, UnitId(99), p(3, 5)),
        [] as [trpg_core::Pos; 0]
    );
}

#[test]
fn the_list_is_kept_for_its_unit_and_tile_and_worked_out_again_for_another() {
    let c = ctx();
    let s = quick(&c);
    assert!(s.threats.borrow().is_none());
    assert_eq!(shown(&s, &c), Some((p(3, 5), vec![p(7, 4)])));
    let kept = KeptThreat {
        unit: LORD,
        tile: p(3, 5),
        enemies: vec![p(7, 4)],
    };
    assert_eq!(*s.threats.borrow(), Some(kept.clone()));
    // What is kept for this unit on this tile is what is shown...
    plant(&s, LORD, p(3, 5), &[p(0, 0), p(1, 1)]);
    assert_eq!(shown(&s, &c), Some((p(3, 5), vec![p(0, 0), p(1, 1)])));
    // ...and with nobody in it, nothing is.
    plant(&s, LORD, p(3, 5), &[]);
    assert_eq!(shown(&s, &c), None);
    // Kept for another tile, or another unit: worked out again, and kept.
    plant(&s, LORD, p(3, 4), &[p(0, 0)]);
    assert_eq!(shown(&s, &c), Some((p(3, 5), vec![p(7, 4)])));
    assert_eq!(*s.threats.borrow(), Some(kept.clone()));
    plant(&s, UnitId(2), p(3, 5), &[p(0, 0)]);
    assert_eq!(shown(&s, &c), Some((p(3, 5), vec![p(7, 4)])));
    assert_eq!(*s.threats.borrow(), Some(kept));
}

#[test]
fn a_command_drops_the_kept_list() {
    let c = ctx();
    let mut s = quick(&c);
    plant(&s, LORD, p(3, 5), &[p(0, 0)]);
    assert_eq!(shown(&s, &c), Some((p(3, 5), vec![p(0, 0)])));
    // The archer waits: the battle changed.
    s.send(&Command::Act {
        unit: UnitId(3),
        dest: p(2, 4),
        action: UnitAction::Wait,
    });
    assert_eq!(s.history().len(), 1);
    assert_eq!(shown(&s, &c), Some((p(3, 5), vec![p(7, 4)])));
}

#[test]
fn a_rewind_drops_the_kept_list() {
    let mut c = ctx();
    // The lord at (6, 2), between the raider and a brigand.
    let mut s = Stacked::new(BattleScreen::new(skirmish_charged(&c, 20, 3)));
    let real = threat_enemies(s.state(), LORD, p(6, 2));
    assert!(
        real.contains(&p(7, 1)) && real.contains(&p(8, 2)),
        "{real:?}"
    );
    assert_eq!(shown(&s, &c), Some((p(6, 2), real.clone())));
    // The archer (at (8, 4) here) waits; then the battle is rewound to
    // before that.
    s.battle_mut().send(&Command::Act {
        unit: UnitId(3),
        dest: p(8, 4),
        action: UnitAction::Wait,
    });
    plant(&s, LORD, p(6, 2), &[p(0, 0)]);
    for action in [Action::Rewind, Action::Confirm, Action::Confirm] {
        s.frame(&mut c, &[action], FRAME_DT);
    }
    assert!(s.rewind().is_none());
    assert!(s.history().is_empty());
    assert_eq!(shown(&s, &c), Some((p(6, 2), real)));
}

/// The Quick Battle with only the lord and one enemy at (5, 5): stationary,
/// unable to walk (the danger zone would still let it, ticket 0450), with an
/// iron spear (range 1), `change` applied. The cursor is on the lord.
fn spear_enemy(c: &Ctx, change: impl FnOnce(&mut Unit)) -> BattleScreen {
    let (map, mut units) = quick_units(c);
    units.retain(|u| u.id == LORD || u.pos == p(7, 4));
    let enemy = &mut units[1];
    enemy.pos = p(5, 5);
    enemy.ai = AiBehavior::Stationary;
    enemy.stats.set(StatKind::Mov, 0);
    enemy.class = ClassId("guard".to_owned());
    enemy.loadout.weapons[0] = Some(WeaponInstance {
        def: ItemId::new("iron_spear"),
        durability_left: 20,
    });
    change(enemy);
    let objective = Objective::Rout { turn_limit: None };
    BattleScreen::new(battle_with(c, map, units, objective))
}

fn press(s: &mut BattleScreen, c: &mut Ctx, actions: &[Action]) {
    for &action in actions {
        s.update(c, &FrameInput::new(vec![action], FRAME_DT, vec![]));
    }
}

#[test]
fn a_tile_only_an_art_reaches_is_in_the_danger_zone_the_arrows_and_the_shown_range() {
    let mut c = ctx();
    let two_away = p(6, 4);
    let spear_c = |u: &mut Unit| {
        u.weapon_ranks = BTreeMap::from([(WeaponKind::Spear, WeaponRank::C)]);
    };
    let pick = [Action::CursorRight, Action::CursorRight, Action::Confirm];

    let mut boss = spear_enemy(&c, |u| {
        u.role = Role::Boss;
        spear_c(u);
    });
    assert_eq!(threat_enemies(boss.state(), LORD, two_away), [p(5, 5)]);
    press(&mut boss, &mut c, &[Action::DangerZone]);
    assert!(boss.scene(&c).tinted(range::DANGER).contains(&two_away));
    press(&mut boss, &mut c, &[Action::DangerZone]);
    press(&mut boss, &mut c, &pick);
    assert!(boss.scene(&c).tinted(range::ATTACK).contains(&two_away));

    // An ordinary enemy with the same spear and rank doesn't reach it.
    let mut ordinary = spear_enemy(&c, spear_c);
    assert_eq!(
        threat_enemies(ordinary.state(), LORD, two_away),
        [] as [Pos; 0]
    );
    press(&mut ordinary, &mut c, &[Action::DangerZone]);
    assert!(!ordinary.scene(&c).tinted(range::DANGER).contains(&two_away));
    press(&mut ordinary, &mut c, &[Action::DangerZone]);
    press(&mut ordinary, &mut c, &pick);
    assert!(!ordinary.scene(&c).tinted(range::ATTACK).contains(&two_away));
}

/// The Quick Battle (a test battle) at its start: the cursor on the lead
/// (3, 5), the mage below it (3, 6), the knight at (4, 6), the archer at
/// (2, 4), and four enemies.
fn quick_screen(c: &Ctx) -> BattleScreen {
    BattleScreen::new(quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}")))
}

/// The path searches each of `actions` runs, the scene drawn after it as
/// a frame draws it, and the player's unit then under the cursor, if any.
fn searches_per_step(
    s: &mut BattleScreen,
    c: &mut Ctx,
    actions: &[Action],
) -> Vec<(u32, Option<UnitId>)> {
    use trpg_core::Faction;
    use trpg_core::movement::searches;
    let mut out = Vec::new();
    for &action in actions {
        let before = searches();
        press(s, c, &[action]);
        s.scene(c);
        let hovered = s.hovered().filter(|u| u.faction == Faction::Player);
        out.push((searches() - before, hovered.map(|u| u.id)));
    }
    out
}

/// The cursor sweeping the Quick Battle past its units, with no unit
/// selected: reaching a player unit runs no path search, as each enemy's
/// threat area is kept for the battle (ticket 1259; before, each new unit
/// searched every enemy's moves again). `bench/`'s cursor sweep times the
/// same on the biggest battle.
#[test]
fn the_cursor_reaching_a_player_unit_runs_no_path_search() {
    use Action::{CursorDown, CursorLeft, CursorRight, CursorUp};
    let mut c = ctx();
    // From the lead: the mage, the knight, the lead again, the archer.
    let sweep = [
        CursorDown,
        CursorRight,
        CursorUp,
        CursorLeft,
        CursorLeft,
        CursorUp,
    ];
    let mut s = quick_screen(&c);
    // The first frame, on the lead, keeps the battle's areas.
    s.scene(&c);
    let counts = searches_per_step(&mut s, &mut c, &sweep);
    let units = counts.iter().filter(|(_, unit)| unit.is_some()).count();
    assert_eq!(units, 4, "{counts:?}");
    assert!(counts.iter().all(|&(n, _)| n == 0), "{counts:?}");
    // Selecting the archer: its own moves, and its planner from the kept
    // areas, which searches again only the enemies whose moves come next
    // to it, never all four.
    let counts = searches_per_step(&mut s, &mut c, &[Action::Confirm]);
    assert!((1..=4).contains(&counts[0].0), "{counts:?}");
}

/// Cursor moves while the lead is selected in the Quick Battle: the threat
/// arrows reuse the enemies' searches made for it, so a move runs at most
/// 2 path searches, and most none (ticket 1251; each ran one per enemy
/// before). `bench/`'s attack times the same on the biggest battle.
#[test]
fn a_cursor_move_while_a_unit_is_selected_hardly_searches() {
    use Action::{Confirm, CursorRight, CursorUp};
    use trpg_core::movement::searches;
    let mut c = ctx();
    let mut s = quick_screen(&c);
    press(&mut s, &mut c, &[Confirm]);
    s.scene(&c);
    let mut counts = Vec::new();
    for action in [CursorRight, CursorRight, CursorUp, CursorUp, CursorUp] {
        let before = searches();
        press(&mut s, &mut c, &[action]);
        assert!(s.scene(&c).threat.is_some(), "no arrows after {action:?}");
        counts.push(searches() - before);
    }
    assert!(counts.iter().all(|&n| n <= 2), "{counts:?}");
    let none = counts.iter().filter(|&&n| n == 0).count();
    assert!(none * 2 > counts.len(), "{counts:?}");
}
