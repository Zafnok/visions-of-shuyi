//! Scripted tests of the title screen through the real game (ADR-0007
//! layer 4), with the right-handed layout already picked
//! (`docs/design/controls.md`): arrows move, `f` selects, `d` backs out.

use insta::assert_snapshot;
use trpg_content::lang::TEST;
use trpg_content::{FontAtlasDef, LangCode};
use trpg_ui::audio::AudioRequest;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;

/// A launch after the right-handed layout was picked.
fn title() -> Harness {
    Harness::with_layout(Layout::RightHanded)
}

#[test]
fn title_renders() {
    let h = title();
    assert_eq!(h.top_screen(), "title");
    assert_snapshot!(h.snapshot());
}

#[test]
fn down_to_the_last_item_then_select_quits() {
    let mut h = title();
    h.keys("Down Down Down Down f");
    assert!(h.quit_requested());
}

#[test]
fn up_wraps_to_quit() {
    let mut h = title();
    h.keys("Up f");
    assert!(h.quit_requested());
}

#[test]
fn select_opens_new_game() {
    let mut h = title();
    h.keys("f");
    assert_eq!(h.top_screen(), "mode_select");
    assert_eq!(h.screens(), ["title", "mode_select"]);
    assert!(!h.quit_requested());
    assert_snapshot!(h.snapshot());
}

#[test]
fn back_returns_to_the_title() {
    let mut h = title();
    let title = h.snapshot();
    h.keys("f d");
    assert_eq!(h.top_screen(), "title");
    assert_eq!(h.snapshot(), title);
    // Escape also backs out.
    h.keys("f Escape");
    assert_eq!(h.top_screen(), "title");
}

#[test]
fn back_on_the_title_does_nothing() {
    let mut h = title();
    h.keys("d Escape");
    assert_eq!(h.top_screen(), "title");
    assert!(!h.quit_requested());
}

#[test]
fn holding_down_repeats_and_wraps() {
    // Held for 0.54 s: the press plus repeats at 300, 355, 410, 465 and
    // 520 ms makes six moves over five items (the harness has debug tools
    // on, so Quick Battle is there), wrapping round to Quick Battle.
    let mut h = title();
    h.hold("Down", 0.54).wait(0.5).keys("f");
    assert_eq!(h.top_screen(), "preparations");
}

#[test]
fn f2_opens_the_debug_menu() {
    let mut h = title();
    // F12 opens the browser's developer tools on the web; it does nothing here.
    h.keys("F12");
    assert_eq!(h.top_screen(), "title");
    h.keys("F2");
    assert_eq!(h.top_screen(), "debug_menu");
    h.keys("f");
    assert_eq!(h.top_screen(), "glyph_sampler");
    h.keys("d Down f");
    assert_eq!(h.top_screen(), "portrait_viewer");
    h.keys("d d");
    assert_eq!(h.top_screen(), "title");
}

#[test]
fn every_glyph_drawn_is_in_the_font() {
    let font = FontAtlasDef::load().unwrap_or_default();
    let mut h = title();
    for script in ["", "Down", "Up f"] {
        h.keys(script);
        let snap = h.snapshot();
        let glyphs = snap.split("\n--- colours ---").next().unwrap_or("");
        for g in glyphs.chars().filter(|&c| c != '\n') {
            assert!(font.glyph_rect(g).is_some(), "{g:?} missing from the font");
        }
    }
}

/// The music cues the run asked for, in order.
fn music(h: &Harness) -> Vec<String> {
    h.audio_requests()
        .into_iter()
        .filter_map(|r| match r {
            AudioRequest::PlayMusic { cue } => Some(cue),
            _ => None,
        })
        .collect()
}

#[test]
fn the_title_asks_for_its_music_once() {
    let mut h = title();
    h.wait(0.1);
    assert_eq!(
        h.audio_requests(),
        [AudioRequest::PlayMusic {
            cue: "title".into()
        }]
    );
    // Moving around the menu doesn't ask again.
    h.keys("Down Up");
    assert_eq!(music(&h), ["title"]);
    // Back from New Game it asks again (its battles change the music),
    // which doesn't restart a track that is still playing.
    h.clear_audio();
    h.keys("f d").wait(0.1);
    assert_eq!(music(&h), ["title"]);
    assert!(h.music_commands().is_empty(), "{:?}", h.music_commands());
}

/// Menu sounds (ticket 0425): the highlight moving, choosing, backing out;
/// nothing when a key does nothing.
#[test]
fn menu_sounds() {
    let mut h = title();
    h.keys("Down Up");
    assert_eq!(h.sounds(), ["menu_move", "menu_move"]);
    // Nothing to back out of on the title screen.
    h.clear_audio().keys("d");
    assert!(h.sounds().is_empty());
    h.keys("f");
    assert_eq!(h.top_screen(), "mode_select");
    assert_eq!(h.sounds(), ["menu_select"]);
    // New Game's first screen sounds like a menu too.
    h.clear_audio().keys("Down");
    assert_eq!(h.sounds(), ["menu_move"]);
    h.clear_audio().keys("d");
    assert_eq!(h.sounds(), ["menu_cancel"]);
    // Holding Down: one tick per step of the highlight.
    h.clear_audio().hold("Down", 0.43);
    assert_eq!(h.sounds(), ["menu_move"; 4]);
}

/// The debug tools sound like the other menus.
#[test]
fn debug_tool_sounds() {
    let mut h = title();
    h.keys("F2").clear_audio().keys("Down f Down Right d d");
    assert_eq!(
        h.sounds(),
        [
            "menu_move",
            "menu_select",
            "menu_move",
            "menu_move",
            "menu_cancel",
            "menu_cancel"
        ]
    );
    assert_eq!(h.top_screen(), "title");
    h.keys("F2 f").clear_audio().keys("d");
    assert_eq!(h.sounds(), ["menu_cancel"]);
}

/// The game's launch on every build, a layout picked before (tickets 0224
/// and 0226, `docs/design/title-screen.md`).
fn waiting_title() -> Harness {
    Harness::at_prompt_with_layout(Layout::RightHanded)
}

#[test]
fn the_title_waits_for_a_key_or_button() {
    let mut h = waiting_title();
    h.wait(0.5);
    let snap = h.snapshot();
    assert!(snap.contains("├─┴┬┴─┴┬┴─┤     Press any key or button     │ ┼ ╭─╮ ◯ │"));
    assert_snapshot!(snap);
    assert!(music(&h).is_empty());
    // Every glyph of the pictures is in the font.
    let font = FontAtlasDef::load().unwrap_or_default();
    let glyphs = snap.split("\n--- colours ---").next().unwrap_or("");
    for g in glyphs.chars().filter(|&c| c != '\n') {
        assert!(font.glyph_rect(g).is_some(), "{g:?} missing from the font");
    }
}

/// The pictures keep their gap whatever the line's length (another
/// language).
#[test]
fn the_pictures_sit_beside_the_line_in_the_test_language() {
    let mut h = waiting_title();
    h.ctx_mut().lang = LangCode::new(TEST).unwrap();
    h.wait(0.1);
    let snap = h.snapshot();
    assert!(snap.contains("├─┴┬┴─┴┬┴─┤     PRESS ANY KEY OR BUTTON     │ ┼ ╭─╮ ◯ │"));
}

#[test]
fn any_key_shows_the_menu_and_starts_the_music() {
    let mut h = waiting_title();
    // `q` is bound to nothing: it still counts.
    h.keys("q");
    assert_eq!(music(&h), ["title"]);
    assert_eq!(h.snapshot(), title().snapshot());
}

#[test]
fn the_key_that_ends_the_wait_does_nothing_else() {
    let mut h = waiting_title();
    // `f` selects, but here it only ends the wait: New Game stays focused
    // and nothing opens, and no menu sound plays.
    h.keys("f");
    assert_eq!(h.screens(), ["title"]);
    assert!(h.sounds().is_empty());
    assert_eq!(h.snapshot(), title().snapshot());
    h.keys("f");
    assert_eq!(h.top_screen(), "mode_select");
}

#[test]
fn back_on_the_title_shows_the_menu_not_the_prompt() {
    let mut h = waiting_title();
    h.keys("f f d");
    assert_eq!(h.top_screen(), "title");
    assert_eq!(h.snapshot(), title().snapshot());
    // Asked for again on the way back from New Game (0807), which the
    // track still playing ignores.
    assert_eq!(music(&h), ["title", "title"]);
    let starts = h
        .music_commands()
        .iter()
        .filter(|c| matches!(c, trpg_ui::audio::MusicCommand::Start { .. }));
    assert_eq!(starts.count(), 1, "{:?}", h.music_commands());
}

/// Whether the title is showing its prompt.
fn prompt_shows(h: &Harness) -> bool {
    let prompt = h.game().ctx().text("title.press_any_key");
    h.snapshot().contains(prompt)
}

/// First launch (`docs/design/controls.md`, *Pick your layout with a
/// controller*, rule 1): the prompt comes first, and a key there opens
/// "Pick your layout"; that key does nothing else.
#[test]
fn first_launch_a_key_at_the_prompt_opens_the_layout_picker() {
    let mut h = Harness::at_prompt();
    assert_eq!(h.screens(), ["title"]);
    assert!(prompt_shows(&h));
    // `s` moves the picker's highlight and `f` picks: here they only open it.
    for key in ["s", "f", "q"] {
        let mut h = Harness::at_prompt();
        h.keys(key);
        assert_eq!(h.screens(), ["title", "layout_picker"], "{key}");
        assert_eq!(h.game().ctx().layout(), None, "{key}");
        assert!(h.snapshot().contains("► Right-handed"), "{key}");
        assert!(music(&h).is_empty());
    }
    // Picked: the title's menu and music, no prompt again.
    h.keys("f Down f").wait(0.1);
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(h.game().ctx().layout(), Some(Layout::LeftHanded));
    assert!(!prompt_shows(&h));
    assert_eq!(music(&h), ["title"]);
    let h = Harness::with_storage(h.into_storage());
    assert_eq!(h.game().ctx().layout(), Some(Layout::LeftHanded));
}

/// Rule 1: a controller button at the prompt goes to the menu; no picker.
#[test]
fn first_launch_a_button_at_the_prompt_skips_the_layout_picker() {
    let mut h = Harness::at_prompt();
    h.pad("South").wait(0.1);
    assert_eq!(h.screens(), ["title"]);
    assert!(!prompt_shows(&h));
    assert_eq!(music(&h), ["title"]);
    assert_eq!(h.game().ctx().layout(), None);
    // The controller plays on: New Game.
    h.pad("South");
    assert_eq!(h.screens(), ["title", "mode_select"]);
}

/// Rule 2: a button while the picker is open closes it without choosing.
#[test]
fn a_button_closes_the_layout_picker_without_choosing() {
    let mut h = Harness::at_prompt();
    h.keys("f");
    assert_eq!(h.top_screen(), "layout_picker");
    // The bottom button would confirm: here it only closes the picker.
    h.pad("South").wait(0.1);
    assert_eq!(h.screens(), ["title"]);
    assert!(!prompt_shows(&h));
    assert_eq!(h.game().ctx().layout(), None);
    assert!(h.sounds().is_empty(), "{:?}", h.sounds());
    let h = Harness::with_storage(h.into_storage());
    assert_eq!(h.game().ctx().layout(), None, "no layout was saved");
}

/// Rules 3 and 4: with no layout picked, a key after playing on the
/// controller opens the picker straight away, over a battle too, and does
/// nothing else; once a layout is picked it never opens again.
#[test]
fn a_key_mid_battle_opens_the_layout_picker_until_one_is_picked() {
    let mut h = Harness::at_prompt();
    // The prompt, then Quick Battle, `Fight!` and the phase banner.
    h.pad("South DpadDown South DpadLeft South South");
    assert_eq!(h.screens(), ["title", "battle"]);
    h.pad("DpadRight");
    let cursor = h.cursor_tile();
    // `Down` is a picker key, so it is bound already: it must not move
    // the battle's cursor.
    h.keys("Down");
    assert_eq!(h.screens(), ["title", "battle", "layout_picker"]);
    assert!(h.snapshot().contains("► Right-handed"));
    // Back to the controller: the picker closes and the battle goes on.
    h.pad("DpadRight");
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(h.cursor_tile(), cursor);
    h.pad("DpadLeft DpadRight");
    assert_eq!(h.cursor_tile(), cursor);
    // The next key asks again; this time a layout is picked.
    h.keys("Down");
    assert_eq!(h.top_screen(), "layout_picker");
    h.keys("f");
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(h.game().ctx().layout(), Some(Layout::RightHanded));
    assert_eq!(h.cursor_tile(), cursor);
    // Keys act now, and switching back and forth never opens it again.
    h.keys("Down");
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_ne!(h.cursor_tile(), cursor);
    h.pad("DpadUp").keys("Down").pad("DpadUp").keys("q");
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(h.cursor_tile(), cursor);
}

/// `app` starts every build with the prompt up (the native `Ctx` too).
#[test]
fn a_later_launch_shows_the_prompt_then_the_menu() {
    let mut h = waiting_title();
    assert_eq!(h.game().ctx().key_prompt, trpg_ui::KeyPrompt::Waiting);
    assert!(prompt_shows(&h));
    h.keys("q");
    assert_eq!(h.screens(), ["title"], "a layout is picked: no picker");
    assert!(!prompt_shows(&h));
}

/// Screen text comes from the language in use (ticket 0233): the test pack
/// is English in capitals, with one stale entry (the subtitle) and one
/// missing (Credits), which stay English.
#[test]
fn the_title_in_the_test_language() {
    let mut h = title();
    h.ctx_mut().lang = LangCode::new(TEST).unwrap();
    // The title labels its menu again when it is back on top.
    h.keys("f d");
    let snap = h.snapshot();
    assert_snapshot!(snap);
    for text in [
        "VISIONS OF SHUYI",
        "NEW GAME",
        "LOAD GAME",
        "QUICK BATTLE",
        "QUIT",
        "arrows MOVE · f SELECT · d BACK",
        // Stale and missing: English.
        "an ASCII tactics game",
        "Credits",
    ] {
        assert!(snap.contains(text), "{text}");
    }
    for text in ["New Game", "AN ASCII", "CREDITS", "Visions"] {
        assert!(!snap.contains(text), "{text}");
    }
}

#[test]
fn the_prompt_in_the_test_language() {
    let mut h = waiting_title();
    h.ctx_mut().lang = LangCode::new(TEST).unwrap();
    h.wait(0.1);
    assert!(h.snapshot().contains("PRESS ANY KEY OR BUTTON"));
}
