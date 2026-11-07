//! The kept session records (ticket 1210, engine ADR-0072): a record of
//! everything the platform gave the game, played from the game's real
//! start ([`trpg_ui::start`]) through `Harness::from_session`, frame by
//! frame, stopping at the first frame whose hash differs. Each frame's hash
//! holds the open screens, the sounds and music asked for, and the battle's
//! rules state; never the picture, so a change of look breaks no record.
//!
//! `docs/session-records.md` says how a tester's record becomes one of
//! these. When a change to the game is meant to play a kept record
//! differently, bless it: `SHUYI_BLESS=1 cargo test -p trpg-ui --test it
//! sessions` writes the file again with the same input and this build's
//! hashes.

use std::path::PathBuf;

use shuyi_platform::harness::{assert_session, check_session};
use shuyi_platform::session::{SessionRecord, Stamps};
use trpg_ui::harness::Core;
use trpg_ui::screens::BattleScreen;
use trpg_ui::start::{Startup, start};

/// The loading card's title, as the binary's window has it.
const TITLE: &str = "Visions of Shuyi";

/// A first launch, into the debug Quick Battle: the layout picker, the
/// battle, the debug menu's "Save session" and the lord's first move.
const QUICK_BATTLE: &str = "quick_battle.session.ron";

/// The game as a device starts it, at the title.
fn title() -> shuyi_platform::Start<trpg_ui::screen::Shuyi> {
    start(TITLE, Ok(Startup::Title))
}

/// Where a kept record is.
fn kept(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!("tests/it/sessions/{name}"))
}

/// The session [`QUICK_BATTLE`] keeps, recorded in the Harness from the
/// game's real start, as a debug build's window records one.
fn record_quick_battle() -> SessionRecord {
    let mut h = Core::start_recording(title()).unwrap_or_else(|e| panic!("no start: {e}"));
    h.wait(0.1);
    // Any key opens the layout picker on a first launch; Confirm picks the
    // right-handed layout.
    h.keys("z f");
    // Quick Battle, then Preparations: Left wraps to `Fight!`; Confirm
    // closes the turn's banner.
    h.keys("Down f Left f f");
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    // The debug menu, "Save session" (above the engine's debug view, its
    // last entry), and back.
    h.keys("F2 Up Up f");
    assert!(h.app_mut().take_session_save());
    h.keys("Escape");
    assert_eq!(h.top_screen(), "battle");
    // The lord, at (3, 5), moves one tile right and waits.
    h.keys("f Right f");
    h.wait(1.0);
    h.keys("f");
    h.wait(1.0);
    h.session()
        .cloned()
        .unwrap_or_else(|| panic!("not recording"))
}

/// Writes [`QUICK_BATTLE`] again, from a new recording: when the script
/// above changes, or the record's format does.
#[test]
#[ignore = "writes the kept session"]
fn write_the_quick_battle_session() {
    std::fs::write(kept(QUICK_BATTLE), record_quick_battle().to_ron()).unwrap();
}

/// Acceptance (1210): the kept record plays to its last frame with no
/// difference (or, with `SHUYI_BLESS=1`, is written again).
#[test]
fn the_kept_quick_battle_session_plays_back() {
    assert_session(title(), kept(QUICK_BATTLE));
    let text = std::fs::read_to_string(kept(QUICK_BATTLE)).unwrap();
    let record = SessionRecord::from_ron(&text).unwrap();
    let mut h = Core::from_session(title(), &record, Stamps::Check).unwrap();
    let played = h.play_all().clone();
    assert_eq!(played.first_difference, None);
    assert_eq!(played.frames as usize, record.frames.len());
    // It ends where it was recorded: in the battle, the lord moved.
    assert_eq!(h.screens(), ["title", "game_flow", "battle"]);
    assert!(h.screen::<BattleScreen>().is_some());
}

/// A new recording of the same script is the kept one: the record holds
/// all the game reads, so nothing else (a clock, a file) gets in.
#[test]
fn the_quick_battle_records_the_same_every_time() {
    let text = std::fs::read_to_string(kept(QUICK_BATTLE)).unwrap();
    let kept = SessionRecord::from_ron(&text).unwrap();
    assert_eq!(record_quick_battle().frames, kept.frames);
}

/// A record whose battle went otherwise is caught at the frame it parts:
/// without the press of Right, the lord moves nowhere and the battle's
/// state hash differs.
#[test]
fn a_record_that_plays_differently_names_its_frame() {
    let text = std::fs::read_to_string(kept(QUICK_BATTLE)).unwrap();
    let mut record = SessionRecord::from_ron(&text).unwrap();
    let right = record.frames.iter().position(|f| {
        let input = format!("{:?}", f.input);
        input.contains("Down") && input.contains("key: Right")
    });
    let right = right.unwrap();
    record.frames[right].input.clear();
    let error = check_session(title(), &record.to_ron()).unwrap_err();
    // Frames count from 1.
    let frame = right + 1;
    assert!(error.contains(&format!("frame {frame}")), "{error}");
}
