//! Saves outside the game (ticket 0872, `docs/design/options.md`, *Saves
//! outside the game*), played through the Harness: the Profile screen
//! exports the whole profile, Load Game one save (asking about the options
//! and keys first), each offered as one file whose keys have no profile
//! prefix; the place line shows the saves' folder on a desktop, nothing on
//! the web; nothing is offered where the platform has no files (the
//! Playdate).

use std::collections::BTreeSet;

use insta::assert_snapshot;
use shuyi_ui::StoragePlace;
use shuyi_ui::storage::bundle::read_bundle;
use trpg_content::new_campaign;
use trpg_core::{GameMode, LeadGender, LeadProfile, SaveFile};
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;
use trpg_ui::save::{self, slot_key};
use trpg_ui::screens::{ProfileScreen, SlotPickerScreen};
use trpg_ui::{Ctx, Storage};

/// A desktop's folder, as Windows writes it.
const FOLDER: &str = r"C:\Users\Player\AppData\Roaming\tactical-rpg";

/// Puts a chapter save in slot `slot` of the profile in use.
fn fill(ctx: &mut Ctx, slot: usize) {
    let lead = LeadProfile::new("Mei", LeadGender::Female);
    let campaign = new_campaign(&ctx.game.content, GameMode::Classic, lead);
    let file = SaveFile::chapter_cleared(campaign);
    save::write(&mut ctx.storage, &slot_key(slot), &file).unwrap_or_else(|e| panic!("{e}"));
}

/// A later launch with the right-handed layout, saves in slots 1 and 2,
/// on a desktop whose saves are in [`FOLDER`].
fn launch() -> Harness {
    let mut h = Harness::with_layout_and(RIGHT_HANDED, |ctx| {
        fill(ctx, 1);
        fill(ctx, 2);
        ctx.platform.storage_place = StoragePlace::Folder(FOLDER.to_owned());
    });
    h.wait(0.1);
    h
}

/// The same launch on the web.
fn launch_on_the_web() -> Harness {
    let mut h = launch();
    h.ctx_mut().platform.storage_place = StoragePlace::Browser {
        prefix: "tactical-rpg/".to_owned(),
    };
    h
}

/// The title's `Profile` item opened: under New Game, Continue? no;
/// Load Game, Quick Battle, Options, Profile.
fn open_profile(h: &mut Harness) {
    h.keys("Down Down Down Down f");
    assert_eq!(h.top_screen(), "profiles", "{}", h.snapshot());
}

/// The title's `Load Game` opened.
fn open_load_game(h: &mut Harness) {
    h.keys("Down f");
    assert_eq!(h.top_screen(), "save_slots", "{}", h.snapshot());
}

/// The one file offered: its name and its entries' keys.
fn offered(h: &Harness) -> (String, BTreeSet<String>) {
    let [(name, text)] = h.offered() else {
        panic!("offered {:?}", h.offered());
    };
    let bundle = read_bundle(text).unwrap_or_else(|e| panic!("{e}"));
    (name.clone(), bundle.entries.keys().cloned().collect())
}

fn keys(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|&k| k.to_owned()).collect()
}

/// What the profile in use has stored, by key (without its prefix).
fn stored(h: &Harness) -> BTreeSet<String> {
    let keys = h.game().ctx().storage.list();
    keys.unwrap_or_else(|e| panic!("{e}")).into_iter().collect()
}

fn shows(h: &Harness, text: &str) -> bool {
    h.snapshot()
        .lines()
        .take(32)
        .any(|line| line.contains(text))
}

/// Acceptance: the Profile screen's export of a profile with two slots
/// and settings offers one file, `visions-of-shuyi-<name>-<n>.ron`, with
/// every key of the profile and none of its prefix; the notice says where
/// it went.
#[test]
fn the_profile_screen_exports_the_whole_profile() {
    let mut h = launch();
    open_profile(&mut h);
    assert!(shows(&h, "e export · x open folder"), "{}", h.snapshot());
    h.keys("e");
    let (name, entries) = offered(&h);
    assert_eq!(name, "visions-of-shuyi-player-1-1.ron");
    assert_eq!(entries, stored(&h));
    assert!(entries.contains("settings") && entries.contains("slot_02"));
    assert!(entries.iter().all(|k| !k.starts_with("p1-")), "{entries:?}");
    h.wait(0.1);
    assert!(
        shows(
            &h,
            "Exported visions-of-shuyi-player-1-1.ron to the exports folder"
        ),
        "{}",
        h.snapshot()
    );
    // Moving on puts the notice away.
    h.keys("Down");
    assert!(!shows(&h, "Exported"));
}

/// Another profile under the cursor exports that profile, not the one in
/// use.
#[test]
fn the_profile_under_the_cursor_is_the_one_exported() {
    let mut h = launch();
    h.ctx_mut()
        .add_profile("Odo Kai")
        .unwrap_or_else(|e| panic!("{e}"));
    open_profile(&mut h);
    h.keys("Down e");
    let (name, entries) = offered(&h);
    assert_eq!(name, "visions-of-shuyi-odo-kai-2.ron");
    assert!(!entries.contains("slot_01"), "{entries:?}");
}

/// Acceptance: Load Game's export asks about the options and keys, the
/// cursor on No; No exports the save alone, Yes the save with them.
#[test]
fn load_game_exports_one_save_after_asking_about_the_options() {
    let mut h = launch();
    open_load_game(&mut h);
    assert!(shows(&h, "e export · x open folder"), "{}", h.snapshot());
    h.keys("Down e");
    let view = h
        .game()
        .screen::<SlotPickerScreen>()
        .map(|s| s.view(h.game().ctx()));
    let question = view.and_then(|v| v.export_question);
    let question = question.unwrap_or_else(|| panic!("no question"));
    assert_eq!(question.text, "Export your options and key bindings too?");
    let focused = question
        .answers
        .iter()
        .find(|a| a.focused)
        .map(|a| a.label.as_str());
    assert_eq!(focused, Some("No"));
    assert_snapshot!("load_game_export_question", h.snapshot());
    h.keys("f");
    let (name, entries) = offered(&h);
    assert_eq!(name, "visions-of-shuyi-player-1-1-slot-02.ron");
    assert_eq!(entries, keys(&["slot_02"]));
    // Yes: the save, and the options and keys the profile has.
    h.keys("e Left f");
    let (name, text) = &h.offered()[1];
    assert_eq!(name, "visions-of-shuyi-player-1-1-slot-02 (2).ron");
    let bundle = read_bundle(text).unwrap_or_else(|e| panic!("{e}"));
    assert!(bundle.entries.contains_key("slot_02"));
    assert!(bundle.entries.contains_key("settings"));
    assert!(!bundle.entries.contains_key("slot_01"));
    // Cancel closes the question and offers nothing.
    h.keys("e d");
    assert_eq!(h.offered().len(), 2);
    assert_eq!(h.top_screen(), "save_slots");
}

/// An empty slot has nothing to export, and saving never exports.
#[test]
fn an_empty_slot_has_nothing_to_export() {
    let mut h = launch();
    open_load_game(&mut h);
    h.keys("Down Down e");
    assert_eq!(h.offered(), [] as [(String, String); 0]);
    assert!(!shows(&h, "e export"), "{}", h.snapshot());
}

/// Acceptance: the place line reads the folder on a desktop; the web
/// shows none (its export is a download).
#[test]
fn the_place_line_shows_the_folder_on_a_desktop_only() {
    let mut h = launch();
    open_load_game(&mut h);
    assert!(
        shows(&h, &format!("Saves are kept in {FOLDER}")),
        "{}",
        h.snapshot()
    );
    assert_snapshot!("load_game_on_a_desktop", h.snapshot());
    let mut h = launch();
    open_profile(&mut h);
    let view = h
        .game()
        .screen::<ProfileScreen>()
        .map(|s| s.view(h.game().ctx()));
    let place = view
        .and_then(|v| v.place)
        .unwrap_or_else(|| panic!("no place"));
    assert_eq!(place.path, FOLDER);
    assert_snapshot!("profile_on_a_desktop", h.snapshot());

    let mut h = launch_on_the_web();
    open_profile(&mut h);
    assert!(!shows(&h, "Saves are kept"), "{}", h.snapshot());
    assert!(shows(&h, "e export · d back"), "{}", h.snapshot());
    assert_snapshot!("profile_on_the_web", h.snapshot());
    let mut h = launch_on_the_web();
    open_load_game(&mut h);
    assert!(!shows(&h, "Saves are kept"));
    assert!(shows(&h, "f load · e export · d back"), "{}", h.snapshot());
}

/// Where the platform has no files a player reaches (the Playdate), the
/// export key does nothing and no help line names it.
#[test]
fn nothing_is_exported_without_files_a_player_reaches() {
    let mut h = launch();
    h.ctx_mut().platform.storage_place = StoragePlace::Unknown;
    open_profile(&mut h);
    h.keys("e");
    assert!(!shows(&h, "export"));
    let mut h = launch();
    h.ctx_mut().platform.storage_place = StoragePlace::Unknown;
    open_load_game(&mut h);
    h.keys("e");
    assert_eq!(h.offered(), [] as [(String, String); 0]);
    assert!(!shows(&h, "export"));
}
