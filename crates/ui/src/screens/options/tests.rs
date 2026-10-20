//! This game's rows on the engine's options screen: which show, what
//! each changes and shows, and what the rows that do something do. The
//! screen itself (the focus, the volume box, leaving) is tested in the
//! engine, on its toy game.

use super::*;
use crate::audio::AudioRequest;
use crate::input::{Action, LEFT_HANDED};
use crate::screen::tests::ctx;
use crate::screen::{FrameInput, ShuyiCtx};
use crate::settings::{SETTINGS_KEY, TextSpeed};
use crate::storage::{Storage, StorageError};
use crate::tips::{TIPS_SEEN_KEY, TipsSeen};
use shuyi_ui::Screen;

fn press(s: &mut OptionsScreen, c: &mut Ctx, actions: &[Action]) -> String {
    let input = FrameInput::new(actions.to_vec(), 0.0, vec![]);
    format!("{:?}", s.update(c, &input))
}

/// The screen focused on `row`.
fn on(row: Row) -> OptionsScreen {
    screen().focused_on(row.key())
}

/// The focused row.
fn focus(s: &OptionsScreen, c: &Ctx) -> Row {
    Row::from_key(s.focus(c)).unwrap()
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

/// Whether the settings in use aren't the ones saved: a change on the
/// screen is saved only when kept on leaving (ticket 0839).
fn unsaved(c: &Ctx) -> bool {
    let text = c.storage.read(SETTINGS_KEY).unwrap();
    text.map(|t| Settings::from_ron(&t).unwrap()).as_ref() != Some(c.settings())
}

#[test]
fn game_mode_and_tutorial_show_only_with_a_campaign() {
    let mut c = ctx();
    let rows = shown_rows(&c);
    assert_eq!(rows.len(), Row::ALL.len() - 4);
    assert!(!rows.contains(&Row::GameMode) && !rows.contains(&Row::Tutorial));
    c.game.campaign_mode = Some(GameMode::Casual);
    c.game.campaign_tutorial = Some(TutorialLevel::Full);
    let without_pictures: Vec<Row> = Row::ALL
        .into_iter()
        .filter(|&r| r != Row::MapLook && r != Row::MapZoom)
        .collect();
    assert_eq!(shown_rows(&c), without_pictures);
    // Map zoom shows under a skin that can zoom (a debug tool's), even
    // without the game's pictures; not under one that can't.
    let with_zoom: Vec<Row> = Row::ALL
        .into_iter()
        .filter(|&r| r != Row::MapLook)
        .collect();
    c.set_map_skin(crate::map_view::skin_named(&c.game.content, "sprite").unwrap());
    assert_eq!(shown_rows(&c), with_zoom);
    c.set_map_skin(crate::map_view::skin_named(&c.game.content, "sprite_units").unwrap());
    assert_eq!(shown_rows(&c), without_pictures);
    // Map look and Map zoom show when the game has pictures.
    let mut tileset = c.game.content.tilesets["test_units"].clone();
    tileset.id = crate::map_view::GAME_TILESET.to_owned();
    c.game.content.tilesets.insert(tileset.id.clone(), tileset);
    assert_eq!(shown_rows(&c), Row::ALL);
}

/// Ticket 1045: with no keyboard there is no layout to pick; on the
/// Playdate's skins the window, the colour theme and the map's look and
/// zoom go too, even with the game's pictures. The PC keeps every row.
#[test]
fn the_playdate_leaves_out_the_rows_that_mean_nothing_there() {
    let mut c = ctx();
    c.game.campaign_mode = Some(GameMode::Casual);
    c.game.campaign_tutorial = Some(TutorialLevel::Full);
    let mut tileset = c.game.content.tilesets["test_units"].clone();
    tileset.id = crate::map_view::GAME_TILESET.to_owned();
    c.game.content.tilesets.insert(tileset.id.clone(), tileset);
    assert_eq!(shown_rows(&c), Row::ALL);
    c.keyboard = Keyboard::Absent;
    let no_layout: Vec<Row> = Row::ALL.into_iter().filter(|&r| r != Row::Layout).collect();
    assert_eq!(shown_rows(&c), no_layout);
    c.use_playdate_skins();
    let playdate = [
        Row::TextSpeed,
        Row::AnimSpeed,
        Row::CombatAnimations,
        Row::EnemyPhaseSpeed,
        Row::AutoEnd,
        Row::Cursor,
        Row::MusicVolume,
        Row::SoundVolume,
        Row::KeyBindings,
        Row::GameMode,
        Row::Tutorial,
        Row::ResetTips,
        Row::RestoreDefaults,
    ];
    assert_eq!(shown_rows(&c), playdate);
    let view = screen().view(&c);
    let keys: Vec<_> = view.rows.iter().map(|r| r.key).collect();
    assert_eq!(keys, playdate.map(Row::key));
    // The Playdate's skins with a keyboard (a test's): its layout shows.
    c.keyboard = Keyboard::Present;
    assert!(shown_rows(&c).contains(&Row::Layout));
    assert!(!shown_rows(&c).contains(&Row::Theme));
}

/// Ticket 0444: while pictures paint the whole map the cursor never
/// glows, so the row doesn't offer `Tile glow`; a player who had picked it
/// is shown `Corners` and steps on from there. On glyphs it is back.
#[test]
fn the_cursor_row_offers_no_tile_glow_while_the_map_is_pictures() {
    use crate::map_view::{MapSkin, skin_named};

    let value = |c: &Ctx, row| match value(c, row) {
        ValueView::Text { text, adjustable } => {
            format!("{text}{}", if adjustable { "*" } else { "" })
        }
        other => panic!("{other:?}"),
    };
    let mut c = ctx();
    c.change_settings(|s| s.game.cursor_style = CursorStyle::TileGlow)
        .unwrap();
    assert_eq!(value(&c, Row::Cursor), "Tile glow*");
    c.set_map_skin(skin_named(&c.game.content, "sprite").unwrap());
    assert_eq!(value(&c, Row::Cursor), "Corners*");
    assert_eq!(c.settings().game.cursor_style, CursorStyle::TileGlow);
    let mut s = on(Row::Cursor);
    let styles: Vec<_> = (0..3)
        .map(|_| {
            press(&mut s, &mut c, &[Action::Confirm]);
            c.settings().game.cursor_style
        })
        .collect();
    let (corners, large) = (CursorStyle::Corners, CursorStyle::LargeCorners);
    assert_eq!(styles, [large, corners, large]);
    assert_eq!(value(&c, Row::Cursor), "Large corners*");
    // Left and Right stop at the ends.
    press(&mut s, &mut c, &[Action::CursorRight]);
    assert_eq!(c.settings().game.cursor_style, large);
    press(&mut s, &mut c, &[Action::CursorLeft, Action::CursorLeft]);
    assert_eq!(c.settings().game.cursor_style, corners);
    // Unit pictures on glyph terrain, and glyphs: the glow is offered.
    let glyphs: MapSkin = MapSkin::glyph();
    for skin in [skin_named(&c.game.content, "sprite_units").unwrap(), glyphs] {
        c.set_map_skin(skin);
        c.change_settings(|s| s.game.cursor_style = large).unwrap();
        press(&mut s, &mut c, &[Action::Confirm]);
        assert_eq!(c.settings().game.cursor_style, CursorStyle::TileGlow);
        assert_eq!(value(&c, Row::Cursor), "Tile glow*");
    }
}

#[test]
fn every_setting_row_changes_its_own_setting() {
    let mut c = ctx();
    // With the game's pictures, so Map look and Map zoom are rows too.
    let mut tileset = c.game.content.tilesets["test_units"].clone();
    tileset.id = crate::map_view::GAME_TILESET.to_owned();
    c.game.content.tilesets.insert(tileset.id.clone(), tileset);
    // The tutorial level is the campaign's, not a setting (below).
    let rows = Row::ALL
        .into_iter()
        .filter(|&r| r.is_setting() && r != Row::Tutorial);
    for row in rows {
        let before = c.settings().clone();
        // Right, or left for a setting already at its last value.
        press(&mut on(row), &mut c, &[Action::CursorRight]);
        if *c.settings() == before {
            press(&mut on(row), &mut c, &[Action::CursorLeft]);
        }
        let s = c.settings().clone();
        let changed = [
            (Row::TextSpeed, s.text_speed != before.text_speed),
            (Row::AnimSpeed, s.game.anim_speed != before.game.anim_speed),
            (
                Row::CombatAnimations,
                s.game.combat_animations != before.game.combat_animations,
            ),
            (
                Row::EnemyPhaseSpeed,
                s.game.enemy_phase_speed != before.game.enemy_phase_speed,
            ),
            (
                Row::AutoEnd,
                s.game.auto_end_turn != before.game.auto_end_turn,
            ),
            (Row::Fullscreen, s.fullscreen != before.fullscreen),
            (Row::Cursor, s.game.cursor_style != before.game.cursor_style),
            (Row::MapLook, s.game.map_look != before.game.map_look),
            (Row::MapZoom, s.game.map_zoom != before.game.map_zoom),
            (Row::Theme, s.theme != before.theme),
            (Row::MusicVolume, s.music_volume != before.music_volume),
            (Row::SoundVolume, s.sound_volume != before.sound_volume),
        ];
        let rows: Vec<Row> = changed.iter().filter(|c| c.1).map(|c| c.0).collect();
        assert_eq!(rows, [row]);
        assert!(unsaved(&c));
    }
}

#[test]
fn rows_show_their_values() {
    let mut c = ctx();
    // What each row shows, as the view says it: the words, or `<level>`
    // for a volume, and `*` if left and right change it.
    let value = |c: &Ctx, row| match value(c, row) {
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
            "Pictures*",
            "2×*",
            "Earthy painterly*",
            "<80/100>",
            "<80/100>",
            "Right-handed",
            "",
            "",
            "",
            "",
            ""
        ]
    );
    c.change_settings(|s| {
        s.game.cursor_style = CursorStyle::LargeCorners;
        s.music_volume = 0;
        s.sound_volume = 100;
        s.game.auto_end_turn = true;
        s.game.anim_speed = AnimSpeed::Fast;
        s.game.enemy_phase_speed = EnemyPhaseSpeed::Fast;
        s.game.map_look = MapLook::Glyphs;
        s.game.map_zoom = MapZoom::X1;
        s.theme = Theme::ALL[2];
    })
    .unwrap();
    c.game.campaign_mode = Some(GameMode::Classic);
    c.use_layout(LEFT_HANDED);
    assert_eq!(value(&c, Row::Cursor), "Large corners*");
    assert_eq!(value(&c, Row::MusicVolume), "<0/100>");
    assert_eq!(value(&c, Row::SoundVolume), "<100/100>");
    assert_eq!(value(&c, Row::AutoEnd), "On*");
    // Each named setting shows the value it has, not its default.
    let named = |key: &str| format!("{}*", c.text(key));
    assert_eq!(value(&c, Row::AnimSpeed), named(AnimSpeed::Fast.key()));
    let fast = EnemyPhaseSpeed::Fast.key();
    assert_eq!(value(&c, Row::EnemyPhaseSpeed), named(fast));
    assert_eq!(value(&c, Row::MapLook), named(MapLook::Glyphs.key()));
    assert_eq!(value(&c, Row::MapZoom), "1×*");
    assert_eq!(value(&c, Row::Theme), named(Theme::ALL[2].key()));
    assert_eq!(value(&c, Row::GameMode), "Classic");
    assert_eq!(value(&c, Row::Layout), "Left-handed");
    assert_eq!(c.text(cursor_key(CursorStyle::TileGlow)), "Tile glow");
    c.game.campaign_mode = Some(GameMode::Casual);
    assert_eq!(value(&c, Row::GameMode), "Casual");
    for (level, shown) in [
        (TutorialLevel::Full, "Full*"),
        (TutorialLevel::Short, "Short*"),
        (TutorialLevel::None, "None*"),
    ] {
        c.game.campaign_tutorial = Some(level);
        assert_eq!(value(&c, Row::Tutorial), shown);
    }
}

/// Ticket 0838: the campaign's tutorial level is a value row. It changes
/// at once, and like any row is kept or put back on leaving.
#[test]
fn the_tutorial_row_steps_the_level_and_leaving_keeps_or_discards_it() {
    let mut c = ctx();
    c.game.campaign_mode = Some(GameMode::Classic);
    c.game.campaign_tutorial = Some(TutorialLevel::Full);
    let mut s = on(Row::Tutorial);
    assert_eq!(s.help(&c), "arrows move · f change · d back");
    // Left stops at Full; right steps to Short, then None, and stops.
    press(&mut s, &mut c, &[Action::CursorLeft]);
    assert_eq!(c.game.campaign_tutorial, Some(TutorialLevel::Full));
    assert!(sounds(&mut c).is_empty());
    assert!(!s.changed(&c));
    press(&mut s, &mut c, &[Action::CursorRight]);
    assert_eq!(c.game.campaign_tutorial, Some(TutorialLevel::Short));
    assert_eq!(sounds(&mut c), ["menu_move"]);
    assert!(s.changed(&c));
    press(&mut s, &mut c, &[Action::CursorRight, Action::CursorRight]);
    assert_eq!(c.game.campaign_tutorial, Some(TutorialLevel::None));
    // Confirm goes round, like the other value rows.
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(c.game.campaign_tutorial, Some(TutorialLevel::Full));
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(c.game.campaign_tutorial, Some(TutorialLevel::Short));
    // No setting changed, but leaving still asks. Discard puts it back.
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(s.leaving(), Some(LeaveChoice::Back));
    press(&mut s, &mut c, &[Action::CursorLeft]);
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "Pop");
    assert_eq!(c.game.campaign_tutorial, Some(TutorialLevel::Full));
    // Keep keeps it.
    let mut s = on(Row::Tutorial);
    press(&mut s, &mut c, &[Action::CursorRight, Action::CursorRight]);
    press(&mut s, &mut c, &[Action::Cancel]);
    let keep = [Action::CursorLeft, Action::CursorLeft, Action::Confirm];
    assert_eq!(press(&mut s, &mut c, &keep), "Pop");
    assert_eq!(c.game.campaign_tutorial, Some(TutorialLevel::None));
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
    // Left and right do nothing on them (Game mode shows with a campaign).
    c.game.campaign_mode = Some(GameMode::Classic);
    let before = c.settings().clone();
    for row in [Row::Layout, Row::KeyBindings, Row::GameMode, Row::ResetTips] {
        let keys = [Action::CursorLeft, Action::CursorRight];
        assert_eq!(press(&mut on(row), &mut c, &keys), "None");
    }
    assert_eq!(*c.settings(), before);
    assert!(sounds(&mut c).is_empty());
}

#[test]
fn classic_switches_to_casual_after_a_confirm_and_never_back() {
    let mut c = ctx();
    c.game.campaign_mode = Some(GameMode::Classic);
    c.game.mode_switch = ModeSwitch::Open;
    let mut s = on(Row::GameMode);
    assert_eq!(s.help(&c), "arrows move · f switch to Casual · d back");
    press(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    // The key that asked isn't the answer.
    assert_eq!(s.asking(), Some(Question::SwitchToCasual.key()));
    assert_eq!(c.game.campaign_mode, Some(GameMode::Classic));
    assert_eq!(s.help(&c), "f yes · d no");
    // Nothing but the answers does anything.
    press(&mut s, &mut c, &[Action::CursorDown, Action::CursorRight]);
    assert_eq!((focus(&s, &c), s.asking().is_some()), (Row::GameMode, true));
    // No: still Classic.
    assert_eq!(press(&mut s, &mut c, &[Action::Cancel]), "None");
    assert_eq!((s.asking(), s.message()), (None, None));
    assert_eq!(c.game.campaign_mode, Some(GameMode::Classic));
    // Yes.
    press(&mut s, &mut c, &[Action::Confirm]);
    sounds(&mut c);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(c.game.campaign_mode, Some(GameMode::Casual));
    assert_eq!(s.message(), Some(CASUAL_MESSAGE));
    assert_eq!(sounds(&mut c), ["menu_select"]);
    // Casual offers nothing.
    assert_eq!(s.help(&c), "arrows move · d back");
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.asking(), None);
    assert_eq!(c.game.campaign_mode, Some(GameMode::Casual));
    assert_eq!(sounds(&mut c), ["menu_cancel"]);
}

/// Nick: not in the middle of a battle, only at Preparations.
#[test]
fn classic_cant_switch_where_the_switch_is_closed() {
    let mut c = ctx();
    c.game.campaign_mode = Some(GameMode::Classic);
    let mut s = on(Row::GameMode);
    assert_eq!(s.help(&c), "arrows move · d back");
    press(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    assert_eq!(s.asking(), None);
    assert_eq!(c.game.campaign_mode, Some(GameMode::Classic));
    assert_eq!(s.message(), Some(MODE_AT_PREP_MESSAGE));
    assert_eq!(sounds(&mut c), ["menu_cancel", "menu_cancel"]);
    // A Casual campaign gets no such message: there is nothing to switch.
    c.game.campaign_mode = Some(GameMode::Casual);
    c.game.mode_switch = ModeSwitch::Open;
    let mut s = on(Row::GameMode);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!((s.asking(), s.message()), (None, None));
}

#[test]
fn reset_tips_forgets_the_tips_seen() {
    let mut c = ctx();
    let mut seen = TipsSeen::default();
    seen.mark("first_move", &mut c.storage);
    assert!(TipsSeen::load(&c.storage).contains("first_move"));
    let mut s = on(Row::ResetTips);
    assert_eq!(s.help(&c), "arrows move · f reset · d back");
    // Nick: it asks first.
    press(&mut s, &mut c, &[Action::Confirm, Action::Confirm]);
    assert_eq!(s.asking(), Some(Question::ResetTips.key()));
    assert!(TipsSeen::load(&c.storage).contains("first_move"));
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!((s.asking(), s.message()), (None, None));
    assert!(TipsSeen::load(&c.storage).contains("first_move"));
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
    c.choose_layout(LEFT_HANDED).unwrap();
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
    assert_eq!(s.asking(), Some(Question::RestoreDefaults.key()));
    press(&mut s, &mut c, &[Action::Cancel]);
    assert_eq!(*c.settings(), changed);
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    let expected = Settings::default().with_layout(LEFT_HANDED);
    assert_eq!(*c.settings(), expected);
    // A change like any other: saved only if kept on leaving.
    assert_eq!(saved(&c), changed);
    assert_eq!(s.message(), Some(RESTORED_MESSAGE));
    assert_eq!(c.layout(), Some(LEFT_HANDED));
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
fn a_failed_save_keeps_the_change_says_so_and_stays() {
    let mut c = ctx().with_storage(Box::new(ReadOnly));
    let mut s = on(Row::ResetTips);
    press(&mut s, &mut c, &[Action::Confirm]);
    press(&mut s, &mut c, &[Action::Confirm]);
    assert_eq!(s.message(), Some(NOT_SAVED_MESSAGE));
    // A change isn't saved yet, so it says nothing.
    let mut s = screen();
    press(&mut s, &mut c, &[Action::CursorRight]);
    assert_eq!(c.settings().text_speed, TextSpeed::Fast);
    assert_eq!(s.message(), None);
    // Keep can't save: the change stays, the screen says so and stays.
    press(&mut s, &mut c, &[Action::Cancel]);
    press(&mut s, &mut c, &[Action::CursorLeft, Action::CursorLeft]);
    assert_eq!(s.leaving(), Some(LeaveChoice::Keep));
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "None");
    assert_eq!((s.leaving(), s.message()), (None, Some(NOT_SAVED_MESSAGE)));
    assert_eq!(c.settings().text_speed, TextSpeed::Fast);
}

#[test]
fn discard_puts_every_setting_back_and_saves_nothing() {
    let mut c = ctx();
    c.choose_layout(LEFT_HANDED).unwrap();
    let before = c.settings().clone();
    let stored = c.storage.read(SETTINGS_KEY);
    let mut s = screen();
    for row in shown_rows(&c) {
        if row.is_setting() {
            s = s.focused_on(row.key());
            press(&mut s, &mut c, &[Action::CursorLeft, Action::CursorRight]);
            press(&mut s, &mut c, &[Action::CursorRight]);
        }
    }
    assert!(s.changed(&c) && c.settings().fullscreen && c.settings().music_volume != 80);
    press(&mut s, &mut c, &[Action::Cancel]);
    press(&mut s, &mut c, &[Action::CursorLeft]);
    assert_eq!(s.leaving(), Some(LeaveChoice::Discard));
    assert_eq!(press(&mut s, &mut c, &[Action::Confirm]), "Pop");
    assert_eq!(*c.settings(), before);
    assert_eq!(c.storage.read(SETTINGS_KEY), stored);
    assert_eq!(
        sounds(&mut c).last().map(String::as_str),
        Some("menu_cancel")
    );
}

#[test]
fn help_names_what_confirm_does_on_each_kind_of_row() {
    let c = ctx();
    assert_eq!(screen().help(&c), "arrows move · f change · d back");
    assert_eq!(on(Row::Layout).help(&c), "arrows move · f open · d back");
    assert_eq!(
        on(Row::KeyBindings).help(&c),
        "arrows move · f open · d back"
    );
}
