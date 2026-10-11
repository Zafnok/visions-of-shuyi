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

/// What the focused row shows beside its name.
fn focused_value(s: &OptionsScreen, c: &Ctx) -> ValueView {
    let view = s.view(c);
    view.focused().map(|r| r.value.clone()).unwrap()
}

/// The volume box as the view shows it: its digits, and whether it is
/// typed in.
fn boxed(digits: &str, typing: bool) -> ValueView {
    ValueView::NumberBox {
        digits: digits.to_owned(),
        typing,
    }
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
    assert_eq!(focused_value(&s, &c), boxed("", true));
    // The game's keys do nothing while it is open.
    press(&mut s, &mut c, &[Action::CursorDown, Action::Cancel]);
    assert!(s.typing().is_some());
    // Digits only, three at most.
    type_in(&mut s, &mut c, "3x7", &[]);
    assert_eq!(s.typing().and_then(NumberBox::value), Some(37));
    assert_eq!(sounds(&mut c), ["menu_cancel"], "the x is refused");
    assert_eq!(focused_value(&s, &c), boxed("37", true));
    // The other rows show their values as ever.
    let music = s.view(&c).row(Row::MusicVolume).map(|r| r.value.clone());
    assert_eq!(
        music,
        Some(ValueView::Volume {
            level: 80,
            max: 100
        })
    );
    type_in(&mut s, &mut c, "55", &[]);
    assert_eq!(s.typing().map(NumberBox::digits), Some("375"));
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
    assert_eq!(focused_value(&s, &c), boxed("80", false));
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
    // What each row shows, as the view says it: the words, or `<level>`
    // for a volume, and `*` if left and right change it.
    let value = |c: &Ctx, row| match OptionsScreen::value(c, row) {
        ValueView::None => String::new(),
        ValueView::Text { text, adjustable } => {
            format!("{text}{}", if adjustable { "*" } else { "" })
        }
        ValueView::Volume { level, max } => format!("<{level}/{max}>"),
        ValueView::NumberBox { .. } => "box".to_owned(),
    };
    let shown: Vec<String> = Row::ALL.iter().map(|&r| value(&c, r)).collect();
    assert_eq!(
        shown,
        [
            "Normal*",
            "Normal*",
            "On*",
            "Normal*",
            "Off*",
            "Off*",
            "Corners*",
            "<80/100>",
            "<80/100>",
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
    assert_eq!(value(&c, Row::Cursor), "Large corners*");
    assert_eq!(value(&c, Row::MusicVolume), "<0/100>");
    assert_eq!(value(&c, Row::SoundVolume), "<100/100>");
    assert_eq!(value(&c, Row::AutoEnd), "On*");
    assert_eq!(value(&c, Row::GameMode), "Classic");
    assert_eq!(value(&c, Row::Layout), "Left-handed");
    assert_eq!(c.text(cursor_key(CursorStyle::TileGlow)), "Tile glow");
    c.campaign_mode = Some(GameMode::Casual);
    assert_eq!(value(&c, Row::GameMode), "Casual");
}

/// The view is the whole screen as data: what a skin paints.
#[test]
fn the_view_says_what_the_screen_shows() {
    let mut c = ctx();
    let s = on(Row::Layout);
    let view = s.view(&c);
    assert_eq!(view.title, "Options");
    assert_eq!(view.help, s.help(&c));
    assert_eq!((view.message.clone(), view.question.clone()), (None, None));
    let rows: Vec<Row> = view.rows.iter().map(|r| r.row).collect();
    assert_eq!(rows, OptionsScreen::rows(&c));
    let labels: Vec<&str> = view.rows.iter().map(|r| r.label.as_str()).collect();
    assert_eq!(labels[0], "Text speed");
    assert_eq!(labels[rows.len() - 1], "Restore defaults");
    // One row in focus; the groups start at Layout and Reset tips.
    let focused: Vec<Row> = view
        .rows
        .iter()
        .filter(|r| r.focused)
        .map(|r| r.row)
        .collect();
    assert_eq!(focused, [Row::Layout]);
    assert_eq!(view.focused().map(|r| r.row), Some(Row::Layout));
    let groups: Vec<Row> = view
        .rows
        .iter()
        .filter(|r| r.starts_group)
        .map(|r| r.row)
        .collect();
    assert_eq!(groups, [Row::Layout, Row::ResetTips]);
    for row in &view.rows {
        assert_eq!(
            row.value,
            OptionsScreen::value(&c, row.row),
            "{:?}",
            row.row
        );
    }
    assert_eq!(view.row(Row::GameMode), None);
    assert_eq!(
        view.row(Row::Layout).and_then(|r| r.value.text()),
        Some("Right-handed")
    );
    assert_eq!(
        view.row(Row::MusicVolume).and_then(|r| r.value.text()),
        None
    );
    // A question and a message, in the player's words and keys.
    let mut s = on(Row::ResetTips);
    press(&mut s, &mut c, &[Action::Confirm]);
    let question = s.view(&c).question;
    assert_eq!(
        question,
        Some(QuestionView {
            text: "Show every tip again?".to_owned(),
            answers: "f yes / d no".to_owned(),
        })
    );
    press(&mut s, &mut c, &[Action::Confirm]);
    let view = s.view(&c);
    assert_eq!(view.question, None);
    assert_eq!(view.message.as_deref(), Some("Tips will show again"));
    // With a campaign, its mode is a row.
    c.campaign_mode = Some(GameMode::Classic);
    let mode = s.view(&c).row(Row::GameMode).map(|r| r.value.clone());
    assert_eq!(mode.as_ref().and_then(ValueView::text), Some("Classic"));
}

/// Drawing is the glyph skin painting the view, nothing more.
#[test]
fn draw_paints_the_view_with_the_glyph_skin() {
    use crate::console::{CONSOLE_H, CONSOLE_W};
    let mut c = ctx();
    let mut s = on(Row::SoundVolume);
    press(&mut s, &mut c, &[Action::CursorLeft]);
    let bg = c.palette.get(crate::color::UiColor::Black);
    let blank = crate::glyph_buffer::Cell::new(' ', bg, bg);
    let mut drawn = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, blank);
    s.draw(&c, &mut drawn);
    let mut painted = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, blank);
    glyph::paint(&c, &s.view(&c), &mut painted);
    assert_eq!(drawn, painted);
    assert!(s.as_any().is_some());
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
