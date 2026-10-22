use proptest::prelude::*;
use shuyi_core::bots::playtest::{Found, RateVerdict, Tried, judge, play_tries};

use super::*;
use crate::persona::Personas;
use crate::personas_from_ron;
use crate::testkit::{content, enemy, lord, player, rigged, setup, stage, weak};

fn targets() -> Targets {
    Targets::from_ron(include_str!("../../targets.ron")).unwrap()
}

fn personas() -> Personas {
    personas_from_ron(include_str!("../../personas.ron")).unwrap()
}

const PERSONAS: [&str; 3] = ["casual", "normal", "hardcore"];

/// A try with these measures.
fn tried(seed: u64, measures: &[(&str, i64)]) -> Tried {
    Tried {
        seed,
        outcome: None,
        turns: 1,
        measures: measures.iter().map(|&(n, v)| (n.to_owned(), v)).collect(),
    }
}

/// The successes of each persona among `tries` (every persona playing
/// the same tries), judged by the engine with `targets.ron`'s checks.
fn successes(tries: &[Tried]) -> Vec<u32> {
    let checks = targets().checks(Difficulty::Normal, &PERSONAS);
    let played = PERSONAS
        .iter()
        .map(|&n| (n.to_owned(), tries.to_vec()))
        .collect();
    let report = judge(played, &checks).unwrap();
    report
        .checks
        .iter()
        .filter_map(|j| match j.found {
            Found::Rate { successes, .. } => Some(successes),
            Found::Median { .. } => None,
        })
        .collect()
}

#[test]
fn success_rules_per_persona() {
    let won = |deaths, items| tried(1, &[("won", 1), ("deaths", deaths), ("items", items)]);
    // Casual, Normal, Hardcore.
    assert_eq!(successes(&[won(1, 2)]), [1, 0, 0]);
    assert_eq!(successes(&[won(0, 3)]), [1, 1, 0]);
    assert_eq!(successes(&[won(0, 1)]), [1, 1, 1]);
    assert_eq!(successes(&[won(0, 0)]), [1, 1, 1]);
    assert_eq!(successes(&[won(0, 4)]), [1, 0, 0]);
    let lost = tried(1, &[("won", 0), ("deaths", 0), ("items", 0)]);
    assert_eq!(successes(&[lost]), [0, 0, 0]);
}

#[test]
fn band_check_flags_too_easy_and_too_hard() {
    let checks = targets().checks(Difficulty::Normal, &["casual"]);
    let verdict = |won: u64| {
        let tries = (0..100)
            .map(|i| tried(i, &[("won", i64::from(i < won))]))
            .collect();
        let report = judge(vec![("casual".to_owned(), tries)], &checks).unwrap();
        match report.checks[0].found {
            Found::Rate {
                verdict, blocked, ..
            } => (verdict, blocked, report.blocked),
            Found::Median { .. } => unreachable!(),
        }
    };
    assert_eq!(verdict(59), (RateVerdict::Below, false, false));
    assert_eq!(verdict(60), (RateVerdict::Within, false, false));
    assert_eq!(verdict(70), (RateVerdict::Within, false, false));
    assert_eq!(verdict(85), (RateVerdict::Within, false, false));
    assert_eq!(verdict(90), (RateVerdict::Above, false, false));
    // Under 20%: the change is blocked.
    assert_eq!(verdict(20), (RateVerdict::Below, false, false));
    assert_eq!(verdict(19), (RateVerdict::Below, true, true));
}

#[test]
fn targets_give_the_design_docs_bands_per_tier() {
    let targets = targets();
    let band = |tier, name: &str| match &targets.checks(tier, &[name])[0] {
        Check::Rate { band, .. } => (band.min, band.max),
        Check::Median { .. } => unreachable!(),
    };
    let rows = [
        (Difficulty::Easy, [(75, 100), (65, 90), (65, 90)]),
        (Difficulty::Normal, [(60, 85), (50, 75), (50, 75)]),
        (Difficulty::Hard, [(55, 80), (40, 65), (40, 65)]),
        (Difficulty::Finale, [(50, 75), (35, 60), (35, 60)]),
    ];
    for (tier, bands) in rows {
        for (name, want) in PERSONAS.iter().zip(bands) {
            assert_eq!(band(tier, name), want, "{tier:?} {name}");
        }
    }
}

#[test]
fn checks_are_a_band_each_then_hardcore_faster_than_normal() {
    let targets = targets();
    let checks = targets.checks(
        Difficulty::Normal,
        &["baseline", "casual", "normal", "hardcore"],
    );
    assert_eq!(checks.len(), 4, "no check for the baseline");
    let floors: Vec<(String, Option<u32>)> = checks[..3]
        .iter()
        .map(|c| match c {
            Check::Rate { entrant, floor, .. } => (entrant.clone(), *floor),
            Check::Median { .. } => unreachable!(),
        })
        .collect();
    assert_eq!(
        floors,
        [
            ("casual".to_owned(), Some(20)),
            ("normal".to_owned(), None),
            ("hardcore".to_owned(), None),
        ]
    );
    assert_eq!(
        checks[3],
        Check::Median {
            entrant: "hardcore".to_owned(),
            measure: "turns".to_owned(),
            among: targets.success["hardcore"].clone(),
            want: Want::Lower,
            than: "normal".to_owned(),
        }
    );
    // Without both of them, no faster check.
    assert_eq!(targets.checks(Difficulty::Normal, &["hardcore"]).len(), 1);
    assert_eq!(targets.checks(Difficulty::Normal, &["normal"]).len(), 1);
    assert!(targets.checks(Difficulty::Normal, &["baseline"]).is_empty());
    assert!(Targets::from_ron("(success: {})").is_err());
}

#[test]
fn hardcore_faster_is_judged_on_the_turns_median() {
    let checks = targets().checks(Difficulty::Normal, &["normal", "hardcore"]);
    let success = |seed, turns| {
        tried(
            seed,
            &[("won", 1), ("deaths", 0), ("items", 0), ("turns", turns)],
        )
    };
    let report = judge(
        vec![
            ("normal".to_owned(), vec![success(1, 14), success(2, 12)]),
            ("hardcore".to_owned(), vec![success(1, 10), success(2, 11)]),
        ],
        &checks,
    )
    .unwrap();
    assert_eq!(
        report.checks[2].found,
        Found::Median {
            median: Some(10),
            than: Some(12),
            verdict: shuyi_core::bots::playtest::MedianVerdict::Pass,
        }
    );
}

// ---- Entrants and measures ----------------------------------------------------

fn measures(tries: &[Tried]) -> Vec<Vec<(String, i64)>> {
    tries
        .iter()
        .map(|t| t.measures.iter().map(|(n, v)| (n.clone(), *v)).collect())
        .collect()
}

fn baseline(mode: GameMode, stage: &BattleStage) -> BattleEntrant {
    BattleEntrant::new("baseline", mode, Player::Baseline, content().ai, stage)
}

/// 0505's report of the baseline on `quick`, seeds 1–6, the same in both
/// modes: won, who fell on which turn, items, turns.
const QUICK_0505: [(bool, &[(&str, i64)], i64, i64); 6] = [
    (true, &[("Test Mage", 1)], 0, 4),
    (true, &[], 0, 2),
    (true, &[], 0, 2),
    (false, &[("Test Lord", 1)], 0, 1),
    (false, &[("Test Lord", 1)], 0, 1),
    (true, &[], 0, 2),
];

#[test]
fn the_baseline_report_numbers_are_0505s() {
    let stage = stage("quick");
    for mode in [GameMode::Classic, GameMode::Casual] {
        let entrant = baseline(mode, &stage);
        assert_eq!(entrant.mode(), mode);
        let tries = play_tries(&stage, &entrant, 1..7, 60).unwrap();
        for (t, (won, fell, items, turns)) in tries.iter().zip(QUICK_0505) {
            let m = &t.measures;
            let falls = i64::try_from(fell.len()).unwrap();
            let lord = fell.iter().any(|(name, _)| *name == "Test Lord");
            assert_eq!(m["won"], i64::from(won), "{mode:?} {t:?}");
            assert_eq!(m["items"], items, "{t:?}");
            assert_eq!(m["turns"], turns, "{t:?}");
            assert_eq!(m["lord fell"], i64::from(lord), "{t:?}");
            let (deaths, retreats) = match mode {
                GameMode::Classic => (falls, 0),
                GameMode::Casual => (0, falls),
            };
            assert_eq!((m["deaths"], m["retreats"]), (deaths, retreats), "{t:?}");
            for &(name, turn) in fell {
                assert_eq!(m[&format!("fell {name}")], 1, "{t:?}");
                assert_eq!(m[&format!("fell {name} turn")], turn, "{t:?}");
            }
            assert_eq!(m.len(), 6 + 2 * fell.len(), "{t:?}");
        }
    }
}

#[test]
fn a_try_stopped_by_the_cap_counts_the_cap_as_its_turns() {
    let stage = stage("quick");
    let tries = play_tries(&stage, &baseline(GameMode::Classic, &stage), 1..2, 2).unwrap();
    assert_eq!(tries[0].outcome, None);
    assert_eq!(tries[0].turns, 3);
    assert_eq!(tries[0].measures["turns"], 2);
    assert_eq!(tries[0].measures["won"], 0);
    let none = play_tries(&stage, &baseline(GameMode::Classic, &stage), 1..2, 0).unwrap();
    assert_eq!(none[0].measures["turns"], 0);
}

#[test]
fn an_entrant_plays_its_mode_with_the_trys_seed() {
    let stage = stage("quick");
    let normal = BattleEntrant::persona("normal", personas()["normal"], content().ai, &stage);
    let casual = BattleEntrant::persona("casual", personas()["casual"], content().ai, &stage);
    assert_eq!(normal.name(), "normal");
    assert_eq!(casual.mode(), GameMode::Casual);
    let state = casual.start(&stage, 9);
    let mut want = stage.casual.clone();
    want.seed = 9;
    assert_eq!(state, BattleState::new(want).0);
    let sides: Vec<Phase> = normal.bots(1).iter().map(|(side, _)| *side).collect();
    assert_eq!(sides, [Phase::Player, Phase::Enemy, Phase::Other]);
}

#[test]
fn a_fallen_unit_the_cast_doesnt_know_is_named_by_its_id() {
    // A lord falls on the enemy's first strike.
    let battle = BattleStage {
        classic: setup(vec![lord(weak(player(1, 5))), rigged(enemy(2, 6))]),
        casual: setup(vec![]),
    };
    let mut entrant = baseline(GameMode::Classic, &battle);
    let tries = play_tries(&battle, &entrant, 1..2, 5).unwrap();
    assert_eq!(tries[0].measures["lord fell"], 1);
    assert_eq!(tries[0].measures["fell Swordsman"], 1);
    entrant.cast.clear();
    let tries = play_tries(&battle, &entrant, 1..2, 5).unwrap();
    assert_eq!(tries[0].measures["fell unit 1"], 1);
    assert_eq!(tries[0].measures["lord fell"], 0);
}

#[test]
fn every_persona_plays_the_same_seed_the_same_way() {
    let stage = stage("quick");
    for name in PERSONAS {
        let entrant = BattleEntrant::persona(name, personas()[name], content().ai, &stage);
        let once = play_tries(&stage, &entrant, 3..4, 2).unwrap();
        assert_eq!(
            measures(&play_tries(&stage, &entrant, 3..4, 2).unwrap()),
            measures(&once)
        );
    }
}

// ---- Random battles -----------------------------------------------------------

/// A small rout on the test map's plain row: one to three player
/// swordsmen (the first a lord) and at least one enemy, at distinct tiles.
fn arb_battle() -> impl Strategy<Value = BattleStage> {
    let tiles = proptest::sample::subsequence((0..9).collect::<Vec<i32>>(), 2..=6);
    (tiles, 0..3usize).prop_map(|(tiles, more)| {
        let players = (1 + more).min(tiles.len() - 1);
        let units: Vec<_> = tiles
            .iter()
            .enumerate()
            .map(|(i, &x)| {
                let id = u32::try_from(i).unwrap() + 1;
                match i {
                    0 => lord(player(id, x)),
                    i if i < players => player(id, x),
                    _ => enemy(id, x),
                }
            })
            .collect();
        let in_mode = |mode| BattleSetup {
            mode,
            ..setup(units.clone())
        };
        BattleStage {
            classic: in_mode(GameMode::Classic),
            casual: in_mode(GameMode::Casual),
        }
    })
}

proptest! {
    #![proptest_config(ProptestConfig { cases: 6, ..ProptestConfig::default() })]

    /// Every persona's commands are accepted, to the battle's end or the
    /// turn cap.
    #[test]
    fn personas_play_random_battles_without_a_refused_command(battle in arb_battle(), seed in 1..1000u64) {
        for name in PERSONAS {
            let entrant = BattleEntrant::persona(name, personas()[name], content().ai, &battle);
            let tries = play_tries(&battle, &entrant, seed..seed + 1, 2);
            prop_assert!(tries.is_ok(), "{name}: {:?}", tries.err().map(|e| e.to_string()));
        }
    }
}
