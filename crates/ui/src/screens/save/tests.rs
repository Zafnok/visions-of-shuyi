use insta::assert_snapshot;
use trpg_content::new_campaign;
use trpg_core::{LeadGender, LeadProfile, SAVE_VERSION};

use super::*;
use crate::audio::AudioRequest;
use crate::harness::Harness;
use crate::save::{SLOTS, SaveError, read};
use crate::screen::tests::ctx;
use crate::screens::save::SlotEntryView;
use crate::storage::{Storage, StorageError};
use crate::words::Words;

fn campaign(c: &Ctx, name: &str, playtime_s: u64) -> Campaign {
    let lead = LeadProfile::new(name, LeadGender::Female);
    let mut campaign = new_campaign(&c.content, GameMode::Classic, lead);
    campaign.playtime_s = playtime_s;
    campaign
}

/// Puts a chapter save of `campaign` in slot `slot`.
fn fill(c: &mut Ctx, slot: usize, campaign: Campaign) {
    let file = SaveFile::chapter_cleared(campaign);
    save::write(c.storage.as_mut(), &slot_key(slot), &file).unwrap();
}

/// The context with saves in slots 2 (Classic) and 4 (Casual), junk in
/// slot 5 and another version's save in slot 6.
fn saves() -> Ctx {
    let mut c = ctx();
    let first = campaign(&c, "Mara", 3725);
    let mut second = campaign(&c, "Ilse", 59);
    second.mode = GameMode::Casual;
    second.roster.truncate(1);
    fill(&mut c, 2, first.clone());
    fill(&mut c, 4, second);
    c.storage.write(&slot_key(5), "junk").unwrap();
    let mut old = SaveFile::chapter_cleared(first);
    old.version = SAVE_VERSION + 1;
    let old = ron::to_string(&old).unwrap();
    c.storage.write(&slot_key(6), &old).unwrap();
    c
}

/// The cue of a refused Confirm.
const DENIED: &str = MenuSound::Denied.cue();

fn press(s: &mut dyn Screen, c: &mut Ctx, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(c, &input))
}

/// The sounds played since the last call, as cue names.
fn sounds(c: &mut Ctx) -> Vec<String> {
    c.audio
        .take()
        .iter()
        .filter(|r| matches!(r, AudioRequest::PlaySound { .. }))
        .filter_map(|r| r.cue().map(str::to_owned))
        .collect()
}

/// The slots the picker shows, as `(number, focused)`.
fn numbers(s: &SlotPickerScreen, c: &Ctx) -> Vec<(usize, bool)> {
    s.view(c)
        .slots
        .iter()
        .map(|v| (v.number, v.focused))
        .collect()
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn the_prompt_can_be_found_by_its_type() {
    let s = SavePromptScreen::new(&ctx());
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<SavePromptScreen>().is_some());
}

/// The tests (and the game flow) reach the screen through `as_any`.
#[test]
fn the_picker_can_be_found_by_its_type() {
    let s = SlotPickerScreen::load(&ctx());
    let any = Screen::as_any(&s).expect("opts into as_any");
    assert!(any.downcast_ref::<SlotPickerScreen>().is_some());
}

#[test]
fn the_prompt_asks_yes_or_no() {
    let mut c = ctx();
    let mut s = SavePromptScreen::new(&c);
    assert_eq!(s.name(), "save_prompt");
    assert!(!s.is_overlay());
    // Nothing to back out of.
    assert_eq!(press(&mut s, &mut c, &[Action::Cancel]), "None");
    assert_eq!(s.result(), None);
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "Pop");
    assert_eq!(s.result(), Some(true));
    let mut s = SavePromptScreen::new(&c);
    let no = [Action::CursorDown, Action::Confirm, Action::CursorUp];
    assert_eq!(press(&mut s, &mut c, &no), "Pop");
    assert_eq!(s.result(), Some(false));
    assert_eq!(SavePromptScreen::help(&c), "arrows choose · f select");
}

#[test]
fn the_prompt_view_asks_with_yes_focused() {
    let c = ctx();
    let mut s = SavePromptScreen::new(&c);
    let view = s.view(&c);
    assert_eq!(view.question, c.text(SAVE_QUESTION));
    let labels: Vec<_> = view.menu.items.iter().map(|i| i.label.as_str()).collect();
    assert_eq!(labels, ["Yes", "No"]);
    assert_eq!(view.menu.focused().unwrap().label, "Yes");
    assert_eq!(view.help, SavePromptScreen::help(&c));
    press(&mut s, &mut ctx(), &[Action::CursorDown]);
    assert_eq!(s.view(&c).menu.focused().unwrap().label, "No");
}

#[test]
fn save_prompt_snapshot() {
    let h = Harness::with_screen(Box::new(SavePromptScreen::new(&ctx())));
    assert_snapshot!(h.snapshot());
}

/// Nick (PR #141): saving always opens on the first empty slot.
#[test]
fn saving_opens_on_the_first_empty_slot() {
    let mut c = saves();
    let game = campaign(&c, "Mara", 10);
    let s = SlotPickerScreen::save(&c, game.clone());
    assert_eq!(s.name(), "save_slots");
    assert!(s.is_saving());
    assert_eq!(s.focused_slot(), 1);
    assert!(!s.would_overwrite());
    fill(&mut c, 1, game.clone());
    // Slot 2 holds a save: the next empty one is 3.
    assert_eq!(SlotPickerScreen::save(&c, game.clone()).focused_slot(), 3);
    // An empty slot past the first screenful is scrolled into view.
    for slot in 1..=24 {
        fill(&mut c, slot, game.clone());
    }
    let s = SlotPickerScreen::save(&c, game.clone());
    assert_eq!(s.focused_slot(), 25);
    assert_eq!(s.view(&c).range, "6-25 of 30");
    // Every slot full: the first, which says it will be overwritten.
    for slot in 1..=SLOTS {
        fill(&mut c, slot, game.clone());
    }
    let s = SlotPickerScreen::save(&c, game);
    assert_eq!(s.focused_slot(), 1);
    assert!(s.would_overwrite());
}

#[test]
fn loading_opens_on_the_first_save() {
    let c = saves();
    let s = SlotPickerScreen::load(&c);
    assert!(!s.is_saving());
    assert_eq!(s.focused_slot(), 2);
    assert!(!s.would_overwrite(), "loading replaces nothing");
    assert_eq!(SlotPickerScreen::load(&ctx()).focused_slot(), 1);
}

/// Nick (PR #141): a slot that would be overwritten says so before it is
/// chosen.
#[test]
fn a_full_slot_says_it_will_be_overwritten() {
    let mut c = saves();
    let mut s = SlotPickerScreen::save(&c, campaign(&c, "Mara", 0));
    assert_eq!(s.help(&c), "arrows choose · f save here · d back");
    // Slot 2 holds a save.
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert!(s.would_overwrite());
    assert_eq!(s.help(&c), "arrows choose · f overwrite · d back");
    assert_eq!(s.view(&c).help, s.help(&c));
    // Slot 3 is empty; slot 5 can't be read, and would be replaced too.
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert!(!s.would_overwrite());
    assert_eq!(s.help(&c), "arrows choose · f save here · d back");
    press(&mut s, &mut c, &[Action::CursorDown, Action::CursorDown]);
    assert_eq!(s.focused_slot(), 5);
    assert!(s.would_overwrite());
    // Loading never says it.
    let s = SlotPickerScreen::load(&c);
    assert_eq!(s.help(&c), "arrows choose · f load · d back");
}

#[test]
fn the_focus_wraps_and_the_list_scrolls_with_it() {
    let mut c = saves();
    let mut s = SlotPickerScreen::load(&c).at(1);
    let shown = numbers(&s, &c);
    assert_eq!(shown.len(), 20);
    assert_eq!(shown[0], (1, true));
    assert_eq!(s.view(&c).range, "1-20 of 30");
    // Up from the first slot: the last, scrolled into view on the last row.
    assert_eq!(press(&mut s, &mut c, &[Action::CursorUp]), "None");
    assert_eq!(s.focused_slot(), SLOTS);
    assert_eq!(sounds(&mut c), ["menu_move"]);
    let shown = numbers(&s, &c);
    assert_eq!(s.view(&c).range, "11-30 of 30");
    assert_eq!(shown.first(), Some(&(11, false)));
    assert_eq!(shown.last(), Some(&(30, true)));
    // Down wraps to the first again.
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert_eq!(s.focused_slot(), 1);
    assert_eq!(s.view(&c).range, "1-20 of 30");
    // Down to slot 21: one row scrolled, the slot on the last row.
    let down = [Action::CursorDown; 20];
    press(&mut s, &mut c, &down);
    assert_eq!(s.focused_slot(), 21);
    assert_eq!(s.view(&c).range, "2-21 of 30");
    assert_eq!(numbers(&s, &c).last(), Some(&(21, true)));
    // Back up inside the window: no scrolling.
    press(&mut s, &mut c, &[Action::CursorUp]);
    assert_eq!(s.view(&c).range, "2-21 of 30");
    // Other keys do nothing, silently.
    sounds(&mut c);
    assert_eq!(
        press(&mut s, &mut c, &[Action::CursorLeft, Action::Info]),
        "None"
    );
    assert_eq!(s.focused_slot(), 20);
    assert!(sounds(&mut c).is_empty());
}

/// The picker always lists every slot, but moving never divides by a
/// count of none, and with one slot there is nowhere to move.
#[test]
fn moving_needs_two_slots() {
    let mut c = ctx();
    for n in 0..=1 {
        let mut s = SlotPickerScreen::new(Purpose::Load, vec![Slot::Empty; n], None);
        let moves = [Action::CursorDown, Action::CursorUp];
        assert_eq!(press(&mut s, &mut c, &moves), "None");
        assert_eq!(s.focused_slot(), 1);
        assert!(sounds(&mut c).is_empty(), "{n} slots");
    }
    let mut s = SlotPickerScreen::new(Purpose::Load, vec![Slot::Empty; 2], None);
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert_eq!(s.focused_slot(), 2);
    assert_eq!(sounds(&mut c), ["menu_move"]);
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert_eq!(s.focused_slot(), 1);
}

#[test]
fn a_slot_row_shows_chapter_mode_army_and_playtime() {
    let c = saves();
    let s = SlotPickerScreen::load(&c);
    let view = s.view(&c);
    let army = c.content.new_game.roster.len();
    let (title, _) = save::next_chapter_title(&c.content, Words::ENGLISH, &campaign(&c, "Mara", 0));
    let entry = |slot: usize| &view.slots[slot - 1].entry;
    let SlotEntryView::Saved {
        chapter,
        mode,
        mode_name: shown_name,
        army: army_shown,
        playtime,
    } = entry(2)
    else {
        panic!("slot 2 holds a save");
    };
    assert!(chapter.contains(&title), "{chapter:?} lacks {title:?}");
    assert_eq!(
        (*mode, shown_name.as_str(), army_shown, playtime.as_str()),
        (
            GameMode::Classic,
            "Classic",
            &army_text(&c, army),
            "1:02:05"
        )
    );
    let SlotEntryView::Saved {
        mode,
        army,
        playtime,
        ..
    } = entry(4)
    else {
        panic!("slot 4 holds a save");
    };
    assert_eq!(
        (*mode, army.as_str(), playtime.as_str()),
        (GameMode::Casual, "1 unit", "0:00:59")
    );
    assert_eq!(entry(1), &SlotEntryView::Empty("Empty".to_owned()));
    let SlotEntryView::Unreadable(why) = entry(5) else {
        panic!("slot 5 can't be read");
    };
    assert_eq!(why, "This save can't be read");
    let SlotEntryView::Unreadable(why) = entry(6) else {
        panic!("slot 6 is another version's");
    };
    assert!(why.contains("incompatible version"), "{why}");
    assert_eq!(view.title, "Load Game");
    assert_eq!(army_text(&c, 0), "0 units");
    assert_eq!(army_text(&c, 1), "1 unit");
    assert_eq!(mode_name(&c, GameMode::Classic), "Classic");
    assert_eq!(mode_name(&c, GameMode::Casual), "Casual");
}

/// The picker names its columns and shows a window of 20 slots with the
/// focused one marked.
#[test]
fn the_view_names_the_columns_and_marks_the_focused_slot() {
    let c = saves();
    let view = SlotPickerScreen::load(&c).view(&c);
    let columns = &view.columns;
    assert_eq!(
        [
            &columns.slot,
            &columns.chapter,
            &columns.mode,
            &columns.army,
            &columns.time
        ]
        .map(String::as_str),
        ["Slot", "Chapter", "Mode", "Army", "Time"]
    );
    assert_eq!(view.slots.len(), VISIBLE);
    let focused: Vec<_> = view.slots.iter().filter(|v| v.focused).collect();
    assert_eq!(focused.len(), 1);
    assert_eq!(focused[0].number, 2);
    assert_eq!(view.message, None);
    assert_eq!(view.question, None);
    assert_eq!(view.help, "arrows choose · f load · d back");
}

#[test]
fn saving_into_an_empty_slot_writes_it_at_once() {
    let mut c = saves();
    let game = campaign(&c, "Mara", 77);
    let mut s = SlotPickerScreen::save(&c, game.clone());
    assert_eq!(s.view(&c).title, "Save");
    assert_eq!(s.help(&c), "arrows choose · f save here · d back");
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "Pop");
    assert_eq!(sounds(&mut c), ["menu_select"]);
    assert_eq!(s.result(), Some(&SlotOutcome::Saved(1)));
    let written = read(c.storage.as_ref(), &slot_key(1)).unwrap();
    assert_eq!(written, Some(SaveFile::chapter_cleared(game)));
    assert_eq!(s.into_result(), Some(SlotOutcome::Saved(1)));
}

#[test]
fn saving_over_a_slot_asks_first() {
    let mut c = saves();
    let game = campaign(&c, "Nell", 5);
    let before = read(c.storage.as_ref(), &slot_key(2)).unwrap();
    let mut s = SlotPickerScreen::save(&c, game.clone()).at(2);
    // The key that opens the question doesn't answer it.
    let twice = [Action::Confirm, Action::Confirm];
    assert_eq!(press(&mut s, &mut c, &twice), "None");
    assert!(s.is_asking());
    assert_eq!(s.overwrite_question(&c), "Overwrite slot 02?");
    assert_eq!(s.help(&c), "f yes · d no");
    let question = s.view(&c).question.unwrap();
    assert_eq!(question.question, "Overwrite slot 02?");
    assert_eq!(question.answers, "f yes / d no");
    assert_eq!(read(c.storage.as_ref(), &slot_key(2)).unwrap(), before);
    // The cursor keys do nothing while it asks; Cancel backs out.
    sounds(&mut c);
    let no = [Action::CursorDown, Action::Cancel];
    assert_eq!(press(&mut s, &mut c, &no), "None");
    assert_eq!(sounds(&mut c), ["menu_cancel"]);
    assert!(!s.is_asking());
    assert_eq!(s.focused_slot(), 2);
    assert_eq!(s.view(&c).question, None);
    assert_eq!(read(c.storage.as_ref(), &slot_key(2)).unwrap(), before);
    assert_eq!(s.result(), None);
    // Asked again and confirmed: written.
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "Pop");
    assert_eq!(s.result(), Some(&SlotOutcome::Saved(2)));
    let written = read(c.storage.as_ref(), &slot_key(2)).unwrap();
    assert_eq!(written, Some(SaveFile::chapter_cleared(game.clone())));
    // A slot that can't be read is asked about too, and can be replaced.
    let mut s = SlotPickerScreen::save(&c, game.clone()).at(5);
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "None");
    assert!(s.is_asking());
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "Pop");
    let written = read(c.storage.as_ref(), &slot_key(5)).unwrap();
    assert_eq!(written, Some(SaveFile::chapter_cleared(game)));
}

#[test]
fn loading_takes_only_a_slot_that_can_be_read() {
    let mut c = saves();
    let mut s = SlotPickerScreen::load(&c).at(1);
    assert_eq!(s.help(&c), "arrows choose · f load · d back");
    // An empty slot.
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "None");
    assert_eq!(sounds(&mut c), [DENIED]);
    assert_eq!(s.message(), None);
    assert!(!s.is_asking());
    // Junk, then another version's save: each says why, and nothing loads.
    for (slot, error) in [(5, SaveError::Corrupt), (6, SaveError::Incompatible)] {
        let mut s = SlotPickerScreen::load(&c).at(slot);
        assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "None");
        assert_eq!(sounds(&mut c), [DENIED]);
        assert_eq!(s.message(), Some(error.to_string().as_str()));
        assert_eq!(s.view(&c).message, Some(error.to_string()));
        assert_eq!(s.result(), None);
        // Moving on clears the message.
        press(&mut s, &mut c, &[Action::CursorUp]);
        assert_eq!(s.message(), None);
        sounds(&mut c);
    }
    // A save.
    let mut s = SlotPickerScreen::load(&c);
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "Pop");
    assert_eq!(sounds(&mut c), ["menu_select"]);
    let expected = SaveFile::chapter_cleared(campaign(&c, "Mara", 3725));
    assert_eq!(
        s.into_result(),
        Some(SlotOutcome::Loaded(2, Box::new(expected)))
    );
}

#[test]
fn cancel_goes_back_without_a_result() {
    let mut c = saves();
    for mut s in [
        SlotPickerScreen::load(&c),
        SlotPickerScreen::save(&c, campaign(&c, "Mara", 0)),
    ] {
        // Nothing after the key that closes the picker counts.
        let back = [Action::Cancel, Action::Confirm];
        assert_eq!(press(&mut s, &mut c, &back), "Pop");
        assert_eq!(sounds(&mut c), ["menu_cancel"]);
        assert_eq!(s.result(), None);
    }
    assert_eq!(read(c.storage.as_ref(), &slot_key(1)), Ok(None));
}

/// A storage that can't be written to.
#[derive(Debug)]
struct ReadOnly;

impl Storage for ReadOnly {
    fn read(&self, _key: &str) -> Result<Option<String>, StorageError> {
        Ok(None)
    }
    fn write(&mut self, _key: &str, _value: &str) -> Result<(), StorageError> {
        Err(StorageError::Backend("disk full".into()))
    }
    fn delete(&mut self, _key: &str) -> Result<(), StorageError> {
        Ok(())
    }
    fn list(&self) -> Result<Vec<String>, StorageError> {
        Ok(Vec::new())
    }
}

#[test]
fn a_failed_save_says_so_and_stays_open() {
    let mut c = ctx();
    c.storage = Box::new(ReadOnly);
    let mut s = SlotPickerScreen::save(&c, campaign(&c, "Mara", 0));
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "None");
    assert_eq!(sounds(&mut c), [DENIED]);
    assert_eq!(s.message(), Some("Save storage error: disk full"));
    assert_eq!(s.result(), None);
    assert_eq!(
        s.view(&c).message.as_deref(),
        Some("Save storage error: disk full")
    );
}

/// The slots with two saves, an unreadable one and another version's; the
/// second save focused.
#[test]
fn slot_picker_snapshot() {
    let c = saves();
    let picker = SlotPickerScreen::load(&c).at(4);
    let mut h = Harness::from_game(crate::Game::new(c, Box::new(picker)));
    h.wait(0.0);
    assert_snapshot!(h.snapshot());
}

/// Saving over a slot: the question over the list.
#[test]
fn overwrite_question_snapshot() {
    let c = saves();
    let picker = SlotPickerScreen::save(&c, campaign(&c, "Mara", 0)).at(2);
    let mut h = Harness::from_game(crate::Game::new(c, Box::new(picker)));
    h.keys("f");
    assert_snapshot!(h.snapshot());
}
