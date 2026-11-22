//! Tests of the scenes' lines the battle keeps loaded beside the guide's
//! (ticket 1296), on the Quick Battle's triggers
//! (`assets-test/battles/quick.ron`, `dialogue/test_triggers.dlg`).

use core::fmt::Write as _;

use trpg_content::LineId;
use trpg_core::{BattleSetup, Objective, Phase, Trigger, TriggerWhen};

use super::testing::{quick_units, setup};
use super::*;
use crate::AudioRequest;
use crate::screen::tests::ctx;

/// The Quick Battle's trigger scenes.
const SCENES: [&str; 9] = [
    "test_turn_3",
    "test_fort",
    "test_engage",
    "test_rogue_half",
    "test_rogue_falls",
    "test_lord_falls",
    "test_knight_dies",
    "test_knight_retreats",
    "test_talk",
];

/// The test context with the lessons on, and a clip for every line of
/// the trigger scenes and every line the guide says.
fn voiced() -> Ctx {
    let mut c = ctx();
    c.game.lessons = crate::screen::Lessons::On;
    let content = &c.game.content;
    let guide = trpg_content::voice::guide_scene(&content.lessons, &content.tips);
    let scenes = SCENES.iter().map(|id| &content.dialogue.scenes[*id]);
    let mut clips = String::new();
    for scene in scenes.chain([&guide]) {
        for l in scene.lines() {
            let _ = writeln!(
                clips,
                "(line: \"{}\", spoken: \"{}\", voice: \"v\", made_by: Recorded(actor: \"a\")),",
                l.id, l.text
            );
        }
    }
    c.set_voice_manifest(&format!("(clips: [\n{clips}])"));
    assert_eq!(c.take_warnings(), [] as [String; 0]);
    c.audio.take();
    c
}

/// Scene `id`'s lines (each of these has three or fewer: all are first).
fn lines_of(c: &Ctx, id: &str) -> Vec<LineId> {
    let scene = &c.game.content.dialogue.scenes[id];
    scene.lines().into_iter().map(|l| l.id.clone()).collect()
}

/// The sets kept since the last call.
fn kept(c: &mut Ctx) -> Vec<Vec<LineId>> {
    c.audio
        .take()
        .into_iter()
        .filter_map(|r| match r {
            AudioRequest::KeepVoices { lines } => Some(lines.into_iter().map(|(l, _)| l).collect()),
            _ => None,
        })
        .collect()
}

/// One frame with no input; the screen's answer.
fn frame(s: &mut BattleScreen, c: &mut Ctx) -> String {
    format!("{:?}", s.update(c, &FrameInput::new(vec![], 0.0, vec![])))
}

/// Acceptance: the battle's first set holds the first lines of every
/// trigger scene it may play (the rogue's too: he arrives on turn 2) and
/// the guide's lines; not the knight's retreat line, which plays only in
/// Casual (the Quick Battle is Classic).
#[test]
fn the_first_set_holds_the_trigger_scenes_and_the_guides_lines() {
    let mut c = voiced();
    let mut s = BattleScreen::new(quick_battle(&c.game.content).unwrap()).in_battle(QUICK_BATTLE);
    let guide = s.guide.coming_guide_lines(&c, s.battle_id.as_deref());
    assert_ne!(guide, [] as [LineId; 0]);
    assert_eq!(frame(&mut s, &mut c), "None");
    let sets = kept(&mut c);
    let [set] = sets.as_slice() else {
        panic!("{sets:?}");
    };
    for id in SCENES.iter().filter(|id| **id != "test_knight_retreats") {
        for line in lines_of(&c, id) {
            assert!(set.contains(&line), "{id}'s {line} not in {set:?}");
        }
    }
    for line in lines_of(&c, "test_knight_retreats") {
        assert!(!set.contains(&line), "{line} kept in Classic");
    }
    for line in &guide {
        assert!(set.contains(line), "the guide's {line} not kept");
    }
}

/// A scene whose character is neither on the map nor coming, or whose
/// turn has passed, is left out: the trigger can't fire.
#[test]
fn a_scene_that_cannot_play_is_left_out() {
    let c = voiced();
    let (map, units) = quick_units(&c);
    let mut triggers = c.game.content.battles[QUICK_BATTLE].triggers.clone();
    triggers.push(Trigger {
        when: TriggerWhen::TurnStart {
            turn: 0,
            phase: Phase::Player,
        },
        scene: "test_knight_retreats".into(),
        once: true,
    });
    let state = BattleState::new(BattleSetup {
        triggers,
        ..setup(&c, map, units, Objective::Rout { turn_limit: None })
    })
    .0;
    let lines = BattleScreen::new(state).coming_scene_lines(&c);
    let had = |id: &str| lines_of(&c, id).iter().all(|l| lines.contains(l));
    assert!(had("test_lord_falls") && had("test_knight_dies") && had("test_fort"));
    for gone in [
        "test_engage",
        "test_rogue_half",
        "test_rogue_falls",
        "test_talk",
        "test_knight_retreats",
    ] {
        assert!(
            !lines_of(&c, gone).iter().any(|l| lines.contains(l)),
            "{gone}"
        );
    }
}

/// A scene fired and waiting in the queue is kept though its trigger is
/// spent, and stays kept up to the frame it is pushed.
#[test]
fn a_fired_scene_is_kept_until_it_plays() {
    let mut c = voiced();
    let (map, units) = quick_units(&c);
    let (state, events) = BattleState::new(BattleSetup {
        triggers: vec![Trigger {
            when: TriggerWhen::TurnStart {
                turn: 1,
                phase: Phase::Player,
            },
            scene: "test_turn_3".into(),
            once: true,
        }],
        ..setup(&c, map, units, Objective::Rout { turn_limit: None })
    });
    assert!(state.has_fired(0));
    let mut s = BattleScreen::start(state, &events);
    let line = lines_of(&c, "test_turn_3");
    let mut last: Vec<LineId> = Vec::new();
    let mut pushed = false;
    for _ in 0..4 {
        let answer = format!(
            "{:?}",
            s.update(&mut c, &FrameInput::new(vec![Action::Confirm], 0.0, vec![]))
        );
        if let Some(set) = kept(&mut c).pop() {
            last = set;
        }
        if answer == "Push(dialogue)" {
            pushed = true;
            break;
        }
    }
    assert!(pushed);
    assert!(line.iter().all(|l| last.contains(l)), "{last:?}");
}

/// The set is built when due, while the battle stays on top or as it
/// pushes a scene; never as another screen goes over it, and never when
/// not due.
#[test]
fn the_set_is_built_when_due_on_top_or_pushing_a_scene() {
    for stays in [false, true] {
        for scene in [false, true] {
            for due in [false, true] {
                let want = due && (stays || scene);
                let got = super::guide::Guide::set_due(stays, scene, due);
                assert_eq!(got, want, "stays {stays}, scene {scene}, due {due}");
            }
        }
    }
}

/// A scene the battle pushes on its first frame is in the first set,
/// sent that frame.
#[test]
fn a_scene_pushed_on_the_first_frame_is_in_its_first_set() {
    let mut c = voiced();
    let (map, units) = quick_units(&c);
    let (state, events) = BattleState::new(BattleSetup {
        triggers: vec![Trigger {
            when: TriggerWhen::TurnStart {
                turn: 1,
                phase: Phase::Player,
            },
            scene: "test_turn_3".into(),
            once: true,
        }],
        ..setup(&c, map, units, Objective::Rout { turn_limit: None })
    });
    let mut s = BattleScreen::start(state, &events);
    s.queue.retain(|q| matches!(q, Queued::Scene(_)));
    assert_eq!(frame(&mut s, &mut c), "Push(dialogue)");
    let sets = kept(&mut c);
    let [set] = sets.as_slice() else {
        panic!("{sets:?}");
    };
    for line in lines_of(&c, "test_turn_3") {
        assert!(set.contains(&line), "{line} not in {set:?}");
    }
}

/// A turn-start scene still to come this turn (turn 1's enemy phase, at
/// the battle's start) is kept.
#[test]
fn a_scene_later_this_turn_is_kept() {
    let c = voiced();
    let (map, units) = quick_units(&c);
    let state = BattleState::new(BattleSetup {
        triggers: vec![Trigger {
            when: TriggerWhen::TurnStart {
                turn: 1,
                phase: Phase::Enemy,
            },
            scene: "test_turn_3".into(),
            once: true,
        }],
        ..setup(&c, map, units, Objective::Rout { turn_limit: None })
    })
    .0;
    assert!(!state.has_fired(0));
    let lines = BattleScreen::new(state).coming_scene_lines(&c);
    assert_eq!(lines, lines_of(&c, "test_turn_3"));
}
