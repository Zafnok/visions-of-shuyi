//! The battle screen and Act 1's battle rules (ticket 0319): an escape
//! map's objective line, its `Escape` entry and its exit area on the map;
//! the forecast against a unit that can't be killed, and that unit drawn
//! down on its tile once beaten; and the enemies left running when a
//! boss falls (0845).

use insta::assert_snapshot;
use trpg_core::{
    BattleState, Command, Defeat, Faction, GameMode, LeadGender, LeadProfile, Leftovers, Objective,
    Pos, TileRect, UnitAction, UnitId,
};

use super::BattleScreen;
use super::flight::FLEE_S;
use super::map_menu::objective_text;
use super::mode::{MenuEntry, Mode};
use super::testing::{Values, battle_with};
use super::view::ForecastView;
use crate::harness::Harness;
use crate::map_view::scene::range;
use crate::screen::tests::ctx;
use crate::words::Words;
use trpg_content::{LangCode, battle_campaign};
use trpg_core::lead::DEFAULT_NAME;

/// The Quick Battle as an escape map whose exit is the lord's own tile
/// (3, 5) and the one right of it.
fn escape() -> BattleState {
    let c = ctx();
    let quick = super::quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
    let area = TileRect {
        x: 3,
        y: 5,
        w: 2,
        h: 1,
    };
    let objective = Objective::Escape { area };
    battle_with(&c, quick.map().clone(), quick.units().to_vec(), objective)
}

#[test]
fn the_escape_objective_counts_who_is_out() {
    let mut s = escape();
    let text = |s: &BattleState| objective_text(s, None, Words::ENGLISH);
    assert_eq!(text(&s), "Get everyone to the exit (0/4 out)");
    let out = Command::Act {
        unit: UnitId(1),
        dest: Pos::new(3, 5),
        action: UnitAction::Escape,
    };
    s.apply(&out).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(text(&s), "Get everyone to the exit (1/4 out)");
}

/// An escape map whose battle file names its exit (0324) says where to
/// go, in English and in the test pack; one without keeps `the exit`.
#[test]
fn an_escape_map_names_its_exit() {
    let c = ctx();
    let content = &c.game.content;
    let def = &content.battles["test_escape"];
    let exit = def.exit_name.as_deref();
    let lead = LeadProfile::new(DEFAULT_NAME, LeadGender::Male);
    let campaign = battle_campaign(content, def, GameMode::Classic, lead);
    let (s, _) = BattleState::new(campaign.battle_setup(def, &content.tables()));
    assert_eq!(
        objective_text(&s, exit, Words::ENGLISH),
        "Get everyone to the east road (0/3 out)"
    );
    let test = LangCode::new("test").unwrap();
    let words = Words::new(&content.lang, &test);
    assert_eq!(
        objective_text(&s, exit, words),
        "GET EVERYONE TO THE EAST ROAD (0/3 OUT)"
    );
    assert_eq!(
        objective_text(&s, None, words),
        "GET EVERYONE TO THE EXIT (0/3 OUT)"
    );
}

#[test]
fn the_action_menu_offers_escape_in_the_exit_area_but_never_opens_on_it() {
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(escape())));
    // The lord, staying on its tile of the exit.
    h.keys("f f").wait(0.5);
    let Some(Mode::ActionMenu { menu, .. }) = h.battle().map(BattleScreen::mode) else {
        panic!("no action menu");
    };
    assert!(
        menu.values().contains(&MenuEntry::Escape),
        "{:?}",
        menu.values()
    );
    assert_ne!(menu.value(menu.focus()), Some(&MenuEntry::Escape));
    // Off the exit: no Escape.
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(escape())));
    h.keys("f Up f").wait(0.5);
    let Some(Mode::ActionMenu { menu, .. }) = h.battle().map(BattleScreen::mode) else {
        panic!("no action menu");
    };
    assert!(!menu.values().contains(&MenuEntry::Escape));
}

#[test]
fn the_exit_area_is_on_the_map() {
    let h = Harness::with_screen(Box::new(BattleScreen::new(escape())));
    assert!(h.tints_at(Pos::new(3, 5)).contains(&range::EXIT));
    assert!(h.tints_at(Pos::new(4, 5)).contains(&range::EXIT));
    assert!(!h.tints_at(Pos::new(5, 5)).contains(&range::EXIT));
}

/// The attack skirmish (the lord at (6, 2) next to a brigand at (8, 2)
/// with `brigand_hp` HP) with that brigand unkillable or not.
fn skirmish(brigand_hp: i32, defeat: Defeat) -> BattleState {
    let c = ctx();
    let quick = super::quick_battle(&c.game.content).unwrap_or_else(|e| panic!("{e}"));
    let mut units = quick.units().to_vec();
    units[0].pos = Pos::new(6, 2);
    units[2].pos = Pos::new(8, 4);
    units[3].hp = brigand_hp;
    units[3].defeat = defeat;
    battle_with(
        &c,
        quick.map().clone(),
        units,
        Objective::Rout { turn_limit: None },
    )
}

/// The lord at (7, 2) aiming at the brigand.
fn aiming(state: BattleState) -> Harness {
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    h.keys("f Right f").wait(0.5);
    h.keys("f Down");
    h
}

fn forecast(h: &Harness) -> ForecastView {
    let battle = h.battle().expect("a battle");
    battle.view(h.game().ctx()).forecast.expect("a forecast")
}

/// The forecast against a unit that can't be killed is the normal one: its
/// HP reaches 0 (Nick, 0069).
#[test]
fn the_forecast_against_a_unit_that_cant_be_killed_is_the_normal_one() {
    let unkillable = aiming(skirmish(5, Defeat::GoesDown));
    let plain = aiming(skirmish(5, Defeat::Falls));
    assert_eq!(forecast(&unkillable), forecast(&plain));
    assert_snapshot!(unkillable.snapshot());
}

#[test]
fn a_unit_that_cant_be_killed_is_drawn_down_on_its_tile() {
    let mut h = aiming(skirmish(5, Defeat::GoesDown));
    h.keys("f").wait(5.0);
    let s = h.battle().expect("a battle").state();
    assert_eq!(s.down().len(), 1, "{:?}", s.fallen());
    let at = s.down()[0].pos;
    let unit = h.unit_at(at).expect("still drawn");
    assert!(unit.is_down());
    assert_eq!(unit.hp.0, 0);
}

/// The attack skirmish won by felling its brigand at 1 HP (unit 4), the
/// rest fleeing (0845): the lord aiming at it, under map skin `skin`.
fn boss_fight(skin: &str) -> Harness {
    let mut state = skirmish(1, Defeat::Falls);
    let objective = Objective::DefeatUnit {
        unit: UnitId(4),
        turn_limit: None,
        leftovers: Leftovers::Flee,
    };
    let c = ctx();
    state = battle_with(&c, state.map().clone(), state.units().to_vec(), objective);
    let mut h = Harness::with_screen(Box::new(BattleScreen::new(state)));
    h.with_map_skin(skin);
    h.keys("f Right f").wait(0.5);
    h.keys("f Down");
    h
}

/// Strikes the brigand, then waits for the leftovers' moment to start,
/// checking that the leftovers are drawn standing until it does.
fn fell_the_boss(h: &mut Harness) -> Vec<(UnitId, Pos)> {
    h.keys("f");
    let fled: Vec<(UnitId, Pos)> = {
        let s = h.battle().expect("a battle").state();
        s.fled().iter().map(|u| (u.id, u.pos)).collect()
    };
    assert_eq!(fled.len(), 3, "the other enemies flee");
    for _ in 0..200 {
        if h.battle().and_then(BattleScreen::flight).is_some() {
            return fled;
        }
        let scene = h.map_scene().expect("a map");
        for &(id, pos) in &fled {
            let unit = scene
                .units
                .iter()
                .find(|u| u.id == id)
                .expect("still drawn");
            assert_eq!((unit.pos, unit.fade), (pos, 0.0));
        }
        h.wait(0.05);
    }
    panic!("the leftovers never left");
}

#[test]
fn the_leftovers_walk_off_and_fade_in_both_skins() {
    for skin in ["glyph", "sprite"] {
        let mut h = boss_fight(skin);
        let fled = fell_the_boss(&mut h);
        h.wait(0.6 * FLEE_S);
        let scene = h.map_scene().expect("a map");
        let (w, ht) = {
            let tiles = &h.battle().expect("a battle").state().map().tiles;
            (i32::from(tiles.width()), i32::from(tiles.height()))
        };
        let to_edge = |p: Pos| p.x.min(w - 1 - p.x).min(p.y).min(ht - 1 - p.y);
        for (id, from) in fled {
            let unit = scene.units.iter().find(|u| u.id == id).expect("leaving");
            assert!(unit.fade > 0.0 && unit.fade < 1.0, "{skin}: {unit:?}");
            assert!(
                to_edge(unit.pos) < to_edge(from) || unit.between_tiles() || to_edge(from) == 0,
                "{skin}: {id:?} from {from:?}: {unit:?}"
            );
        }
        // Gone, and the victory banner up.
        h.wait(FLEE_S);
        let battle = h.battle().expect("a battle");
        assert!(battle.flight().is_none(), "{skin}");
        assert!(battle.banner().is_some(), "{skin}");
        let scene = h.map_scene().expect("a map");
        assert!(
            scene.units.iter().all(|u| u.faction != Faction::Enemy),
            "{skin}"
        );
    }
}

#[test]
fn confirm_skips_the_leftovers_leaving() {
    let mut h = boss_fight("glyph");
    fell_the_boss(&mut h);
    h.keys("f");
    let battle = h.battle().expect("a battle");
    assert!(battle.flight().is_none());
    assert!(battle.banner().is_some());
}

#[test]
fn the_leftovers_leaving() {
    let mut h = boss_fight("glyph");
    fell_the_boss(&mut h);
    h.wait(0.4 * FLEE_S);
    assert_snapshot!(h.snapshot());
}
