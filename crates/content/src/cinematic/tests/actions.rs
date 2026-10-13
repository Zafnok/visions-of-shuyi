//! Tests of a map shot's action (ticket 0827), on the test battle: the
//! lead at (3, 5) and the knight at (3, 6), the brigand the battle file
//! calls `road_brigand` at (8, 3).

use trpg_core::{CharacterId, Pos, UnitId};

use super::*;

/// The lead's, the knight's and the brigand's unit ids in the test battle.
const LEAD: UnitId = UnitId(1);
const KNIGHT: UnitId = UnitId(2);
const BRIGAND: UnitId = UnitId(3);

fn p(x: i32, y: i32) -> Pos {
    Pos::new(x, y)
}

/// The test cinematic's action, as loaded.
pub fn duel() -> Vec<ActionDef> {
    let strike = |at_ms, hp_left| ActionDef {
        at_ms,
        act: Act::Strike {
            unit: LEAD,
            target: BRIGAND,
            hp_left,
        },
    };
    let path = [(3, 5), (3, 4), (3, 3), (4, 3), (5, 3), (6, 3), (7, 3)];
    let walk = Act::Walk {
        unit: LEAD,
        path: path.iter().map(|&(x, y)| p(x, y)).collect(),
        ms: 3000,
    };
    let walk = ActionDef {
        at_ms: 500,
        act: walk,
    };
    vec![walk, strike(4000, 40), strike(5000, 0)]
}

/// A ten-second pan over the test battle with `action` written after its
/// zoom.
fn pan(action: &str) -> String {
    format!("(at: 0, shot: MapPan(battle: \"test\", from: (0, 0), to: (0, 0), zoom: 1, {action}))")
}

/// A cinematic of one pan whose action is the list `entries`.
fn acted(entries: &str) -> String {
    source("Seconds(10)", &pan(&format!("action: [{entries}]")))
}

/// The action of the first shot of `source`.
fn action(c: &Content, source: &str) -> ShotAction {
    let def = load(c, source).unwrap_or_else(|e| panic!("{e:?}"));
    match &def.shots[0].shot {
        Shot::MapPan {
            action: Some(action),
            ..
        } => action.clone(),
        other => panic!("no action: {other:?}"),
    }
}

/// The problems of the action list `entries`, each without the "shot 1: "
/// it starts with.
fn wrong(c: &Content, entries: &str) -> Vec<String> {
    let strip = |m: String| m.strip_prefix("shot 1: ").map(str::to_owned).unwrap_or(m);
    problems(c, &acted(entries))
        .into_iter()
        .map(strip)
        .collect()
}

const WALK: &str = "(at: 1, do: Walk(unit: \"lead\", path: [(3, 5), (5, 5)], secs: 2))";

fn strike(at: &str, unit: &str, target: &str, hp_left: &str) -> String {
    format!("(at: {at}, do: Strike(unit: \"{unit}\", target: \"{target}\", hp_left: {hp_left}))")
}

fn walk(at: &str, unit: &str, path: &str, secs: &str) -> String {
    format!("(at: {at}, do: Walk(unit: \"{unit}\", path: [{path}], secs: {secs}))")
}

#[test]
fn a_pan_without_an_action_has_none() {
    let c = content();
    let def = load(&c, &source("Seconds(10)", PAN)).unwrap();
    assert!(matches!(
        def.shots[0].shot,
        Shot::MapPan { action: None, .. }
    ));
}

#[test]
fn a_list_in_place_is_kept_in_milliseconds_with_its_units_ids() {
    let c = content();
    let entries = [
        WALK.to_owned(),
        walk("3", "test_knight", "(3, 6), (3, 7)", "0.5"),
        strike("3.25", "lead", "road_brigand", "100"),
        strike("3.5", "test_knight", "road_brigand", "1"),
    ]
    .join(", ");
    let got = action(&c, &acted(&entries));
    assert_eq!(got.from_ms, 0);
    let kept: Vec<(u32, Act)> = got.actions.into_iter().map(|a| (a.at_ms, a.act)).collect();
    let walked = |unit, path: &[(i32, i32)], ms| Act::Walk {
        unit,
        path: path.iter().map(|&(x, y)| p(x, y)).collect(),
        ms,
    };
    let struck = |unit, hp_left| Act::Strike {
        unit,
        target: BRIGAND,
        hp_left,
    };
    assert_eq!(
        kept,
        [
            (1000, walked(LEAD, &[(3, 5), (4, 5), (5, 5)], 2000)),
            (3000, walked(KNIGHT, &[(3, 6), (3, 7)], 500)),
            (3250, struck(LEAD, 100)),
            (3500, struck(KNIGHT, 1)),
        ]
    );
}

#[test]
fn a_walk_goes_tile_by_tile_through_its_corners_and_back() {
    let c = content();
    let entries = [
        walk("0", "lead", "(3, 5), (3, 3), (5, 3), (5, 2)", "1"),
        // The next starts where that one ended, the moment it ended.
        walk("1", "lead", "(5, 2), (5, 3), (4, 3)", "0.001"),
    ]
    .join(", ");
    let paths: Vec<Vec<Pos>> = action(&c, &acted(&entries))
        .actions
        .into_iter()
        .map(|a| match a.act {
            Act::Walk { path, ms, .. } => {
                assert!(ms == 1000 || ms == 1);
                path
            }
            Act::Strike { .. } => panic!("a strike"),
        })
        .collect();
    let tiles = |path: &[(i32, i32)]| path.iter().map(|&(x, y)| p(x, y)).collect::<Vec<_>>();
    assert_eq!(
        paths,
        [
            tiles(&[(3, 5), (3, 4), (3, 3), (4, 3), (5, 3), (5, 2)]),
            tiles(&[(5, 2), (5, 3), (4, 3)]),
        ]
    );
}

#[test]
fn a_shot_starts_its_action_at_action_from() {
    let c = content();
    let from = |at: &str| {
        source(
            "Seconds(10)",
            &pan(&format!("action_from: {at}, action: [{WALK}]")),
        )
    };
    assert_eq!(action(&c, &from("2.5")).from_ms, 2500);
    assert_eq!(action(&c, &from("0")).from_ms, 0);
    // Past the list's end is fine: the shot shows how it all ended.
    assert_eq!(action(&c, &from("900")).from_ms, 900_000);
    for at in ["-1", "NaN", "inf"] {
        assert_eq!(
            problems(&c, &from(at)),
            [format!("shot 1: action_from {at} is not a time")]
        );
    }
    assert_eq!(
        problems(&c, &source("Seconds(10)", &pan("action_from: 1"))),
        ["shot 1: action_from without an action"]
    );
}

#[test]
fn shots_share_a_named_list() {
    let c = content();
    let named = |table: &str, shots: &str| {
        format!("(id: \"t\", length: Seconds(10), actions: {{{table}}}, shots: [{shots}])")
    };
    let two = format!(
        "{}, {}",
        pan("action: \"march\""),
        pan("action_from: 4, action: \"march\"").replace("at: 0", "at: 5")
    );
    let def = load(&c, &named(&format!("\"march\": [{WALK}]"), &two)).unwrap();
    let actions: Vec<&ShotAction> = def
        .shots
        .iter()
        .map(|s| match &s.shot {
            Shot::MapPan { action, .. } => action.as_ref().unwrap(),
            other => panic!("{other:?}"),
        })
        .collect();
    assert_eq!((actions[0].from_ms, actions[1].from_ms), (0, 4000));
    assert_eq!(actions[0].actions, actions[1].actions);
    assert_eq!(actions[0].actions.len(), 1);
    // A list's problems are reported once, however many shots use it.
    let bad = walk("0", "nobody", "(0, 0), (0, 1)", "1");
    assert_eq!(
        problems(&c, &named(&format!("\"march\": [{bad}]"), &two)),
        [
            "actions \"march\" (on battle \"test\"): action 1: no unit \"nobody\" on the map of battle \"test\" at its start"
        ]
    );
    // A name the table lacks; a list no shot uses (so never checked).
    assert_eq!(
        problems(
            &c,
            &named(&format!("\"march\": [{bad}]"), &pan("action: \"charge\""))
        ),
        [
            "shot 1: no actions named \"charge\"",
            "actions \"march\": no shot uses it"
        ]
    );
    // On a battle that doesn't exist only the battle is wrong.
    let nowhere = pan("action: \"march\"").replace("\"test\"", "\"nope\"");
    assert_eq!(
        problems(&c, &named(&format!("\"march\": [{bad}]"), &nowhere)),
        ["shot 1: no battle \"nope\""]
    );
    let nowhere = acted(&bad).replace("\"test\"", "\"nope\"");
    assert_eq!(problems(&c, &nowhere), ["shot 1: no battle \"nope\""]);
}

#[test]
fn entries_are_in_order_of_time() {
    let c = content();
    assert_eq!(wrong(&c, ""), ["an action needs at least one entry"]);
    let late = strike("0.999", "lead", "road_brigand", "50");
    assert_eq!(
        wrong(&c, &format!("{WALK}, {late}")),
        [
            "action 2: at 0.999 is before the action before it (1 seconds); actions are in order of time",
            // It is taken to happen with the entry before it.
            "action 2: \"lead\" is still walking then (until 3 seconds)",
        ]
    );
    for at in ["-1", "NaN", "inf"] {
        let entry = strike(at, "lead", "road_brigand", "50");
        assert_eq!(
            wrong(&c, &entry),
            [format!("action 1: at {at} is not a time")]
        );
    }
}

#[test]
fn units_are_those_on_the_battles_map_at_its_start() {
    let c = content();
    let none =
        |name: &str| format!("no unit \"{name}\" on the map of battle \"test\" at its start");
    assert_eq!(
        wrong(&c, &strike("0", "test_brigand", "nobody", "50")),
        [
            format!("action 1: {}", none("test_brigand")),
            format!("action 1: {}", none("nobody")),
        ]
    );
    assert_eq!(
        wrong(&c, &walk("0", "road_brigand2", "(8, 3), (8, 2)", "1")),
        [format!("action 1: {}", none("road_brigand2"))]
    );
    // An enemy that is a character goes by the character: Chapter 1's
    // captain, who has no `id` of his own.
    let captain = strike("0", "red_captain", "lead", "50").replace("at: 0", "at: 1");
    let on_ch01 = acted(&captain).replace("\"test\"", "\"ch01\"");
    let action = action(&c, &on_ch01);
    let boss = c.battles["ch01"]
        .enemies
        .iter()
        .find(|u| u.character == Some(CharacterId("red_captain".into())))
        .unwrap();
    assert_eq!(
        action.actions[0].act,
        Act::Strike {
            unit: boss.id,
            target: UnitId(1),
            hp_left: 50
        }
    );
}

#[test]
fn a_walk_takes_time_and_two_tiles_from_where_its_unit_is() {
    let c = content();
    for secs in ["0", "0.0004", "-1", "NaN"] {
        assert_eq!(
            wrong(&c, &walk("0", "lead", "(3, 5), (4, 5)", secs)),
            [format!(
                "action 1: secs {secs}; a walk takes more than 0 seconds"
            )]
        );
    }
    for path in ["", "(3, 5)"] {
        assert_eq!(
            wrong(&c, &walk("0", "lead", path, "1")),
            ["action 1: path: a walk needs at least two tiles"]
        );
    }
    assert_eq!(
        wrong(&c, &walk("0", "lead", "(4, 5), (5, 5)", "1")),
        ["action 1: path starts at (4, 5), but \"lead\" is at (3, 5) then"]
    );
    // After a walk it is at that walk's end, even one that was wrong.
    let again = format!("{WALK}, {}", walk("3", "lead", "(3, 5), (4, 5)", "1"));
    assert_eq!(
        wrong(&c, &again),
        ["action 2: path starts at (3, 5), but \"lead\" is at (5, 5) then"]
    );
    let after_wrong = format!(
        "{}, {}",
        walk("0", "lead", "(4, 5), (6, 5)", "1"),
        walk("1", "lead", "(6, 5), (6, 6)", "1")
    );
    assert_eq!(wrong(&c, &after_wrong).len(), 1);
    // One walk at a time.
    let early = format!("{WALK}, {}", walk("2.999", "lead", "(5, 5), (4, 5)", "1"));
    assert_eq!(
        wrong(&c, &early),
        ["action 2: \"lead\" is still walking then (until 3 seconds)"]
    );
    // Another unit may walk meanwhile.
    let both = format!(
        "{WALK}, {}",
        walk("1.5", "test_knight", "(3, 6), (3, 7)", "1")
    );
    assert_eq!(action(&c, &acted(&both)).actions.len(), 2);
}

#[test]
fn a_walk_stays_on_the_map_in_straight_runs_on_tiles_its_unit_can_enter() {
    let c = content();
    assert_eq!(
        wrong(&c, &walk("0", "lead", "(3, 5), (3, 8), (-1, 8)", "1")),
        [
            "action 1: path: (3, 8) is not on the map (14×8 tiles)",
            "action 1: path: (-1, 8) is not on the map (14×8 tiles)",
        ]
    );
    // A corner far off the map is only that: no tiles are walked.
    assert_eq!(
        wrong(&c, &walk("0", "lead", "(3, 5), (2000000000, 5)", "1")),
        ["action 1: path: (2000000000, 5) is not on the map (14×8 tiles)"]
    );
    assert_eq!(
        wrong(
            &c,
            &walk("0", "lead", "(3, 5), (4, 4), (4, 4), (4, 3)", "1")
        ),
        [
            "action 1: path: (3, 5) to (4, 4) is not a straight run along a row or a column",
            "action 1: path: (4, 4) to (4, 4) is not a straight run along a row or a column",
        ]
    );
    // Row 5 of the test map: plain, a fort, plain, then the keep's wall.
    let lead = &c.characters.characters[&CharacterId("lead".into())];
    let class = &c.classes.get(&lead.class).unwrap().name;
    assert_eq!(
        wrong(&c, &walk("0", "lead", "(3, 5), (10, 5)", "1")),
        [
            format!("action 1: path: a {class} can't walk on (9, 5)"),
            // The floor inside can be walked on.
        ]
    );
    assert!(load(&c, &acted(&walk("0", "lead", "(3, 5), (8, 5)", "1"))).is_ok());
    // The brigand can't walk through it either: every tile is named.
    let brigand = &c.battles["test"].enemies[0];
    let class = &c.classes.get(&brigand.class).unwrap().name;
    assert_eq!(
        wrong(
            &c,
            &walk("0", "road_brigand", "(8, 3), (8, 4), (10, 4)", "1")
        ),
        [
            format!("action 1: path: a {class} can't walk on (9, 4)"),
            format!("action 1: path: a {class} can't walk on (10, 4)"),
        ]
    );
}

#[test]
fn a_strike_leaves_its_target_no_more_hp_than_it_had() {
    let c = content();
    let hit = |hp: &str| strike("0", "lead", "road_brigand", hp);
    for hp in ["-1", "101"] {
        assert_eq!(
            wrong(&c, &hit(hp)),
            [format!("action 1: hp_left {hp}; it is 0 to 100")]
        );
    }
    for hp in ["0", "100"] {
        assert!(load(&c, &acted(&hit(hp))).is_ok(), "{hp}");
    }
    assert_eq!(
        wrong(&c, &format!("{}, {}, {}", hit("40"), hit("40"), hit("41"))),
        ["action 3: hp_left 41 is more than the 40 \"road_brigand\" has left; HP never rises"]
    );
    // A strike that was wrong took no HP.
    assert_eq!(
        wrong(&c, &format!("{}, {}", hit("101"), hit("100"))),
        ["action 1: hp_left 101; it is 0 to 100"]
    );
    assert_eq!(
        wrong(&c, &strike("0", "lead", "lead", "50")),
        ["action 1: \"lead\" can't strike itself"]
    );
    // Each unit has its own HP.
    let each = format!(
        "{}, {}",
        hit("10"),
        strike("0", "road_brigand", "lead", "90")
    );
    assert!(load(&c, &acted(&each)).is_ok());
    // Striking in the middle of a walk; being struck in one is fine.
    let mid = format!(
        "{WALK}, {}, {}",
        strike("2", "lead", "road_brigand", "50"),
        strike("2", "road_brigand", "lead", "50")
    );
    assert_eq!(
        wrong(&c, &mid),
        ["action 2: \"lead\" is still walking then (until 3 seconds)"]
    );
}

#[test]
fn a_unit_that_fell_does_nothing_more() {
    let c = content();
    let fell = strike("1", "lead", "road_brigand", "0");
    let after = [
        fell.clone(),
        strike("2", "road_brigand", "lead", "50"),
        strike("2", "lead", "road_brigand", "0"),
        walk("2", "road_brigand", "(8, 3), (8, 2)", "1"),
        strike("3", "lead", "test_knight", "50"),
    ]
    .join(", ");
    let gone =
        |n: usize| format!("action {n}: \"road_brigand\" fell at action 1 and can do nothing more");
    assert_eq!(wrong(&c, &after), [gone(2), gone(3), gone(4)]);
    assert!(load(&c, &acted(&fell)).is_ok());
}

#[test]
fn every_problem_of_an_action_is_reported_at_once() {
    let c = content();
    let entries = [
        walk("2", "lead", "(0, 0), (1, 1)", "0"),
        strike("1", "nobody", "lead", "200"),
    ]
    .join(", ");
    assert_eq!(
        wrong(&c, &entries),
        [
            "action 1: secs 0; a walk takes more than 0 seconds",
            "action 1: path starts at (0, 0), but \"lead\" is at (3, 5) then",
            "action 1: path: (0, 0) to (1, 1) is not a straight run along a row or a column",
            "action 2: at 1 is before the action before it (2 seconds); actions are in order of time",
            "action 2: no unit \"nobody\" on the map of battle \"test\" at its start",
            "action 2: hp_left 200; it is 0 to 100",
        ]
    );
    // An entry the format doesn't have is refused.
    let unknown = "(at: 0, do: Cast(unit: \"lead\"))";
    assert_eq!(problems(&c, &acted(unknown)).len(), 1);
    // So is an action that is neither a list nor a name, saying what one is.
    let neither = problems(&c, &source("Seconds(10)", &pan("action: 3")));
    assert_eq!(neither.len(), 1);
    assert!(
        neither[0].contains("a list of actions, or the name of one in `actions`"),
        "{neither:?}"
    );
}

#[test]
fn the_battles_name_their_enemies_by_unit_id() {
    let names = crate::battle::unit_names;
    assert_eq!(
        names("test"),
        BTreeMap::from([("road_brigand".to_owned(), BRIGAND)])
    );
    // Chapter 1's file names the three the title cinematic moves; a
    // battle that has no file names none.
    assert_eq!(
        names("ch01").into_keys().collect::<Vec<_>>(),
        ["footbridge_brigand", "road_archer", "road_brigand"]
    );
    assert_eq!(names("nope"), BTreeMap::new());
    assert_eq!(names(""), BTreeMap::new());
    // Ids follow the slots that aren't numbered after the enemies.
    let of = crate::battle::unit_names_of;
    let src = "(id: \"b\", map: \"m\",
        player_slots: [
            (character: \"a\", pos: (0, 0)),
            (character: \"b\", pos: (0, 1), after_enemies: true),
            (character: \"c\", pos: (0, 2)),
        ],
        enemies: [
            (template: \"x\", pos: (1, 0)),
            (template: \"x\", pos: (1, 1), id: \"second\"),
            (template: \"x\", pos: (1, 2), id: \"third\"),
        ],
        reinforcements: [(turn: 2, unit: (template: \"x\", pos: (2, 2), id: \"late\"))],
        objective: Rout(), difficulty: Normal, music: Pool(\"skirmish\"), seed: 1)";
    assert_eq!(
        of(src),
        BTreeMap::from([
            ("second".to_owned(), UnitId(4)),
            ("third".to_owned(), UnitId(5)),
        ])
    );
    assert_eq!(of("(not a battle"), BTreeMap::new());
}
