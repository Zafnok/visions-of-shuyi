//! Tests of an opening's cast over time, on the test opening
//! (`assets-test/openings/test_opening.ron`, ADR-0118): the lead (unit 1,
//! from (1, 0), facing right) stepping in and back, and the test knight
//! (unit 2, from (3, 0), facing left) walking right to (4, 0) and down to
//! (4, 2).

use trpg_content::ActionDef;
use trpg_core::{CharacterId, Faction, LeadGender, Pos, UnitId};

use super::*;
use crate::map_view::Facing;
use crate::screen::tests::ctx;

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

fn lead() -> LeadProfile {
    LeadProfile::new("Wren", LeadGender::Female)
}

/// The test opening.
fn test_opening(c: &Ctx) -> OpeningDef {
    c.game.content.openings["test_opening"].clone()
}

/// Each unit's place and facing.
fn standing(units: &[UnitView]) -> Vec<(UnitId, Pos, Facing)> {
    units.iter().map(|u| (u.id, u.pos, u.facing)).collect()
}

#[test]
fn the_cast_stand_as_characters_on_no_side() {
    let c = ctx();
    let def = test_opening(&c);
    let cast = cast_units(&c, &def, &lead());
    assert_eq!(
        standing(&cast),
        [
            (UnitId(1), p(1, 0), Facing::Right),
            (UnitId(2), p(3, 0), Facing::Left)
        ]
    );
    let lead_unit = &cast[0];
    assert_eq!(lead_unit.character, Some(CharacterId("lead".into())));
    // The lead's label is from the name the player gave; the knight's is
    // its own map label.
    assert_eq!(lead_unit.label, "Wr");
    assert_eq!(cast[1].label, "Kn");
    assert!(
        cast.iter()
            .all(|u| !u.sided && u.faction == Faction::Neutral)
    );
    // Each has its class, though nobody fights in an opening.
    assert_eq!(lead_unit.class.0, "exile");
    assert_eq!(cast[1].class.0, "guard");
}

#[test]
fn a_step_back_keeps_facing_the_other_way() {
    let c = ctx();
    let def = test_opening(&c);
    let start = cast_units(&c, &def, &lead());
    let at = |ms| standing(&opening_units(&start, &def, ms));
    let lead_at = |pos, facing| (UnitId(1), pos, facing);
    assert_eq!(at(0)[0], lead_at(p(1, 0), Facing::Right));
    // The lead stepped in, and stands there facing right.
    assert_eq!(at(250)[0], lead_at(p(2, 0), Facing::Right));
    // Stepping back: still facing right.
    assert_eq!(at(400)[0].2, Facing::Right);
    assert_eq!(at(550)[0], lead_at(p(1, 0), Facing::Right));
    // The knight stands facing left until its walk.
    assert_eq!(at(550)[1], (UnitId(2), p(3, 0), Facing::Left));
    // At the end, and long after, as it left them.
    let end = [
        lead_at(p(1, 0), Facing::Right),
        (UnitId(2), p(4, 2), Facing::Down),
    ];
    assert_eq!(at(u64::from(def.length_ms)), end);
    assert_eq!(at(60_000), end);
}

#[test]
fn a_walker_faces_the_way_it_goes_and_keeps_it() {
    let c = ctx();
    let mut def = test_opening(&c);
    // Then a step up, so the way it last went isn't the camera's.
    def.actions.push(ActionDef {
        at_ms: 1100,
        act: Act::Walk {
            unit: UnitId(2),
            path: vec![p(4, 2), p(4, 1)],
            ms: 200,
            facing: None,
        },
    });
    def.length_ms = 1300;
    let start = cast_units(&c, &def, &lead());
    let at = |ms| opening_units(&start, &def, ms)[1].clone();
    // Going right, between its first two tiles.
    let across = at(650);
    assert_eq!(across.facing, Facing::Right);
    assert!(across.between_tiles());
    // Then down.
    assert_eq!(at(900).facing, Facing::Down);
    assert_eq!((at(1000).pos, at(1000).facing), (p(4, 2), Facing::Down));
    // After the step up it keeps facing up, not the camera.
    let end = at(u64::from(def.length_ms));
    assert_eq!(
        (end.pos, end.facing, end.frame),
        (p(4, 1), Facing::Up, STANDING_FRAME)
    );
    assert!(!end.between_tiles());
    assert_eq!(at(60_000).facing, Facing::Up);
}

#[test]
fn the_place_darkens_after_the_movement() {
    let c = ctx();
    let def = test_opening(&c);
    let length = u64::from(def.length_ms);
    assert_eq!(opening_ms(&def), def.length_ms + DARKEN_MS);
    assert!(darkened(&def, 0) == 0.0 && darkened(&def, length) == 0.0);
    let half = darkened(&def, length + u64::from(DARKEN_MS / 2));
    assert!((half - 0.5).abs() < 0.01, "{half}");
    assert!((darkened(&def, u64::from(opening_ms(&def))) - 1.0).abs() < f32::EPSILON);
    assert!((darkened(&def, 60_000) - 1.0).abs() < f32::EPSILON);
}
