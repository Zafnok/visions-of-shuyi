use super::*;
use crate::save::tests::{campaign, suspended, written_by};
use crate::screen::tests::ctx;

fn verdict(c: &Ctx, key: &str, text: &str) -> Verdict {
    GameCheck::new(&c.game.content).check(key, text)
}

fn chapter_save(c: &Ctx) -> SaveFile {
    SaveFile::chapter_cleared(campaign(c))
}

fn text_of(save: &SaveFile) -> String {
    save::encode(save).unwrap_or_else(|e| panic!("{e}"))
}

/// A slot reads as the Load Game list reads it: a chapter save of this
/// version is ready; another version is incompatible; damaged text, or a
/// battle save (never written in a slot), can't be read.
#[test]
fn a_slot_is_ready_only_as_a_chapter_save_of_this_version() {
    let c = ctx();
    let save = chapter_save(&c);
    for key in ["slot_01", "slot_30"] {
        assert_eq!(verdict(&c, key, &text_of(&save)), Verdict::Ready, "{key}");
    }
    let world = SaveFile::world_map(campaign(&c));
    assert_eq!(verdict(&c, "slot_02", &text_of(&world)), Verdict::Ready);
    assert_eq!(
        verdict(&c, "slot_02", &written_by(&save, 1)),
        Verdict::Incompatible { found: 1, wants: 2 }
    );
    let text = text_of(&save);
    let damaged = &text[..text.len() / 2];
    assert_eq!(verdict(&c, "slot_02", damaged), Verdict::Unreadable);
    assert_eq!(
        verdict(&c, "slot_02", &text_of(&suspended(&c))),
        Verdict::Unreadable
    );
    // Past the last slot is no slot.
    assert_eq!(verdict(&c, "slot_31", &text), Verdict::Unknown);
}

/// The suspend save is a battle of a chapter the content has: Continue
/// couldn't go on with another.
#[test]
fn a_suspend_save_needs_a_battle_of_a_chapter_the_game_has() {
    let c = ctx();
    let battle = suspended(&c);
    assert_eq!(verdict(&c, SUSPEND_KEY, &text_of(&battle)), Verdict::Ready);
    let mut gone = battle.clone();
    gone.campaign.chapter = "a_chapter_cut".to_owned();
    assert_eq!(
        verdict(&c, SUSPEND_KEY, &text_of(&gone)),
        Verdict::Unreadable
    );
    let chapter = chapter_save(&c);
    assert_eq!(
        verdict(&c, SUSPEND_KEY, &text_of(&chapter)),
        Verdict::Unreadable
    );
    assert_eq!(
        verdict(&c, SUSPEND_KEY, &written_by(&battle, 1)),
        Verdict::Incompatible { found: 1, wants: 2 }
    );
}

/// The options read as the game reads them: this version and the older
/// ones it still reads are ready, a later one is incompatible.
#[test]
fn the_options_are_read_by_their_version() {
    let c = ctx();
    let now = Settings::to_ron(c.settings());
    assert_eq!(verdict(&c, SETTINGS_KEY, &now), Verdict::Ready);
    let old = now.replacen("version: 3", "version: 1", 1);
    assert_ne!(old, now);
    assert_eq!(verdict(&c, SETTINGS_KEY, &old), Verdict::Ready);
    let later = now.replacen("version: 3", "version: 9", 1);
    assert_eq!(
        verdict(&c, SETTINGS_KEY, &later),
        Verdict::Incompatible { found: 9, wants: 3 }
    );
    assert_eq!(verdict(&c, SETTINGS_KEY, "(nonsense"), Verdict::Unreadable);
}

/// The key bindings and the layout read as the engine reads them.
#[test]
fn the_keys_and_the_layout_are_read_as_the_engine_reads_them() {
    let c = ctx();
    let keys = c.player_keys().to_ron();
    assert_eq!(verdict(&c, KEYBINDINGS_KEY, &keys), Verdict::Ready);
    let later = keys.replacen("version: 2", "version: 7", 1);
    assert_ne!(later, keys);
    assert_eq!(
        verdict(&c, KEYBINDINGS_KEY, &later),
        Verdict::Incompatible { found: 7, wants: 2 }
    );
    assert_eq!(verdict(&c, KEYBINDINGS_KEY, "(("), Verdict::Unreadable);
    // Its version reads, the rest doesn't.
    let broken = "PlayerKeys(version: 2, layouts: 5)";
    assert_eq!(verdict(&c, KEYBINDINGS_KEY, broken), Verdict::Unreadable);
    assert_eq!(verdict(&c, LAYOUT_KEY, "RightHanded\n"), Verdict::Ready);
    assert_eq!(verdict(&c, LAYOUT_KEY, "Dvorak"), Verdict::Unreadable);
}

/// The tips seen and the set-up mark are free text; a key the game
/// doesn't keep is unknown, and left out.
#[test]
fn other_keys_of_a_profile_are_ready_and_the_rest_unknown() {
    let c = ctx();
    for key in [TIPS_SEEN_KEY, SET_UP_KEY, LAST_SAVE_KEY] {
        assert_eq!(verdict(&c, key, "anything"), Verdict::Ready, "{key}");
    }
    for key in ["debug_view", "profiles", "high_scores"] {
        assert_eq!(verdict(&c, key, "anything"), Verdict::Unknown, "{key}");
    }
}

/// A file holding `entries`, made by this build.
fn file(entries: &[(&str, &str)]) -> String {
    let entries = entries
        .iter()
        .map(|&(k, v)| (k.to_owned(), v.to_owned()))
        .collect();
    shuyi_ui::storage::bundle::SaveBundle::new(made(), entries).to_text()
}

fn verdict_in(plan: &ImportPlan, key: &str) -> Option<Verdict> {
    plan.entry(key).map(|e| e.verdict)
}

/// `last_save` is ready only if the save it names is imported too, or
/// already in the profile it goes into.
#[test]
fn last_save_needs_the_save_it_names() {
    let c = ctx();
    let save = text_of(&chapter_save(&c));
    let content = &c.game.content;
    let with = file(&[("slot_01", &save), (LAST_SAVE_KEY, "slot_01")]);
    let planned =
        super::plan(&with, &MemoryStorage::new(), content).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(verdict_in(&planned, LAST_SAVE_KEY), Some(Verdict::Ready));
    let without = file(&[("slot_01", &save), (LAST_SAVE_KEY, "slot_02")]);
    let planned =
        super::plan(&without, &MemoryStorage::new(), content).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        verdict_in(&planned, LAST_SAVE_KEY),
        Some(Verdict::Unreadable)
    );
    let mut held = MemoryStorage::new();
    held.write("slot_02", &save)
        .unwrap_or_else(|e| panic!("{e}"));
    let planned = super::plan(&without, &held, content).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(verdict_in(&planned, LAST_SAVE_KEY), Some(Verdict::Ready));
    // A save named that can't be imported doesn't count.
    let old = written_by(&chapter_save(&c), 1);
    let refused = file(&[("slot_01", &old), (LAST_SAVE_KEY, "slot_01")]);
    let planned =
        super::plan(&refused, &MemoryStorage::new(), content).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        verdict_in(&planned, LAST_SAVE_KEY),
        Some(Verdict::Unreadable)
    );
}

/// The profile's name comes back out of the file's name, as `export_name`
/// wrote it.
#[test]
fn the_name_in_a_file_name() {
    for (file, name) in [
        ("visions-of-shuyi-odo-kai-2.ron", "Odo Kai"),
        ("visions-of-shuyi-player-1-1.ron", "Player 1"),
        ("visions-of-shuyi-mei-1-slot-07.ron", "Mei"),
        ("visions-of-shuyi-mei-1-slot-07 (2).ron", "Mei"),
        // A name with a number in it keeps it.
        ("visions-of-shuyi-odo-2-1-slot-07.ron", "Odo 2"),
        ("visions-of-shuyi-2.ron", ""),
        // Renamed by the player: no number to drop.
        ("visions-of-shuyi-odo.ron", "Odo"),
        ("my-saves.ron", ""),
        ("visions-of-shuyi-abcdefghijklmnop-1.ron", "Abcdefghijkl"),
    ] {
        assert_eq!(name_in(file), name, "{file}");
    }
    assert_eq!(with_number("Odo", 2), "Odo 2");
    assert_eq!(with_number("Abcdefghijkl", 3), "Abcdefghij 3");
}

/// The options' line takes the first verdict that isn't ready, whichever
/// key it is on.
#[test]
fn the_options_verdict_is_the_first_that_isnt_ready() {
    let c = ctx();
    let keys = c.player_keys().to_ron();
    let settings = Settings::to_ron(c.settings()).replacen("version: 3", "version: 9", 1);
    let text = file(&[(KEYBINDINGS_KEY, &keys), (SETTINGS_KEY, &settings)]);
    let report = Report::read(&c, "a.ron", &text, Channel::Profile);
    let report = report.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        report.options,
        Some(Verdict::Incompatible { found: 9, wants: 3 })
    );
}

/// A suspend save that decodes but can't be continued shows no chapter:
/// only its verdict.
#[test]
fn a_refused_suspend_save_shows_no_chapter() {
    let c = ctx();
    let mut gone = suspended(&c);
    gone.campaign.chapter = "a_chapter_cut".to_owned();
    let text = file(&[(SUSPEND_KEY, &text_of(&gone))]);
    let report = Report::read(&c, "a.ron", &text, Channel::Profile);
    let report = report.unwrap_or_else(|e| panic!("{e}"));
    let [line] = report.saves.as_slice() else {
        panic!("{:?}", report.saves);
    };
    assert_eq!(line.verdict, Verdict::Unreadable);
    assert_eq!((line.chapter.as_deref(), line.playtime_s), (None, None));
    let ready = file(&[(SUSPEND_KEY, &text_of(&suspended(&c)))]);
    let report = Report::read(&c, "a.ron", &ready, Channel::Profile);
    let report = report.unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(report.saves[0].playtime_s, Some(3725));
}

/// Another profile's storage reads, writes, lists and deletes its own
/// keys, with its prefix in the computer's storage.
#[test]
fn another_profiles_storage_keeps_to_its_prefix() {
    let mut computer = MemoryStorage::new();
    computer
        .write("p1-slot_01", "mine")
        .unwrap_or_else(|e| panic!("{e}"));
    let id = ron::from_str::<ProfileId>("2").unwrap_or_else(|e| panic!("{e}"));
    let mut other = OtherProfile {
        computer: &mut computer,
        id,
    };
    assert_eq!(other.read("slot_01"), Ok(None));
    other
        .write("slot_01", "theirs")
        .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(other.read("slot_01"), Ok(Some("theirs".to_owned())));
    assert_eq!(other.list(), Ok(vec!["slot_01".to_owned()]));
    other.delete("slot_01").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(other.list(), Ok(Vec::new()));
    assert_eq!(computer.read("p1-slot_01"), Ok(Some("mine".to_owned())));
}
