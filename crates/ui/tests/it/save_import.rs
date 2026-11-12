//! Importing a file of saves (ticket 0873, `docs/design/options.md`,
//! *Saves outside the game*), played through the Harness: the import key
//! on the Profile screen or Load Game waits for the file (dropped on a
//! desktop, picked on the web), reports each save's verdict before
//! anything is written, and writes the ready ones into a new profile, or
//! a one-save file through Load Game into this profile.

use std::sync::{Arc, Mutex};

use insta::assert_snapshot;
use shuyi_platform::player_files::ChosenFile;
use shuyi_ui::profiles::ProfileStore;
use shuyi_ui::storage::bundle::{SaveBundle, read_bundle};
use shuyi_ui::{FileDrops, FilePicker, StoragePlace};
use trpg_content::new_campaign;
use trpg_core::{GameMode, LeadGender, LeadProfile, SaveFile};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::save::{self, Slot, slot_key};
use trpg_ui::save_files::made;
use trpg_ui::screens::save_import::ImportView;
use trpg_ui::screens::{ImportScreen, SlotPickerScreen};
use trpg_ui::storage::StorageError;
use trpg_ui::{Ctx, ShuyiCtx, Storage};

/// A desktop's folder.
const FOLDER: &str = r"C:\Users\Player\Documents\My Games\Visions of Shuyi";

/// A chapter save of a new campaign, `seconds` played.
fn chapter_save(ctx: &Ctx, seconds: u64) -> SaveFile {
    let lead = LeadProfile::new("Mei", LeadGender::Female);
    let mut campaign = new_campaign(&ctx.game.content, GameMode::Classic, lead);
    campaign.playtime_s = seconds;
    SaveFile::chapter_cleared(campaign)
}

/// Puts a chapter save in each of `slots` of the profile in use.
fn fill(ctx: &mut Ctx, slots: impl IntoIterator<Item = usize>) {
    for slot in slots {
        let file = chapter_save(ctx, 60 * slot as u64);
        save::write(&mut ctx.storage, &slot_key(slot), &file).unwrap_or_else(|e| panic!("{e}"));
    }
}

/// A desktop whose dropped files the game hears, no picker.
fn on_a_desktop(ctx: &mut Ctx) {
    ctx.platform.storage_place = StoragePlace::Folder(FOLDER.to_owned());
    ctx.platform.file_drops = FileDrops::Heard;
}

/// A later launch on a desktop, with saves in `slots`.
fn launch(slots: impl IntoIterator<Item = usize>) -> Harness {
    let mut h = Harness::with_layout_and(RIGHT_HANDED, |ctx| {
        fill(ctx, slots);
        on_a_desktop(ctx);
    });
    h.wait(0.1);
    h
}

/// The text of a save written by `version`.
fn written_by(save: &SaveFile, version: u32) -> String {
    let mut save = save.clone();
    save.version = version;
    ron::to_string(&save).unwrap_or_else(|e| panic!("{e}"))
}

/// A file of `entries`, made by this build.
fn file_of(entries: &[(&str, String)]) -> Vec<u8> {
    let entries = entries.iter().map(|(k, v)| ((*k).to_owned(), v.clone()));
    SaveBundle::new(made(), entries.collect())
        .to_text()
        .into_bytes()
}

/// The title's `Profile` item opened, and the import key pressed.
fn import_from_profile(h: &mut Harness) {
    h.keys("Down Down Down Down f");
    assert_eq!(h.top_screen(), "profiles", "{}", h.snapshot());
    h.keys("Space");
    assert_eq!(h.top_screen(), "save_import", "{}", h.snapshot());
}

/// The title's `Load Game` opened, the cursor moved `down`, and the import
/// key pressed.
fn import_from_load_game(h: &mut Harness, down: usize) {
    h.keys("Down f");
    assert_eq!(h.top_screen(), "save_slots", "{}", h.snapshot());
    for _ in 0..down {
        h.keys("Down");
    }
    h.keys("Space");
    assert_eq!(h.top_screen(), "save_import", "{}", h.snapshot());
}

/// The Import screen's view.
fn view(h: &Harness) -> ImportView {
    let screen = h.game().screen::<ImportScreen>();
    let view = screen.map(|s| s.view(h.game().ctx()));
    view.unwrap_or_else(|| panic!("no import screen: {}", h.snapshot()))
}

/// Each row's label and verdict.
fn verdicts(view: &ImportView) -> Vec<(String, String)> {
    let rows = view.rows.iter();
    rows.map(|r| (r.label.clone(), r.verdict.clone())).collect()
}

fn pairs(list: &[(&str, &str)]) -> Vec<(String, String)> {
    let pairs = list.iter();
    pairs.map(|&(a, b)| (a.to_owned(), b.to_owned())).collect()
}

/// The keys profile `n` has stored, without its prefix.
fn keys_of(h: &Harness, n: u32) -> Vec<String> {
    let prefix = format!("p{n}-");
    let keys = h.game().ctx().storage.computer().list();
    let keys = keys.unwrap_or_else(|e| panic!("{e}")).into_iter();
    let mut keys: Vec<String> = keys
        .filter_map(|k| k.strip_prefix(&prefix).map(str::to_owned))
        .collect();
    keys.sort();
    keys
}

/// The profiles' names.
fn profile_names(h: &Harness) -> Vec<String> {
    let profiles = h.game().ctx().profiles().all().iter();
    profiles.map(|p| p.name.clone()).collect()
}

/// The answer in focus.
fn focused_answer(view: &ImportView) -> Option<String> {
    let question = view.question.as_ref()?;
    let focused = question.answers.iter().find(|a| a.focused);
    focused.map(|a| a.label.clone())
}

/// Acceptance: a file with a save of this version, one of save version 1,
/// a damaged one and a key the game doesn't keep shows "Ready", the
/// incompatible version and "Can't be read", and leaves the unknown key
/// out; Import writes only the current save, into a new profile named from
/// the file.
#[test]
fn the_report_gives_each_save_its_verdict_and_import_writes_the_ready_one() {
    let mut h = launch([1]);
    let ctx = h.game().ctx();
    let current = save::encode(&chapter_save(ctx, 125)).unwrap_or_else(|e| panic!("{e}"));
    let old = written_by(&chapter_save(ctx, 60), 1);
    let damaged = current[..current.len() / 2].to_owned();
    let file = file_of(&[
        ("slot_01", current),
        ("slot_02", old),
        ("slot_03", damaged),
        ("high_scores", "(best: 9)".to_owned()),
    ]);
    import_from_profile(&mut h);
    assert_eq!(
        view(&h).waiting.as_deref(),
        Some("Drop the exported save file on the window.")
    );
    h.drop_file("visions-of-shuyi-odo-2.ron", file).wait(0.02);
    let report = view(&h);
    assert_eq!(
        verdicts(&report),
        pairs(&[
            ("Slot 01", "Ready"),
            (
                "Slot 02",
                "From an incompatible version (save version 1, this game reads 2)"
            ),
            ("Slot 03", "Can't be read"),
        ])
    );
    let first = &report.rows[0];
    assert_eq!(first.chapter.as_deref(), Some("Test Chapter (cleared)"));
    assert_eq!(first.playtime.as_deref(), Some("0:02:05"));
    assert_eq!(
        report.to.as_deref(),
        Some("The saves go into a new profile, Odo.")
    );
    assert_eq!(report.made_by, None);
    assert_eq!(focused_answer(&report).as_deref(), Some("Cancel"));
    assert_snapshot!("report_through_profile", h.snapshot());
    // Nothing is written before the answer.
    assert_eq!(profile_names(&h), ["Player 1"]);
    h.keys("Left f");
    assert_eq!(h.top_screen(), "profiles", "{}", h.snapshot());
    assert_eq!(profile_names(&h), ["Player 1", "Odo"]);
    assert_eq!(keys_of(&h, 2), ["slot_01"]);
    assert!(
        h.snapshot()
            .contains("Added the profile Odo: 1 imported, 2 refused"),
        "{}",
        h.snapshot()
    );
}

/// Acceptance: a file made by another version of the game, whose saves
/// are of this save version, imports every save and says who made it.
#[test]
fn a_file_of_another_build_imports_when_its_saves_are_this_version() {
    let mut h = launch([1]);
    let ctx = h.game().ctx();
    let a = save::encode(&chapter_save(ctx, 1)).unwrap_or_else(|e| panic!("{e}"));
    let b = save::encode(&chapter_save(ctx, 2)).unwrap_or_else(|e| panic!("{e}"));
    let tips = ("tips_seen", "intro".to_owned());
    let text = String::from_utf8(file_of(&[("slot_04", a), ("slot_09", b), tips]));
    let mut bundle = read_bundle(&text.unwrap_or_default()).unwrap_or_else(|e| panic!("{e}"));
    bundle.made.game = "0.0.1".to_owned();
    import_from_profile(&mut h);
    let name = "visions-of-shuyi-player-1-1.ron";
    h.drop_file(name, bundle.to_text().into_bytes()).wait(0.02);
    let report = view(&h);
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        report.made_by,
        Some(format!("Made by version 0.0.1; this is {version}."))
    );
    assert_eq!(
        verdicts(&report),
        pairs(&[("Slot 04", "Ready"), ("Slot 09", "Ready")])
    );
    // "Player 1" is taken: the next free name.
    assert_eq!(
        report.to.as_deref(),
        Some("The saves go into a new profile, Player 1 2.")
    );
    h.keys("Left f");
    assert_eq!(keys_of(&h, 2), ["slot_04", "slot_09", "tips_seen"]);
    // The notice counts the saves, not the tips seen.
    assert!(
        h.snapshot()
            .contains("Added the profile Player 1 2: 2 imported, 0 refused"),
        "{}",
        h.snapshot()
    );
}

/// A storage that fails the first write of one key, and works again.
#[derive(Debug)]
struct FailsOnce {
    inner: Box<dyn Storage>,
    key: Arc<Mutex<Option<String>>>,
}

impl Storage for FailsOnce {
    fn read(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.inner.read(key)
    }

    fn write(&mut self, key: &str, value: &str) -> Result<(), StorageError> {
        let mut failing = self.key.lock().unwrap_or_else(|e| panic!("{e}"));
        if failing.as_deref() == Some(key) {
            *failing = None;
            return Err(StorageError::Backend("the disk is full".to_owned()));
        }
        self.inner.write(key, value)
    }

    fn delete(&mut self, key: &str) -> Result<(), StorageError> {
        self.inner.delete(key)
    }

    fn list(&self) -> Result<Vec<String>, StorageError> {
        self.inner.list()
    }
}

/// Acceptance: Cancel writes nothing; and a storage that fails during
/// the import leaves the profile as it was (the engine's `apply`).
#[test]
fn cancel_writes_nothing_and_a_failed_import_changes_nothing() {
    let mut h = launch([1, 2]);
    let save = save::encode(&chapter_save(h.game().ctx(), 7)).unwrap_or_else(|e| panic!("{e}"));
    let file = file_of(&[("slot_05", save.clone())]);
    let whole = file_of(&[("slot_05", save.clone()), ("slot_06", save)]);
    import_from_profile(&mut h);
    h.drop_file("a.ron", file.clone()).wait(0.02);
    let before = h.game().ctx().storage.computer().list();
    h.keys("f");
    assert_eq!(h.top_screen(), "profiles");
    assert_eq!(h.game().ctx().storage.computer().list(), before);
    assert_eq!(profile_names(&h), ["Player 1"]);

    let failing = Arc::new(Mutex::new(Some("p1-slot_03".to_owned())));
    let shared = Arc::clone(&failing);
    let mut h = Harness::with_layout_and(RIGHT_HANDED, |ctx| {
        fill(ctx, [1, 2]);
        on_a_desktop(ctx);
        let store = std::mem::take(&mut ctx.storage);
        let id = store.profile();
        let inner = store.into_computer();
        let key = shared;
        ctx.storage = ProfileStore::new(Box::new(FailsOnce { inner, key }), id);
    });
    h.wait(0.1);
    let slots_before = slots_of(&h);
    import_from_load_game(&mut h, 0);
    h.drop_file("one.ron", file).wait(0.02);
    assert_eq!(
        verdicts(&view(&h)),
        pairs(&[("Slot 05", "Ready — goes in slot 03")])
    );
    h.keys("Left f");
    assert_eq!(h.top_screen(), "save_import");
    assert_eq!(
        view(&h).message.as_deref(),
        Some("The import failed; nothing was changed.")
    );
    assert_eq!(*failing.lock().unwrap_or_else(|e| panic!("{e}")), None);
    assert_eq!(slots_of(&h), slots_before);
    let left = keys_of(&h, 1);
    assert!(left.iter().all(|k| !k.starts_with("_bundle")), "{left:?}");

    // Into a new profile: it is taken away again, with what it held.
    *failing.lock().unwrap_or_else(|e| panic!("{e}")) = Some("p2-slot_05".to_owned());
    h.keys("Left f d d Down Down Down f Space");
    h.drop_file("visions-of-shuyi-kai-1.ron", whole).wait(0.02);
    h.keys("Left f");
    assert_eq!(
        view(&h).message.as_deref(),
        Some("The import failed; nothing was changed.")
    );
    assert_eq!(profile_names(&h), ["Player 1"]);
    assert_eq!(keys_of(&h, 2), [] as [String; 0]);
}

/// What the profile in use's slots hold.
fn slots_of(h: &Harness) -> Vec<Slot> {
    let ctx = h.game().ctx();
    save::slots(&ctx.storage, &ctx.game.content, ctx.words())
}

/// Acceptance: without a picker, Import says to drop the file, and a file
/// dropped then opens the report; where there is a picker (the web) it
/// opens with the press, and closing it goes back. A dropped file that
/// isn't an export is refused, and one dropped anywhere else is ignored.
#[test]
fn the_file_is_dropped_on_a_desktop_and_picked_on_the_web() {
    let mut h = launch([1]);
    let save = save::encode(&chapter_save(h.game().ctx(), 3)).unwrap_or_else(|e| panic!("{e}"));
    let file = file_of(&[("slot_01", save)]);
    // Dropped on the title: nothing happens.
    h.drop_file("a.ron", file.clone()).wait(0.02);
    assert_eq!(h.top_screen(), "title");
    import_from_profile(&mut h);
    assert_snapshot!("waiting_for_a_drop", h.snapshot());
    // Two files at once aren't one to import.
    let two = vec![
        shuyi_platform::dropped::DroppedFile::new("a.ron", file.clone()),
        shuyi_platform::dropped::DroppedFile::new("b.ron", file.clone()),
    ];
    h.drop_files(two).wait(0.02);
    assert!(view(&h).waiting.is_some());
    h.drop_file("notes.txt", b"my notes".to_vec()).wait(0.02);
    assert_eq!(
        view(&h).message.as_deref(),
        Some("That isn't an exported save file.")
    );
    assert_snapshot!("refused", h.snapshot());
    h.keys("d");
    assert_eq!(h.top_screen(), "profiles");

    let mut h = launch([1]);
    h.ctx_mut().platform.file_picker = FilePicker::Opens;
    h.ctx_mut().platform.storage_place = StoragePlace::Browser {
        prefix: "tactical-rpg/".to_owned(),
    };
    let text = String::from_utf8(file).unwrap_or_default();
    h.answer_next_ask(ChosenFile::picked("visions-of-shuyi-mei-1.ron", text));
    import_from_profile(&mut h);
    h.wait(0.02);
    assert_eq!(verdicts(&view(&h)), pairs(&[("Slot 01", "Ready")]));
    h.keys("d");
    // No answer queued: the player closes the picker, and is back.
    h.keys("Space").wait(0.05);
    assert_eq!(h.top_screen(), "profiles", "{}", h.snapshot());
}

/// Acceptance: the Profile screen's export of a profile, imported as a new
/// profile, gives the same Load Game list.
#[test]
fn an_exported_profile_imports_as_the_same_saves() {
    let mut h = launch([1, 2, 7]);
    let before = slots_of(&h);
    h.keys("Down Down Down Down f e");
    let [(name, text)] = h.offered() else {
        panic!("offered {:?}", h.offered());
    };
    let (name, text) = (name.clone(), text.clone());
    h.keys("Space");
    h.drop_file(&name, text.into_bytes()).wait(0.02);
    assert_eq!(view(&h).rows.len(), 3);
    h.keys("Left f");
    let names = profile_names(&h);
    assert_eq!(names, ["Player 1", "Player 1 2"]);
    let id = h.game().ctx().profiles().all()[1].id;
    h.ctx_mut()
        .switch_profile(id)
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(slots_of(&h), before);
}

/// Through Load Game, a file of one save goes into this profile: the first
/// empty slot, without its options; with every slot full, the slot under
/// the cursor, replacing it; a whole profile warns that it becomes a new
/// one.
#[test]
fn load_game_takes_one_save_into_this_profile() {
    let mut h = launch([1, 2]);
    let ctx = h.game().ctx();
    let save = save::encode(&chapter_save(ctx, 99)).unwrap_or_else(|e| panic!("{e}"));
    let settings = ctx
        .storage
        .read("settings")
        .ok()
        .flatten()
        .unwrap_or_default();
    let one = file_of(&[("slot_12", save.clone()), ("settings", settings)]);
    import_from_load_game(&mut h, 1);
    h.drop_file("one.ron", one.clone()).wait(0.02);
    let report = view(&h);
    assert_eq!(
        verdicts(&report),
        pairs(&[("Slot 12", "Ready — goes in slot 03")])
    );
    assert_eq!(
        report.to.as_deref(),
        Some("The save goes into this profile.")
    );
    assert_eq!(
        report.options.as_deref(),
        Some("The file's options and key bindings are left out: this profile keeps its own.")
    );
    assert_snapshot!("report_through_load_game", h.snapshot());
    h.keys("Left f");
    assert_eq!(h.top_screen(), "save_slots");
    let picker = h.game().screen::<SlotPickerScreen>();
    assert_eq!(picker.map(SlotPickerScreen::focused_slot), Some(3));
    assert!(
        h.snapshot().contains("Imported the save into slot 03"),
        "{}",
        h.snapshot()
    );
    assert!(matches!(slots_of(&h)[2], Slot::Saved(_)));
    assert_eq!(profile_names(&h), ["Player 1"]);

    // Every slot full: the one under the cursor, the cursor on Cancel.
    let mut h = launch(1..=30);
    import_from_load_game(&mut h, 4);
    h.drop_file("one.ron", one).wait(0.02);
    let report = view(&h);
    assert_eq!(
        verdicts(&report),
        pairs(&[("Slot 12", "Ready — replaces slot 05")])
    );
    assert_eq!(focused_answer(&report).as_deref(), Some("Cancel"));
    h.keys("Left f");
    let Slot::Saved(fifth) = &slots_of(&h)[4] else {
        panic!("slot 5");
    };
    assert_eq!(fifth.playtime_s, 99);

    // A whole profile: a warning, then a new profile.
    let mut h = launch([1]);
    let two = file_of(&[("slot_01", save.clone()), ("slot_02", save)]);
    import_from_load_game(&mut h, 0);
    h.drop_file("visions-of-shuyi-kai-3.ron", two).wait(0.02);
    assert_eq!(
        view(&h).to.as_deref(),
        Some("This file holds a whole profile: it will be added as a new profile, Kai.")
    );
    h.keys("Left f");
    assert_eq!(profile_names(&h), ["Player 1", "Kai"]);
    assert_eq!(
        slots_of(&h).iter().filter(|s| **s != Slot::Empty).count(),
        1
    );
}

/// More saves than fit scroll, the range in the panel's border; a file of
/// another build of this version says so; the options are judged on their
/// own line.
#[test]
fn a_long_report_scrolls_and_names_another_build() {
    let mut h = launch([1]);
    let ctx = h.game().ctx();
    let save = save::encode(&chapter_save(ctx, 5)).unwrap_or_else(|e| panic!("{e}"));
    let settings = ctx
        .storage
        .read("settings")
        .ok()
        .flatten()
        .unwrap_or_default();
    let mut entries: Vec<(String, String)> =
        (1..=20).map(|n| (slot_key(n), save.clone())).collect();
    entries.push(("settings".to_owned(), settings));
    let entries: Vec<(&str, String)> = entries
        .iter()
        .map(|(k, v)| (k.as_str(), v.clone()))
        .collect();
    let text = String::from_utf8(file_of(&entries)).unwrap_or_default();
    let mut bundle = read_bundle(&text).unwrap_or_else(|e| panic!("{e}"));
    bundle.made.content = shuyi_core::made::ContentHash(1);
    import_from_profile(&mut h);
    h.drop_file("b.ron", bundle.to_text().into_bytes())
        .wait(0.02);
    let report = view(&h);
    let version = env!("CARGO_PKG_VERSION");
    assert_eq!(
        report.made_by,
        Some(format!("Made by another build of version {version}."))
    );
    assert_eq!(
        report.options.as_deref(),
        Some("Options and key bindings: Ready")
    );
    assert_eq!(report.rows.len(), 16);
    assert_eq!(report.range.as_deref(), Some("1-16 of 20"));
    h.keys("Up");
    assert_eq!(view(&h).range.as_deref(), Some("1-16 of 20"));
    h.keys("Down Down Down Down Down");
    let report = view(&h);
    assert_eq!(report.range.as_deref(), Some("5-20 of 20"));
    assert_eq!(report.rows[0].label, "Slot 05");
    assert_snapshot!("report_scrolled", h.snapshot());
    h.keys("Up");
    assert_eq!(view(&h).range.as_deref(), Some("4-19 of 20"));
    // Scrolling leaves the answer on Cancel.
    assert_eq!(focused_answer(&view(&h)).as_deref(), Some("Cancel"));
}

/// With the most profiles, a file has nowhere to go: no question, and
/// back. A file of nothing this build reads writes nothing and adds no
/// profile. Where the platform has no files a player reaches (the
/// Playdate), the key does nothing.
#[test]
fn a_file_with_nowhere_to_go_or_nothing_to_take_writes_nothing() {
    let mut h = launch([1]);
    let old = written_by(&chapter_save(h.game().ctx(), 8), 1);
    let file = file_of(&[("slot_01", old)]);
    for n in 2..=8 {
        let name = format!("P{n}");
        h.ctx_mut()
            .add_profile(&name)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    import_from_profile(&mut h);
    h.drop_file("visions-of-shuyi-odo-1.ron", file.clone())
        .wait(0.02);
    let report = view(&h);
    assert_eq!(
        report.to.as_deref(),
        Some("There can be at most 8 profiles: delete one to import this file.")
    );
    assert!(report.question.is_none());
    assert_eq!(report.help, "d back");
    h.keys("f");
    assert_eq!(h.top_screen(), "profiles");

    // The last place taken while the report is open: Import says so.
    let mut h = launch([1]);
    let save = save::encode(&chapter_save(h.game().ctx(), 6)).unwrap_or_else(|e| panic!("{e}"));
    for n in 2..=7 {
        let name = format!("P{n}");
        h.ctx_mut()
            .add_profile(&name)
            .unwrap_or_else(|e| panic!("{e}"));
    }
    import_from_profile(&mut h);
    h.drop_file("y.ron", file_of(&[("slot_01", save)]))
        .wait(0.02);
    h.ctx_mut()
        .add_profile("P8")
        .unwrap_or_else(|e| panic!("{e}"));
    h.keys("Left f");
    assert_eq!(
        view(&h).message.as_deref(),
        Some("There can be at most 8 profiles: delete one to import this file.")
    );

    let mut h = launch([1]);
    import_from_profile(&mut h);
    h.drop_file("x.ron", file).wait(0.02);
    assert_eq!(
        view(&h).to.as_deref(),
        Some("The saves go into a new profile, Player 2.")
    );
    h.keys("Left f");
    assert_eq!(profile_names(&h), ["Player 1"]);
    assert!(
        h.snapshot().contains("Nothing was imported: 1 refused"),
        "{}",
        h.snapshot()
    );

    let mut h = launch([1]);
    h.ctx_mut().platform.storage_place = StoragePlace::Unknown;
    h.keys("Down Down Down Down f Space");
    assert_eq!(h.top_screen(), "profiles");
    // No help line names the key there, on a profile or the row that adds one.
    assert!(!h.snapshot().contains("import"), "{}", h.snapshot());
    h.keys("Down");
    assert!(!h.snapshot().contains("import"), "{}", h.snapshot());
}

/// A picker that couldn't open the file, and a file of another version
/// of the file format, are refused; once the report is up, another file
/// dropped changes nothing.
#[test]
fn a_file_the_platform_or_the_format_refuses_is_refused() {
    let mut h = launch([1]);
    h.ctx_mut().platform.file_picker = FilePicker::Opens;
    h.answer_next_ask(ChosenFile::Refused("over the limit".to_owned()));
    import_from_profile(&mut h);
    h.wait(0.02);
    assert_eq!(
        view(&h).message.as_deref(),
        Some("The file couldn't be opened.")
    );

    let mut h = launch([1]);
    let save = save::encode(&chapter_save(h.game().ctx(), 4)).unwrap_or_else(|e| panic!("{e}"));
    let file = file_of(&[("slot_01", save)]);
    let text = String::from_utf8(file.clone()).unwrap_or_default();
    let mut bundle = read_bundle(&text).unwrap_or_else(|e| panic!("{e}"));
    bundle.version = 99;
    import_from_profile(&mut h);
    h.drop_file("new.ron", bundle.to_text().into_bytes())
        .wait(0.02);
    assert_eq!(
        view(&h).message.as_deref(),
        Some("That file was exported by a version of the game this one can't read.")
    );
    h.keys("f");
    assert_eq!(h.top_screen(), "profiles");
    h.keys("Space");
    h.drop_file("visions-of-shuyi-mei-1.ron", file).wait(0.02);
    let before = view(&h);
    h.drop_file("notes.txt", b"notes".to_vec()).wait(0.02);
    assert_eq!(view(&h), before);
}

/// On the row that adds a profile, the help line names the import key.
#[test]
fn the_add_row_names_the_import_key() {
    let mut h = launch([1]);
    h.keys("Down Down Down Down f Down");
    assert!(
        h.snapshot().contains("f add · Space import · d back"),
        "{}",
        h.snapshot()
    );
}
