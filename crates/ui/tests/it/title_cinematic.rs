//! The title's intro cinematic through the real game (ticket 0819;
//! `docs/design/title-screen.md`, *Intro cinematic*): left alone, the menu
//! fades out and `assets/cinematics/title.ron` plays in time with the
//! title song; a press skips to the logo and the menu.
//!
//! The song is 133.743 s long; its cinematic's last shot, the logo, starts
//! at 117 s. What shows is read from the title's view, never from cells.

use trpg_ui::audio::{AudioRequest, MusicCommand};
use trpg_ui::cinematic::Frame;
use trpg_ui::glyph_buffer::Rect;
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::screens::TitleScreen;
use trpg_ui::screens::title::TitleView;

const SCREEN: Rect = Rect::new(0, 0, 100, 32);
/// The title song's length, in seconds (`assets/audio/audio.ron`).
const SONG: f32 = 133.743;
/// Where the logo shot starts, in seconds.
const LOGO: f32 = 117.0;

/// A launch after the right-handed layout was picked, at the menu.
fn title() -> Harness {
    Harness::with_layout(Layout::RightHanded)
}

fn screen(h: &Harness) -> &TitleScreen {
    let screen = h.game().screen::<TitleScreen>();
    screen.unwrap_or_else(|| panic!("no title screen"))
}

fn view(h: &Harness) -> TitleView {
    screen(h).view(h.game().ctx())
}

/// Seconds into the cinematic, as the title has it.
fn time(h: &Harness) -> f32 {
    screen(h).cinematic_time()
}

/// How far into the title song the music is (not a number in silence).
fn song(h: &Harness) -> f32 {
    let clock = h.music_clock().filter(|c| c.cue == "title");
    clock.map_or(f32::NAN, |c| c.position)
}

/// Lets `seconds` pass, however many.
fn wait(h: &mut Harness, seconds: f32) {
    let mut left = seconds;
    while left > 0.0 {
        h.wait(left.min(30.0));
        left -= 30.0;
    }
}

/// Waits until the song is `position` seconds in (later in this loop).
fn wait_until(h: &mut Harness, position: f32) {
    if song(h).is_nan() {
        // Nothing has run yet: the first frame starts the song.
        h.wait(0.05);
    }
    let now = song(h);
    assert!(position >= now, "{position} is before {now}");
    wait(h, position - now);
}

/// What shows.
#[derive(Debug, PartialEq)]
enum Shows {
    /// The logo and the menu, this solid (255: fully).
    Menu(u8),
    /// The menu fading out over the cinematic.
    Fading,
    /// The cinematic alone, this many seconds in.
    Alone(f32),
}

fn shows(h: &Harness) -> Shows {
    let view = view(h);
    let Some(shot) = &view.cinematic else {
        assert!(view.menu.is_some() && view.help.is_some());
        return Shows::Menu(view.menu_opacity);
    };
    if shot.brightness < 255 {
        assert!(view.menu.is_some());
        assert_eq!(view.menu_opacity, 255 - shot.brightness);
        return Shows::Fading;
    }
    assert_eq!(
        (&view.menu, &view.help, &view.notice),
        (&None, &None, &None)
    );
    assert_eq!(view.prompt, None);
    #[allow(clippy::cast_precision_loss)]
    let t = shot.t_ms as f32 / 1000.0;
    Shows::Alone(t)
}

/// Asserts the cinematic is showing alone, within a frame or two of `t`
/// seconds.
fn assert_alone_at(h: &Harness, t: f32) {
    match shows(h) {
        Shows::Alone(at) if (at - t).abs() < 0.05 => {}
        other => panic!("not the cinematic at {t}: {other:?}"),
    }
}

/// The cinematic's frame now (nothing if it isn't showing).
fn frame(h: &Harness) -> Frame {
    let shot = view(h).cinematic;
    #[allow(clippy::cast_precision_loss)]
    let shown = shot.map(|s| {
        s.player
            .frame(h.game().ctx(), s.t_ms as f32 / 1000.0, SCREEN)
    });
    shown.unwrap_or(Frame::Blank)
}

/// The music cues the run asked for, in order.
fn music(h: &Harness) -> Vec<String> {
    let requests = h.audio_requests().into_iter();
    requests
        .filter_map(|r| match r {
            AudioRequest::PlayMusic { cue } => Some(cue),
            _ => None,
        })
        .collect()
}

/// The focused menu line.
fn focused(h: &Harness) -> Option<String> {
    let menu = view(h).menu?;
    menu.focused().map(|i| i.label.clone())
}

/// Acceptance: after the first press the menu shows; left alone, the
/// cinematic takes over 15 s in, where the song is; the logo has the
/// menu; and it loops with the song.
#[test]
fn left_alone_the_cinematic_joins_the_song_shows_the_logo_and_loops() {
    let mut h = Harness::at_prompt_with_layout(Layout::RightHanded);
    // Before the first press: the prompt, however long; no cinematic.
    h.wait(20.0);
    let waiting = view(&h);
    assert!(waiting.prompt.is_some() && waiting.cinematic.is_none());
    assert_eq!(waiting.menu_opacity, 255);
    h.keys("z");
    assert_eq!(shows(&h), Shows::Menu(255));
    wait_until(&mut h, 10.0);
    assert_eq!(shows(&h), Shows::Menu(255));
    wait_until(&mut h, 14.9);
    assert_eq!(shows(&h), Shows::Menu(255));
    // The menu fades for a second over the cinematic.
    wait_until(&mut h, 15.5);
    assert_eq!(shows(&h), Shows::Fading);
    let half = view(&h).cinematic.map_or(0, |c| c.brightness);
    assert!((100..156).contains(&half), "{half}");
    wait_until(&mut h, 17.0);
    assert_alone_at(&h, 17.0);
    // The stand-in's shot (ticket 0820 writes the real ones): a pan over
    // the Chapter 1 map at 3×, which has moved on by the next look.
    let Frame::MapPan { origin_px, zoom: 3 } = frame(&h) else {
        panic!("not the pan: {:?}", frame(&h));
    };
    assert!(h.snapshot().contains("zoom 3 ---"));
    wait_until(&mut h, 60.0);
    assert_alone_at(&h, 60.0);
    let Frame::MapPan {
        origin_px: later, ..
    } = frame(&h)
    else {
        panic!("not the pan: {:?}", frame(&h));
    };
    assert!(
        later.0 > origin_px.0 && later.1 < origin_px.1,
        "{origin_px:?} to {later:?}"
    );
    wait_until(&mut h, LOGO - 0.1);
    assert_alone_at(&h, LOGO - 0.1);
    // The logo: the menu fades in on it over half a second.
    wait_until(&mut h, LOGO + 0.25);
    match shows(&h) {
        Shows::Menu(solid) if (90..170).contains(&solid) => {}
        other => panic!("{other:?}"),
    }
    wait_until(&mut h, 120.0);
    assert_eq!(shows(&h), Shows::Menu(255));
    // It is the title's own screen: the same picture as at launch.
    assert_eq!(h.snapshot(), title().snapshot());
    wait_until(&mut h, SONG - 0.1);
    assert_eq!(shows(&h), Shows::Menu(255));
    // The song loops: the menu fades and the pan plays from its start.
    wait(&mut h, 0.6);
    assert_eq!(shows(&h), Shows::Fading);
    wait(&mut h, 1.0);
    assert_alone_at(&h, 1.5);
    // All by itself: one request for the song, and not a sound.
    assert_eq!(music(&h), ["title"]);
    assert!(h.sounds().is_empty(), "{:?}", h.sounds());
}

/// Acceptance: while the track loads the picture holds at its start, then
/// follows the music.
#[test]
fn the_picture_holds_at_its_start_until_the_music_sounds() {
    let mut h = title();
    h.music_load_delay(2.0);
    h.wait(1.9);
    assert!(song(&h).is_nan());
    assert!(time(&h).abs() < f32::EPSILON, "{}", time(&h));
    h.wait(0.6);
    assert!((song(&h) - 0.5).abs() < 0.05, "{}", song(&h));
    assert!((time(&h) - song(&h)).abs() < f32::EPSILON);
    // The 15 seconds are the player's, not the song's: the cinematic
    // shows 13 seconds into the song.
    wait(&mut h, 14.0);
    assert_alone_at(&h, song(&h));
    assert!((song(&h) - 14.5).abs() < 0.05, "{}", song(&h));
}

/// Acceptance: with no music at all the cinematic runs on the frame clock
/// after three seconds, and still loops at the track's length.
#[test]
fn without_music_the_cinematic_runs_anyway_and_loops() {
    let mut h = title();
    h.without_music();
    h.wait(2.9);
    assert!(time(&h).abs() < f32::EPSILON, "{}", time(&h));
    h.wait(1.1);
    assert!((time(&h) - 1.0).abs() < 0.05, "{}", time(&h));
    wait(&mut h, 16.0);
    assert_alone_at(&h, 17.0);
    wait(&mut h, 103.0);
    assert_eq!(shows(&h), Shows::Menu(255));
    assert!((time(&h) - 120.0).abs() < 0.2, "{}", time(&h));
    // 3 s held and the song's length: the start again.
    wait(&mut h, SONG - 120.0 + 2.0);
    assert_alone_at(&h, 2.0);
    assert_eq!(music(&h), ["title"]);
}

/// Claude's starting rule: any press while the menu is up restarts the 15
/// seconds, moving the cursor included.
#[test]
fn a_press_at_14_seconds_keeps_the_menu_another_15() {
    let mut h = title();
    wait_until(&mut h, 14.0);
    h.keys("Down");
    assert_eq!(focused(&h).as_deref(), Some("Quick Battle"));
    wait_until(&mut h, 28.9);
    assert_eq!(shows(&h), Shows::Menu(255));
    wait_until(&mut h, 30.5);
    assert_alone_at(&h, 30.5);
    // A key bound to nothing and a controller button count too.
    for pad in [false, true] {
        let mut h = title();
        wait_until(&mut h, 14.0);
        if pad {
            h.pad("North");
        } else {
            h.keys("z");
        }
        wait_until(&mut h, 28.9);
        assert_eq!(shows(&h), Shows::Menu(255), "{pad}");
        assert_eq!(focused(&h).as_deref(), Some("New Game"));
        wait_until(&mut h, 30.5);
        assert_alone_at(&h, 30.5);
    }
    // A press while the menu fades brings it back, and acts on it.
    let mut h = title();
    wait_until(&mut h, 15.5);
    assert_eq!(shows(&h), Shows::Fading);
    h.keys("Down");
    assert_eq!(shows(&h), Shows::Menu(255));
    assert_eq!(focused(&h).as_deref(), Some("Quick Battle"));
    assert_eq!(h.sounds(), ["menu_move"]);
}

/// Rule 4: a press during the cinematic plays the skip sound, stops the
/// song and shows the logo and menu; it does nothing else. The song
/// starts again when its silence would have ended.
#[test]
fn a_press_during_the_cinematic_skips_to_the_logo_and_the_silence() {
    for press in ["f", "Down", "z"] {
        let mut h = title();
        wait_until(&mut h, 40.0);
        assert_alone_at(&h, 40.0);
        h.clear_audio();
        h.keys(press);
        assert_eq!(h.sounds(), ["hit_sword"], "{press}");
        assert_eq!(
            h.audio_requests(),
            [
                AudioRequest::PlaySound {
                    cue: "hit_sword".into(),
                    volume: 1.0
                },
                AudioRequest::StopMusic
            ]
        );
        // The logo and the menu at once; the press chose and moved nothing.
        assert_eq!(shows(&h), Shows::Menu(255), "{press}");
        assert_eq!(h.screens(), ["title"], "{press}");
        assert_eq!(focused(&h).as_deref(), Some("New Game"), "{press}");
        assert!((time(&h) - LOGO).abs() < 0.05, "{}", time(&h));
        // The song fades out as any music does, and is gone.
        h.wait(1.0);
        assert!(song(&h).is_nan());
        assert!(h.music_commands().contains(&MusicCommand::Stop {
            cue: "title".into()
        }));
        // The silence lasts as long as the song's own: 16.7 s.
        h.clear_audio();
        wait(&mut h, SONG - LOGO - 1.2);
        assert_eq!(shows(&h), Shows::Menu(255));
        assert!(h.audio_requests().is_empty(), "{:?}", h.audio_requests());
        h.wait(0.3);
        assert_eq!(music(&h), ["title"]);
        assert!(h.music_commands().contains(&MusicCommand::Start {
            cue: "title".into()
        }));
        // Left alone, the menu fades and the cinematic plays from its
        // start, with the song.
        h.wait(2.0);
        assert_alone_at(&h, song(&h));
        assert!(song(&h) < 2.2, "{}", song(&h));
        assert_eq!(music(&h), ["title"], "asked once");
    }
}

/// Rule 4 again: the menu is one press away, and works at once.
#[test]
fn after_a_skip_the_menu_works_and_a_second_skip_is_another_press() {
    let mut h = title();
    wait_until(&mut h, 20.0);
    h.keys("f f");
    assert_eq!(h.screens(), ["title", "mode_select"]);
    // Back on the title its music is asked for again, from its start
    // (the skip had stopped it), and the 15 seconds start over.
    h.wait(1.0);
    h.keys("d");
    h.wait(0.5);
    assert!(song(&h) < 0.6, "{}", song(&h));
    assert!((time(&h) - song(&h)).abs() < f32::EPSILON);
    assert_eq!(shows(&h), Shows::Menu(255));
    wait_until(&mut h, 16.5);
    assert_alone_at(&h, 16.5);
    h.clear_audio().pad("South");
    assert_eq!(h.sounds(), ["hit_sword"]);
    assert_eq!(shows(&h), Shows::Menu(255));
    assert_eq!(h.screens(), ["title"]);
    // Out and back before the song has faded out (half a second, as any
    // music): asked for again it plays on (ADR-0026), and the picture
    // follows it instead of counting the silence out.
    h.keys("f d");
    assert!((song(&h) - 16.6).abs() < 0.2, "{}", song(&h));
    assert!((time(&h) - song(&h)).abs() < f32::EPSILON);
    assert_eq!(shows(&h), Shows::Menu(255));
    h.wait(15.0);
    assert_eq!(shows(&h), Shows::Fading);
}

/// Claude's starting rule: a press in the silence puts the menu on the
/// 15-second rule, so it stays past the loop.
#[test]
fn a_press_in_the_silence_keeps_the_menu_past_the_loop() {
    let mut h = title();
    wait_until(&mut h, 125.0);
    assert_eq!(shows(&h), Shows::Menu(255));
    h.keys("Down");
    // The song loops under the menu.
    wait(&mut h, 12.0);
    assert!(song(&h) < 4.0, "{}", song(&h));
    assert_eq!(shows(&h), Shows::Menu(255));
    wait(&mut h, 2.9);
    assert_eq!(shows(&h), Shows::Menu(255));
    // 15 seconds after the press the cinematic joins the song.
    wait(&mut h, 1.3);
    assert_alone_at(&h, song(&h));
    assert!(
        (song(&h) - (125.0 + 16.2 - SONG)).abs() < 0.2,
        "{}",
        song(&h)
    );

    // After a skip too.
    let mut h = title();
    wait_until(&mut h, 30.0);
    h.keys("z");
    h.wait(5.0);
    h.keys("Down");
    // The silence ends 16.7 s after the skip: the menu is still up, and
    // stays 15 s from the press.
    wait(&mut h, 14.5);
    assert_eq!(shows(&h), Shows::Menu(255));
    wait(&mut h, 1.7);
    assert_alone_at(&h, song(&h));

    // 15 seconds that run out on the logo wait for the loop: the logo
    // shot always has the menu.
    let mut h = title();
    wait_until(&mut h, 117.5);
    h.keys("Down");
    wait_until(&mut h, 133.0);
    assert_eq!(shows(&h), Shows::Menu(255));
    wait_until(&mut h, SONG - 0.1);
    assert_eq!(shows(&h), Shows::Menu(255));
    wait(&mut h, 1.6);
    assert_alone_at(&h, 1.5);
}

/// Claude's starting rule: the 15 seconds only run on the title itself.
#[test]
fn the_timer_does_not_run_under_load_game_or_options() {
    let mut h = title();
    let slot = trpg_ui::save::slot_key(3);
    if let Err(e) = h.ctx_mut().storage.write(&slot, "junk") {
        panic!("{e}");
    }
    // The title looks for saves when it is shown again.
    h.keys("f d");
    h.wait(10.0);
    assert_eq!(focused(&h).as_deref(), Some("New Game"));
    h.keys("Down f");
    assert_eq!(h.screens(), ["title", "save_slots"]);
    wait(&mut h, 40.0);
    assert_eq!(h.screens(), ["title", "save_slots"]);
    h.keys("d");
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(shows(&h), Shows::Menu(255));
    // 15 seconds from coming back, the song having played on.
    h.wait(14.5);
    assert_eq!(shows(&h), Shows::Menu(255));
    h.wait(1.7);
    assert_alone_at(&h, song(&h));
    assert!(song(&h) > 60.0, "{}", song(&h));

    let mut h = title();
    h.wait(10.0);
    // Load Game can't be chosen: Quick Battle, then Options.
    h.keys("Down Down f");
    assert_eq!(h.screens(), ["title", "options"]);
    wait(&mut h, 40.0);
    h.keys("d");
    assert_eq!(h.screens(), ["title"]);
    h.wait(14.5);
    assert_eq!(shows(&h), Shows::Menu(255));
    h.wait(1.7);
    assert_alone_at(&h, song(&h));
}

/// Rule 7: back from a battle the title shows its menu, the song starts
/// from its beginning, and the same rules run.
#[test]
fn back_from_a_battle_the_menu_shows_and_the_song_starts_over() {
    let mut h = title();
    wait_until(&mut h, 20.0);
    assert_alone_at(&h, 20.0);
    // Skip, then Quick Battle: `Fight!`, the banner.
    h.keys("z Down f Left f f");
    assert_eq!(h.screens(), ["title", "battle"]);
    wait(&mut h, 40.0);
    assert!(song(&h).is_nan(), "the battle's music");
    // The map menu's Suspend, confirmed: the title.
    h.clear_audio();
    h.keys("d Down Down Down f f");
    assert_eq!(h.screens(), ["title"]);
    assert_eq!(shows(&h), Shows::Menu(255));
    assert_eq!(music(&h), ["title"]);
    // The battle's music fades for half a second; the picture waits at
    // its start, then follows the song from 0.
    h.wait(0.3);
    assert!(time(&h).abs() < f32::EPSILON, "{}", time(&h));
    h.wait(1.2);
    assert!((0.5..1.5).contains(&song(&h)), "{}", song(&h));
    assert!((time(&h) - song(&h)).abs() < f32::EPSILON);
    wait_until(&mut h, 13.0);
    assert_eq!(shows(&h), Shows::Menu(255));
    wait_until(&mut h, 16.0);
    assert_alone_at(&h, 16.0);
}
