//! Tests of the dialogue screen: reveal timing, paging, skipping and what
//! it shows (its view). The look is tested in `glyph/tests.rs`; scripted
//! runs and snapshots are in `crates/ui/tests/it/dialogue.rs`.

use crate::widgets::menu::LabelText;
use trpg_content::{Side, Stage, Step};
use trpg_core::CharacterId;

use super::*;
use crate::audio::AudioRequest;
use crate::screen::tests::ctx;
use crate::settings::TextSpeed;

/// One frame of `dt` seconds with `actions`, Confirm held if `held`.
fn frame(actions: &[Action], dt: f32, held: bool) -> FrameInput {
    let held = if held { vec![Action::Confirm] } else { vec![] };
    FrameInput::new(actions.to_vec(), dt, held)
}

fn press(s: &mut DialogueScreen, c: &mut Ctx, action: Action) -> Transition {
    s.update(c, &frame(&[action], 0.0, false))
}

fn say(speaker: &str, text: &str) -> Step {
    Step::Say {
        speaker: CharacterId(speaker.into()),
        expression: None,
        text: text.into(),
        line: trpg_content::LineId::default(),
    }
}

fn place(side: Side, character: &str) -> Step {
    Step::Place {
        side,
        character: CharacterId(character.into()),
        expression: "neutral".into(),
    }
}

/// `scene` full-screen, with the default lead.
fn full(scene: &Scene) -> DialogueScreen {
    DialogueScreen::new(
        scene,
        ctx().game.lead,
        ctx().game.content.names,
        &Present::Everyone,
    )
}

/// `scene` as an overlay, with the default lead.
fn over(scene: &Scene) -> DialogueScreen {
    DialogueScreen::overlay(
        scene,
        ctx().game.lead,
        ctx().game.content.names,
        &Present::Everyone,
    )
}

fn scene(steps: Vec<Step>) -> Scene {
    Scene {
        id: "t".into(),
        steps,
    }
}

/// Lord on the left speaking to the knight on the right.
fn two_speakers(text: &str) -> Scene {
    scene(vec![
        place(Side::Left, "test_lord"),
        place(Side::Right, "test_knight"),
        say("test_lord", text),
        say("test_knight", "Second."),
    ])
}

/// The text box's lines as revealed so far.
fn revealed(s: &DialogueScreen, c: &Ctx) -> Vec<String> {
    s.view(c).text.revealed()
}

/// The names of who is on the left and the right.
fn names(view: &DialogueView) -> [Option<&str>; 2] {
    [Side::Left, Side::Right].map(|side| view.face(side).map(|f| f.name.as_str()))
}

/// Line counts of each page of the first box of a [`two_speakers`] scene,
/// pressing Confirm twice per page until the second box. Gives up after
/// 20 pages, so a paging bug fails the test instead of hanging it.
fn page_sizes(s: &mut DialogueScreen, c: &mut Ctx) -> Vec<usize> {
    let mut pages = vec![];
    for _ in 0..20 {
        if s.page_lines() == ["Second."] {
            return pages;
        }
        pages.push(s.page_lines().len());
        press(s, c, Action::Confirm);
        press(s, c, Action::Confirm);
    }
    panic!("never reached the second box; pages so far: {pages:?}");
}

#[test]
fn reveals_at_the_text_speed() {
    let mut c = ctx();
    let mut s = full(&two_speakers("Hello there, knight."));
    assert_eq!(s.name(), "dialogue");
    assert!(!s.is_overlay());
    s.update(&mut c, &frame(&[], 0.1, false));
    assert!(
        (s.text.progress() - 6.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
    assert_eq!(revealed(&s, &c), ["Hello "]);
    assert!(s.view(&c).text.waiting.is_none());
    // The player's text speed (0805): Slow is 30 characters a second.
    c.change_settings(|s| s.text_speed = TextSpeed::Slow)
        .unwrap();
    s.update(&mut c, &frame(&[], 0.1, false));
    assert!(
        (s.text.progress() - 9.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
    c.change_settings(|s| s.text_speed = TextSpeed::Fast)
        .unwrap();
    s.update(&mut c, &frame(&[], 0.05, false));
    assert!(
        (s.text.progress() - 15.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
    // Never past the end.
    s.update(&mut c, &frame(&[], 10.0, false));
    assert!(s.is_revealed());
    assert!((s.text.progress() - 20.0).abs() < 1e-3);
}

#[test]
fn instant_text_shows_each_page_at_once() {
    let mut c = ctx();
    c.change_settings(|s| s.text_speed = TextSpeed::Instant)
        .unwrap();
    let mut s = full(&two_speakers("Hello there, knight."));
    // Even a frame of no time.
    s.update(&mut c, &frame(&[], 0.0, false));
    assert!(s.is_revealed());
    assert!((s.text.progress() - 20.0).abs() < 1e-3);
}

#[test]
fn holding_confirm_fast_forwards() {
    let mut c = ctx();
    let mut s = full(&two_speakers(&"word ".repeat(50)));
    // Normal speed up to the delay (0.3 s at 60 a second), then six times it.
    s.update(&mut c, &frame(&[], 0.3, true));
    assert!(
        (s.text.progress() - 18.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
    s.update(&mut c, &frame(&[], 0.1, true));
    assert!(
        (s.text.progress() - 54.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
    // A frame that crosses the delay is fast only past it.
    let mut s = full(&two_speakers(&"word ".repeat(50)));
    s.update(&mut c, &frame(&[], 0.4, true));
    assert!(
        (s.text.progress() - 54.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
}

/// Two long boxes, the first one fully shown.
fn second_page_next(c: &mut Ctx) -> DialogueScreen {
    let long = "word ".repeat(50);
    let mut s = full(&scene(vec![
        say("test_lord", &long),
        say("test_knight", &long),
    ]));
    press(&mut s, c, Action::Confirm);
    assert!(s.is_revealed());
    s
}

#[test]
fn a_confirm_tap_does_not_fast_forward_the_next_page() {
    let mut c = ctx();
    let mut s = second_page_next(&mut c);
    // The press that turns the page, the key down for 0.1 s more.
    s.update(&mut c, &frame(&[Action::Confirm], 0.05, true));
    s.update(&mut c, &frame(&[], 0.05, true));
    assert!(!s.is_revealed());
    let mut plain = second_page_next(&mut c);
    plain.update(&mut c, &frame(&[Action::Confirm], 0.05, false));
    plain.update(&mut c, &frame(&[], 0.05, false));
    assert!(
        (s.text.progress() - 6.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
    assert!(
        (s.text.progress() - plain.text.progress()).abs() < 1e-6,
        "{}",
        plain.text.progress()
    );
}

#[test]
fn the_fast_forward_delay_starts_again_on_each_page() {
    let mut c = ctx();
    let mut s = second_page_next(&mut c);
    // Held for a second on the shown page, and through the page turn.
    s.update(&mut c, &frame(&[], 1.0, true));
    s.update(&mut c, &frame(&[Action::Confirm], 0.0, true));
    s.update(&mut c, &frame(&[], 0.2, true));
    assert!(
        (s.text.progress() - 12.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
    // Still held: fast once the delay has passed on this page.
    s.update(&mut c, &frame(&[], 0.2, true));
    assert!(
        (s.text.progress() - 54.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
}

#[test]
fn letting_go_starts_the_fast_forward_delay_again() {
    let mut c = ctx();
    let mut s = full(&two_speakers(&"word ".repeat(50)));
    s.update(&mut c, &frame(&[], 0.2, true));
    s.update(&mut c, &frame(&[], 0.1, false));
    s.update(&mut c, &frame(&[], 0.2, true));
    assert!(
        (s.text.progress() - 30.0).abs() < 1e-3,
        "{}",
        s.text.progress()
    );
}

#[test]
fn confirm_reveals_then_advances() {
    let mut c = ctx();
    let mut s = full(&two_speakers("First."));
    assert!(!s.is_revealed());
    assert!(matches!(
        press(&mut s, &mut c, Action::Confirm),
        Transition::None
    ));
    assert!(s.is_revealed());
    assert_eq!(s.page_lines(), ["First."]);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Second."]);
    assert!(!s.is_revealed(), "the next box starts hidden");
    press(&mut s, &mut c, Action::Confirm);
    let end = press(&mut s, &mut c, Action::Confirm);
    assert!(matches!(end, Transition::Pop));
    assert!(s.player().is_finished());
}

#[test]
fn end_turn_advances_like_confirm() {
    let mut c = ctx();
    let mut s = full(&two_speakers("First."));
    press(&mut s, &mut c, Action::EndTurn);
    assert!(s.is_revealed());
    press(&mut s, &mut c, Action::EndTurn);
    assert_eq!(s.page_lines(), ["Second."]);
    // Held, it fast-forwards.
    let mut long = full(&two_speakers(&"word ".repeat(50)));
    let held = FrameInput::new(vec![], 0.4, vec![Action::EndTurn]);
    long.update(&mut c, &held);
    assert!(
        (long.text.progress() - 54.0).abs() < 1e-3,
        "{}",
        long.text.progress()
    );
    press(&mut s, &mut c, Action::EndTurn);
    assert!(s.is_revealed());
    // It answers yes to the skip question.
    press(&mut s, &mut c, Action::Cancel);
    assert!(matches!(
        press(&mut s, &mut c, Action::EndTurn),
        Transition::Pop
    ));
}

#[test]
fn long_text_pages_three_lines_at_a_time() {
    let mut c = ctx();
    // 10 lines of 90 characters: four pages.
    let line = format!("{} ", "x".repeat(90));
    let mut s = full(&two_speakers(&line.repeat(10)));
    let pages = page_sizes(&mut s, &mut c);
    assert_eq!(pages, [3, 3, 3, 1]);
}

#[test]
fn text_filling_whole_pages_has_no_empty_page_after() {
    let mut c = ctx();
    let line = format!("{} ", "x".repeat(90));
    let mut s = full(&two_speakers(&line.repeat(6)));
    let pages = page_sizes(&mut s, &mut c);
    assert_eq!(pages, [3, 3]);
}

#[test]
fn cancel_asks_before_skipping() {
    let mut c = ctx();
    let mut s = full(&two_speakers("First."));
    press(&mut s, &mut c, Action::Cancel);
    assert!(s.is_asking_skip());
    let skip = SkipView {
        question: "Skip scene?".to_owned(),
        answers: "f yes / d no".to_owned(),
    };
    assert_eq!(s.view(&c).text.skip, Some(skip));
    // Time stands still while asking.
    s.update(&mut c, &frame(&[], 1.0, false));
    assert!(!s.is_revealed());
    // No: back to the scene where it was.
    press(&mut s, &mut c, Action::Cancel);
    assert!(!s.is_asking_skip());
    assert_eq!(s.view(&c).text.skip, None);
    assert_eq!(s.page_lines(), ["First."]);
    // Yes: the scene ends.
    press(&mut s, &mut c, Action::Cancel);
    assert!(matches!(
        press(&mut s, &mut c, Action::Confirm),
        Transition::Pop
    ));
}

#[test]
fn a_scene_without_text_pops_at_once() {
    let mut c = ctx();
    let mut s = full(&scene(vec![place(Side::Left, "test_lord")]));
    assert!(matches!(
        s.update(&mut c, &frame(&[], 0.0, false)),
        Transition::Pop
    ));
}

#[test]
fn the_speaker_is_the_side_that_says_the_line() {
    let mut c = ctx();
    let mut s = full(&two_speakers("Hi."));
    let view = s.view(&c);
    assert_eq!(names(&view), [Some("Test Lord"), Some("Test Knight")]);
    assert_eq!(view.speaker, Some(Side::Left));
    assert_eq!(view.speaking().map(|f| f.name.as_str()), Some("Test Lord"));
    assert!(!view.text.narration);
    let lord = view.face(Side::Left);
    assert_eq!(lord.map(|f| f.portrait.as_str()), Some("test_lord"));
    assert_eq!(lord.map(|f| f.expression.as_str()), Some("neutral"));
    // Now the knight speaks.
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    let view = s.view(&c);
    assert_eq!(view.speaker, Some(Side::Right));
    assert_eq!(
        view.speaking().map(|f| f.name.as_str()),
        Some("Test Knight")
    );
}

#[test]
fn narration_has_no_speaker() {
    let c = ctx();
    let s = full(&scene(vec![
        place(Side::Left, "test_lord"),
        place(Side::Right, "test_knight"),
        narrate("Rain."),
    ]));
    let view = s.view(&c);
    assert_eq!(view.speaker, None);
    assert!(view.speaking().is_none());
    assert!(view.text.narration);
    assert_eq!(view.text.lines, ["Rain."]);
}

#[test]
fn an_empty_side_has_no_face() {
    let c = ctx();
    let s = full(&scene(vec![
        place(Side::Left, "test_lord"),
        say("test_lord", "Alone."),
    ]));
    assert_eq!(names(&s.view(&c)), [Some("Test Lord"), None]);
}

#[test]
fn the_overlay_says_so() {
    let c = ctx();
    let s = over(&two_speakers("Hi."));
    assert!(s.is_overlay());
    assert!(s.view(&c).overlay);
    assert!(!full(&two_speakers("Hi.")).view(&c).overlay);
}

/// The marker that says "press to go on" appears once the page is shown,
/// blinks, and goes with the page.
#[test]
fn the_page_waits_once_revealed() {
    let mut c = ctx();
    let mut s = full(&two_speakers("Hi."));
    assert!(s.view(&c).text.waiting.is_none());
    press(&mut s, &mut c, Action::Confirm);
    let waiting = s.view(&c).text.waiting;
    let log_hint = c.text_with("dialogue.log_hint", &[]);
    assert_eq!(
        waiting,
        Some(WaitView {
            key: "f".to_owned(),
            log_hint: Some(log_hint),
            marker: true,
        })
    );
    let marker = |s: &DialogueScreen, c: &Ctx| s.view(c).text.waiting.map(|w| w.marker);
    // Out from exactly half a second.
    s.update(&mut c, &frame(&[], 0.5, false));
    assert_eq!(marker(&s, &c), Some(false));
    s.update(&mut c, &frame(&[], 0.1, false));
    assert_eq!(marker(&s, &c), Some(false));
    s.update(&mut c, &frame(&[], 0.5, false));
    assert_eq!(marker(&s, &c), Some(true));
    // The next box types out first.
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Second."]);
    assert!(s.view(&c).text.waiting.is_none());
    // Nor does the skip question wait.
    press(&mut s, &mut c, Action::Confirm);
    assert!(s.view(&c).text.waiting.is_some());
    press(&mut s, &mut c, Action::Cancel);
    assert!(s.view(&c).text.waiting.is_none());
}

/// Without a key for the log, the page doesn't say how to open it.
#[test]
fn no_log_hint_without_a_log_key() {
    let mut c = ctx();
    let layout = crate::input::RIGHT_HANDED;
    let mut keys = c.layout_bindings(layout);
    for slot in 0..crate::input::SLOTS {
        keys.clear(Action::DialogueLog, slot);
    }
    c.set_layout_bindings(layout, keys).unwrap();
    let mut s = full(&two_speakers("Hi."));
    press(&mut s, &mut c, Action::Confirm);
    let waiting = s.view(&c).text.waiting;
    assert_eq!(waiting.map(|w| w.log_hint), Some(None));
}

#[test]
fn the_caption_is_shown() {
    let c = ctx();
    let s = full(&scene(vec![
        Step::Caption {
            text: "Heth".into(),
        },
        say("nobody", "Hm."),
    ]));
    assert_eq!(s.view(&c).caption.as_deref(), Some("Heth"));
    assert_eq!(full(&two_speakers("Hi.")).view(&c).caption, None);
}

#[test]
fn a_speaker_who_is_not_a_unit_is_named_by_the_names_table() {
    let c = ctx();
    let name = c
        .game
        .content
        .names
        .get("retainer")
        .unwrap_or_default()
        .to_owned();
    assert!(!name.is_empty() && name != "retainer", "{name}");
    let s = full(&scene(vec![
        place(Side::Left, "retainer"),
        say("retainer", "My lord."),
    ]));
    let view = s.view(&c);
    assert_eq!(names(&view), [Some(name.as_str()), None]);
    assert_eq!(view.speaking().map(|f| f.name.clone()), Some(name));
}

#[test]
fn a_character_without_an_entry_is_named_by_id() {
    let c = ctx();
    let s = full(&scene(vec![
        place(Side::Left, "stranger"),
        say("stranger", "Who, me?"),
    ]));
    let view = s.view(&c);
    assert_eq!(names(&view), [Some("stranger"), None]);
    assert_eq!(view.speaking().map(|f| f.name.as_str()), Some("stranger"));
}

// --- Reply choices (0708) ---------------------------------------------------

fn reply(text: &str, reaction: &str) -> trpg_content::ChoiceOption {
    trpg_content::ChoiceOption {
        tone: "t".into(),
        text: text.into(),
        line: trpg_content::LineId::default(),
        steps: vec![say("test_knight", reaction)],
    }
}

/// The knight asks; the lord's two replies; the knight answers after.
fn asking() -> Scene {
    scene(vec![
        place(Side::Left, "test_lord"),
        place(Side::Right, "test_knight"),
        say("test_knight", "Ready?"),
        Step::Choice {
            options: vec![reply("Yes.", "Good."), reply("{They} knows.", "Hm.")],
        },
        say("test_knight", "Go."),
    ])
}

/// The sound cues played since the last call.
fn sounds(c: &mut Ctx) -> Vec<String> {
    c.audio
        .take()
        .iter()
        .filter_map(|r| r.cue().map(str::to_owned))
        .collect()
}

/// The labels of the replies on screen, and which one has the focus.
fn replies(s: &DialogueScreen, c: &Ctx) -> Option<(Vec<String>, Option<usize>)> {
    let choices = s.view(c).text.choices?;
    let focus = choices.items.iter().position(|i| i.focused);
    Some((
        choices
            .items
            .into_iter()
            .map(|i| i.label.as_str().to_owned())
            .collect(),
        focus,
    ))
}

#[test]
fn a_choice_lists_its_replies_under_the_question() {
    let mut c = ctx();
    let mut s = full(&asking());
    press(&mut s, &mut c, Action::Confirm);
    assert!(s.menu().is_none());
    assert_eq!(replies(&s, &c), None);
    // The press that opens the choice doesn't pick a reply, even with a
    // second press in the same frame.
    let t = s.update(
        &mut c,
        &frame(&[Action::Confirm, Action::Confirm], 0.0, false),
    );
    assert!(matches!(t, Transition::None));
    assert_eq!(s.menu().map(Menu::focus), Some(0));
    // The question, fully shown, then the replies, the first focused.
    let view = s.view(&c);
    assert!(s.is_revealed());
    assert_eq!(view.text.revealed(), ["Ready?"]);
    let both = vec!["Yes.".to_owned(), "He knows.".to_owned()];
    assert_eq!(replies(&s, &c), Some((both.clone(), Some(0))));
    // Nothing waits for the key that goes on while choosing; the faces
    // stay.
    assert!(view.text.waiting.is_none());
    assert_eq!(names(&view), [Some("Test Lord"), Some("Test Knight")]);
    // The focus follows the cursor.
    press(&mut s, &mut c, Action::CursorDown);
    assert_eq!(replies(&s, &c), Some((both, Some(1))));
    // Picked: the replies go.
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(replies(&s, &c), None);
    assert_eq!(s.page_lines(), ["Hm."]);
}

#[test]
fn replies_are_picked_with_the_cursor_keys_and_confirm() {
    let mut c = ctx();
    let mut s = full(&asking());
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    sounds(&mut c);
    press(&mut s, &mut c, Action::CursorDown);
    assert_eq!(s.menu().map(Menu::focus), Some(1));
    press(&mut s, &mut c, Action::CursorDown);
    assert_eq!(s.menu().map(Menu::focus), Some(0));
    press(&mut s, &mut c, Action::CursorUp);
    assert_eq!(s.menu().map(Menu::focus), Some(1));
    assert_eq!(sounds(&mut c), ["menu_move"; 3]);
    // Cancel doesn't skip past a choice, and makes no sound.
    press(&mut s, &mut c, Action::Cancel);
    assert!(!s.is_asking_skip());
    assert!(sounds(&mut c).is_empty());
    // End turn confirms, like everywhere in dialogue.
    press(&mut s, &mut c, Action::EndTurn);
    assert_eq!(sounds(&mut c), ["menu_select"]);
    assert!(s.menu().is_none());
    assert_eq!(s.page_lines(), ["Hm."]);
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Go."]);
}

#[test]
fn skipping_stops_at_a_choice() {
    let mut c = ctx();
    let mut s = full(&asking());
    press(&mut s, &mut c, Action::Cancel);
    let t = s.update(
        &mut c,
        &frame(&[Action::Confirm, Action::Confirm], 0.0, false),
    );
    assert!(matches!(t, Transition::None));
    assert!(!s.is_asking_skip());
    assert_eq!(s.menu().map(Menu::focus), Some(0));
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Good."]);
    // After the choice, skipping ends the scene.
    press(&mut s, &mut c, Action::Cancel);
    assert!(matches!(
        press(&mut s, &mut c, Action::Confirm),
        Transition::Pop
    ));
}

#[test]
fn a_last_reply_with_nothing_after_ends_the_scene() {
    let mut c = ctx();
    let mut s = full(&scene(vec![Step::Choice {
        options: vec![
            trpg_content::ChoiceOption {
                tone: "t".into(),
                text: "Bye.".into(),
                line: trpg_content::LineId::default(),
                steps: vec![],
            },
            reply("Stay.", "Ok."),
        ],
    }]));
    assert!(s.menu().is_some());
    assert!(matches!(
        press(&mut s, &mut c, Action::Confirm),
        Transition::Pop
    ));
}

#[test]
fn the_lead_shows_the_players_name_and_gendered_portrait() {
    let c = ctx();
    let lead_scene = || scene(vec![place(Side::Left, "lead"), say("lead", "Let's move.")]);
    for (gender, art) in [
        (trpg_core::LeadGender::Male, "lead_m"),
        (trpg_core::LeadGender::Female, "lead_f"),
    ] {
        let s = DialogueScreen::new(
            &lead_scene(),
            LeadProfile::new("Isolde", gender),
            c.game.content.names.clone(),
            &Present::Everyone,
        );
        let view = s.view(&c);
        let lead = view.speaking();
        assert_eq!(lead.map(|f| f.name.as_str()), Some("Isolde"));
        assert_eq!(lead.map(|f| f.portrait.as_str()), Some(art));
    }
}

/// Only a press that opens a choice ends the frame's keys: two presses in
/// one frame otherwise both count.
#[test]
fn two_presses_in_one_frame_both_count() {
    let mut c = ctx();
    let mut s = full(&two_speakers("First."));
    press(&mut s, &mut c, Action::Confirm);
    s.update(
        &mut c,
        &frame(&[Action::Confirm, Action::Confirm], 0.0, false),
    );
    assert_eq!(s.page_lines(), ["Second."]);
    assert!(s.is_revealed());
}

fn music(cue: &str) -> Step {
    Step::Music(MusicLine::Cue(cue.into()))
}

fn narrate(text: &str) -> Step {
    Step::Narrate {
        text: text.into(),
        line: trpg_content::LineId::default(),
    }
}

/// The music requests made since the last call.
fn music_requests(c: &mut Ctx) -> Vec<AudioRequest> {
    let music = |r: &AudioRequest| !matches!(r, AudioRequest::PlaySound { .. });
    c.audio.take().into_iter().filter(music).collect()
}

fn play_music(cue: &str) -> AudioRequest {
    AudioRequest::PlayMusic { cue: cue.into() }
}

/// `@music` asks for the music when the scene reaches its line: the one
/// before the first text box on the first frame, a later one with the text
/// box after it, and `@music stop` fades the music out.
#[test]
fn music_changes_when_its_line_is_reached() {
    let mut c = ctx();
    let mut s = full(&scene(vec![
        music("village"),
        narrate("One."),
        narrate("Two."),
        music("talk_calm"),
        narrate("Three."),
        Step::Music(MusicLine::Stop),
    ]));
    assert!(c.audio.pending().is_empty());
    s.update(&mut c, &frame(&[], 0.0, false));
    assert_eq!(music_requests(&mut c), [play_music("village")]);
    // Not again on later frames, nor when revealing or reading on to a
    // box without a @music before it.
    s.update(&mut c, &frame(&[], 0.0, false));
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Two."]);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(music_requests(&mut c), []);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Three."]);
    assert_eq!(music_requests(&mut c), [play_music("talk_calm")]);
    // The line after the last text box is reached as the scene closes.
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(music_requests(&mut c), []);
    let t = press(&mut s, &mut c, Action::Confirm);
    assert!(matches!(t, Transition::Pop));
    assert_eq!(music_requests(&mut c), [AudioRequest::StopMusic]);
}

/// Skipping asks only for the last `@music` passed, and a scene without
/// one asks for nothing.
#[test]
fn skipping_asks_for_the_last_music_only() {
    let mut c = ctx();
    let mut s = full(&scene(vec![
        narrate("One."),
        music("village"),
        narrate("Two."),
        music("talk_calm"),
        narrate("Three."),
    ]));
    press(&mut s, &mut c, Action::Cancel);
    let t = press(&mut s, &mut c, Action::Confirm);
    assert!(matches!(t, Transition::Pop));
    assert_eq!(music_requests(&mut c), [play_music("talk_calm")]);

    let mut s = full(&two_speakers("First."));
    press(&mut s, &mut c, Action::Cancel);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(music_requests(&mut c), []);
}

/// A reply's reaction changes the music when it is picked; skipping to the
/// choice asks for the music before it.
#[test]
fn music_around_a_choice() {
    let mut c = ctx();
    let reply = |text: &str, steps| trpg_content::ChoiceOption {
        tone: "wry".into(),
        text: text.into(),
        line: trpg_content::LineId::default(),
        steps,
    };
    let mut s = full(&scene(vec![
        narrate("One."),
        music("village"),
        narrate("Well?"),
        Step::Choice {
            options: vec![
                reply("Yes.", vec![music("scene_sad"), narrate("Oh.")]),
                reply("No.", vec![narrate("Good.")]),
            ],
        },
    ]));
    press(&mut s, &mut c, Action::Cancel);
    press(&mut s, &mut c, Action::Confirm);
    assert!(s.menu().is_some());
    assert_eq!(music_requests(&mut c), [play_music("village")]);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Oh."]);
    assert_eq!(music_requests(&mut c), [play_music("scene_sad")]);
}

/// The sound `sound` makes, as [`sounds`] reports it.
fn cue(sound: MenuSound) -> String {
    let mut c = ctx();
    c.audio.menu(sound);
    sounds(&mut c).remove(0)
}

/// Opens the log and moves its cursor `up` rows.
fn open_log(s: &mut DialogueScreen, c: &mut Ctx, up: usize) {
    press(s, c, Action::DialogueLog);
    for _ in 0..up {
        press(s, c, Action::CursorUp);
    }
}

/// Jumping to a line plays the music that was on there, and only when it
/// isn't on already.
#[test]
fn a_jump_in_the_log_plays_that_lines_music() {
    let mut c = ctx();
    let mut s = full(&scene(vec![
        music("village"),
        narrate("One."),
        music("talk_calm"),
        narrate("Two."),
        narrate("Three."),
    ]));
    s.update(&mut c, &frame(&[], 0.0, false));
    assert_eq!(music_requests(&mut c), [play_music("village")]);
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(music_requests(&mut c), [play_music("talk_calm")]);
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Three."]);
    // To "Two.": its music is on already.
    open_log(&mut s, &mut c, 1);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["Two."]);
    assert_eq!(music_requests(&mut c), []);
    // To "One.": the village again, once.
    open_log(&mut s, &mut c, 1);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["One."]);
    assert_eq!(music_requests(&mut c), [play_music("village")]);
    s.update(&mut c, &frame(&[], 0.0, false));
    assert_eq!(music_requests(&mut c), []);
    // Reading on reaches `@music talk_calm` again.
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(music_requests(&mut c), [play_music("talk_calm")]);
}

/// The log sounds like a menu: opening, moving (not past either end),
/// going to a line and closing.
#[test]
fn the_log_makes_the_menu_sounds() {
    let mut c = ctx();
    let mut s = full(&scene(vec![narrate("One."), narrate("Two.")]));
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    sounds(&mut c);
    press(&mut s, &mut c, Action::DialogueLog);
    assert!(s.is_log_open());
    assert_eq!(sounds(&mut c), [cue(MenuSound::Select)]);
    // The log's key again does nothing in the log.
    press(&mut s, &mut c, Action::DialogueLog);
    press(&mut s, &mut c, Action::CursorDown);
    assert_eq!(sounds(&mut c), Vec::<String>::new());
    assert!(s.is_log_open());
    press(&mut s, &mut c, Action::CursorUp);
    assert_eq!(sounds(&mut c), [cue(MenuSound::Move)]);
    press(&mut s, &mut c, Action::CursorUp);
    assert_eq!(sounds(&mut c), Vec::<String>::new(), "at the top already");
    press(&mut s, &mut c, Action::Cancel);
    assert_eq!(sounds(&mut c), [cue(MenuSound::Cancel)]);
    assert!(!s.is_log_open());
    assert_eq!(s.page_lines(), ["Two."]);
    // End turn goes to the line, as Confirm does.
    open_log(&mut s, &mut c, 1);
    sounds(&mut c);
    press(&mut s, &mut c, Action::EndTurn);
    assert_eq!(sounds(&mut c), [cue(MenuSound::Select)]);
    assert_eq!(s.page_lines(), ["One."]);
    assert!(!s.is_revealed(), "it types out from the start");
}

/// Nothing seen yet (a scene with nothing to say): the log doesn't open.
#[test]
fn an_empty_scene_has_no_log() {
    let mut c = ctx();
    let mut s = full(&scene(vec![music("village")]));
    press(&mut s, &mut c, Action::DialogueLog);
    assert!(!s.is_log_open());
}

/// The log is in the view while it is open, and the scene under it.
#[test]
fn the_log_appears_and_goes_in_the_view() {
    let mut c = ctx();
    let mut s = full(&scene(vec![narrate("One."), narrate("Two.")]));
    assert_eq!(s.view(&c).log, None);
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::Confirm);
    press(&mut s, &mut c, Action::DialogueLog);
    let view = s.view(&c);
    assert_eq!(view.log, s.log_view(&c));
    assert!(view.log.is_some());
    assert_eq!(view.text.lines, ["Two."]);
    press(&mut s, &mut c, Action::Cancel);
    assert_eq!(s.view(&c).log, None);
}

// --- The place behind the talk (0732) ---------------------------------------

fn setting(place: PlaceLine) -> Step {
    Step::Directive(Stage::Setting(place))
}

/// A frame that types the page out (the text is instant), then Confirm:
/// the next line.
fn next_line(s: &mut DialogueScreen, c: &mut Ctx) {
    s.update(c, &frame(&[], 0.0, false));
    press(s, c, Action::Confirm);
}

fn at(x: i32, y: i32) -> trpg_core::Pos {
    trpg_core::Pos::new(x, y)
}

/// A map named by `@place` is shown centred on its tile, as much of it as
/// the map skin fits on the screen; the battle only when the screen was
/// given one; `@place none` and an unknown map show none.
#[test]
fn the_view_shows_the_place_the_scene_names() {
    let mut c = ctx();
    c.change_settings(|s| s.text_speed = TextSpeed::Instant)
        .unwrap();
    let ch01 = || PlaceLine::Map {
        map: "ch01".into(),
        at: at(3, 12),
    };
    let steps = vec![
        narrate("Black."),
        setting(ch01()),
        narrate("The green."),
        setting(PlaceLine::Battle { at: at(20, 3) }),
        narrate("The fort."),
        setting(PlaceLine::Map {
            map: "no_such_map".into(),
            at: at(0, 0),
        }),
        narrate("Nowhere."),
        setting(PlaceLine::None),
        narrate("Black again."),
    ];
    let view = screen_tiles(&c);
    let maps = &c.game.content.maps;
    let expected = Place::of_map(&maps["ch01"]).scene(at(3, 12), view);
    let battle = &c.game.content.battles["ch01"];
    let mut field = Place::of_battle(&c.game.content, battle);
    field.units = vec![];
    let places = |s: &mut DialogueScreen, c: &mut Ctx| {
        let mut seen = vec![s.view(c).place];
        for _ in 0..4 {
            next_line(s, c);
            seen.push(s.view(c).place);
        }
        seen
    };
    // Without a battle, `@place battle` shows none.
    let mut s = full(&scene(steps.clone()));
    assert_eq!(s.battle(), None);
    let seen = places(&mut s, &mut c);
    assert_eq!(seen, [None, Some(expected.clone()), None, None, None]);
    // With one, its field, centred on the fort.
    let mut s = full(&scene(steps)).with_battle(field.clone());
    assert_eq!(s.battle(), Some(&field));
    let seen = places(&mut s, &mut c);
    let fort = field.scene(at(20, 3), view);
    assert_eq!(seen, [None, Some(expected), Some(fort), None, None]);
}

/// Jumping in the log shows the line with the place it had.
#[test]
fn a_jump_in_the_log_shows_that_lines_place() {
    let mut c = ctx();
    c.change_settings(|s| s.text_speed = TextSpeed::Instant)
        .unwrap();
    let ch01 = PlaceLine::Map {
        map: "ch01".into(),
        at: at(3, 12),
    };
    let mut s = full(&scene(vec![
        setting(ch01),
        narrate("One."),
        setting(PlaceLine::None),
        narrate("Two."),
    ]));
    let green = s.view(&c).place;
    assert!(green.is_some());
    next_line(&mut s, &mut c);
    assert_eq!(s.page_lines(), ["Two."]);
    assert_eq!(s.view(&c).place, None);
    open_log(&mut s, &mut c, 1);
    press(&mut s, &mut c, Action::Confirm);
    assert_eq!(s.page_lines(), ["One."]);
    assert_eq!(s.view(&c).place, green);
}

/// A key that reaches an `@opening` starts it (0734); the rest of that
/// frame's keys neither skip it nor reveal the text waiting behind it.
#[test]
fn the_keys_after_the_one_that_starts_an_opening_wait() {
    let mut c = ctx();
    let narrate = |text: &str| Step::Narrate {
        text: text.into(),
        line: trpg_content::LineId::default(),
    };
    let yard = Step::Directive(Stage::Setting(trpg_content::PlaceLine::Map {
        map: "harrowby_yard".into(),
        at: trpg_core::Pos::new(12, 8),
    }));
    let mut s = full(&scene(vec![
        narrate("Black."),
        yard,
        Step::Directive(Stage::Opening("ch01_sparring".into())),
        narrate("The yard."),
    ]));
    press(&mut s, &mut c, Action::Confirm);
    assert!(s.is_revealed() && !s.is_opening());
    s.update(
        &mut c,
        &frame(&[Action::Confirm, Action::Confirm], 0.0, false),
    );
    assert!(s.is_opening());
    assert_eq!(s.view(&c).text.lines, ["The yard."]);
    assert!(!s.is_revealed());
    // Once it has played, the line types out from its start.
    s.update(&mut c, &frame(&[], 5.0, false));
    assert!(!s.is_opening());
    assert_eq!(s.view(&c).text.shown, 0);
}
