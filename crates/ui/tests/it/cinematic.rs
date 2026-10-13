//! The "Play test cinematic" debug tool through the Harness (tickets 0817,
//! 0818 and 0827, ADR-0060, ADR-0061): `assets-test/cinematics/test.ron`
//! plays on the screen's own clock (a pan over the test battle's map for
//! six seconds, in which the lead walks up to the brigand and fells it,
//! two text boxes of the test scene for four, three faces crossing for
//! six, the logo for four, each after a hard cut), loops, and closes on
//! Cancel.

use insta::assert_snapshot;
use trpg_content::new_campaign;
use trpg_core::{GameMode, LeadGender, LeadProfile, Pos, SaveFile, UnitId};
use trpg_ui::cinematic::{CinematicPlayer, Frame};
use trpg_ui::color::UiColor;
use trpg_ui::console::{CONSOLE_H, CONSOLE_W};
use trpg_ui::debug::CinematicScreen;
use trpg_ui::glyph_buffer::{Cell, GlyphBuffer, Rect};
use trpg_ui::harness::Harness;
use trpg_ui::input::Layout;
use trpg_ui::map_view::{Facing, MapScene, UnitView};
use trpg_ui::save;

const SCREEN: Rect = Rect::new(0, 0, 100, 32);

/// The title with the test cinematic opened from the debug menu (its last
/// tool but two: the scene camera and the map skin come after it).
fn opened() -> Harness {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.keys("F2 Up Up Up f");
    assert_eq!(h.screens(), ["title", "debug_menu", "cinematic"]);
    h
}

/// Seconds into the cinematic (not a number if it isn't on screen).
fn time(h: &Harness) -> f32 {
    let screen = h.game().screen::<CinematicScreen>();
    screen.map_or(f32::NAN, CinematicScreen::time)
}

/// What the cinematic shows now (nothing if it isn't on screen).
fn frame(h: &Harness) -> Frame {
    let screen = h.game().screen::<CinematicScreen>();
    let shown = screen.and_then(|s| Some(s.player()?.frame(h.game().ctx(), s.time(), SCREEN)));
    shown.unwrap_or(Frame::Blank)
}

/// The map scene the cinematic shows now, if it shows a map.
fn scene(h: &Harness) -> Option<MapScene> {
    let screen = h.game().screen::<CinematicScreen>()?;
    screen.player()?.scene(screen.time())
}

/// The unit `id` as the cinematic shows it now, if it is there.
fn unit(h: &Harness, id: UnitId) -> Option<UnitView> {
    scene(h)?.unit(id).cloned()
}

const LEAD: UnitId = UnitId(1);
const BRIGAND: UnitId = UnitId(3);

/// The left edge of the map picture on screen, in its own pixels.
fn pan_x(h: &Harness) -> f32 {
    match frame(h) {
        Frame::MapPan { origin_px, zoom: 4 } => origin_px.0,
        other => panic!("not the pan: {other:?}"),
    }
}

#[test]
fn the_test_cinematic_plays_cuts_to_the_logo_and_loops() {
    let mut h = opened();
    // The start of the pan: the picture's left edge.
    assert!(time(&h) < 0.05, "{}", time(&h));
    assert!(pan_x(&h) < 0.01);
    assert_snapshot!("cinematic_pan_start", h.snapshot());
    // Everyone stands where the battle starts.
    let (lead, brigand) = (unit(&h, LEAD).unwrap(), unit(&h, BRIGAND).unwrap());
    assert_eq!((lead.pos, brigand.pos), (Pos::new(3, 5), Pos::new(8, 3)));
    let full = brigand.hp;
    assert_eq!(full.0, full.1);
    // Mid-walk: the lead has turned the corner and heads right, between
    // two tiles of row 3.
    h.wait(2.25 - time(&h));
    assert!((time(&h) - 2.25).abs() < 0.02, "{}", time(&h));
    let lead = unit(&h, LEAD).unwrap();
    assert_eq!((lead.pos, lead.facing), (Pos::new(4, 3), Facing::Right));
    assert!(
        lead.offset.0 > 0.3 && lead.offset.0 < 0.7,
        "{:?}",
        lead.offset
    );
    assert_eq!(unit(&h, BRIGAND).unwrap(), brigand);
    assert_snapshot!("cinematic_mid_walk", h.snapshot());
    // The pan's middle: half of the 24 pixels there are to move.
    h.wait(3.0 - time(&h));
    assert!((time(&h) - 3.0).abs() < 0.02, "{}", time(&h));
    assert!((pan_x(&h) - 12.0).abs() < 0.2, "{}", pan_x(&h));
    let middle = h.snapshot();
    assert!(middle.contains("--- backdrop: clip 0,0 100x32  origin 12."));
    assert!(middle.contains("zoom 4 ---"));
    // Mid-strike: the lead, arrived beside the brigand, leans at it, and
    // the brigand flashes with less HP.
    h.wait(4.3 - time(&h));
    let (lead, struck) = (unit(&h, LEAD).unwrap(), unit(&h, BRIGAND).unwrap());
    assert_eq!((lead.pos, lead.facing), (Pos::new(7, 3), Facing::Right));
    assert!(
        lead.offset.0 > 0.0 && lead.offset.1 == 0.0,
        "{:?}",
        lead.offset
    );
    assert!(struck.hp.0 > 0 && struck.hp.0 < full.0, "{:?}", struck.hp);
    assert!(struck.flash > 0.2 && struck.flash < 0.8, "{}", struck.flash);
    assert_eq!((struck.pos, struck.hp.1), (brigand.pos, full.1));
    assert_snapshot!("cinematic_mid_strike", h.snapshot());
    // Just before the cut the pan is nearly at its end, and the brigand
    // has fallen: the others are still there.
    h.wait(5.9 - time(&h));
    assert!(pan_x(&h) > 23.9, "{}", pan_x(&h));
    assert_eq!(unit(&h, BRIGAND), None);
    assert_eq!(scene(&h).unwrap().units.len(), 2);
    assert_eq!(unit(&h, LEAD).unwrap().pos, Pos::new(7, 3));
    // A hard cut to the dialogue screen's look, no map behind it: the
    // first box's text typing out, the lord speaking on the left.
    h.wait(6.2 - time(&h));
    assert_eq!(scene(&h), None);
    let Frame::Talk { index: 0, shown } = frame(&h) else {
        panic!("not the first box: {:?}", frame(&h));
    };
    assert!((10..=14).contains(&shown), "{shown}");
    let typing = h.snapshot();
    assert!(!typing.contains("--- backdrop"));
    assert!(typing.contains("It's always") && !typing.contains("about you."));
    assert_snapshot!("cinematic_talk_typing", typing);
    // Typed out, it holds.
    h.wait(7.5 - time(&h));
    assert_eq!(
        frame(&h),
        Frame::Talk {
            index: 0,
            shown: 22
        }
    );
    let held = h.snapshot();
    assert!(held.contains("It's always about you."));
    // Nothing waits for a key: no key name, no ▼.
    assert!(!held.contains('▼'));
    assert_snapshot!("cinematic_talk_held", held);
    // The second box, to the lead a game without a save has.
    h.wait(9.5 - time(&h));
    assert_eq!(
        frame(&h),
        Frame::Talk {
            index: 1,
            shown: 29
        }
    );
    assert!(h.snapshot().contains("Ellery! You came back for us."));
    // The faces: the lead reaches the middle of the screen while the lord
    // is half in behind.
    h.wait(11.0 - time(&h));
    for _ in 0..600 {
        if matches!(frame(&h), Frame::Characters { x, .. } if x <= 33) {
            break;
        }
        h.wait(1.0 / 240.0);
    }
    assert_eq!(frame(&h), Frame::Characters { x: 33, step: 50 });
    let faces = h.snapshot();
    assert!(faces.contains("Ellery") && faces.contains("Test "));
    assert!(!faces.contains("Knight"));
    assert_snapshot!("cinematic_faces", faces);
    // The logo: a hard cut again.
    h.wait(16.5 - time(&h));
    assert_eq!(frame(&h), Frame::Logo { sparks: None });
    assert_eq!(scene(&h), None);
    let logo = h.snapshot();
    assert!(!logo.contains("--- backdrop"));
    assert_snapshot!("cinematic_logo", logo);
    // Twenty seconds in, it starts again.
    h.wait(4.0);
    assert!(time(&h) < 1.0, "{}", time(&h));
    assert!(pan_x(&h) < 1.0);
    // The brigand is back where it started, unhurt.
    assert_eq!(unit(&h, BRIGAND), Some(brigand));
    // Nothing but Cancel does anything, and nothing makes a sound.
    h.clear_audio();
    h.keys("f Up Down Left Right");
    assert_eq!(h.top_screen(), "cinematic");
    assert!(h.sounds().is_empty(), "{:?}", h.sounds());
}

#[test]
fn cancel_closes_it() {
    let mut h = opened();
    h.wait(7.0);
    h.clear_audio();
    h.keys("d");
    assert_eq!(h.screens(), ["title", "debug_menu"]);
    assert_eq!(h.sounds(), ["menu_cancel"]);
    // Opened again, it starts from the beginning.
    h.keys("f");
    assert_eq!(h.top_screen(), "cinematic");
    assert!(time(&h) < 0.05);
    assert!(matches!(frame(&h), Frame::MapPan { .. }));
}

#[test]
fn the_debug_key_does_nothing_on_it() {
    let mut h = opened();
    h.keys("F2");
    assert_eq!(h.screens(), ["title", "debug_menu", "cinematic"]);
}

#[test]
fn it_follows_the_map_skin_in_use() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    h.with_map_skin("sprite");
    h.keys("F2 Up Up Up f");
    let scene = h.game().buffer().backdrop().unwrap().scene();
    // The test tileset: 14 × 8 tiles of 24 px, in whole cells.
    assert_eq!((scene.width(), scene.height()), (42, 12));
    assert!(!scene.sprites().is_empty());
    // Its units are sprites: a walking one is drawn somewhere else each
    // moment, the ground the same.
    let sprites = |h: &Harness| h.game().buffer().backdrop().unwrap().scene().sprites();
    let standing = sprites(&h);
    h.wait(2.25);
    let walking = sprites(&h);
    assert_eq!(standing.len(), walking.len());
    assert_ne!(standing, walking);
    // Felled, the brigand's sprites are gone.
    h.wait(3.6);
    assert!(sprites(&h).len() < standing.len());
}

/// The frame of `player` at `t`, drawn as the debug tool draws it.
fn drawn(h: &Harness, player: &CinematicPlayer, t: f32) -> GlyphBuffer {
    let ctx = h.game().ctx();
    let p = &ctx.palette;
    let blank = Cell::new(' ', p.get(UiColor::Text), p.get(UiColor::Black));
    let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, blank);
    player.draw(ctx, t, 1.0, &mut buf);
    buf
}

#[test]
fn the_lead_is_the_most_recent_saves_and_the_default_without_one() {
    let mut h = Harness::with_layout(Layout::RightHanded);
    let face = |h: &Harness, id: &str| {
        let portrait = &h.game().ctx().content.portraits[id];
        portrait.expression("neutral").unwrap().image
    };
    // What the second text box (9.5 s) and the faces with the lead in the
    // middle (11.73 s) show: the text, and the portraits drawn.
    let shown = |h: &Harness| {
        let ctx = h.game().ctx();
        let lead = save::latest_lead(ctx.storage.as_ref());
        let player = CinematicPlayer::new(ctx, "test", lead).unwrap();
        let (talk, faces) = (drawn(h, &player, 9.5), drawn(h, &player, 11.73));
        let images =
            |buf: &GlyphBuffer| -> Vec<_> { buf.sprites().iter().map(|s| s.image).collect() };
        let text = [
            talk.to_snapshot(&ctx.palette),
            faces.to_snapshot(&ctx.palette),
        ];
        (text, images(&talk), images(&faces))
    };
    // No save: the default name and the male look.
    let ([talk, faces], in_talk, in_faces) = shown(&h);
    assert!(talk.contains("Ellery! You came back for us."));
    assert!(faces.contains("Ellery"));
    assert_eq!(in_talk, [face(&h, "lead_m")]);
    assert_eq!(in_faces, [face(&h, "lead_m"), face(&h, "test_lord")]);
    // A save whose lead was renamed and has the other look.
    let lead = LeadProfile::new("Mara", LeadGender::Female);
    let campaign = new_campaign(&h.game().ctx().content, GameMode::Classic, lead);
    let file = SaveFile::chapter_cleared(campaign);
    save::write(h.ctx_mut().storage.as_mut(), &save::slot_key(4), &file).unwrap();
    let ([talk, faces], in_talk, in_faces) = shown(&h);
    assert!(talk.contains("Mara! You came back for us."));
    assert!(faces.contains("Mara") && !faces.contains("Ellery"));
    assert!(!talk.contains("Ellery"));
    assert_eq!(in_talk, [face(&h, "lead_f")]);
    assert_eq!(in_faces, [face(&h, "lead_f"), face(&h, "test_lord")]);
    // The debug tool plays it with the default lead whatever is saved.
    h.keys("F2 Up Up Up f");
    h.wait(9.5);
    assert!(h.snapshot().contains("Ellery! You came back for us."));
}
