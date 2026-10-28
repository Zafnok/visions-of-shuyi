//! Chapter 1 against the Playdate's bundle (ticket 1036): the front end
//! starts the game as on the device (`crate::card`, `crate::start`), and
//! the game plays from New Game to "To be continued", with this thread
//! reading the bundle the Playdate embeds (`trpg_content::bundle::
//! playdate_layers`). Every file the game reads must be in that bundle,
//! and every file in it must be read: the bundle holds what the Playdate
//! uses and nothing more (`crates/content/playdate-bundle.txt`). Every
//! track it asks for must be one the Playdate's build ships
//! (`crates/content/playdate-music.txt`, ticket 1047).
//!
//! The battle is won with the commands of the PC's playthrough test
//! (`crates/ui/tests/it/ch01_winnable.rs`); the screens around it are
//! driven with the Playdate's buttons.

use std::cell::RefCell;

use trpg_content::audio;
use trpg_content::bundle::{self, Bundle};
use trpg_content::{MusicLine, Step};
use trpg_core::{BattleState, Command, Outcome, Phase, next_command};
use trpg_ui::MemoryStorage;
use trpg_ui::audio::MusicCommand;
use trpg_ui::harness::Harness;
use trpg_ui::input::PadKind;
use trpg_ui::screen::Lessons;

use crate::{card, start};

/// The PC test's winning playthrough: the player's commands, in order.
const PLAYTHROUGH: &str = include_str!("../../ui/tests/it/fixtures/ch01_winning.ron");

std::thread_local! {
    /// The files this thread reads, while it notes them.
    static SEEN: RefCell<Option<Vec<(String, bool)>>> = const { RefCell::new(None) };
}

/// The test's watcher of the bundle's reads.
fn note(path: &str, found: bool) {
    SEEN.with_borrow_mut(|seen| {
        if let Some(seen) = seen {
            seen.push((path.to_owned(), found));
        }
    });
}

/// Skips every scene on screen: B, then A on "Skip scene?".
fn skip_scenes(h: &mut Harness) {
    for _ in 0..8 {
        if h.top_screen() != "dialogue" {
            return;
        }
        h.pad("East South");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

fn battle(h: &Harness) -> BattleState {
    let battle = h.battle();
    battle
        .unwrap_or_else(|| panic!("no battle: {:?}", h.screens()))
        .state()
        .clone()
}

fn send(h: &mut Harness, command: &Command) {
    let battle = h.battle_mut();
    battle.unwrap_or_else(|| panic!("no battle")).send(command);
}

/// The front end's start, then Chapter 1 from New Game to the title;
/// returns the music cues it started and those its scenes name.
fn play_chapter_1() -> Vec<String> {
    // The card's font comes first, as on the device.
    card::font().unwrap_or_else(|e| panic!("{e}"));
    let mut ctx = start::context(Box::new(MemoryStorage::new()), 1)
        .unwrap_or_else(|e| panic!("the game did not start: {e}"));
    // The guide's tips and lessons are read at the start; off, they never
    // stop the recorded commands.
    ctx.game.tips_enabled = false;
    ctx.game.lessons = Lessons::Off;
    let mut h = Harness::from_game(trpg_ui::game::start(ctx));
    h.use_pad(PadKind::Playdate);
    // A ends the title's wait; then New Game, Classic, and on until the
    // lead's screen.
    for _ in 0..8 {
        if h.top_screen() == "lead_select" {
            break;
        }
        h.pad("South");
    }
    assert_eq!(h.top_screen(), "lead_select", "{:?}", h.screens());
    // Down to Start, past the name, and A.
    h.pad("DpadDown DpadDown South");
    skip_scenes(&mut h);
    // The battle's banner, then its opening scene.
    h.pad("South");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    let commands: Vec<Command> =
        ron::from_str(PLAYTHROUGH).unwrap_or_else(|e| panic!("ch01_winning.ron: {e}"));
    let ai = h.game().ctx().game.content.ai;
    for command in commands {
        loop {
            skip_scenes(&mut h);
            let state = battle(&h);
            assert_eq!(state.outcome(), None);
            if state.phase() == Phase::Player {
                break;
            }
            let theirs = next_command(&state, &ai).unwrap_or(Command::EndPhase);
            send(&mut h, &theirs);
        }
        send(&mut h, &command);
    }
    assert_eq!(battle(&h).outcome(), Some(Outcome::Victory));
    // Harl's last words, the banner, the results, the victory scenes,
    // "Save your progress?" (No), "To be continued", the title.
    for _ in 0..40 {
        match h.top_screen() {
            "dialogue" => skip_scenes(&mut h),
            "save_prompt" => break,
            _ => {
                h.wait(0.5).pad("South");
            }
        }
    }
    assert_eq!(
        h.screens(),
        ["title", "game_flow", "save_prompt"],
        "{}",
        h.snapshot()
    );
    h.pad("DpadDown South");
    assert_eq!(h.screens(), ["title", "game_flow", "to_be_continued"]);
    h.pad("South");
    assert_eq!(h.screens(), ["title"]);
    // What it started, and what the scenes it skipped would have: every
    // `@music` of Chapter 1's own scenes (its intro and victory scenes
    // and its battle's triggers), not of every scene in the bundle: a
    // later chapter's script may name a track the Playdate doesn't ship
    // until that chapter does.
    let started = h
        .music_commands()
        .iter()
        .filter_map(|command| match command {
            MusicCommand::Start { cue } => Some(cue.clone()),
            _ => None,
        });
    let content = &h.game().ctx().game.content;
    let chapter = &content.chapters["ch01"];
    let battle = &content.battles[&chapter.battle];
    let ids = chapter
        .intro_scenes
        .iter()
        .chain(&chapter.victory_scenes)
        .chain(battle.triggers.iter().map(|trigger| &trigger.scene));
    let in_scenes = ids
        .filter_map(|id| content.dialogue.scenes.get(id))
        .flat_map(|scene| &scene.steps)
        .filter_map(|step| match step {
            Step::Music(MusicLine::Cue(cue)) => Some(cue.clone()),
            _ => None,
        });
    started.chain(in_scenes).collect()
}

/// The only test here that sets the bundle's watcher: it is process-wide.
#[test]
fn chapter_1_plays_through_on_the_playdates_bundle_which_holds_only_what_it_reads() {
    assert!(bundle::watch(note).is_ok());
    let playdate: &'static Bundle = Box::leak(Box::new(Bundle::new(bundle::playdate_layers())));
    SEEN.set(Some(Vec::new()));
    let tracks = bundle::reading(playdate, play_chapter_1);
    let seen = SEEN.take().unwrap_or_default();
    // The story's files, as a build that ships has them.
    let story = Bundle::new(bundle::layers());
    let lacking: Vec<&str> = seen
        .iter()
        .filter(|(path, found)| !found && bundle::story_only(|| story.bytes(path).is_some()))
        .map(|(path, _)| path.as_str())
        .collect();
    assert!(
        lacking.is_empty(),
        "the game reads these files, which the Playdate's bundle leaves out \
         (crates/content/playdate-bundle.txt): {lacking:?}"
    );
    let unread: Vec<&str> = playdate
        .files_under("")
        .into_iter()
        .filter(|path| !seen.iter().any(|(read, _)| read == path))
        .collect();
    assert!(
        unread.is_empty(),
        "the Playdate's bundle holds files it never reads; list them in \
         crates/content/playdate-bundle.txt: {unread:?}"
    );
    let manifest = audio::load().unwrap_or_else(|e| panic!("{e:?}"));
    let shipped = audio::playdate_music(&manifest).unwrap_or_else(|e| panic!("{e}"));
    for cue in ["title", "battle_bright", "village_home"] {
        assert!(tracks.iter().any(|t| t == cue), "{cue}: {tracks:?}");
    }
    let silent: Vec<&String> = tracks
        .iter()
        .filter(|cue| !shipped.contains(*cue))
        .collect();
    assert!(
        silent.is_empty(),
        "Chapter 1 plays these tracks, which the Playdate's build does not \
         ship (crates/content/playdate-music.txt): {silent:?}"
    );
}
