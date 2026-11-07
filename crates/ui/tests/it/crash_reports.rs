//! Crash reports (ticket 1245, engine ADR-0079): the record the game adds to
//! one (the open battle's), and the start after a crash, which says so once
//! under the title's menu (`docs/design/crash-reports.md`).

use shuyi_platform::crash::{CrashReport, crash_key};
use shuyi_platform::headless::HeadlessPlatform;
use shuyi_platform::{App, Millis};
use trpg_ui::crash::{BattleRecord, VERSION, crash_record};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::screens::TitleScreen;
use trpg_ui::start::{Startup, start};
use trpg_ui::{MemoryStorage, Storage};

/// The loading card's title, as the binary's window has it.
const TITLE: &str = "Visions of Shuyi";

/// What the title says after a crash.
const CRASHED: &str = "The game closed unexpectedly last time. A report was saved.";

/// At the title, no battle is open: no record.
#[test]
fn there_is_no_record_outside_a_battle() {
    let h = Harness::with_layout(RIGHT_HANDED);
    assert_eq!(crash_record(h.game()), None);
}

/// In the debug Quick Battle after the lord's first move: the battle's
/// history, nothing being applied between frames.
#[test]
fn the_record_in_a_battle_holds_its_history() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    // Quick Battle, then Preparations: Left wraps to `Fight!`; Confirm
    // closes the turn's banner. The lord moves one tile right and waits.
    h.keys("Down f Left f f");
    h.keys("f Right f").wait(1.0).keys("f").wait(1.0);
    let text = crash_record(h.game()).unwrap();
    let record: BattleRecord = ron::from_str(&text).unwrap();
    let battle = h.battle().unwrap();
    assert_eq!(record.version, VERSION);
    assert!(!record.history.is_empty());
    assert_eq!(record.history.commands(), battle.history().commands());
    assert_eq!(record.applying, None);
    assert_eq!(record.commands(), battle.history().commands());
}

/// The game started as a device starts it, at the title, on `storage`
/// (a run's): what the title says under its menu, and the storage after.
fn launch(storage: Box<dyn Storage>) -> (Option<String>, Box<dyn Storage>) {
    let mut app = App::new(start(TITLE, Ok(Startup::Title)));
    let mut platform = HeadlessPlatform::new().with_storage(storage);
    let _ = app.frame(&mut platform, Millis(0));
    let _ = app.frame(&mut platform, Millis(16));
    let game = app.into_game().unwrap_or_else(|| panic!("not loaded"));
    let title = game.screen::<TitleScreen>();
    let title = title.unwrap_or_else(|| panic!("not at the title"));
    let notice = title.notice().map(str::to_owned);
    (notice, game.into_ctx().storage.into_computer())
}

/// Acceptance (1245): with a report not yet offered, the title says the
/// game closed unexpectedly; the start after says nothing.
#[test]
fn the_start_after_a_crash_says_so_once() {
    let mut storage = MemoryStorage::new();
    let report = CrashReport::new(1_000, "a panic on purpose").to_text();
    storage.write(&crash_key(1_000), &report).unwrap();
    let (notice, storage) = launch(Box::new(storage));
    assert_eq!(notice.as_deref(), Some(CRASHED));
    let (notice, _) = launch(storage);
    assert_eq!(notice, None);
}

/// Without a report, the title says nothing.
#[test]
fn a_start_without_a_crash_says_nothing() {
    let (notice, _) = launch(Box::new(MemoryStorage::new()));
    assert_eq!(notice, None);
}
