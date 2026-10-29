//! Tests of the camp events' conditions and seen state.

use super::*;
use crate::battle::GameMode;
use crate::battle::tests::{p, unit};
use crate::campaign::Campaign;
use crate::item::Stock;
use crate::lead::{LeadGender, LeadProfile};
use crate::unit::{Faction, Unit};

fn c(id: &str) -> CharacterId {
    CharacterId(id.into())
}

/// A campaign whose army is `ann` and `ben`.
fn campaign() -> Campaign {
    let named = |n: u32, id: &str| Unit {
        character: Some(c(id)),
        ..unit(n, Faction::Player, p(0, 0))
    };
    Campaign::new_game(
        GameMode::Classic,
        LeadProfile::new("Mara", LeadGender::Female),
        "ch01",
        vec![named(1, "ann"), named(2, "ben")],
        0,
        Stock::default(),
    )
}

/// An event with no conditions.
fn event() -> CampEvent {
    CampEvent {
        id: "fireside".into(),
        title: "By the fire".into(),
        scene: "camp_fireside".into(),
        flags: vec![],
        battles: vec![],
        units: vec![],
    }
}

#[test]
fn an_event_with_no_conditions_is_always_available() {
    assert!(event().available(&campaign()));
}

#[test]
fn an_event_waits_for_every_flag_it_names() {
    let e = CampEvent {
        flags: vec!["met_hedda".into(), "river_fort".into()],
        ..event()
    };
    let mut game = campaign();
    assert!(!e.available(&game));
    game.flags.insert("met_hedda".into(), true);
    assert!(!e.available(&game), "one flag of two");
    game.flags.insert("river_fort".into(), false);
    assert!(!e.available(&game), "a flag set to false isn't set");
    game.flags.insert("river_fort".into(), true);
    assert!(e.available(&game));
}

#[test]
fn an_event_waits_for_every_battle_it_names() {
    let e = CampEvent {
        battles: vec!["ch01".into()],
        ..event()
    };
    let mut game = campaign();
    assert!(!e.available(&game));
    game.battles_won.insert("ch02".into());
    assert!(!e.available(&game), "another battle");
    game.battles_won.insert("ch01".into());
    assert!(e.available(&game));
}

#[test]
fn an_event_needs_every_character_it_names_in_the_army() {
    let e = CampEvent {
        units: vec![c("ann"), c("ben")],
        ..event()
    };
    let mut game = campaign();
    assert!(e.available(&game));
    // Ben dies (or was never recruited).
    game.roster.retain(|u| u.character != Some(c("ben")));
    assert!(!e.available(&game));
    let only_ann = CampEvent {
        units: vec![c("ann")],
        ..event()
    };
    assert!(only_ann.available(&game));
}

#[test]
fn seeing_an_event_is_remembered_once() {
    let mut game = campaign();
    assert!(game.camp_seen.is_empty());
    assert!(game.see_camp_event("fireside"));
    assert!(!game.see_camp_event("fireside"), "seen already");
    assert!(game.camp_seen.contains("fireside"));
}

/// The seen events and the battles won are saved; a save made before
/// they existed (neither field) loads with none, and a campaign with
/// none writes neither, so old saves still read back to the byte.
#[test]
fn seen_events_and_battles_won_are_saved_and_old_saves_load() {
    let mut game = campaign();
    let empty = ron::to_string(&game).unwrap();
    assert!(!empty.contains("camp_seen") && !empty.contains("battles_won"));
    let back: Campaign = ron::from_str(&empty).unwrap();
    assert_eq!(back, game);
    game.see_camp_event("fireside");
    game.battles_won.insert("ch01".into());
    let text = ron::to_string(&game).unwrap();
    let back: Campaign = ron::from_str(&text).unwrap();
    assert_eq!(back.camp_seen, game.camp_seen);
    assert_eq!(back.battles_won, game.battles_won);
}
