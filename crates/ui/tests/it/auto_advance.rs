//! Auto-advance (ticket 0859): with the Options row on, a scene's text box
//! moves on by itself once its voice clip and a pause, or its reading time,
//! are over; a reply choice always waits. Time is stepped with
//! [`Harness::wait`].

use trpg_content::{ChoiceOption, Present, Scene, Step};
use trpg_core::{CharacterId, LeadGender, LeadProfile};
use trpg_ui::harness::{FRAME_DT, Harness};
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::DialogueScreen;
use trpg_ui::screens::options::{Row, shown_rows};
use trpg_ui::settings::{AutoAdvance, TextSpeed};

/// Forty characters: read in 1 s + 40 × 0.05 s = 3 s; said, as clips are
/// taken to last until the platform reports their end, in 40 × 0.08 s =
/// 3.2 s, then the 0.5 s pause.
const LONG: &str = "The river is high, and the ford is gone.";

fn narrate(text: &str) -> Step {
    Step::Narrate {
        direction: None,
        text: text.into(),
        line: trpg_content::LineId::default(),
    }
}

/// A game playing `steps`, its lines given ids, the whole page shown at
/// once, and auto-advance `on` or off.
fn playing(steps: Vec<Step>, on: bool) -> Harness {
    let mut scene = Scene {
        id: "auto".into(),
        steps,
    };
    scene.assign_line_ids();
    let names = trpg_content::load_embedded().map(|c| c.names);
    let mut h = Harness::with_screen(Box::new(DialogueScreen::new(
        &scene,
        LeadProfile::new("Ellery", LeadGender::Male),
        names.unwrap_or_default(),
        &Present::Everyone,
    )));
    let set = |s: &mut trpg_ui::settings::Settings| {
        s.text_speed = TextSpeed::Instant;
        s.game.auto_advance = if on {
            AutoAdvance::On
        } else {
            AutoAdvance::Off
        };
    };
    assert!(h.ctx_mut().change_settings(set).is_ok());
    h
}

/// [`playing`], with a voice clip for every line of `steps`.
fn voiced(steps: Vec<Step>) -> Harness {
    let mut scene = Scene {
        id: "auto".into(),
        steps: steps.clone(),
    };
    scene.assign_line_ids();
    let clip = |l: &trpg_content::dialogue::Line| {
        format!(
            "(line: \"{}\", spoken: \"{}\", voice: \"v\", \
             made_by: Generated(tool: \"t\", model: \"m\", date: \"2026-11-06\")),\n",
            l.id, l.text
        )
    };
    let clips: String = scene.lines().iter().map(clip).collect();
    let mut h = playing(steps, true);
    // The manifest's lines must be the content's.
    let scenes = &mut h.ctx_mut().game.content.dialogue.scenes;
    scenes.insert(scene.id.clone(), scene);
    h.ctx_mut()
        .set_voice_manifest(&format!("(clips: [\n{clips}])"));
    assert_eq!(h.ctx_mut().take_warnings(), [] as [String; 0]);
    h
}

/// The text box's page, in full, its lines joined by spaces.
fn page(h: &Harness) -> String {
    let Some(screen) = h.game().screen::<DialogueScreen>() else {
        return String::new();
    };
    screen.view(h.game().ctx()).text.lines.join(" ")
}

/// Acceptance: off, a page waits for a key however long it is left.
#[test]
fn off_nothing_advances_by_itself() {
    let mut h = playing(vec![narrate("One."), narrate("Two.")], false);
    h.wait(30.0);
    assert_eq!(page(&h), "One.");
    h.keys("f");
    assert_eq!(page(&h), "Two.");
}

/// Acceptance: on, a silent line moves on after its reading time, 1 s and
/// 0.05 s a character, and the last one closes the scene.
#[test]
fn a_silent_line_advances_after_its_reading_time() {
    let mut h = playing(vec![narrate(LONG), narrate("Two.")], true);
    h.wait(2.9);
    assert_eq!(page(&h), LONG);
    h.wait(0.2);
    assert_eq!(page(&h), "Two.");
    // "Two.": 1.2 s.
    h.wait(1.1);
    assert_eq!(h.top_screen(), "dialogue");
    h.wait(0.2);
    assert_eq!(h.top_screen(), "");
}

/// Acceptance: on, a voiced line moves on once its clip has ended and the
/// pause passed: later than reading alone would.
#[test]
fn a_voiced_line_advances_after_its_clip_and_the_pause() {
    let mut h = voiced(vec![narrate(LONG), narrate("Two.")]);
    h.wait(3.6);
    assert_eq!(page(&h), LONG);
    assert_eq!(h.voices().len(), 1, "its clip plays");
    h.wait(0.2);
    assert_eq!(page(&h), "Two.");
}

/// The reading time starts once the page is all typed: at Normal speed
/// (60 characters a second) [`LONG`] takes about 0.67 s to type, then 3 s
/// to read.
#[test]
fn the_reading_time_starts_once_the_page_is_typed() {
    let mut h = playing(vec![narrate(LONG), narrate("Two.")], true);
    let normal = |s: &mut trpg_ui::settings::Settings| s.text_speed = TextSpeed::Normal;
    assert!(h.ctx_mut().change_settings(normal).is_ok());
    h.wait(3.4);
    assert_eq!(page(&h), LONG);
    h.wait(0.5);
    assert_eq!(page(&h), "Two.");
}

/// "Skip scene?" holds the page however long it is open, and the wait
/// starts over once it closes.
#[test]
fn the_skip_question_holds_the_page() {
    let mut h = playing(vec![narrate("One."), narrate("Two.")], true);
    h.keys("d");
    let asking = |h: &Harness| {
        let screen = h.game().screen::<DialogueScreen>();
        screen.is_some_and(DialogueScreen::is_asking_skip)
    };
    assert!(asking(&h));
    h.wait(30.0);
    assert!(asking(&h));
    assert_eq!(page(&h), "One.");
    // Cancel answers no: back to the page, which waits its reading time
    // again (1.2 s).
    h.keys("d");
    assert!(!asking(&h));
    h.wait(1.0);
    assert_eq!(page(&h), "One.");
    h.wait(0.4);
    assert_eq!(page(&h), "Two.");
}

/// A press still works as usual, and starts the wait over on the next
/// line.
#[test]
fn a_press_still_moves_on_at_once() {
    let mut h = playing(vec![narrate(LONG), narrate(LONG), narrate("Three.")], true);
    h.wait(2.0).keys("f");
    assert_eq!(page(&h), LONG);
    h.wait(2.5);
    assert_eq!(page(&h), LONG, "the second line waits its own time");
    h.wait(0.6);
    assert_eq!(page(&h), "Three.");
}

/// Acceptance: on, a reply choice never moves on by itself.
#[test]
fn a_reply_choice_always_waits() {
    let say = |text: &str| Step::Say {
        direction: None,
        speaker: CharacterId("test_knight".into()),
        expression: None,
        text: text.into(),
        line: trpg_content::LineId::default(),
    };
    let reply = |text: &str, reaction: &str| ChoiceOption {
        tone: "earnest".into(),
        text: text.into(),
        line: trpg_content::LineId::default(),
        steps: vec![say(reaction)],
    };
    let choice = Step::Choice {
        options: vec![reply("I will.", "Good."), reply("No.", "Pity.")],
    };
    let mut h = voiced(vec![say("Will you lead?"), choice]);
    h.wait(30.0);
    let screen = h.game().screen::<DialogueScreen>();
    assert!(
        screen.and_then(DialogueScreen::menu).is_some(),
        "still choosing"
    );
    h.keys("f");
    assert_eq!(page(&h), "Good.");
}

/// Acceptance: the row, under the voice rows, turns it on, and it
/// survives a restart.
#[test]
fn the_row_turns_it_on_and_it_survives_a_restart() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.keys("Down Down f");
    let rows = shown_rows(h.game().ctx());
    let at = rows.iter().position(|&r| r == Row::AutoAdvance).unwrap();
    assert_eq!(rows[at - 1], Row::LogVoice);
    h.keys(&vec!["Down"; at].join(" ")).keys("Right");
    assert_eq!(h.game().ctx().settings().game.auto_advance, AutoAdvance::On);
    // Kept (Cancel asks, two left is Keep), then a restart.
    h.keys("d Left Left f");
    let mut h = Harness::with_storage(h.into_storage());
    h.wait(FRAME_DT);
    assert_eq!(h.game().ctx().settings().game.auto_advance, AutoAdvance::On);
}
