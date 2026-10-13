//! Tests of the talk and characters shots (ticket 0818), on the test
//! cinematic (`assets-test/cinematics/test.ron`): from 6 s, boxes 6 and 7
//! of the test scene for two seconds each ("It's always about you.",
//! "{lead}! You came back for us."); from 10 s, the lead, the test lord
//! and the test knight crossing from the right for six seconds.

use proptest::prelude::*;
use trpg_content::{LangCode, Present, Side};
use trpg_core::CharacterId;

use super::faces::{FACE_SPACING, faces, first_x};
use super::talk::{HOLD_SHARE, boxes, talk_at};
use super::*;
use crate::console::{CELL_W_PX, CONSOLE_H, CONSOLE_W};
use crate::input::Action;
use crate::screen::tests::ctx;
use crate::screen::{FrameInput, Screen};
use crate::screens::dialogue::{DialogueScreen, FRAME};
use crate::settings::TextSpeed;

const SCREEN: Rect = Rect::new(0, 0, 100, 32);

fn player(ctx: &Ctx) -> CinematicPlayer {
    CinematicPlayer::new(ctx, "test", None).unwrap()
}

fn mara() -> LeadProfile {
    LeadProfile::new("Mara", LeadGender::Female)
}

/// `player`'s frame at `t`, over a stale buffer.
fn draw(ctx: &Ctx, player: &CinematicPlayer, t: f32) -> GlyphBuffer {
    let p = &ctx.palette;
    let stale = Cell::new('x', p.get(UiColor::Enemy), p.get(UiColor::PanelBg));
    let mut buf = GlyphBuffer::new(CONSOLE_W, CONSOLE_H, stale);
    player.draw(ctx, t, 1.0, &mut buf);
    buf
}

/// Row `y` of `buf` as text.
fn row(buf: &GlyphBuffer, y: i32) -> String {
    (0..i32::from(buf.width()))
        .map(|x| buf.get(x, y).unwrap().glyph)
        .collect()
}

/// The first text row of `buf`'s text box, without the box's border.
fn text_row(buf: &GlyphBuffer) -> String {
    let row = row(buf, 23);
    row.trim_matches(|c: char| c == '│' || c == ' ').to_owned()
}

fn with_speed(speed: TextSpeed) -> Ctx {
    let mut ctx = ctx();
    ctx.change_settings(|s| s.text_speed = speed).unwrap();
    ctx
}

#[test]
fn each_box_types_out_then_holds_for_its_share_of_the_shot() {
    // Two boxes over four seconds at 60 characters a second: two seconds
    // each, of which the last third always shows the whole text.
    let at = |seconds: f32| talk_at(seconds / 4.0, 4.0, &[22, 31], Some(60.0));
    assert_eq!(at(0.0), Some((0, 0)));
    assert_eq!(at(0.01), Some((0, 0)));
    assert_eq!(at(0.11), Some((0, 6)));
    assert_eq!(at(0.21), Some((0, 12)));
    assert_eq!(at(0.36), Some((0, 21)));
    // Typed out after 22 / 60 s; held until the box changes.
    assert_eq!(at(0.38), Some((0, 22)));
    assert_eq!(at(1.0), Some((0, 22)));
    assert_eq!(at(1.99), Some((0, 22)));
    assert_eq!(at(2.0), Some((1, 0)));
    assert_eq!(at(2.26), Some((1, 15)));
    assert_eq!(at(2.51), Some((1, 30)));
    assert_eq!(at(2.53), Some((1, 31)));
    assert_eq!(at(3.99), Some((1, 31)));
    // The very end is still the last box.
    assert_eq!(at(4.0), Some((1, 31)));
    // Slower text types slower.
    let slow = |seconds: f32| talk_at(seconds / 4.0, 4.0, &[22, 31], Some(30.0));
    assert_eq!(slow(0.21), Some((0, 6)));
    assert_eq!(slow(0.72), Some((0, 21)));
    assert_eq!(slow(0.75), Some((0, 22)));
    // Instant text is all there from the box's first moment.
    let instant = |seconds: f32| talk_at(seconds / 4.0, 4.0, &[22, 31], None);
    assert_eq!(instant(0.0), Some((0, 22)));
    assert_eq!(instant(2.0), Some((1, 31)));
}

#[test]
fn a_long_box_types_faster_to_be_whole_for_the_last_third() {
    assert!((HOLD_SHARE - 1.0 / 3.0).abs() < f32::EPSILON);
    // One box of 200 characters in three seconds: at 60 a second it would
    // take 3.3 s, so it types in the first two (100 a second).
    let at = |seconds: f32| talk_at(seconds / 3.0, 3.0, &[200], Some(60.0));
    assert_eq!(at(0.0), Some((0, 0)));
    assert_eq!(at(0.505), Some((0, 50)));
    assert_eq!(at(1.005), Some((0, 100)));
    assert_eq!(at(1.985), Some((0, 198)));
    assert_eq!(at(2.0), Some((0, 200)));
    assert_eq!(at(2.5), Some((0, 200)));
    // A box that types in time anyway keeps the player's speed: 90
    // characters at 60 a second, not at the 45 that would fill two seconds.
    let short = |seconds: f32| talk_at(seconds / 3.0, 3.0, &[90], Some(60.0));
    assert_eq!(short(1.005), Some((0, 60)));
    assert_eq!(short(1.6), Some((0, 90)));
}

#[test]
fn talk_timing_never_divides_by_nothing() {
    assert_eq!(talk_at(0.5, 4.0, &[], Some(60.0)), None);
    assert_eq!(talk_at(0.5, 4.0, &[], None), None);
    // A progress that isn't one counts as the start; one past an end, as
    // that end.
    assert_eq!(talk_at(f32::NAN, 4.0, &[9, 9], Some(60.0)), Some((0, 0)));
    assert_eq!(talk_at(-1.0, 4.0, &[9, 9], Some(60.0)), Some((0, 0)));
    assert_eq!(talk_at(7.0, 4.0, &[9, 9], Some(60.0)), Some((1, 9)));
    // A shot of no time, or of a time that isn't one: the text is whole.
    for shot_s in [0.0, -3.0, f32::NAN] {
        assert_eq!(talk_at(0.0, shot_s, &[9], Some(60.0)), Some((0, 9)));
    }
    // A speed that isn't one types at the pace that fills the time.
    for speed in [0.0, -5.0, f32::NAN] {
        let at = talk_at(0.25, 4.0, &[80], Some(speed));
        assert_eq!(at, Some((0, 30)), "{speed}");
    }
    // An empty box is whole at once.
    assert_eq!(talk_at(0.0, 4.0, &[0, 5], Some(60.0)), Some((0, 0)));
}

proptest! {
    #[test]
    fn typed_text_never_passes_its_box_and_never_shrinks_within_one(
        lens in prop::collection::vec(0usize..300, 1..6),
        shot_s in 0.1f32..60.0,
        speed in prop::option::of(1f32..500.0),
        a in 0f32..=1.0,
        b in 0f32..=1.0,
    ) {
        let (early, late) = if a <= b { (a, b) } else { (b, a) };
        let (i, shown) = talk_at(early, shot_s, &lens, speed).unwrap();
        let (j, later) = talk_at(late, shot_s, &lens, speed).unwrap();
        prop_assert!(i < lens.len() && j < lens.len());
        prop_assert!(shown <= lens[i] && later <= lens[j]);
        // Boxes come in order; within one, the text only grows.
        prop_assert!(i <= j);
        if i == j {
            prop_assert!(shown <= later);
        }
        // The last third of a box's share shows it whole.
        #[allow(clippy::cast_precision_loss)]
        let held = (i as f32 + 1.0 - HOLD_SHARE * 0.99) / lens.len() as f32;
        if held < 1.0 {
            let (k, whole) = talk_at(held, shot_s, &lens, speed).unwrap();
            prop_assert_eq!((k, whole), (i, lens[i]));
        }
    }

    #[test]
    fn faces_cross_one_way_at_one_spacing(
        count in 0usize..9,
        a in 0f32..=1.0,
        b in 0f32..=1.0,
        width in 20i32..200,
    ) {
        let (early, late) = if a <= b { (a, b) } else { (b, a) };
        let (x0, step) = first_x(early, count, Side::Right, width);
        let (x1, _) = first_x(late, count, Side::Right, width);
        prop_assert_eq!(step, FACE_SPACING);
        prop_assert!(x1 <= x0 && x0 <= width);
        // From the left it is the mirror picture.
        let (m0, back) = first_x(early, count, Side::Left, width);
        prop_assert_eq!(back, -FACE_SPACING);
        prop_assert_eq!(m0 + FRAME.0, width - x0);
    }
}

#[test]
fn the_boxes_of_a_talk_shot_are_the_scenes_as_the_lead_reads_them() {
    let ctx = ctx();
    let scene = &ctx.content.dialogue.scenes["test"];
    let got = boxes(&ctx, scene, &mara(), 6, 2);
    let lines: Vec<_> = got.iter().map(|b| b.lines().to_vec()).collect();
    assert_eq!(
        lines,
        [["It's always about you."], ["Mara! You came back for us."]]
    );
    assert_eq!((got[0].len(), got[1].len()), (22, 27));
    assert!(!got[0].is_empty());
    // The first boxes: narration under the caption, before anyone is up.
    let opening = boxes(&ctx, scene, &mara(), 0, 2);
    assert_eq!(
        opening[0].lines(),
        ["The rain had not stopped for three days."]
    );
    assert_eq!(opening[1].lines(), ["You're late."]);
    // It stops at the reply choice, and at the scene's end.
    assert_eq!(boxes(&ctx, scene, &mara(), 6, 9).len(), 2);
    assert_eq!(boxes(&ctx, scene, &mara(), 8, 3), []);
    assert_eq!(boxes(&ctx, scene, &mara(), usize::MAX, 3), []);
    assert_eq!(boxes(&ctx, scene, &mara(), 3, 0), []);
    let intro = &ctx.content.dialogue.scenes["test_intro"];
    let all = boxes(&ctx, intro, &mara(), 0, 99);
    assert!(!all.is_empty() && all.len() < 99);
    // In the player's language.
    let mut other = crate::screen::tests::ctx();
    other.lang = LangCode::new("test").unwrap();
    let told = boxes(&other, scene, &mara(), 7, 1);
    assert_eq!(told[0].lines(), ["Mara! YOU CAME BACK FOR US."]);
}

#[test]
fn a_talk_shot_shows_the_box_and_the_text_typed_so_far() {
    let ctx = ctx();
    let player = player(&ctx);
    let talk = player.talk_boxes(1).unwrap();
    assert_eq!(talk.len(), 2);
    assert_eq!(talk[1].lines(), ["Ellery! You came back for us."]);
    assert!(player.talk_boxes(0).is_none() && player.talk_boxes(9).is_none());
    let shown = |ctx: &Ctx, t: f32| match player.frame(ctx, t, SCREEN) {
        Frame::Talk { index, shown } => (index, shown),
        other => panic!("{t}: {other:?}"),
    };
    // Two seconds a box, at the player's text speed.
    assert_eq!(shown(&ctx, 6.0), (0, 0));
    assert_eq!(shown(&ctx, 6.21), (0, 12));
    assert_eq!(shown(&ctx, 7.5), (0, 22));
    assert_eq!(shown(&ctx, 8.0), (1, 0));
    assert_eq!(shown(&ctx, 8.26), (1, 15));
    assert_eq!(shown(&ctx, 9.9), (1, 29));
    assert_eq!(shown(&with_speed(TextSpeed::Slow), 6.21), (0, 6));
    assert_eq!(shown(&with_speed(TextSpeed::Instant), 6.0), (0, 22));
    // A round later, the same.
    assert_eq!(shown(&ctx, 26.21), (0, 12));
    // The text on screen is as much as that.
    let text = |t: f32| text_row(&draw(&ctx, &player, t));
    assert_eq!(text(6.0), "");
    assert_eq!(text(6.21), "It's always");
    assert_eq!(text(7.5), "It's always about you.");
    assert_eq!(text(8.26), "Ellery! You cam");
    assert_eq!(text(9.9), "Ellery! You came back for us.");
}

#[test]
fn a_talk_shot_looks_like_the_dialogue_screen_without_what_waits_for_a_key() {
    // Instant text: every Confirm is the next box.
    let mut ctx = with_speed(TextSpeed::Instant);
    let scene = ctx.content.dialogue.scenes["test"].clone();
    let lead = ctx.lead.clone();
    let mut screen = DialogueScreen::told(&ctx, &scene, lead.clone(), &Present::Everyone);
    let confirm = FrameInput::new(vec![Action::Confirm], 0.0, vec![]);
    let player = CinematicPlayer::new(&ctx, "test", Some(lead)).unwrap();
    // The first frame shows the first box whole.
    screen.update(&mut ctx, &FrameInput::new(vec![], 0.0, vec![]));
    for _ in 0..6 {
        screen.update(&mut ctx, &confirm);
    }
    for t in [7.0, 9.0] {
        let mut expected = draw(&ctx, &player, 0.0);
        screen.draw(&ctx, &mut expected);
        let got = draw(&ctx, &player, t);
        // The dialogue screen's `▼` and the key that goes on, at the
        // right of the text box's last row, are not in the shot.
        let marker = |x: i32, y: i32| y == 26 && (80..98).contains(&x);
        let blank = got.get(4, 26).copied();
        for y in 0..i32::from(CONSOLE_H) {
            for x in 0..i32::from(CONSOLE_W) {
                if marker(x, y) {
                    assert_eq!(got.get(x, y).copied(), blank, "({x}, {y})");
                } else {
                    assert_eq!(got.get(x, y), expected.get(x, y), "{t}: ({x}, {y})");
                }
            }
        }
        assert!(row(&expected, 26).contains('▼'));
        assert!(!row(&got, 26).contains('▼'));
        assert_eq!(got.sprites(), expected.sprites(), "{t}");
        assert!(got.backdrop().is_none());
        screen.update(&mut ctx, &confirm);
    }
    // The next thing in the scene is the reply choice: the shot ended
    // before it.
    assert!(screen.menu().is_some());
}

#[test]
fn a_talk_shot_shows_the_lead_it_was_given() {
    let ctx = ctx();
    let portrait = |id: &str| {
        ctx.content.portraits[id]
            .expression("neutral")
            .unwrap()
            .image
    };
    let shown = |lead: Option<LeadProfile>| {
        let player = CinematicPlayer::new(&ctx, "test", lead).unwrap();
        let buf = draw(&ctx, &player, 9.5);
        let images: Vec<_> = buf.sprites().iter().map(|s| s.image).collect();
        let lead = player.lead().clone();
        (text_row(&buf), row(&buf, 19), images, lead)
    };
    // The archer who speaks has no portrait: the lead's is the only one.
    let (text, plates, images, lead) = shown(None);
    assert_eq!(text, "Ellery! You came back for us.");
    assert!(plates.contains("Ellery") && plates.contains("Test Archer"));
    assert_eq!(images, [portrait("lead_m")]);
    assert_eq!(lead, LeadProfile::new("Ellery", LeadGender::Male));
    let (text, plates, images, lead) = shown(Some(mara()));
    assert_eq!(text, "Mara! You came back for us.");
    assert!(plates.contains("Mara") && !plates.contains("Ellery"));
    assert_eq!(images, [portrait("lead_f")]);
    assert_eq!(lead, mara());
}

#[test]
fn a_talk_shot_without_its_scene_or_its_boxes_is_blank() {
    let mut ctx = ctx();
    ctx.content.dialogue.scenes.remove("test");
    let player = player(&ctx);
    assert_eq!(player.talk_boxes(1), Some(&[][..]));
    assert_eq!(player.frame(&ctx, 7.0, SCREEN), Frame::Blank);
    let buf = draw(&ctx, &player, 7.0);
    let c = |u| ctx.palette.get(u);
    let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
    for y in 0..i32::from(CONSOLE_H) {
        for x in 0..i32::from(CONSOLE_W) {
            assert_eq!(buf.get(x, y), Some(&blank), "({x}, {y})");
        }
    }
    assert!(buf.items().is_empty());
}

#[test]
fn faces_cross_from_just_off_one_edge_to_just_off_the_other() {
    // Three faces on 100 columns: 100 + 2 × 50 + 34 columns to go.
    let right = |p: f32| first_x(p, 3, Side::Right, 100);
    assert_eq!(right(0.0), (100, 50));
    assert_eq!(right(0.5), (-17, 50));
    assert_eq!(right(1.0), (-134, 50));
    // The last face's frame (34 wide) is then just off the left edge.
    assert_eq!(right(1.0).0 + 2 * 50 + FRAME.0, 0);
    // In whole cells: 0.29 of 234 is 67.86 columns gone.
    assert_eq!(right(0.29), (33, 50));
    // From the left, the mirror picture: the others follow on its left.
    let left = |p: f32| first_x(p, 3, Side::Left, 100);
    assert_eq!(left(0.0), (-34, -50));
    assert_eq!(left(0.5), (83, -50));
    assert_eq!(left(1.0), (200, -50));
    assert_eq!(left(1.0).0 - 2 * 50, 100);
    // One face, or none, has only the screen and itself to cross.
    assert_eq!(first_x(1.0, 1, Side::Right, 100), (-34, 50));
    assert_eq!(first_x(1.0, 0, Side::Right, 100), (-34, 50));
    assert_eq!(first_x(0.5, 1, Side::Right, 100), (33, 50));
    // A progress that isn't one is the start; past an end, that end.
    assert_eq!(right(f32::NAN), (100, 50));
    assert_eq!(right(-2.0), (100, 50));
    assert_eq!(right(2.0), (-134, 50));
    // A crowd beyond counting doesn't overflow.
    assert_eq!(first_x(0.0, usize::MAX, Side::Right, 100), (100, 50));
}

#[test]
fn a_face_has_the_short_name_and_the_lead_the_one_given() {
    let ctx = ctx();
    let who: Vec<_> = ["lead", "retainer", "test_lord", "stranger"]
        .iter()
        .map(|id| CharacterId((*id).to_owned()))
        .collect();
    let named = |lead: &LeadProfile| -> Vec<(String, String)> {
        let faces = faces(&ctx, &who, lead);
        faces.into_iter().map(|f| (f.portrait, f.name)).collect()
    };
    let pair = |portrait: &str, name: &str| (portrait.to_owned(), name.to_owned());
    // "Hollis Marr" is Hollis; a name without a short form is whole; an
    // id the names table lacks is shown as it is.
    assert_eq!(ctx.content.names.get("retainer"), Some("Hollis Marr"));
    assert_eq!(
        named(&mara()),
        [
            pair("lead_f", "Mara"),
            pair("retainer", "Hollis"),
            pair("test_lord", "Test Lord"),
            pair("stranger", "stranger"),
        ]
    );
    let odo = LeadProfile::new("Odo", LeadGender::Male);
    assert_eq!(named(&odo)[0], pair("lead_m", "Odo"));
    // In the player's language.
    let mut other = crate::screen::tests::ctx();
    other.lang = LangCode::new("test").unwrap();
    let knight = [CharacterId("test_knight".to_owned())];
    assert_eq!(faces(&ctx, &knight, &mara())[0].name, "Test Knight");
    assert_eq!(faces(&other, &knight, &mara())[0].name, "TEST KNIGHT");
}

#[test]
fn a_characters_shot_moves_its_faces_across_the_screen() {
    let ctx = ctx();
    let player = player(&ctx);
    let names: Vec<_> = player.faces(2).unwrap().iter().map(|f| &f.name).collect();
    assert_eq!(names, ["Ellery", "Test Lord", "Test Knight"]);
    assert!(player.faces(1).is_none() && player.faces(9).is_none());
    let at = |t: f32| match player.frame(&ctx, t, SCREEN) {
        Frame::Characters { x, step } => (x, step),
        other => panic!("{t}: {other:?}"),
    };
    // Six seconds for 234 columns: 39 a second.
    assert_eq!(at(10.0), (100, 50));
    assert_eq!(at(11.0), (61, 50));
    assert_eq!(at(11.73), (33, 50));
    assert_eq!(at(13.0), (-17, 50));
    assert_eq!(at(15.999).0, -133);
    // On a narrower screen there is less to cross.
    let narrow = player.frame(&ctx, 13.0, Rect::new(0, 0, 60, 32));
    assert_eq!(narrow, Frame::Characters { x: -37, step: 50 });
}

#[test]
fn a_characters_shot_draws_each_face_in_its_frame_over_its_name() {
    let ctx = ctx();
    let player = player(&ctx);
    let image = |id: &str| {
        ctx.content.portraits[id]
            .expression("neutral")
            .unwrap()
            .image
    };
    let cell_w = i32::from(CELL_W_PX);
    // The lead in the middle of the screen, the lord half in behind.
    let buf = draw(&ctx, &player, 11.73);
    // A frame is 18 rows and its name the row under: rows 6 to 23, and 24.
    let top = row(&buf, 6);
    let frame: String = top.chars().skip(33).take(34).collect();
    assert_eq!(frame, format!("╔{}╗", "═".repeat(32)));
    assert_eq!(top.chars().nth(83), Some('╔'));
    assert_eq!(row(&buf, 5).trim(), "");
    assert_eq!(row(&buf, 23).chars().nth(33), Some('╚'));
    let plates = row(&buf, 24);
    assert_eq!(plates.trim_start().find("Ellery"), Some(0));
    assert_eq!(plates.find("Ellery"), Some(33 + (34 - 6) / 2));
    // The lord's name is cut by the screen's edge: it starts at 95.
    assert!(plates.ends_with("Test "), "{plates:?}");
    assert_eq!(row(&buf, 25).trim(), "");
    let sprites = buf.sprites();
    let shown: Vec<_> = sprites.iter().map(|s| s.image).collect();
    assert_eq!(shown, [image("lead_m"), image("test_lord")]);
    // Inside the frame's border, looking the way they go (mirrored),
    // at full colour; the lord's is cut at the screen's edge.
    assert_eq!(sprites[0].dest.x, 34 * cell_w);
    assert_eq!(sprites[1].dest.x, 84 * cell_w);
    assert!(sprites.iter().all(|s| s.flip_x && s.opacity == 255));
    assert_eq!(sprites[0].clip, sprites[0].dest);
    assert_eq!(sprites[1].clip.x + sprites[1].clip.w, 100 * cell_w);
    assert!(sprites[1].clip.w < sprites[1].dest.w);
    assert!(buf.backdrop().is_none());
    // Nothing is left of the frame before, and nothing but black beside.
    let c = |u| ctx.palette.get(u);
    let blank = Cell::new(' ', c(UiColor::Text), c(UiColor::Black));
    assert_eq!(buf.get(0, 0), Some(&blank));
    assert_eq!(buf.get(32, 10), Some(&blank));
    assert_eq!(buf.get(67, 10), Some(&blank));
    // Before anyone is in, the screen is empty; then the lead's frame
    // comes in at the right edge.
    let empty = draw(&ctx, &player, 10.0);
    assert!(empty.sprites().is_empty());
    assert_eq!(row(&empty, 6).trim(), "");
    let entering = draw(&ctx, &player, 10.1);
    assert_eq!(row(&entering, 6).trim_start(), "╔══");
    // The last one out is the knight.
    let leaving = draw(&ctx, &player, 15.5);
    let shown: Vec<_> = leaving.sprites().iter().map(|s| s.image).collect();
    assert_eq!(shown, [image("test_knight")]);
    assert!(row(&leaving, 24).contains("Knight"));
}

#[test]
fn a_characters_shot_shows_the_lead_it_was_given_and_comes_from_its_side() {
    let mut ctx = ctx();
    let def = ctx.content.cinematics.get_mut("test").unwrap();
    let Shot::Characters { from, .. } = &mut def.shots[2].shot else {
        panic!("not the characters shot");
    };
    *from = Side::Left;
    let player = CinematicPlayer::new(&ctx, "test", Some(mara())).unwrap();
    // The mirror of the frame at 33 is the frame at 33: the lead is in
    // the middle at the same moment, the lord half in on the left.
    assert_eq!(
        player.frame(&ctx, 11.73, SCREEN),
        Frame::Characters { x: 33, step: -50 }
    );
    let buf = draw(&ctx, &player, 11.73);
    let image = |id: &str| {
        ctx.content.portraits[id]
            .expression("neutral")
            .unwrap()
            .image
    };
    let sprites = buf.sprites();
    let shown: Vec<_> = sprites.iter().map(|s| s.image).collect();
    assert_eq!(shown, [image("lead_f"), image("test_lord")]);
    // Going right, they look right: as the art is.
    assert!(sprites.iter().all(|s| !s.flip_x));
    let plates = row(&buf, 24);
    assert_eq!(plates.find("Mara"), Some(33 + (34 - 4) / 2));
    // The lord's name is cut by the screen's edge: it starts at -5.
    assert!(plates.starts_with("Lord "), "{plates:?}");
    assert_eq!(row(&buf, 6).chars().nth(16), Some('╗'));
}

#[test]
fn a_characters_shot_with_nobody_is_blank() {
    let mut ctx = ctx();
    let def = ctx.content.cinematics.get_mut("test").unwrap();
    let Shot::Characters { who, .. } = &mut def.shots[2].shot else {
        panic!("not the characters shot");
    };
    who.clear();
    let player = player(&ctx);
    assert_eq!(player.faces(2), Some(&[][..]));
    assert_eq!(player.frame(&ctx, 12.0, SCREEN), Frame::Blank);
    assert!(draw(&ctx, &player, 12.0).items().is_empty());
}
