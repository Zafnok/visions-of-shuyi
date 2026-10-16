//! Tests of an opening's cast over time, on Chapter 1's two openings: the
//! lead (unit 1, from (11, 8), facing right) and Hollis (unit 2, from
//! (13, 8), facing left) stepping in and back in the yard, and Dace
//! pacing the study from (9, 8) to (15, 8) and back to (12, 8).

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

/// Each unit's place and facing.
fn standing(units: &[UnitView]) -> Vec<(UnitId, Pos, Facing)> {
    units.iter().map(|u| (u.id, u.pos, u.facing)).collect()
}

#[test]
fn the_cast_stand_as_characters_without_marks() {
    let c = ctx();
    let def = &c.content.openings["ch01_sparring"];
    let cast = cast_units(&c, def, &lead());
    assert_eq!(
        standing(&cast),
        [
            (UnitId(1), p(11, 8), Facing::Right),
            (UnitId(2), p(13, 8), Facing::Left)
        ]
    );
    let lead_unit = &cast[0];
    assert_eq!(lead_unit.character, Some(CharacterId("lead".into())));
    // The lead's label is from the name the player gave.
    assert_eq!(lead_unit.label, "Wr");
    assert_eq!(cast[1].label, "Ho");
    assert!(
        cast.iter()
            .all(|u| !u.marks && u.faction == Faction::Neutral)
    );
    assert_eq!(lead_unit.class.0, "exile");
    // Dace has no class (he never fights in Chapter 1): his picture is the
    // skin's for his character.
    let dace = &cast_units(&c, &c.content.openings["ch01_pacing"], &lead())[0];
    assert_eq!((dace.class.0.as_str(), dace.label.as_str()), ("", "Da"));
}

#[test]
fn sparring_partners_step_in_and_back_facing_each_other() {
    let c = ctx();
    let def = &c.content.openings["ch01_sparring"];
    let start = cast_units(&c, def, &lead());
    let at = |ms| standing(&opening_units(&start, def, ms));
    let (lead_at, hollis_at) = (
        |pos, facing| (UnitId(1), pos, facing),
        |pos, facing| (UnitId(2), pos, facing),
    );
    assert_eq!(
        at(0),
        [
            lead_at(p(11, 8), Facing::Right),
            hollis_at(p(13, 8), Facing::Left)
        ]
    );
    // The lead stepped in, and stands there facing Hollis.
    assert_eq!(at(250)[0], lead_at(p(12, 8), Facing::Right));
    // Stepping back: still facing him.
    assert_eq!(at(400)[0].2, Facing::Right);
    assert_eq!(at(550)[0], lead_at(p(11, 8), Facing::Right));
    // Hollis's turn: in and back, facing the lead all along.
    assert_eq!(at(800)[1], hollis_at(p(12, 8), Facing::Left));
    assert_eq!(at(950)[1].2, Facing::Left);
    // At the end, and long after, as it left them.
    let end = [
        lead_at(p(11, 8), Facing::Right),
        hollis_at(p(13, 8), Facing::Left),
    ];
    assert_eq!(at(u64::from(def.length_ms)), end);
    assert_eq!(at(60_000), end);
}

#[test]
fn a_walker_faces_the_way_it_goes_and_keeps_it() {
    let c = ctx();
    let def = &c.content.openings["ch01_pacing"];
    let start = cast_units(&c, def, &lead());
    let at = |ms| opening_units(&start, def, ms)[0].clone();
    let across = at(400);
    assert_eq!(across.facing, Facing::Right);
    assert!((9..15).contains(&across.pos.x));
    // At the far end, before turning back: still the way it went.
    assert_eq!((at(900).pos, at(900).facing), (p(15, 8), Facing::Right));
    assert_eq!(at(1200).facing, Facing::Left);
    let end = at(u64::from(def.length_ms));
    assert_eq!(
        (end.pos, end.facing, end.frame),
        (p(12, 8), Facing::Left, STANDING_FRAME)
    );
    assert!(!end.between_tiles());
}

#[test]
fn the_place_darkens_after_the_movement() {
    let c = ctx();
    let def = &c.content.openings["ch01_sparring"];
    let length = u64::from(def.length_ms);
    assert_eq!(opening_ms(def), def.length_ms + DARKEN_MS);
    assert!(darkened(def, 0) == 0.0 && darkened(def, length) == 0.0);
    let half = darkened(def, length + u64::from(DARKEN_MS / 2));
    assert!((half - 0.5).abs() < 0.01, "{half}");
    assert!((darkened(def, u64::from(opening_ms(def))) - 1.0).abs() < f32::EPSILON);
    assert!((darkened(def, 60_000) - 1.0).abs() < f32::EPSILON);
}
