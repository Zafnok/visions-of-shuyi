use super::*;
use crate::audio::AudioRequest;
use crate::input::{Chord, Key, Layout};
use crate::screen::tests::ctx;
use crate::settings::SETTINGS_KEY;
use crate::storage::{Storage, StorageError};
use crate::tips::{TIPS_SEEN_KEY, TipsSeen};

fn press(s: &mut OptionsScreen, c: &mut Ctx, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(c, &input))
}

/// The screen focused on `row`.
fn on(row: Row) -> OptionsScreen {
    let mut s = OptionsScreen::new();
    s.row = row;
    s
}

/// The sounds asked for since the last call.
fn sounds(c: &mut Ctx) -> Vec<String> {
    let cue = |r: AudioRequest| r.cue().map(str::to_owned);
    c.audio.take().into_iter().filter_map(cue).collect()
}

fn saved(c: &Ctx) -> Settings {
    let text = c.storage.read(SETTINGS_KEY).unwrap().unwrap();
    Settings::from_ron(&text).unwrap()
}

#[test]
fn game_mode_shows_only_with_a_campaign() {
    let mut c = ctx();
    let rows = OptionsScreen::rows(&c);
    assert_eq!(rows.len(), Row::ALL.len() - 1);
    assert!(!rows.contains(&Row::GameMode));
    c.campaign_mode = Some(GameMode::Casual);
    assert_eq!(OptionsScreen::rows(&c), Row::ALL);
}

#[test]
fn up_and_down_move_the_focus_and_wrap() {
    use Action::{CursorDown, CursorUp};
    let mut c = ctx();
    let mut s = OptionsScreen::new();
    assert_eq!(s.name(), "options");
    assert!(!s.is_overlay());
    assert_eq!(s.focus(), Row::TextSpeed);
    press(&mut s, &mut c, &[CursorDown]);
    assert_eq!(s.focus(), Row::AnimSpeed);
    assert_eq!(sounds(&mut c), ["menu_move"]);
    press(&mut s, &mut c, &[CursorUp, CursorUp]);
    assert_eq!(s.focus(), Row::RestoreDefaults);
    press(&mut s, &mut c, &[CursorUp, CursorUp]);
    // No campaign: Game mode is skipped.
    assert_eq!(s.focus(), Row::KeyBindings);
    press(&mut s, &mut c, &[CursorDown, CursorDown, CursorDown]);
    assert_eq!(s.focus(), Row::TextSpeed);
}

#[test]
fn left_and_right_step_a_setting_and_stop_at_the_ends() {
    use Action::{CursorLeft, CursorRight};
    let mut c = ctx();
    let mut s = OptionsScreen::new();
    press(&mut s, &mut c, &[CursorRight]);
    assert_eq!(c.settings().text_speed, TextSpeed::Fast);
    assert_eq!(saved(&c).text_speed, TextSpeed::Fast);
    assert_eq!(sounds(&mut c), ["menu_move"]);
    press(&mut s, &mut c, &[CursorRight, CursorRight, CursorRight]);
    assert_eq!(c.settings().text_speed, TextSpeed::Instant);
    // Only the step that changed something sounds.
    assert_eq!(sounds(&mut c), ["menu_move"]);
    press(&mut s, &mut c, &[CursorLeft; 5]);
    assert_eq!(c.settings().text_speed, TextSpeed::Slow);
    assert_eq!(sounds(&mut c).len(), 3);
}

#[test]
fn confirm_steps_a_setting_and_goes_round() {
    let mut c = ctx();
    let mut s = on(Row::Cursor);
    let styles: Vec<_> = (0..3)
        .map(|_| {
            press(&mut s, &mut c, &[Action::Confirm]);
            c.settings().cursor_style
        })
        .collect();
    assert_eq!(
        styles,
        [
            CursorStyle::LargeCorners,
            CursorStyle::TileGlow,
            CursorStyle::Corners
        ]
    );
}

/// One frame of typing: the characters `text`, then the keys `keys`.
fn type_in(s: &mut OptionsScreen, c: &mut Ctx, text: &str, keys: &[Key]) {
    let chords = keys.iter().map(|&k| Chord::plain(k)).collect();
    let input = FrameInput::new(vec![], 0.0, vec![]).with_typing(chords, text.chars().collect());
    s.update(c, &input);
}

/// Nick: a volume is 0 to 100, with a slider.
#[test]
fn left_and_right_move_a_volumes_slider_five_at_a_time() {
    use Action::{CursorLeft, CursorRight};
    let mut c = ctx();
    let mut s = on(Row::MusicVolume);
    assert_eq!(s.help(&c), "arrows move · f type a number · d back");
    press(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(c.settings().music_volume, 75);
    assert_eq!(saved(&c).music_volume, 75);
    press(&mut s, &mut c, &[CursorRight; 6]);
    assert_eq!(c.settings().music_volume, 100);
    // Five steps changed it; the sixth did nothing.
    assert_eq!(sounds(&mut c).len(), 6);
    c.change_settings(|s| s.music_volume = 3).unwrap();
    press(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(c.settings().music_volume, 0);
    press(&mut s, &mut c, &[CursorLeft]);
    assert_eq!(c.settings().music_volume, 0);
    assert_eq!(c.settings().sound_volume, 80);
}

/// Nick: and an input box for the exact number.
#[test]
fn confirm_on_a_volume_opens_a_box_to_type_the_number() {
    let mut c = ctx();
    let mut s = on(Row::SoundVolume);
    // The key that opens the box types nothing, nor do the keys after it.
    press(&mut s, &mut c, &[Action::Confirm, Action::CursorDown]);
    assert_eq!(s.typing().map(NumberBox::value), Some(None));
    assert_eq!(s.focus(), Row::SoundVolume);
    assert_eq!(sounds(&mut c), ["menu_select"]);
    assert_eq!(s.help(&c), "Enter done · Backspace delete · Escape cancel");
    assert!(
        drawn_row(&s, &c, 13).contains(" _ "),
        "{}",
        drawn_row(&s, &c, 13)
    );
    // The game's keys do nothing while it is open.
    press(&mut s, &mut c, &[Action::CursorDown, Action::Cancel]);
    assert!(s.typing().is_some());
    // Digits only, three at most.
    type_in(&mut s, &mut c, "3x7", &[]);
    assert_eq!(s.typing().and_then(NumberBox::value), Some(37));
    assert_eq!(sounds(&mut c), ["menu_cancel"], "the x is refused");
    assert!(drawn_row(&s, &c, 13).contains(" 37_ "));
    type_in(&mut s, &mut c, "55", &[]);
    assert_eq!(s.typing().map(NumberBox::text), Some("375_".to_owned()));
    // Backspace deletes; nothing changes until Enter.
    type_in(&mut s, &mut c, "", &[Key::Backspace, Key::Backspace]);
    assert_eq!(s.typing().and_then(NumberBox::value), Some(3));
    assert_eq!(c.settings().sound_volume, 80);
    type_in(&mut s, &mut c, "7", &[Key::Enter]);
    assert!(s.typing().is_none());
    assert_eq!(c.settings().sound_volume, 37);
    assert_eq!(saved(&c).sound_volume, 37);
    assert_eq!(c.settings().music_volume, 80);
    // More than 100 is 100.
    press(&mut s, &mut c, &[Action::Confirm]);
    type_in(&mut s, &mut c, "250", &[Key::Enter]);
    assert_eq!(c.settings().sound_volume, 100);
    // Escape, or Enter on an empty box, leaves the volume alone.
    press(&mut s, &mut c, &[Action::Confirm]);
    type_in(&mut s, &mut c, "12", &[Key::Escape]);
    assert!(s.typing().is_none());
    press(&mut s, &mut c, &[Action::Confirm]);
    type_in(&mut s, &mut c, "", &[Key::Enter]);
    assert!(s.typing().is_none());
    press(&mut s, &mut c, &[Action::Confirm]);
    type_in(&mut s, &mut c, "", &[Key::Backspace, Key::Enter]);
    assert_eq!(c.settings().sound_volume, 100);
    // The music row's box sets the music.
    let mut s = on(Row::MusicVolume);
    press(&mut s, &mut c, &[Action::Confirm]);
    type_in(&mut s, &mut c, "0", &[Key::Enter]);
    assert_eq!(c.settings().music_volume, 0);
    assert_eq!(c.settings().sound_volume, 100);
}

/// On a controller there is nothing to type with: the box holds the
/// volume and the cursor steps it by one.
#[test]
fn the_volume_box_steps_by_one_on_a_controller() {
    let pad = |actions: &[Action]| FrameInput::new(actions.to_vec(), 0.0, vec![]).with_pad(true);
    let mut c = ctx();
    let mut s = on(Row::MusicVolume);
    s.update(&mut c, &pad(&[Action::Confirm]));
    assert_eq!(s.typing().and_then(NumberBox::value), Some(80));
    assert_eq!(s.help(&c), "arrows change · f done · d cancel");
    assert!(drawn_row(&s, &c, 12).contains(" 80 "));
    sounds(&mut c);
    let steps = [Action::CursorUp, Action::CursorRight, Action::CursorUp];
    s.update(&mut c, &pad(&steps));
    assert_eq!(s.typing().and_then(NumberBox::value), Some(83));
    assert_eq!(sounds(&mut c).len(), 3);
    s.update(&mut c, &pad(&[Action::CursorDown, Action::Info]));
    assert_eq!(s.typing().and_then(NumberBox::value), Some(82));
    // Not until Confirm.
    assert_eq!(c.settings().music_volume, 80);
    s.update(&mut c, &pad(&[Action::Confirm, Action::CursorDown]));
    assert!(s.typing().is_none());
    assert_eq!(c.settings().music_volume, 82);
    assert_eq!(s.focus(), Row::MusicVolume);
    // Cancel keeps the old volume; the ends hold.
    s.update(&mut c, &pad(&[Action::Confirm]));
    s.update(&mut c, &pad(&[Action::CursorLeft, Action::Cancel]));
    assert_eq!(c.settings().music_volume, 82);
    c.change_settings(|s| s.music_volume = 100).unwrap();
    s.update(&mut c, &pad(&[Action::Confirm]));
    sounds(&mut c);
    s.update(&mut c, &pad(&[Action::CursorUp]));
    assert_eq!(s.typing().and_then(NumberBox::value), Some(100));
    assert!(sounds(&mut c).is_empty());
    c.change_settings(|s| s.music_volume = 0).unwrap();
    s.update(&mut c, &pad(&[Action::Cancel]));
    s.update(&mut c, &pad(&[Action::Confirm]));
    s.update(&mut c, &pad(&[Action::CursorDown, Action::Confirm]));
    assert_eq!(c.settings().music_volume, 0);
}

#[test]
fn every_setting_row_changes_its_own_setting() {
    let mut c = ctx();
    for row in Row::ALL.into_iter().filter(|r| r.is_setting()) {
        let before = c.settings().clone();
        // Right, or left for a setting already at its last value.
        press(&mut on(row), &mut c, &[Action::CursorRight]);
        if *c.settings() == before {
            press(&mut on(row), &mut c, &[Action::CursorLeft]);
        }
        let s = c.settings().clone();
        let changed = [
            (Row::TextSpeed, s.text_speed != before.text_speed),
            (Row::AnimSpeed, s.anim_speed != before.anim_speed),
            (
                Row::CombatAnimations,
                s.combat_animations != before.combat_animations,
            ),
            (
                Row::EnemyPhaseSpeed,
                s.enemy_phase_speed != before.enemy_phase_speed,
            ),
            (Row::AutoEnd, s.auto_end_turn != before.auto_end_turn),
            (Row::Fullscreen, s.fullscreen != before.fullscreen),
            (Row::Cursor, s.cursor_style != before.cursor_style),
            (Row::MusicVolume, s.music_volume != before.music_volume),
            (Row::SoundVolume, s.sound_volume != before.sound_volume),
        ];
        let rows: Vec<Row> = changed.iter().filter(|c| c.1).map(|c| c.0).collect();
        assert_eq!(rows, [row]);
        assert_eq!(saved(&c), s);
    }
}

#[test]
fn rows_show_their_values() {
    let mut c = ctx();
    let value = |c: &Ctx, row| OptionsScreen::value(c, row);
    let shown: Vec<String> = Row::ALL.iter().map(|&r| value(&c, r)).collect();
    assert_eq!(
        shown,
        [
            "Normal",
            "Normal",
            "On",
            "Normal",
            "Off",
            "Off",
            "Corners",
            "████████░░  80",
            "████████░░  80",
            "Right-handed",
            "",
            "",
            "",
            ""
        ]
    );
    c.change_settings(|s| {
        s.cursor_style = CursorStyle::LargeCorners;
        s.music_volume = 0;
        s.sound_volume = 100;
        s.auto_end_turn = true;
    })
    .unwrap();
    c.campaign_mode = Some(GameMode::Classic);
    c.use_layout(Layout::LeftHanded);
    assert_eq!(value(&c, Row::Cursor), "Large corners");
    assert_eq!(value(&c, Row::MusicVolume), "░░░░░░░░░░   0");
    assert_eq!(value(&c, Row::SoundVolume), "██████████ 100");
    // A half cell for the odd five; under it rounds down.
    assert_eq!(volume_text(85), "████████▒░  85");
    assert_eq!(volume_text(84), "████████░░  84");
    assert_eq!(volume_text(5), "▒░░░░░░░░░   5");
    assert_eq!(volume_text(99), "█████████▒  99");
    assert_eq!(volume_text(200), "██████████ 100");
    assert_eq!(value(&c, Row::AutoEnd), "On");
    assert_eq!(value(&c, Row::GameMode), "Classic");
    assert_eq!(value(&c, Row::Layout), "Left-handed");
    assert_eq!(c.text(cursor_key(CursorStyle::TileGlow)), "Tile glow");
    c.campaign_mode = Some(GameMode::Casual);
    assert_eq!(value(&c, Row::GameMode), "Casual");
}

#[test]
fn layout_and_key_bindings_open_their_screens() {
    let mut c = ctx();
    assert_eq!(
        press(&mut on(Row::Layout), &mut c, &[Action::Confirm]),
        "Push(layout_picker)"
    );
    assert_eq!(
        press(&mut on(Row::KeyBindings), &mut c, &[Action::Confirm]),
        "Push(key_bindings)"
    );
    assert_eq!(sounds(&mut c), ["menu_select", "menu_select"]);
    // Left and right do nothing on them.
    let before = c.settings().clone();
    for row in [Row::Layout, Row::KeyBindings, Row::GameMode, Row::ResetTips] {
        let keys = [Action::CursorLeft, Action::CursorRight];
        assert_eq!(press(&mut on(row), &mut c, &keys), "None");
    }
    assert_eq!(*c.settings(), before);
    assert!(sounds(&mut c).is_empty());
}

#[test]
fn cancel_closes_the_screen() {
    let mut c = ctx();
    let mut s = OptionsScreen::new();
    assert_eq!(press(&mut s, &mut c, &[Action::Cancel]), "Pop");
    assert_eq!(sounds(&mut c), ["menu_cancel"]);
}

#[test]
fn classic_switches_to_casual_after_a_confirm_and_never_back() {
    let mut c = ctx();
    c.campaign_mode = Some(GameMode::Classic);
    c.mode_switch = ModeSwitch::Open;
    let mut s = on(Row::GameMode);
    assert_eq!(s.help(&c), "arrows move · f switch to Casual · d back");
    press(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    // The key that asked isn't the answer.
    assert_eq!(s.asking(), Some(Question::SwitchToCasual));
    assert_eq!(c.campaign_mode, Some(GameMode::Classic));
    assert_eq!(s.help(&c), "f yes · d no");
    // Nothing but the answers does anything.
    press(&mut s, &mut c, &[Action::CursorDown, Action::CursorRight]);
    assert_eq!((s.focus(), s.asking().is_some()), (Row::GameMode, true));
    // No: still Classic.
    assert_eq!(press(&mut s, &mut c, &[Action::Cancel]), "None");
    assert_eq!((s.asking(), s.message()), (None, None));
    assert_eq!(c.campaign_mode, Some(GameMode::Classic));
    // Yes.
    press(&mut s, &mut c, &[Action::Confirm]);
    sounds(&mut c);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(c.campaign_mode, Some(GameMode::Casual));
    assert_eq!(s.message(), Some(CASUAL_MESSAGE));
    assert_eq!(sounds(&mut c), ["menu_select"]);
    // Casual offers nothing.
    assert_eq!(s.help(&c), "arrows move · d back");
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.asking(), None);
    assert_eq!(c.campaign_mode, Some(GameMode::Casual));
    assert_eq!(sounds(&mut c), ["menu_cancel"]);
}

/// Nick: not in the middle of a battle, only at Preparations.
#[test]
fn classic_cant_switch_where_the_switch_is_closed() {
    let mut c = ctx();
    c.campaign_mode = Some(GameMode::Classic);
    let mut s = on(Row::GameMode);
    assert_eq!(s.help(&c), "arrows move · d back");
    press(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    assert_eq!(s.asking(), None);
    assert_eq!(c.campaign_mode, Some(GameMode::Classic));
    assert_eq!(s.message(), Some(MODE_AT_PREP_MESSAGE));
    assert_eq!(sounds(&mut c), ["menu_cancel", "menu_cancel"]);
    // A Casual campaign gets no such message: there is nothing to switch.
    c.campaign_mode = Some(GameMode::Casual);
    c.mode_switch = ModeSwitch::Open;
    let mut s = on(Row::GameMode);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!((s.asking(), s.message()), (None, None));
}

#[test]
fn reset_tips_forgets_the_tips_seen() {
    let mut c = ctx();
    let mut seen = TipsSeen::default();
    seen.mark("first_move", &mut *c.storage);
    assert!(TipsSeen::load(&*c.storage).contains("first_move"));
    let mut s = on(Row::ResetTips);
    assert_eq!(s.help(&c), "arrows move · f reset · d back");
    // Nick: it asks first.
    press(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    assert_eq!(s.asking(), Some(Question::ResetTips));
    assert!(TipsSeen::load(&*c.storage).contains("first_move"));
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!((s.asking(), s.message()), (None, None));
    assert!(TipsSeen::load(&*c.storage).contains("first_move"));
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(c.storage.read(TIPS_SEEN_KEY), Ok(None));
    assert_eq!(s.message(), Some(TIPS_RESET_MESSAGE));
    // Moving on clears the message.
    press(&mut s, &mut c, &[Action::CursorDown]);
    assert_eq!(s.message(), None);
}

#[test]
fn restore_defaults_asks_first_and_keeps_the_layout() {
    let mut c = ctx();
    c.choose_layout(Layout::LeftHanded).unwrap();
    c.change_settings(|s| {
        s.text_speed = TextSpeed::Slow;
        s.fullscreen = true;
        s.music_volume = 2;
    })
    .unwrap();
    let changed = c.settings().clone();
    let mut s = on(Row::RestoreDefaults);
    assert_eq!(s.help(&c), "wasd move · j restore · k back");
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.asking(), Some(Question::RestoreDefaults));
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(*c.settings(), changed);
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    let expected = Settings::default().with_layout(Layout::LeftHanded);
    assert_eq!(*c.settings(), expected);
    assert_eq!(saved(&c), expected);
    assert_eq!(s.message(), Some(RESTORED_MESSAGE));
    assert_eq!(c.layout(), Some(Layout::LeftHanded));
}

/// A storage that reads nothing and can't be written.
#[derive(Debug)]
struct ReadOnly;

impl Storage for ReadOnly {
    fn read(&self, _: &str) -> Result<Option<String>, StorageError> {
        Ok(None)
    }
    fn write(&mut self, _: &str, _: &str) -> Result<(), StorageError> {
        Err(StorageError::Backend("full".into()))
    }
    fn delete(&mut self, _: &str) -> Result<(), StorageError> {
        Err(StorageError::Backend("full".into()))
    }
    fn list(&self) -> Result<Vec<String>, StorageError> {
        Ok(Vec::new())
    }
}

#[test]
fn a_failed_save_still_changes_the_setting_and_says_so() {
    let mut c = ctx().with_storage(Box::new(ReadOnly));
    let mut s = OptionsScreen::new();
    press(&mut s, &mut c, &[Action::CursorRight]);
    assert_eq!(c.settings().text_speed, TextSpeed::Fast);
    assert_eq!(s.message(), Some(NOT_SAVED_MESSAGE));
    let mut s = on(Row::ResetTips);
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.message(), Some(NOT_SAVED_MESSAGE));
    let mut s = on(Row::SoundVolume);
    press(&mut s, &mut c, &[Action::Confirm]);
    type_in(&mut s, &mut c, "9", &[Key::Enter]);
    assert_eq!(c.settings().sound_volume, 9);
    assert_eq!(s.message(), Some(NOT_SAVED_MESSAGE));
    let mut s = on(Row::RestoreDefaults);
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(c.settings().text_speed, TextSpeed::Normal);
    assert_eq!(s.message(), Some(NOT_SAVED_MESSAGE));
}

#[test]
fn help_names_what_confirm_does_on_each_kind_of_row() {
    let c = ctx();
    assert_eq!(
        OptionsScreen::new().help(&c),
        "arrows move · f change · d back"
    );
    assert_eq!(on(Row::Layout).help(&c), "arrows move · f open · d back");
    assert_eq!(
        on(Row::KeyBindings).help(&c),
        "arrows move · f open · d back"
    );
}

#[test]
fn stepping_helpers_stop_or_wrap() {
    let all = [1, 2, 3];
    assert_eq!(stepped(&all, 3, true, false), 3);
    assert_eq!(stepped(&all, 3, true, true), 1);
    assert_eq!(stepped(&all, 1, false, false), 1);
    assert_eq!(stepped(&all, 1, false, true), 3);
    assert_eq!(stepped(&all, 2, true, false), 3);
    assert_eq!(stepped(&all, 2, false, false), 1);
    assert_eq!(stepped_volume(100, true), 100);
    assert_eq!(stepped_volume(97, true), 100);
    assert_eq!(stepped_volume(50, true), 55);
    assert_eq!(stepped_volume(50, false), 45);
    assert_eq!(stepped_volume(3, false), 0);
    assert_eq!(stepped_volume(0, false), 0);
}

/// Row `y` of the screen as drawn, trimmed.
fn drawn_row(s: &OptionsScreen, c: &Ctx, y: i32) -> String {
    use crate::console::{CONSOLE_H, CONSOLE_W};
    let black = c.palette.get(UiColor::Black);
    let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, Cell::new(' ', black, black));
    s.draw(c, &mut buf);
    let glyphs = (0..i32::from(CONSOLE_W)).map(|x| buf.get(x, y).map_or(' ', |c| c.glyph));
    glyphs.collect::<String>().trim().to_owned()
}

/// The message sits one blank row under the panel's bottom border.
#[test]
fn the_message_is_drawn_under_the_panel() {
    let mut c = ctx();
    let mut s = on(Row::ResetTips);
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    let bottom = PANEL.y + PANEL.h - 1;
    assert!(drawn_row(&s, &c, bottom).starts_with('└'));
    assert_eq!(drawn_row(&s, &c, bottom + 1), "");
    assert_eq!(drawn_row(&s, &c, bottom + 2), "Tips will show again");
    assert_eq!(drawn_row(&s, &c, bottom + 3), "");
}
