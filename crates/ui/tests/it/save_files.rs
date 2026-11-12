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
use trpg_ui::input::{LEFT_HANDED, RIGHT_HANDED};
use trpg_ui::save::{self, slot_key};
use trpg_ui::screens::{ProfileScreen, SlotPickerScreen, TitleScreen};
use trpg_ui::tips::TipScreen;
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
    assert!(
        shows(&h, "e export · Space import · x open folder"),
        "{}",
        h.snapshot()
    );
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
    assert!(
        shows(&h, "e export · Space import · x open folder"),
        "{}",
        h.snapshot()
    );
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
    assert!(
        shows(&h, "e export · Space import · d back"),
        "{}",
        h.snapshot()
    );
    assert_snapshot!("profile_on_the_web", h.snapshot());
    let mut h = launch_on_the_web();
    open_load_game(&mut h);
    assert!(!shows(&h, "Saves are kept"));
    assert!(
        shows(&h, "f load · e export · Space import · d back"),
        "{}",
        h.snapshot()
    );
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

/// The guide's note beside the title's menu (ticket 0874), as the title
/// shows it now: its heading and text.
fn title_note(h: &Harness) -> Option<(String, String)> {
    let title = h.game().screen::<TitleScreen>();
    let view = title.map(|t| t.view(h.game().ctx()));
    let note = view.and_then(|v| v.note)?;
    Some((note.heading, note.lines.concat()))
}

/// The title's item under the cursor.
fn title_focus(h: &Harness) -> Option<String> {
    let title = h.game().screen::<TitleScreen>();
    let menu = title.and_then(|t| t.view(h.game().ctx()).menu);
    let focused = menu.and_then(|m| m.focused().map(|i| i.label.clone()));
    focused.map(|l| h.game().ctx().label(&l))
}

/// Acceptance (0874): the cursor on `Load Game` shows the note on
/// exporting a save, on `Profile` the one on exporting the profile, on
/// any other item none; the cursor still moves through the menu as
/// before, the note taking no key.
#[test]
fn the_title_note_follows_the_cursor_onto_load_game_and_profile() {
    let mut h = launch();
    let items = [
        ("New Game", None),
        ("Load Game", Some("export any of your saves")),
        ("Quick Battle", None),
        ("Options", None),
        ("Profile", Some("export your whole profile")),
        ("Credits", None),
        ("Quit", None),
        ("New Game", None),
    ];
    for (i, (item, text)) in items.into_iter().enumerate() {
        if i > 0 {
            h.keys("Down");
        }
        assert_eq!(title_focus(&h).as_deref(), Some(item));
        let note = title_note(&h);
        match text {
            Some(text) => {
                let (heading, lines) = note.unwrap_or_else(|| panic!("no note on {item}"));
                assert_eq!(heading, "Did you know?");
                assert!(lines.contains(text), "{item}: {lines}");
            }
            None => assert_eq!(note, None, "{item}"),
        }
    }
    h.keys("Up Up Up");
    assert_eq!(title_focus(&h).as_deref(), Some("Profile"));
    assert_snapshot!("title_note_on_profile", h.snapshot());
    h.keys("Up Up Up");
    assert_eq!(title_focus(&h).as_deref(), Some("Load Game"));
    assert_snapshot!("title_note_on_load_game", h.snapshot());
    // The note takes no key: Confirm opens Load Game.
    h.keys("f");
    assert_eq!(h.top_screen(), "save_slots");
}

/// No export, no note: the web still has one (a download), a platform
/// without files a player reaches (the Playdate) none.
#[test]
fn no_title_note_without_files_a_player_reaches() {
    let mut h = launch_on_the_web();
    h.keys("Down");
    assert!(title_note(&h).is_some());
    let mut h = launch();
    h.ctx_mut().platform.storage_place = StoragePlace::Unknown;
    h.keys("Down");
    assert_eq!(title_focus(&h).as_deref(), Some("Load Game"));
    assert_eq!(title_note(&h), None);
}

/// The tip box on top, if one is: its heading and its instructions.
fn tip(h: &Harness) -> Option<(String, String)> {
    let tip = h.game().screen::<TipScreen>()?;
    let view = tip.view(h.game().ctx())?;
    Some((view.heading, view.how.join("\n")))
}

/// Acceptance (0874): the first time Load Game opens with a save in it,
/// and the first time the Profile screen opens, the guide's tip on
/// exporting shows over it, naming the export key of the player's keymap;
/// Confirm closes it; once per profile.
#[test]
fn each_export_tip_shows_once_per_profile_naming_the_export_key() {
    let mut h = launch();
    h.with_tips();
    open_under_a_tip(&mut h, "Down f");
    let (heading, how) = tip(&h).unwrap_or_else(|| panic!("{}", h.snapshot()));
    assert_eq!(heading, "Exporting a save");
    assert_eq!(how, "Export the save under the cursor: e");
    assert_snapshot!("load_game_export_tip", h.snapshot());
    h.keys("f");
    assert_eq!(h.top_screen(), "save_slots");
    // Seen: Load Game opens alone from now on.
    h.keys("d f");
    assert_eq!(h.top_screen(), "save_slots", "{}", h.snapshot());
    open_under_a_tip(&mut h, "d Down Down Down f");
    let (heading, how) = tip(&h).unwrap_or_else(|| panic!("no tip"));
    assert_eq!(heading, "Exporting a profile");
    assert_eq!(how, "Export the profile under the cursor: e");
    assert_snapshot!("profile_export_tip", h.snapshot());
    h.keys("d");
    assert_eq!(h.top_screen(), "profiles");
    h.keys("d f");
    assert_eq!(h.top_screen(), "profiles", "{}", h.snapshot());

    // Another profile has not seen them; the left-handed keymap names its
    // own key.
    let mut h = Harness::with_layout_and(LEFT_HANDED, |ctx| {
        fill(ctx, 1);
        ctx.platform.storage_place = StoragePlace::Folder(FOLDER.to_owned());
    });
    h.with_tips().wait(0.1);
    // The left-handed layout moves with wasd and confirms with j.
    open_under_a_tip(&mut h, "s j");
    let (_, how) = tip(&h).unwrap_or_else(|| panic!("no tip"));
    assert_eq!(how, "Export the save under the cursor: i");
}

/// A screen opened from the title by `keys`, with the tip over it.
fn open_under_a_tip(h: &mut Harness, keys: &str) {
    // Load Game is opened by the game flow, a frame after the key.
    h.keys(keys).wait(0.02);
    assert_eq!(h.top_screen(), "tip", "{}", h.snapshot());
}

/// The Profile screen's tip needs no save; where nothing can be exported
/// (the Playdate), neither tip shows.
#[test]
fn the_profile_tip_needs_no_save_and_no_tip_shows_without_files() {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.with_tips();
    h.ctx_mut().platform.storage_place = StoragePlace::Folder(FOLDER.to_owned());
    h.wait(0.1);
    // Load Game is off without a save: Down skips it.
    open_under_a_tip(&mut h, "Down Down Down f");
    let (heading, _) = tip(&h).unwrap_or_else(|| panic!("no tip"));
    assert_eq!(heading, "Exporting a profile");
    let mut h = launch();
    h.with_tips();
    h.ctx_mut().platform.storage_place = StoragePlace::Unknown;
    open_load_game(&mut h);
    h.keys("d Down Down Down f");
    assert_eq!(h.top_screen(), "profiles", "{}", h.snapshot());
}
