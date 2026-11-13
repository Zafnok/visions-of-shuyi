//! Scripted tests of the checklist lessons (ticket 0456) through the real
//! game: New Game on the test chapter at each familiarity answer, the test
//! lesson (`assets-test/data/lessons.ron`: the cursor's four directions on
//! the first turn), the turn waiting for it, and a done lesson staying
//! done after the battle is suspended and continued. The guide says a
//! lesson's and a tip's line as its box opens (ticket 0476).

use trpg_content::{TipTrigger, Variant};
use trpg_core::{Phase, TutorialLevel};
use trpg_ui::AudioRequest;
use trpg_ui::ShuyiCtx;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::save::SUSPEND_KEY;
use trpg_ui::screens::battle::BattleScreen;
use trpg_ui::{Ctx, MemoryStorage, Storage};

/// The test lesson.
const TEST_LESSON: &str = "test_cursor";
/// What the campaign keeps once the box at the end of the lessons is answered.
const END_ANSWERED: &str = "lessons_end_answered";

/// A launch on `storage` with the right-handed layout picked, New Game on
/// the test chapter, and the lessons on (as the game has them).
fn launch(storage: Box<dyn Storage>) -> Harness {
    let mut storage = storage;
    if let Err(e) = storage.write("layout", RIGHT_HANDED.name()) {
        panic!("saving the layout: {e}");
    }
    let mut ctx = Ctx::embedded()
        .unwrap_or_else(|e| panic!("{e}"))
        .with_storage(storage);
    ctx.game.content.new_game = trpg_content::NewGameDef::test_chapter();
    // The story's lessons start in any battle; theirs are in `ch01_lessons`.
    ctx.game
        .content
        .lessons
        .lessons
        .retain(|l| l.id == TEST_LESSON);
    let mut h = Harness::from_game(trpg_ui::game::start(ctx));
    h.with_lessons();
    h
}

/// From the title: New Game in Classic, the familiarity answer `downs`
/// rows below "Not familiar", the default lead, the intro scene skipped,
/// the battle notes and turn 1's banner closed.
fn to_battle(h: &mut Harness, downs: &str) {
    h.keys("f f").keys(downs).keys("f");
    from_the_lead(h);
}

/// From the title with voice clips found, at "Not familiar": the voices
/// question (0826) given `answer`'s keys, then on as [`to_battle`].
fn to_battle_answering_voices(h: &mut Harness, answer: &str) {
    h.keys("f f f");
    assert_eq!(h.top_screen(), "voices_question");
    h.keys(answer);
    from_the_lead(h);
}

/// From the lead screen: the default lead, the intro scene skipped, the
/// battle notes and turn 1's banner closed.
fn from_the_lead(h: &mut Harness) {
    h.keys("Up f");
    assert_eq!(h.top_screen(), "dialogue");
    h.keys("d f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.keys("f").wait(0.1).keys("f").wait(0.1);
}

fn battle(h: &Harness) -> &BattleScreen {
    let battle = h.battle();
    battle.unwrap_or_else(|| panic!("no battle"))
}

fn lessons_done(h: &Harness) -> Vec<String> {
    let campaign = h.flow().and_then(|f| f.campaign_now(h.game().ctx()));
    let campaign = campaign.unwrap_or_else(|| panic!("no campaign"));
    campaign.lessons_done.iter().cloned().collect()
}

/// Acceptance: at `Full` the test lesson opens on the first turn; the
/// four directions, in any order, close it, and the campaign keeps it.
/// While it is open End Turn is refused; after, it works.
#[test]
fn at_full_the_lesson_opens_holds_the_turn_and_closes() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    to_battle(&mut h, "");
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    assert!(h.snapshot().contains("Moving the cursor"));
    // End turn: nothing, and no prompt.
    h.keys("Space");
    assert!(!h.snapshot().contains("End turn"), "no End turn prompt");
    assert_eq!(battle(&h).state().phase(), Phase::Player);
    h.keys("Down Right Left");
    assert!(battle(&h).lesson_open());
    assert_eq!(lessons_done(&h), [] as [std::string::String; 0]);
    h.keys("Up");
    assert_eq!(battle(&h).shown_lesson(), None);
    assert!(!h.snapshot().contains("Moving the cursor"));
    assert_eq!(lessons_done(&h), [TEST_LESSON]);
    // Keep going at the end of the lessons (0465); now End turn asks, and
    // yes ends the phase.
    h.keys("f Space f");
    assert_ne!(battle(&h).state().phase(), Phase::Player);
}

/// Acceptance: at `None` the lesson never opens, nor at `Short` (it is a
/// `Full` lesson); the turn ends as usual.
#[test]
fn at_short_and_none_the_full_lesson_never_opens() {
    for (downs, level) in [
        ("Down", TutorialLevel::Short),
        ("Down Down", TutorialLevel::None),
    ] {
        let mut h = launch(Box::new(MemoryStorage::new()));
        to_battle(&mut h, downs);
        let campaign = h
            .flow()
            .and_then(|f| f.campaign_now(h.game().ctx()))
            .map(|c| c.tutorial);
        assert_eq!(campaign, Some(level));
        h.wait(1.0);
        assert_eq!(battle(&h).shown_lesson(), None, "{level:?}");
        assert!(!battle(&h).lesson_open());
        h.keys("Space f");
        assert_ne!(battle(&h).state().phase(), Phase::Player, "{level:?}");
    }
}

/// Acceptance: a done lesson doesn't reopen after saving and loading: the
/// battle suspended (the map menu's `Suspend`) and continued in the next
/// launch, back on its first turn's player phase.
#[test]
fn a_done_lesson_stays_done_after_suspend_and_continue() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    to_battle(&mut h, "");
    h.keys("Up Down Left Right");
    assert_eq!(lessons_done(&h), [TEST_LESSON]);
    // Keep going at the end of the lessons (0465).
    h.keys("f");
    assert_eq!(lessons_done(&h), [END_ANSWERED, TEST_LESSON]);
    // The map menu's Suspend, then yes.
    h.keys("d Down Down Down Down f f");
    assert_eq!(h.screens(), ["title"]);
    let saved = h.game().ctx().storage.read(SUSPEND_KEY);
    let saved = saved
        .ok()
        .flatten()
        .unwrap_or_else(|| panic!("no suspend save"));
    assert!(saved.contains("\"test_cursor\""), "{saved}");
    // The next launch: Continue.
    let mut h = launch(h.into_storage());
    h.keys("f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    h.wait(1.0);
    assert_eq!(battle(&h).state().turn(), 1);
    assert_eq!(battle(&h).shown_lesson(), None);
    assert_eq!(lessons_done(&h), [END_ANSWERED, TEST_LESSON]);
}

/// Not done when suspended: the lesson opens again on Continue, from
/// scratch (its ticks belong to the battle screen that showed it).
#[test]
fn a_lesson_left_open_opens_again_after_continue() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    to_battle(&mut h, "");
    h.keys("Up Down");
    h.keys("d Down Down Down Down f f");
    assert_eq!(h.screens(), ["title"]);
    let mut h = launch(h.into_storage());
    h.keys("f").wait(0.1);
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    assert_eq!(battle(&h).lesson_ticks(), [false; 4]);
    assert_eq!(lessons_done(&h), [] as [std::string::String; 0]);
}

/// A voice manifest with a clip for every line the guide says (0476).
fn guide_manifest(h: &Harness) -> String {
    let content = &h.game().ctx().game.content;
    let scene = trpg_content::voice::guide_scene(&content.lessons, &content.tips);
    let clip = |l: &trpg_content::dialogue::Line| {
        format!(
            "(line: \"{}\", spoken: \"{}\", voice: \"guide\", \
             made_by: Generated(tool: \"t\", model: \"m\", date: \"2026-11-08\")),\n",
            l.id, l.text
        )
    };
    let clips: String = scene.lines().iter().map(clip).collect();
    format!("(clips: [\n{clips}])")
}

/// The voice requests of the run so far: what was asked for and stopped.
fn voice_requests(h: &Harness) -> Vec<AudioRequest> {
    let voice =
        |r: &AudioRequest| matches!(r, AudioRequest::PlayVoice { .. } | AudioRequest::StopVoice);
    h.audio_requests().into_iter().filter(voice).collect()
}

fn play(text: &str) -> AudioRequest {
    AudioRequest::PlayVoice {
        line: trpg_content::voice::guide_line(text),
        variant: Variant::None,
    }
}

/// Acceptance (0476): the lesson opening, with a clip in the manifest,
/// queues one `PlayVoice` of its text; ticking its last item closes it
/// and queues `StopVoice`.
#[test]
fn the_lesson_box_says_her_line_and_closing_it_stops_her() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    let manifest = guide_manifest(&h);
    h.ctx_mut().set_voice_manifest(&manifest);
    assert_eq!(h.ctx_mut().take_warnings(), [] as [String; 0]);
    let text = h.game().ctx().game.content.lessons.lessons[0].text.clone();
    to_battle_answering_voices(&mut h, "f");
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    let asked: Vec<AudioRequest> = voice_requests(&h)
        .into_iter()
        .filter(|r| matches!(r, AudioRequest::PlayVoice { .. }))
        .collect();
    assert_eq!(asked, [play(&text)]);
    // Moving on the map with the box up says nothing more.
    h.clear_audio().keys("Down Right Left");
    assert_eq!(voice_requests(&h), [] as [AudioRequest; 0]);
    h.keys("Up");
    assert_eq!(battle(&h).shown_lesson(), None);
    assert_eq!(voice_requests(&h), [AudioRequest::StopVoice]);
}

/// Acceptance (0476): a tip's box says its line; closing it stops her and
/// the lesson's box that comes next says its own.
#[test]
fn a_tip_says_its_line_and_the_next_box_takes_over() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    h.with_tips();
    let manifest = guide_manifest(&h);
    h.ctx_mut().set_voice_manifest(&manifest);
    let content = &h.game().ctx().game.content;
    let tip = content.tips.for_trigger(TipTrigger::FirstBattleStart);
    let tip = tip.map(|t| t.text.clone()).unwrap_or_default();
    let lesson = content.lessons.lessons[0].text.clone();
    to_battle_answering_voices(&mut h, "f");
    assert_eq!(battle(&h).shown_tip(), Some(TipTrigger::FirstBattleStart));
    let asked = voice_requests(&h);
    assert_eq!(asked.last(), Some(&play(&tip)));
    h.clear_audio().keys("f");
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    assert_eq!(voice_requests(&h), [AudioRequest::StopVoice, play(&lesson)]);
}

/// Acceptance (0476): with voices off (No to the voices question), no
/// box asks for her voice.
#[test]
fn with_voices_off_no_box_asks_for_her() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    h.with_tips();
    let manifest = guide_manifest(&h);
    h.ctx_mut().set_voice_manifest(&manifest);
    to_battle_answering_voices(&mut h, "Down f");
    assert!(!h.game().ctx().settings().voices);
    assert!(battle(&h).shown_tip().is_some());
    h.keys("f");
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    let asked = h.audio_requests().into_iter().filter(|r| {
        matches!(
            r,
            AudioRequest::PlayVoice { .. } | AudioRequest::PreloadVoices { .. }
        )
    });
    assert_eq!(asked.count(), 0);
}

/// Acceptance (0494): the battle preloads her lines before her first box
/// asks for its clip (on the web a clip is fetched, and one not loaded
/// 0.3 s after it is asked for is skipped): the queued tip's first, then
/// the lesson's. Once the tip is closed, the list is sent again without
/// it, before the lesson's clip is asked for.
#[test]
fn the_battle_preloads_her_lines_before_her_box_asks_for_one() {
    let mut h = launch(Box::new(MemoryStorage::new()));
    h.with_tips();
    let manifest = guide_manifest(&h);
    h.ctx_mut().set_voice_manifest(&manifest);
    let content = &h.game().ctx().game.content;
    let tip = content.tips.for_trigger(TipTrigger::FirstBattleStart);
    let tip = tip.map(|t| t.text.clone()).unwrap_or_default();
    let lesson = content.lessons.lessons[0].text.clone();
    let line = trpg_content::voice::guide_line;
    to_battle_answering_voices(&mut h, "f");
    assert_eq!(battle(&h).shown_tip(), Some(TipTrigger::FirstBattleStart));
    let guide = |r: &AudioRequest| match r {
        AudioRequest::PreloadVoices { lines } => {
            let ids: Vec<_> = lines.iter().map(|(id, _)| id.clone()).collect();
            ids.iter().all(|id| id.scene_id() == "vix").then_some(ids)
        }
        AudioRequest::PlayVoice { line, .. } if line.scene_id() == "vix" => Some(vec![]),
        _ => None,
    };
    let asked: Vec<Vec<_>> = h.audio_requests().iter().filter_map(guide).collect();
    // Preloads (the first on the battle's first frame), then her clip
    // (an empty list here), last; the preload just before it starts with
    // the tip's line, then the lesson's.
    let Some((played, preloads)) = asked.split_last() else {
        panic!("nothing asked");
    };
    assert_eq!(played, &[] as &[trpg_content::LineId]);
    let Some(preload) = preloads.last() else {
        panic!("her clip was asked for with nothing preloaded: {asked:?}");
    };
    assert_eq!(preload[..2], [line(&tip), line(&lesson)]);
    h.clear_audio().keys("f");
    assert_eq!(battle(&h).shown_lesson(), Some(TEST_LESSON));
    let after: Vec<AudioRequest> = h
        .audio_requests()
        .into_iter()
        .filter(|r| guide(r).is_some() || *r == AudioRequest::StopVoice)
        .collect();
    let [
        AudioRequest::PreloadVoices { lines },
        AudioRequest::StopVoice,
        voice,
    ] = after.as_slice()
    else {
        panic!("{after:?}");
    };
    assert_eq!(lines[0].0, line(&lesson));
    assert!(!lines.iter().any(|(id, _)| *id == line(&tip)), "seen: gone");
    assert_eq!(voice, &play(&lesson));
}
