use trpg_content::new_campaign;
use trpg_core::{
    BattleHistory, BattleState, CharacterId, LeadGender, LeadProfile, SAVE_VERSION, SupportPair,
    SupportRank,
};

use super::*;
use crate::screen::Ctx;
use crate::screen::tests::ctx;
use crate::storage::MemoryStorage;

fn campaign(c: &Ctx) -> Campaign {
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let mut campaign = new_campaign(&c.game.content, GameMode::Casual, lead);
    campaign.playtime_s = 3725;
    campaign
}

/// A suspend save of the test chapter's battle, just started.
fn suspended(c: &Ctx) -> SaveFile {
    let campaign = campaign(c);
    let def = &c.game.content.battles[&c.game.content.chapters[&campaign.chapter].battle];
    let (state, _) = BattleState::new(campaign.battle_setup(def, &c.game.content.tables()));
    SaveFile::suspended(campaign, BattleHistory::new(state))
}

/// `save` as text, written by `version`.
fn written_by(save: &SaveFile, version: u32) -> String {
    let mut save = save.clone();
    save.version = version;
    ron::to_string(&save).unwrap()
}

#[test]
fn slot_keys_are_valid_storage_keys() {
    assert_eq!(SLOTS, 30);
    assert_eq!(slot_key(1), "slot_01");
    assert_eq!(slot_key(30), "slot_30");
    for n in 1..=SLOTS {
        assert!(crate::storage::is_valid_key(&slot_key(n)));
    }
    assert!(crate::storage::is_valid_key(SUSPEND_KEY));
}

#[test]
fn a_save_round_trips_through_text() {
    let c = ctx();
    let save = SaveFile::chapter_cleared(campaign(&c));
    let text = encode(&save).unwrap();
    assert_eq!(decode(&text), Ok(save));
    // A battle comes back without its content tables (ADR-0020), so it is
    // compared as text.
    let save = suspended(&c);
    let text = encode(&save).unwrap();
    let back = decode(&text).unwrap();
    assert_eq!(back.campaign, save.campaign);
    assert!(matches!(back.point, SavePoint::Battle(_)));
    assert_eq!(encode(&back).unwrap(), text);
}

#[test]
fn support_state_survives_a_save_and_a_load() {
    let c = ctx();
    let table = &c.game.content.supports;
    let id = |s: &str| CharacterId(s.into());
    let knight = SupportPair::new(id("test_lord"), id("test_knight"));
    let mage = SupportPair::new(id("test_mage"), id("test_lord"));
    // The knight's pair at rank C with 3 points more; the mage's at 7.
    let mut game = campaign(&c);
    game.supports.gain(&knight, 20, table);
    game.supports.view(&knight, table).unwrap();
    game.supports.gain(&knight, 3, table);
    game.supports.gain(&mage, 7, table);
    let supports = game.supports.clone();
    let text = encode(&SaveFile::chapter_cleared(game.clone())).unwrap();
    let back = decode(&text).unwrap().campaign;
    assert_eq!(back.supports, supports);
    let state = back.supports.state(&knight, table).unwrap();
    assert_eq!(
        (state.points(), state.rank(), state.unlocked()),
        (23, Some(SupportRank::C), None)
    );
    let points = back.supports.state(&mage, table).map(|s| s.points());
    assert_eq!(points, Some(7));
    // A suspended battle holds them too, in the campaign and in the
    // battle's own state.
    let def = &c.game.content.battles[&c.game.content.chapters[&game.chapter].battle];
    let (state, _) = BattleState::new(game.battle_setup(def, &c.game.content.tables()));
    let save = SaveFile::suspended(game, BattleHistory::new(state));
    let back = decode(&encode(&save).unwrap()).unwrap();
    assert_eq!(back.campaign.supports, supports);
    let SavePoint::Battle(history) = back.point else {
        panic!("not a suspend save");
    };
    assert_eq!(history.state_at(0).supports(), &supports);
}

#[test]
fn another_versions_save_is_incompatible_and_junk_is_corrupt() {
    let c = ctx();
    let save = SaveFile::chapter_cleared(campaign(&c));
    for version in [SAVE_VERSION - 1, SAVE_VERSION + 1] {
        let text = written_by(&save, version);
        assert_eq!(decode(&text), Err(SaveError::Incompatible));
    }
    // Another version whose fields this build can't parse at all.
    let old = "SaveFile(version:0,army:[\"lord\"])";
    assert_eq!(decode(old), Err(SaveError::Incompatible));
    // This version, but not a save.
    let text = format!("SaveFile(version:{SAVE_VERSION},army:[])");
    assert_eq!(decode(&text), Err(SaveError::Corrupt));
    for junk in ["", "hello", "(", "SaveFile(version:\"one\")", "[1,2,3]"] {
        assert_eq!(decode(junk), Err(SaveError::Corrupt), "{junk:?}");
    }
    // Cut short.
    let text = encode(&save).unwrap();
    assert_eq!(decode(&text[..text.len() / 2]), Err(SaveError::Corrupt));
}

#[test]
fn the_errors_say_what_the_player_reads() {
    assert_eq!(
        SaveError::Incompatible.to_string(),
        "This save is from an incompatible version"
    );
    assert_eq!(SaveError::Corrupt.to_string(), "This save can't be read");
    assert_eq!(SaveError::Missing.to_string(), "There is no save here");
    let storage = SaveError::Storage(StorageError::Backend("disk full".into()));
    assert_eq!(storage.to_string(), "Save storage error: disk full");
}

#[test]
fn saves_are_written_to_and_read_from_storage() {
    let c = ctx();
    let mut storage = MemoryStorage::new();
    assert_eq!(read(&storage, SUSPEND_KEY), Ok(None));
    assert!(!has_suspend(&storage));
    assert!(!any_slot(&storage));
    let save = suspended(&c);
    write(&mut storage, SUSPEND_KEY, &save).unwrap();
    assert!(has_suspend(&storage));
    assert!(!any_slot(&storage));
    let back = read(&storage, SUSPEND_KEY).unwrap().unwrap();
    assert_eq!(encode(&back), encode(&save));
    // The last slot counts as a slot; junk in one does too.
    storage.write(&slot_key(SLOTS), "junk").unwrap();
    assert!(any_slot(&storage));
    assert_eq!(read(&storage, &slot_key(SLOTS)), Err(SaveError::Corrupt));
    // A key that isn't one is a storage error, not a panic.
    let bad = read(&storage, "No Such Key");
    assert!(matches!(bad, Err(SaveError::Storage(_))), "{bad:?}");
    let bad = write(&mut storage, "No Such Key", &suspended(&c));
    assert!(matches!(bad, Err(SaveError::Storage(_))), "{bad:?}");
    assert!(!has_suspend(&MemoryStorage::new()));
}

#[test]
fn a_slot_shows_the_chapter_it_goes_on_with() {
    let mut c = ctx();
    let game = campaign(&c);
    assert_eq!(game.chapter, "test");
    let title = c.game.content.chapters["test"].title.clone();
    // No chapter after it yet.
    assert_eq!(
        next_chapter_title(&c.game.content, Words::ENGLISH, &game),
        (title.clone(), true)
    );
    // The chapter after it.
    c.game.content.chapters.get_mut("test").unwrap().next = Some("quick".into());
    let next = c.game.content.chapters["quick"].title.clone();
    assert_ne!(next, title);
    assert_eq!(
        next_chapter_title(&c.game.content, Words::ENGLISH, &game),
        (next, false)
    );
    // A next chapter the game doesn't have: as if there were none.
    c.game.content.chapters.get_mut("test").unwrap().next = Some("gone".into());
    assert_eq!(
        next_chapter_title(&c.game.content, Words::ENGLISH, &game),
        (title.clone(), true)
    );
    // A cleared chapter the game no longer has: its id.
    let mut lost = game;
    lost.chapter = "old_ch".into();
    assert_eq!(
        next_chapter_title(&c.game.content, Words::ENGLISH, &lost),
        ("old_ch".to_owned(), false)
    );
}

/// A world-map save's slot shows the chapter being played (1007).
#[test]
fn a_world_map_slot_shows_the_chapter_being_played() {
    let mut c = ctx();
    let mut game = campaign(&c);
    game.chapter = "test_wm1".into();
    let file = SaveFile::world_map(game);
    write(&mut c.storage, &slot_key(1), &file).unwrap();
    let Slot::Saved(summary) = slot(&c.storage, &c.game.content, Words::ENGLISH, 1) else {
        panic!("slot 1 holds the save");
    };
    assert_eq!(
        (summary.chapter.as_str(), summary.cleared),
        ("Test World I", false)
    );
}

/// A slot's chapter title is in the player's language (ticket 0235).
#[test]
fn a_slots_chapter_is_titled_in_the_players_language() {
    let mut c = ctx();
    let game = campaign(&c);
    c.lang = trpg_content::LangCode::new(trpg_content::lang::TEST).unwrap();
    assert_eq!(
        next_chapter_title(&c.game.content, c.words(), &game),
        ("TEST CHAPTER".to_owned(), true)
    );
    // The chapter after it is one the test pack doesn't title.
    c.game.content.chapters.get_mut("quick").unwrap().next = Some("test".into());
    let mut quick = game.clone();
    quick.chapter = "quick".into();
    assert_eq!(
        next_chapter_title(&c.game.content, c.words(), &quick),
        ("TEST CHAPTER".to_owned(), false)
    );
    quick.chapter = "test".into();
    c.game.content.chapters.get_mut("test").unwrap().next = Some("quick".into());
    assert_eq!(
        next_chapter_title(&c.game.content, c.words(), &quick),
        ("Quick Battle".to_owned(), false)
    );
    // A slot read from storage is titled the same way.
    let mut storage = MemoryStorage::new();
    let save = SaveFile::chapter_cleared(game);
    write(&mut storage, &slot_key(1), &save).unwrap();
    let Slot::Saved(summary) = slot(&storage, &c.game.content, c.words(), 1) else {
        panic!("no save in slot 1");
    };
    assert_eq!(summary.chapter, "Quick Battle");
    let all = slots(&storage, &c.game.content, c.words());
    assert_eq!(all[0], Slot::Saved(summary));
}

#[test]
fn slots_are_empty_saved_or_unreadable() {
    let c = ctx();
    let mut storage = MemoryStorage::new();
    let game = campaign(&c);
    let save = SaveFile::chapter_cleared(game.clone());
    write(&mut storage, &slot_key(2), &save).unwrap();
    storage.write(&slot_key(3), "junk").unwrap();
    let old = written_by(&save, SAVE_VERSION + 1);
    storage.write(&slot_key(4), &old).unwrap();
    // A battle save is never written to a slot.
    write(&mut storage, &slot_key(5), &suspended(&c)).unwrap();

    let all = slots(&storage, &c.game.content, Words::ENGLISH);
    assert_eq!(all.len(), SLOTS);
    assert_eq!(all[0], Slot::Empty);
    let Slot::Saved(summary) = &all[1] else {
        panic!("{:?}", all[1]);
    };
    assert_eq!(
        (summary.chapter.clone(), summary.cleared),
        next_chapter_title(&c.game.content, Words::ENGLISH, &game)
    );
    // No chapter follows the test chapter: the slot says it is cleared.
    assert!(summary.cleared);
    assert_eq!(summary.title(&c), format!("{} (cleared)", summary.chapter));
    let mut next = (**summary).clone();
    next.cleared = false;
    assert_eq!(next.title(&c), summary.chapter);
    assert_eq!(summary.mode, GameMode::Casual);
    assert_eq!(summary.roster, game.roster.len());
    assert!(summary.roster > 0);
    assert_eq!(summary.playtime_s, 3725);
    assert_eq!(summary.save, save);
    assert_eq!(all[2], Slot::Unreadable(SaveError::Corrupt));
    assert_eq!(all[3], Slot::Unreadable(SaveError::Incompatible));
    assert_eq!(all[4], Slot::Unreadable(SaveError::Corrupt));
    assert!(all[5..].iter().all(|s| *s == Slot::Empty));
    assert_eq!(slot(&storage, &c.game.content, Words::ENGLISH, 2), all[1]);
}

#[test]
fn playtime_reads_as_hours_minutes_seconds() {
    assert_eq!(playtime_text(0), "0:00:00");
    assert_eq!(playtime_text(59), "0:00:59");
    assert_eq!(playtime_text(60), "0:01:00");
    assert_eq!(playtime_text(3725), "1:02:05");
    assert_eq!(playtime_text(3600 * 123 + 59 * 60 + 59), "123:59:59");
}

#[test]
fn a_written_save_is_noted_as_the_last_one() {
    let c = ctx();
    let mut storage = MemoryStorage::new();
    assert_eq!(storage.read(LAST_SAVE_KEY), Ok(None));
    write(&mut storage, &slot_key(3), &suspended(&c)).unwrap();
    assert_eq!(storage.read(LAST_SAVE_KEY), Ok(Some(slot_key(3))));
    write(&mut storage, SUSPEND_KEY, &suspended(&c)).unwrap();
    assert_eq!(storage.read(LAST_SAVE_KEY).unwrap().unwrap(), SUSPEND_KEY);
    // A save that wasn't written isn't noted.
    assert!(write(&mut storage, "No Such Key", &suspended(&c)).is_err());
    assert_eq!(storage.read(LAST_SAVE_KEY).unwrap().unwrap(), SUSPEND_KEY);
}

/// A save whose lead is `name`, of `gender`.
pub(crate) fn save_of(c: &Ctx, name: &str, gender: LeadGender) -> SaveFile {
    let mut save = suspended(c);
    save.campaign.lead = LeadProfile::new(name, gender);
    save
}

#[test]
fn the_latest_lead_is_the_lead_of_the_save_written_last() {
    let c = ctx();
    let lead = |name: &str, gender| Some(LeadProfile::new(name, gender));
    let (male, female) = (LeadGender::Male, LeadGender::Female);
    let mut storage = MemoryStorage::new();
    assert_eq!(latest_lead(&storage), None);
    write(&mut storage, &slot_key(2), &save_of(&c, "Mara", female)).unwrap();
    assert_eq!(latest_lead(&storage), lead("Mara", female));
    // A later save in an earlier slot, then the suspend save, then a
    // later slot: always the one written last.
    write(&mut storage, &slot_key(1), &save_of(&c, "Odo", male)).unwrap();
    assert_eq!(latest_lead(&storage), lead("Odo", male));
    write(&mut storage, SUSPEND_KEY, &save_of(&c, "Anna", female)).unwrap();
    assert_eq!(latest_lead(&storage), lead("Anna", female));
    write(&mut storage, &slot_key(9), &save_of(&c, "Piet", male)).unwrap();
    assert_eq!(latest_lead(&storage), lead("Piet", male));
    // The last one gone, or unreadable: the suspend save's.
    storage.delete(&slot_key(9)).unwrap();
    assert_eq!(latest_lead(&storage), lead("Anna", female));
    storage.write(&slot_key(9), "junk").unwrap();
    assert_eq!(latest_lead(&storage), lead("Anna", female));
    // A note that names no key at all is no worse.
    storage.write(LAST_SAVE_KEY, "No Such Key").unwrap();
    assert_eq!(latest_lead(&storage), lead("Anna", female));
    // No suspend save: the first slot that holds a save.
    storage.delete(SUSPEND_KEY).unwrap();
    assert_eq!(latest_lead(&storage), lead("Odo", male));
    storage.write(&slot_key(1), "junk").unwrap();
    assert_eq!(latest_lead(&storage), lead("Mara", female));
    // Saves from before the note was kept: the same order.
    storage.delete(LAST_SAVE_KEY).unwrap();
    assert_eq!(latest_lead(&storage), lead("Mara", female));
    let text = encode(&save_of(&c, "Anna", female)).unwrap();
    storage.write(SUSPEND_KEY, &text).unwrap();
    assert_eq!(latest_lead(&storage), lead("Anna", female));
    // Nothing readable: nobody.
    storage.delete(SUSPEND_KEY).unwrap();
    storage.delete(&slot_key(2)).unwrap();
    assert_eq!(latest_lead(&storage), None);
}
