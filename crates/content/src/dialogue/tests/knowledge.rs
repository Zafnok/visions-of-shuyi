//! Where the party learned what it tells Dace about Wren (0753). The
//! finale's reveal (`ch08_victory`, then Dace alone in `ch08_dock`) repeats
//! two things only Rue saw: Wren walked off the Ashfields on her own feet,
//! and two keepers in gray had her. In Classic, Rue may have fallen before
//! she could say so (`ch06_after`), and an `@if` block can only ask who is
//! there now, not whether a fact was said earlier. These tests play the
//! real scenes, resolved for who is in the army on each path, and read the
//! player text: every relay of a fact needs an earlier source on that path,
//! and "Rue saw her" needs Rue to have said it herself.

use std::collections::BTreeMap;

use super::*;

const HOLLIS: &str = "retainer";
const RUE: &str = "heretic";
const CRANE: &str = "vowmaster";
/// Everyone else the army can have by Chapter 6; none of them carries
/// these facts, but the scenes are played as they would be with them.
const OTHERS: [&str; 7] = [
    "sergeant",
    "poacher",
    "keeper",
    "defector",
    "shieldbearer",
    "battlemage",
    "prizefighter",
];

/// A fact about how Wren left the field, as the player text carries it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Fact {
    /// She walked off the field, after, alive.
    WalkedOff,
    /// Two keepers in gray had her.
    KeepersInGray,
    /// The attribution: Rue is the one who saw it.
    RueSaw,
}

impl Fact {
    fn all() -> [Fact; 3] {
        [Fact::WalkedOff, Fact::KeepersInGray, Fact::RueSaw]
    }

    /// Whether `text` (rendered, lower case) carries the fact.
    fn is_in(self, text: &str) -> bool {
        let any = |needles: &[&str]| needles.iter().any(|n| text.contains(n));
        match self {
            Fact::WalkedOff => any(&[
                "walked out",
                "walked off",
                "walking off",
                "walked her off",
                "walk out of the fire",
            ]),
            Fact::KeepersInGray => any(&["keepers in gray", "keeper in gray"]),
            Fact::RueSaw => any(&["rue saw", "what the girl saw"]),
        }
    }
}

/// A scene on a path: its id and who is in the army (or on the map, for a
/// battle scene) when it plays, besides the lead and [`OTHERS`].
type PathStep = (&'static str, &'static [&'static str]);

/// What a path's scenes established, read in play order.
#[derive(Debug, Default)]
struct Knowledge {
    /// Where each fact was first said, by anyone who could know it.
    first_said: BTreeMap<Fact, &'static str>,
    /// Whether Rue herself has said she saw Wren walk off.
    rue_told: bool,
    /// Every problem found: a relay of a fact nobody established, or
    /// "Rue saw her" where she never said so.
    problems: Vec<String>,
}

fn present(there: &[&str]) -> Present {
    let everyone = ["lead"]
        .into_iter()
        .chain(OTHERS)
        .chain(there.iter().copied())
        .map(id);
    Present::only(everyone)
}

/// Plays `steps` in order and reads what each scene tells the player, as
/// the game renders it for those there.
fn play(steps: &[PathStep]) -> Knowledge {
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let names = &content.names;
    let mut known = Knowledge::default();
    for &(scene_id, there) in steps {
        let scene = content
            .dialogue
            .get(scene_id)
            .unwrap_or_else(|| panic!("no scene {scene_id}"));
        let resolved = scene.resolved(&present(there));
        for line in resolved.lines() {
            let text = names.substitute(line.text).to_lowercase();
            let facts: Vec<Fact> = Fact::all().into_iter().filter(|f| f.is_in(&text)).collect();
            if facts.is_empty() {
                continue;
            }
            // Rue saw it; Crane has her. Anyone else repeats what they heard.
            let source = line.speaker == RUE || line.speaker == CRANE;
            for fact in &facts {
                let known_before = known.first_said.contains_key(fact);
                match fact {
                    Fact::RueSaw if !known.rue_told => known.problems.push(format!(
                        "{scene_id}: {} says Rue saw her, but Rue never said so on this path: \
                         {text:?}",
                        line.speaker
                    )),
                    Fact::WalkedOff | Fact::KeepersInGray if !source && !known_before => {
                        known.problems.push(format!(
                            "{scene_id}: {} says {fact:?} with no earlier source on this path: \
                             {text:?}",
                            line.speaker
                        ));
                    }
                    _ => {}
                }
                known.first_said.entry(*fact).or_insert(scene_id);
            }
            if line.speaker == RUE && facts.contains(&Fact::WalkedOff) {
                known.rue_told = true;
            }
        }
    }
    known
}

/// The lines of `scene_id` for those `there`, rendered and lower case.
fn rendered(scene_id: &str, there: &[&str]) -> Vec<(String, String)> {
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let scene = &content.dialogue.scenes[scene_id];
    scene
        .resolved(&present(there))
        .lines()
        .iter()
        .map(|l| {
            (
                l.speaker.to_owned(),
                content.names.substitute(l.text).to_lowercase(),
            )
        })
        .collect()
}

/// The reveal reaches Dace on every path: the gangway and the dock carry
/// both facts, with a source on that path and no attribution Rue never
/// made; and where each fact was first said is where the path says.
fn check(name: &str, steps: &[PathStep], first_said: &[(Fact, &str)], rue_told: bool) -> Knowledge {
    let known = play(steps);
    assert_eq!(known.problems, Vec::<String>::new(), "{name}");
    let expected: BTreeMap<Fact, &str> = first_said.iter().copied().collect();
    assert_eq!(
        known.first_said, expected,
        "{name}: where each fact was first said"
    );
    assert_eq!(
        known.rue_told, rue_told,
        "{name}: whether Rue told it herself"
    );
    for &(scene_id, there) in steps {
        if scene_id != "ch08_victory" && scene_id != "ch08_dock" {
            continue;
        }
        let text: String = rendered(scene_id, there)
            .into_iter()
            .map(|(_, t)| t)
            .collect::<Vec<_>>()
            .join("\n");
        for fact in [Fact::WalkedOff, Fact::KeepersInGray] {
            assert!(
                fact.is_in(&text),
                "{name}: {scene_id} doesn't carry {fact:?}:\n{text}"
            );
        }
    }
    known
}

const DEAD_GROUND: &str = "ch06_after";
const CRANE_SCENE: &str = "ch07_crane";
const GANGWAY: &str = "ch08_victory";
const DOCK: &str = "ch08_dock";
const HOLLIS_FALLS: &str = "ch08_death_retainer";

// --- Rue dead before Chapter 6: nobody saw it for the party ---------------------

/// Hollis lives to the gangway. Nobody told him on the dead ground; Crane
/// tells him under Ossary, and on the quay he says what he knows without
/// Rue's name.
#[test]
fn rue_fell_before_the_ashfields_and_hollis_dies_on_the_quay() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS]),
        (CRANE_SCENE, &[HOLLIS]),
        (GANGWAY, &[HOLLIS]),
        (DOCK, &[]),
    ];
    let known = check(
        "Rue fell before Chapter 6, Hollis dies on the quay",
        steps,
        &[
            (Fact::WalkedOff, CRANE_SCENE),
            (Fact::KeepersInGray, CRANE_SCENE),
        ],
        false,
    );
    assert!(!known.first_said.contains_key(&Fact::RueSaw));
}

/// Both gone before the Ashfields: the lead asks Crane, and the lead tells
/// Dace.
#[test]
fn rue_and_hollis_both_fell_before_the_ashfields() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[]),
        (CRANE_SCENE, &[]),
        (GANGWAY, &[]),
        (DOCK, &[]),
    ];
    check(
        "Rue and Hollis fell before Chapter 6",
        steps,
        &[
            (Fact::WalkedOff, CRANE_SCENE),
            (Fact::KeepersInGray, CRANE_SCENE),
        ],
        false,
    );
    // The lead carries Hollis's question to Crane.
    let crane = rendered(CRANE_SCENE, &[]);
    assert!(
        crane
            .iter()
            .any(|(s, t)| s == "lead" && t.contains("where is she")),
        "{crane:?}"
    );
}

/// Rue dead before the Ashfields; Hollis heard Crane and fell in the
/// Undercroft. The lead was in the room.
#[test]
fn rue_fell_before_the_ashfields_and_hollis_in_the_undercroft() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS]),
        (CRANE_SCENE, &[HOLLIS]),
        (GANGWAY, &[]),
        (DOCK, &[]),
    ];
    check(
        "Rue fell before Chapter 6, Hollis in the Undercroft",
        steps,
        &[
            (Fact::WalkedOff, CRANE_SCENE),
            (Fact::KeepersInGray, CRANE_SCENE),
        ],
        false,
    );
}

/// Hollis falls in the docks battle with Rue long dead: his last words
/// hand the lead the job without "what the girl saw".
#[test]
fn rue_fell_before_the_ashfields_and_hollis_in_the_docks_battle() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS]),
        (CRANE_SCENE, &[HOLLIS]),
        (HOLLIS_FALLS, &[HOLLIS]),
        (GANGWAY, &[]),
        (DOCK, &[]),
    ];
    check(
        "Rue fell before Chapter 6, Hollis in the docks battle",
        steps,
        &[
            (Fact::WalkedOff, CRANE_SCENE),
            (Fact::KeepersInGray, CRANE_SCENE),
        ],
        false,
    );
}

// --- Rue told it, then died: what was learned stays learned ----------------------

/// Rue told Hollis on the dead ground and fell in the Undercroft: on the
/// quay he still says she walked off with two keepers in gray.
#[test]
fn rue_told_hollis_then_fell_in_the_undercroft() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS, RUE]),
        (CRANE_SCENE, &[HOLLIS, RUE]),
        (GANGWAY, &[HOLLIS]),
        (DOCK, &[]),
    ];
    check(
        "Rue told Hollis, then fell in the Undercroft",
        steps,
        &[
            (Fact::WalkedOff, DEAD_GROUND),
            (Fact::KeepersInGray, DEAD_GROUND),
        ],
        true,
    );
}

/// Rue told Hollis on the dead ground, and both are gone before the ship:
/// the lead, who sat with them, tells Dace.
#[test]
fn rue_told_hollis_then_both_fell_before_the_ship() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS, RUE]),
        (CRANE_SCENE, &[HOLLIS]),
        (GANGWAY, &[]),
        (DOCK, &[]),
    ];
    check(
        "Rue told Hollis, then both fell",
        steps,
        &[
            (Fact::WalkedOff, DEAD_GROUND),
            (Fact::KeepersInGray, DEAD_GROUND),
        ],
        true,
    );
}

/// Rue fell after the dead ground but before Ossary: Crane confirms what
/// she said, which is no contradiction, and the party already knew.
#[test]
fn crane_confirms_what_rue_told_when_she_has_since_fallen() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS, RUE]),
        (CRANE_SCENE, &[HOLLIS]),
        (GANGWAY, &[HOLLIS]),
        (DOCK, &[]),
    ];
    check(
        "Rue told Hollis and fell in the crossing to Ossary",
        steps,
        &[
            (Fact::WalkedOff, DEAD_GROUND),
            (Fact::KeepersInGray, DEAD_GROUND),
        ],
        true,
    );
    let crane = rendered(CRANE_SCENE, &[HOLLIS]);
    assert!(
        crane
            .iter()
            .any(|(s, t)| s == CRANE && Fact::WalkedOff.is_in(t) && Fact::KeepersInGray.is_in(t)),
        "{crane:?}"
    );
}

// --- Rue alive: her own account, and her name -----------------------------------

/// Everyone lives to the quay: Hollis dies there and names her.
#[test]
fn everyone_lives_and_hollis_names_rue_on_the_quay() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS, RUE]),
        (CRANE_SCENE, &[HOLLIS, RUE]),
        (GANGWAY, &[HOLLIS, RUE]),
        (DOCK, &[RUE]),
    ];
    let known = check(
        "everyone lives",
        steps,
        &[
            (Fact::WalkedOff, DEAD_GROUND),
            (Fact::KeepersInGray, DEAD_GROUND),
            (Fact::RueSaw, GANGWAY),
        ],
        true,
    );
    assert_eq!(known.first_said[&Fact::RueSaw], GANGWAY);
    // With Rue in the room, Crane doesn't repeat what she told them.
    let crane = rendered(CRANE_SCENE, &[HOLLIS, RUE]);
    assert!(
        !crane
            .iter()
            .any(|(s, t)| s == CRANE && Fact::WalkedOff.is_in(t)),
        "{crane:?}"
    );
}

/// Hollis fell before the Ashfields; Rue told the lead on the dead ground
/// and says what she saw to Dace herself.
#[test]
fn hollis_fell_before_the_ashfields_and_rue_tells_dace_herself() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[RUE]),
        (CRANE_SCENE, &[RUE]),
        (GANGWAY, &[RUE]),
        (DOCK, &[RUE]),
    ];
    check(
        "Hollis fell before Chapter 6, Rue lives",
        steps,
        &[
            (Fact::WalkedOff, DEAD_GROUND),
            (Fact::KeepersInGray, DEAD_GROUND),
        ],
        true,
    );
    let gangway = rendered(GANGWAY, &[RUE]);
    assert!(
        gangway
            .iter()
            .any(|(s, t)| s == RUE && t.contains("i saw her")),
        "{gangway:?}"
    );
}

/// Hollis falls in the docks battle with Rue on the map: his last words
/// may name what the girl saw, because she told him.
#[test]
fn hollis_falls_in_the_docks_battle_with_rue_there() {
    let steps: &[PathStep] = &[
        (DEAD_GROUND, &[HOLLIS, RUE]),
        (CRANE_SCENE, &[HOLLIS, RUE]),
        (HOLLIS_FALLS, &[HOLLIS, RUE]),
        (GANGWAY, &[RUE]),
        (DOCK, &[RUE]),
    ];
    check(
        "Hollis falls in the docks battle, Rue there",
        steps,
        &[
            (Fact::WalkedOff, DEAD_GROUND),
            (Fact::KeepersInGray, DEAD_GROUND),
            (Fact::RueSaw, HOLLIS_FALLS),
        ],
        true,
    );
}

/// The reader of the facts sees them in the lines that carry them, and
/// not in a line about something else.
#[test]
fn the_facts_are_read_from_the_words() {
    assert!(Fact::WalkedOff.is_in("she walked out of the fire."));
    assert!(Fact::WalkedOff.is_in("walking off the field, after"));
    assert!(!Fact::WalkedOff.is_in("somebody's walked in off the king's side"));
    assert!(Fact::KeepersInGray.is_in("every keeper in gray between this coast"));
    assert!(!Fact::KeepersInGray.is_in("the gray cloak's in the boat"));
    assert!(Fact::RueSaw.is_in("rue saw her."));
    assert!(Fact::RueSaw.is_in("you tell him what the girl saw on the plain"));
    assert!(!Fact::RueSaw.is_in("i saw her. after."));
}
