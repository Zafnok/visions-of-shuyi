//! Scripted tests of the title screen through the real game (ADR-0007
//! layer 4), with the right-handed layout already picked
//! (`docs/design/controls.md`): arrows move, `f` selects, `d` backs out.
//!
//! What the screen says is read from its view (`TitleScreen::view`: plain
//! data), not from the glyphs a skin painted; the snapshots pin the glyph
//! look.

use insta::assert_snapshot;
use trpg_content::lang::TEST;
use trpg_content::{FontAtlasDef, LangCode};
use trpg_ui::audio::AudioRequest;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::screens::TitleScreen;
use trpg_ui::screens::title::TitleView;

/// A launch after the right-handed layout was picked.
fn title() -> Harness {
    Harness::with_layout(Layout::RightHanded)
}

/// The title screen on the stack, as it shows itself now.
fn view(h: &Harness) -> TitleView {
    let screen = h.game().screen::<TitleScreen>();
    screen
        .unwrap_or_else(|| panic!("no title screen"))
        .view(h.game().ctx())
}

/// The labels of the title's menu, and the focused one.
fn menu(h: &Harness) -> (Vec<String>, Option<String>) {
    let menu = view(h).menu.unwrap_or_else(|| panic!("no menu"));
    let labels = menu.items.iter().map(|i| i.label.clone()).collect();
    (labels, menu.focused().map(|i| i.label.clone()))
}

#[test]
fn title_renders() {
    let h = title();
    assert_eq!(h.top_screen(), "title");
    let (labels, focused) = menu(&h);
    // The harness has debug tools on, so Quick Battle is there.
    let want = [
        "New Game",
        "Load Game",
        "Quick Battle",
        "Options",
        "Credits",
        "Quit",
    ];
    assert_eq!(labels, want);
    assert_eq!(focused.as_deref(), Some("New Game"));
    assert_eq!(
        view(&h).help.as_deref(),
        Some("arrows move · f select · d back")
    );
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

/// The web build's launch (ticket 0224, `docs/design/title-screen.md`).
fn web_title() -> Harness {
    Harness::on_web_with_layout(Layout::RightHanded)
}

#[test]
fn web_title_waits_for_a_key() {
    let mut h = web_title();
    h.wait(0.5);
    let shown = view(&h);
    assert_eq!(shown.prompt.as_deref(), Some("Press any key"));
    assert!(shown.menu.is_none() && shown.help.is_none());
    assert_snapshot!(h.snapshot());
    assert!(music(&h).is_empty());
}

#[test]
fn any_key_shows_the_menu_and_starts_the_music() {
    let mut h = web_title();
    // `q` is bound to nothing: it still counts.
    h.keys("q");
    assert_eq!(music(&h), ["title"]);
    assert_eq!(view(&h), view(&title()));
    assert_eq!(h.snapshot(), title().snapshot());
}

#[test]
fn the_key_that_ends_the_wait_does_nothing_else() {
    let mut h = web_title();
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
fn back_on_the_web_title_shows_the_menu_not_the_prompt() {
    let mut h = web_title();
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

#[test]
fn picking_a_layout_first_skips_the_prompt() {
    // First launch on the web: the layout picker takes the first keys, so
    // the title shows its menu (and music) straight away after it.
    let mut h = Harness::on_web();
    assert_eq!(h.game().ctx().key_prompt, trpg_ui::KeyPrompt::Waiting);
    h.keys("Down f");
    assert_eq!(h.top_screen(), "title");
    h.wait(0.1);
    assert_eq!(view(&h).prompt, None);
    assert!(view(&h).menu.is_some());
    assert_eq!(music(&h), ["title"]);
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
    assert_snapshot!(h.snapshot());
    let shown = view(&h);
    assert_eq!(shown.title, "VISIONS OF SHUYI");
    // Stale and missing: English.
    assert_eq!(shown.subtitle, "an ASCII tactics game");
    assert_eq!(
        shown.help.as_deref(),
        Some("arrows MOVE · f SELECT · d BACK")
    );
    let (labels, _) = menu(&h);
    assert_eq!(
        labels,
        [
            "NEW GAME",
            "LOAD GAME",
            "QUICK BATTLE",
            "OPTIONS",
            "Credits",
            "QUIT"
        ]
    );
}

#[test]
fn the_web_prompt_in_the_test_language() {
    let mut h = web_title();
    h.ctx_mut().lang = LangCode::new(TEST).unwrap();
    h.wait(0.1);
    assert_eq!(view(&h).prompt.as_deref(), Some("PRESS ANY KEY"));
}
