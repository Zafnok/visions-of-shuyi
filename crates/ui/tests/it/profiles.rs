//! Player profiles (ticket 0836, ADR-0069): each profile keeps its own
//! options, keys, tips seen and saves; the game opens on the last one used;
//! a player from before profiles becomes `Player 1`.

use std::collections::BTreeMap;

use trpg_content::new_campaign;
use trpg_core::{GameMode, LeadGender, LeadProfile, SaveFile};
use trpg_ui::harness::Harness;
use trpg_ui::input::{Action, Chord, Layout};
use trpg_ui::profiles::{PROFILES_KEY, ProfileError, ProfileId};
use trpg_ui::save::{self, SUSPEND_KEY, slot_key};
use trpg_ui::screens::title::BACKDROP_KEY;
use trpg_ui::settings::TextSpeed;
use trpg_ui::tips::TipsSeen;
use trpg_ui::{MemoryStorage, Storage};

fn chord(s: &str) -> Chord {
    Chord::parse(s).unwrap_or_else(|e| panic!("{e}"))
}

fn ok<T, E: std::fmt::Display>(result: Result<T, E>) -> T {
    result.unwrap_or_else(|e| panic!("{e}"))
}

/// Puts Confirm's first key of the layout in use on `key`.
fn confirm_on(h: &mut Harness, key: &str) {
    let layout = h
        .game()
        .ctx()
        .layout()
        .unwrap_or_else(|| panic!("no layout"));
    let mut b = h.game().ctx().layout_bindings(layout);
    ok(b.bind(Action::Confirm, 0, chord(key)));
    ok(h.ctx_mut().set_layout_bindings(layout, b));
}

fn set_text_speed(h: &mut Harness, speed: TextSpeed) {
    ok(h.ctx_mut().change_settings(|s| s.text_speed = speed));
}

fn mark_tip(h: &mut Harness, id: &str) {
    let mut seen = TipsSeen::load(&h.game().ctx().storage);
    seen.mark(id, &mut h.ctx_mut().storage);
}

fn tip_seen(h: &Harness, id: &str) -> bool {
    TipsSeen::load(&h.game().ctx().storage).contains(id)
}

/// Saves a chapter save of a lead called `name` at `key` of the profile
/// in use.
fn save_at(h: &mut Harness, key: &str, name: &str) {
    let lead = LeadProfile::new(name, LeadGender::Female);
    let campaign = new_campaign(&h.game().ctx().content, GameMode::Classic, lead);
    let file = SaveFile::chapter_cleared(campaign);
    ok(save::write(&mut h.ctx_mut().storage, key, &file));
}

/// The lead's name of the save at `key` of the profile in use.
fn saved_lead(h: &Harness, key: &str) -> Option<String> {
    let save = ok(save::read(&h.game().ctx().storage, key));
    save.map(|s| s.campaign.lead.name)
}

fn text_speed(h: &Harness) -> TextSpeed {
    h.game().ctx().settings().text_speed
}

fn confirm_key(h: &Harness, key: &str) -> bool {
    h.game().ctx().keymap.action(chord(key)) == Some(Action::Confirm)
}

fn names(h: &Harness) -> Vec<String> {
    let profiles = h.game().ctx().profiles();
    profiles.all().iter().map(|p| p.name.clone()).collect()
}

fn add(h: &mut Harness, name: &str) -> ProfileId {
    ok(h.ctx_mut().add_profile(name))
}

fn switch(h: &mut Harness, id: ProfileId) {
    ok(h.ctx_mut().switch_profile(id));
}

/// Every key in `storage` and its value.
fn everything(storage: &dyn Storage) -> BTreeMap<String, String> {
    let keys = ok(storage.list());
    let value = |k: &String| ok(storage.read(k)).unwrap_or_else(|| panic!("{k}"));
    keys.iter().map(|k| (k.clone(), value(k))).collect()
}

/// A storage as a player had it before profiles: the left-handed layout
/// with Confirm on `g`, slow text, a tip seen, a save in slot 3 and a
/// suspend save, all under their own keys; and the title's backdrop.
fn storage_from_before_profiles() -> MemoryStorage {
    let mut h = Harness::with_layout(Layout::LeftHanded);
    set_text_speed(&mut h, TextSpeed::Slow);
    confirm_on(&mut h, "g");
    mark_tip(&mut h, "first_move");
    save_at(&mut h, &slot_key(3), "Mara");
    save_at(&mut h, SUSPEND_KEY, "Odo");
    let now = h.into_storage();
    let mut old = MemoryStorage::new();
    for (key, value) in everything(&*now) {
        let key = key.strip_prefix(&ProfileId::FIRST.prefix()).unwrap_or(&key);
        if key != PROFILES_KEY {
            ok(old.write(key, &value));
        }
    }
    ok(old.write(BACKDROP_KEY, "backdrop.png"));
    old
}

#[test]
fn a_first_launch_makes_player_1_and_asks_for_the_layout() {
    let mut h = Harness::new();
    assert_eq!(names(&h), ["Player 1"]);
    assert_eq!(h.game().ctx().profiles().current(), ProfileId::FIRST);
    h.keys("z");
    assert_eq!(h.top_screen(), "layout_picker");
}

#[test]
fn a_storage_from_before_profiles_loads_as_player_1() {
    let old = storage_from_before_profiles();
    let before = everything(&old);
    assert!(before.contains_key("settings") && before.contains_key("keybindings"));
    let h = Harness::with_storage(Box::new(old));
    assert_eq!(names(&h), ["Player 1"]);
    // Every option, key, tip seen, save and suspend save is there.
    assert_eq!(h.screens(), ["title"], "the layout picker doesn't open");
    assert_eq!(h.game().ctx().layout(), Some(Layout::LeftHanded));
    assert_eq!(text_speed(&h), TextSpeed::Slow);
    assert!(confirm_key(&h, "g"));
    assert!(tip_seen(&h, "first_move"));
    assert_eq!(saved_lead(&h, &slot_key(3)).as_deref(), Some("Mara"));
    assert_eq!(saved_lead(&h, SUSPEND_KEY).as_deref(), Some("Odo"));
    assert_eq!(
        save::latest_lead(&h.game().ctx().storage).map(|l| l.name),
        Some("Odo".to_owned())
    );
    // Moved, not copied: only the backdrop stays with the computer.
    let ctx = h.game().ctx();
    for (key, value) in &before {
        let (profile, computer) = (ctx.storage.read(key), ctx.storage.computer().read(key));
        if key == BACKDROP_KEY {
            assert_eq!(computer, Ok(Some(value.clone())));
        } else {
            assert_eq!(profile, Ok(Some(value.clone())), "{key}");
            assert_eq!(computer, Ok(None), "{key}");
        }
    }
    // The move runs once: an old key turning up later isn't moved again.
    let mut storage = h.into_storage();
    ok(storage.write("settings", "Settings(version: 1, text_speed: Fast)"));
    let h = Harness::with_storage(storage);
    assert_eq!(text_speed(&h), TextSpeed::Slow);
    assert_eq!(names(&h), ["Player 1"]);
}

#[test]
fn each_profile_keeps_its_own_options_keys_tips_and_saves() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    set_text_speed(&mut h, TextSpeed::Slow);
    confirm_on(&mut h, "g");
    mark_tip(&mut h, "first_move");
    save_at(&mut h, &slot_key(1), "Mara");
    let mei = add(&mut h, "Mei");
    assert_eq!(names(&h), ["Player 1", "Mei"]);
    assert_eq!(h.game().ctx().profiles().current(), ProfileId::FIRST);

    // A new profile starts from the defaults, with no layout picked.
    switch(&mut h, mei);
    assert_eq!(h.game().ctx().layout(), None);
    assert_eq!(text_speed(&h), TextSpeed::Normal);
    assert!(!confirm_key(&h, "g"));
    assert!(!tip_seen(&h, "first_move"));
    assert_eq!(saved_lead(&h, &slot_key(1)), None);
    assert!(!save::any_slot(&h.game().ctx().storage));
    ok(h.ctx_mut().choose_layout(Layout::LeftHanded));
    set_text_speed(&mut h, TextSpeed::Fast);
    confirm_on(&mut h, "h");
    save_at(&mut h, &slot_key(2), "Odo");
    save_at(&mut h, SUSPEND_KEY, "Piet");

    for _ in 0..2 {
        switch(&mut h, ProfileId::FIRST);
        assert_eq!(h.game().ctx().layout(), Some(Layout::RightHanded));
        assert_eq!(text_speed(&h), TextSpeed::Slow);
        assert!(confirm_key(&h, "g") && !confirm_key(&h, "h"));
        assert!(tip_seen(&h, "first_move"));
        assert_eq!(saved_lead(&h, &slot_key(1)).as_deref(), Some("Mara"));
        assert_eq!(saved_lead(&h, &slot_key(2)), None);
        assert!(!save::has_suspend(&h.game().ctx().storage));
        // The keys work at once.
        h.keys("g");
        assert_eq!(h.screens(), ["title", "mode_select"]);
        h.keys("d");

        switch(&mut h, mei);
        assert_eq!(h.game().ctx().layout(), Some(Layout::LeftHanded));
        assert_eq!(text_speed(&h), TextSpeed::Fast);
        assert!(confirm_key(&h, "h") && !confirm_key(&h, "g"));
        assert!(!tip_seen(&h, "first_move"));
        assert_eq!(saved_lead(&h, &slot_key(1)), None);
        assert_eq!(saved_lead(&h, &slot_key(2)).as_deref(), Some("Odo"));
        assert_eq!(saved_lead(&h, SUSPEND_KEY).as_deref(), Some("Piet"));
    }
}

#[test]
fn the_next_launch_opens_on_the_last_profile_used() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    let mei = add(&mut h, "Mei");
    switch(&mut h, mei);
    ok(h.ctx_mut().choose_layout(Layout::LeftHanded));
    set_text_speed(&mut h, TextSpeed::Fast);
    let mut h = Harness::with_storage(h.into_storage());
    assert_eq!(h.game().ctx().profiles().current(), mei);
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(h.game().ctx().layout(), Some(Layout::LeftHanded));
    assert_eq!(text_speed(&h), TextSpeed::Fast);
    switch(&mut h, ProfileId::FIRST);
    let h = Harness::with_storage(h.into_storage());
    assert_eq!(h.game().ctx().profiles().current(), ProfileId::FIRST);
    assert_eq!(h.game().ctx().layout(), Some(Layout::RightHanded));
    assert_eq!(text_speed(&h), TextSpeed::Normal);
}

#[test]
fn deleting_a_profile_removes_its_keys_and_no_others() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    save_at(&mut h, &slot_key(1), "Mara");
    let (mei, odo) = (add(&mut h, "Mei"), add(&mut h, "Odo"));
    for id in [mei, odo] {
        switch(&mut h, id);
        ok(h.ctx_mut().choose_layout(Layout::LeftHanded));
        mark_tip(&mut h, "first_move");
        save_at(&mut h, &slot_key(1), "Piet");
    }
    ok(h.ctx_mut()
        .storage
        .computer_mut()
        .write(BACKDROP_KEY, "b.png"));
    let before = everything(h.game().ctx().storage.computer());
    assert!(before.keys().any(|k| k.starts_with(&mei.prefix())));

    // Deleting the profile in use: the first one left is used.
    ok(h.ctx_mut().delete_profile(odo));
    assert_eq!(names(&h), ["Player 1", "Mei"]);
    assert_eq!(h.game().ctx().profiles().current(), ProfileId::FIRST);
    assert_eq!(h.game().ctx().layout(), Some(Layout::RightHanded));
    ok(h.ctx_mut().delete_profile(mei));
    let after = everything(h.game().ctx().storage.computer());
    let kept: BTreeMap<String, String> = before
        .into_iter()
        .filter(|(k, _)| !k.starts_with(&mei.prefix()) && !k.starts_with(&odo.prefix()))
        .filter(|(k, _)| k != PROFILES_KEY)
        .collect();
    let after_but_list: BTreeMap<String, String> = after
        .into_iter()
        .filter(|(k, _)| k != PROFILES_KEY)
        .collect();
    assert_eq!(after_but_list, kept);
    assert_eq!(saved_lead(&h, &slot_key(1)).as_deref(), Some("Mara"));

    // The only profile left stays, and a new one never gets a deleted
    // one's keys.
    assert_eq!(
        h.ctx_mut().delete_profile(ProfileId::FIRST),
        Err(ProfileError::LastOne)
    );
    let new = add(&mut h, "Mei");
    assert!(new != mei && new != odo);
    let h = Harness::with_storage(h.into_storage());
    assert_eq!(names(&h), ["Player 1", "Mei"]);
}

#[test]
fn a_refused_change_leaves_the_list_as_it_was() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    let mei = add(&mut h, "Mei");
    let ctx = h.ctx_mut();
    assert_eq!(ctx.add_profile("player 1"), Err(ProfileError::NameTaken));
    assert_eq!(ctx.rename_profile(mei, ""), Err(ProfileError::NameEmpty));
    ok(ctx.switch_profile(mei));
    ok(ctx.rename_profile(mei, "Shuyi"));
    let h = Harness::with_storage(h.into_storage());
    assert_eq!(names(&h), ["Player 1", "Shuyi"]);
    assert_eq!(h.game().ctx().profiles().current(), mei);
}
