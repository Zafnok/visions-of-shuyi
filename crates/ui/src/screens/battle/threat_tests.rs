//! Tests of the threat arrows' list on the battle screen (ticket 0448): it
//! is worked out once for a unit and a tile and kept, and dropped when the
//! battle changes. What the scene shows while playing is in
//! `crates/ui/tests/it/threat.rs`.

use trpg_core::{Command, Pos, UnitAction, UnitId};

use super::testing::skirmish_charged;
use super::{BattleScreen, KeptThreat, quick_battle, threat_enemies};
use crate::harness::FRAME_DT;
use crate::input::Action;
use crate::map_view::ThreatView;
use crate::screen::tests::ctx;
use crate::screen::{Ctx, FrameInput, Screen};

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
    assert!(threat_enemies(state, UnitId(8), p(3, 6)).is_empty());
    // A unit the battle doesn't have: nothing, not a panic.
    assert!(threat_enemies(state, UnitId(99), p(3, 5)).is_empty());
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
    let mut s = BattleScreen::new(skirmish_charged(&c, 20, 3));
    let real = threat_enemies(s.state(), LORD, p(6, 2));
    assert!(
        real.contains(&p(7, 1)) && real.contains(&p(8, 2)),
        "{real:?}"
    );
    assert_eq!(shown(&s, &c), Some((p(6, 2), real.clone())));
    // The archer (at (8, 4) here) waits; then the battle is rewound to
    // before that.
    s.send(&Command::Act {
        unit: UnitId(3),
        dest: p(8, 4),
        action: UnitAction::Wait,
    });
    plant(&s, LORD, p(6, 2), &[p(0, 0)]);
    for action in [Action::Rewind, Action::Confirm, Action::Confirm] {
        s.update(&mut c, &FrameInput::new(vec![action], FRAME_DT, vec![]));
    }
    assert!(s.rewind().is_none());
    assert!(s.history().is_empty());
    assert_eq!(shown(&s, &c), Some((p(6, 2), real)));
}
