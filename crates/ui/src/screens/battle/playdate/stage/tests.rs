//! The combat picture's look on the Playdate (ADR-0038: a look's tests
//! read cells, items and pixels): where the backdrop, the fighters, the
//! plates and the result go on the 400 × 240 screen, and one frame of a
//! strike through the one-bit renderer. What the stage shows is tested on
//! its view (`stage/tests.rs`).

use shuyi_assets::bitmap::convert::{Rgba, convert};
use shuyi_assets::bitmap::{Bitmap, encode};
use trpg_content::ImageId;
use trpg_content::image::ImageInfo;
use trpg_content::playdate_images::{FIGHTER_RIM, Job, jobs, with_rim};
use trpg_core::Faction;

use super::*;
use crate::map_view::playdate::tests::{SCREEN, c as ink, font};
use crate::onebit::{FRAME_BYTES, Frame, Image, Images, is_white, render};
use crate::screens::battle::stage::motion::Pose;
use crate::screens::battle::stage::view::Numbers;
use crate::screens::playdate::tests::{blank, cell, find, is_bar, pd_ctx, row};

/// A picture `w` × `h` px of `image`, facing `faces` at `scale`.
fn picture(image: ImageId, (w, h): (u32, u32), faces: Facing, scale: (u32, u32)) -> Battler {
    Battler {
        id: "t".to_owned(),
        image,
        size: ImageInfo {
            width: w,
            height: h,
        },
        faces,
        scale,
    }
}

fn fighter(name: &str, faction: Faction, picture: Battler) -> FighterView {
    FighterView {
        name: name.to_owned(),
        faction,
        hp: 13,
        max: 20,
        lit: false,
        numbers: Some(Numbers {
            damage: 8,
            heals: false,
            hit: 96,
            crit: 3,
        }),
        picture: Some(picture),
        pose: Pose::default(),
    }
}

/// The default battler's image and the brigand's: two pictures.
fn images(c: &Ctx) -> (ImageId, ImageId) {
    let battlers = &c.game.content.battlers.battlers;
    (battlers["default"].image, battlers["brigand"].image)
}

/// The lord (48×72, facing right) attacks a brigand (64×64 at 1.25×,
/// facing left), hand to hand, the lord on the left.
fn stage(c: &Ctx) -> StageView {
    let (lord, brigand) = images(c);
    StageView {
        backdrop: c.game.content.battlers.backdrops.default,
        shade: trpg_content::Light::PLAIN,
        banner: None,
        fighters: [
            fighter(
                "Ellery",
                Faction::Player,
                picture(lord, (48, 72), Facing::Right, (1, 1)),
            ),
            fighter(
                "Brigand",
                Faction::Enemy,
                picture(brigand, (64, 64), Facing::Left, (5, 4)),
            ),
        ],
        left: 0,
        melee: true,
        striker: None,
        result: None,
        veil: false,
    }
}

/// The lord's strike landing: it has run up and is lit, the brigand is
/// hit (a solid shape) and `HIT -8` is over it.
fn strike(c: &Ctx) -> StageView {
    let mut view = stage(c);
    view.striker = Some(0);
    view.fighters[0].lit = true;
    view.fighters[0].pose.lunge = 1.0;
    view.fighters[1].pose.flash = true;
    view.result = Some((1, "HIT -8".to_owned()));
    view
}

/// The `n` glyphs of row `y` from column `x`.
fn text(buf: &GlyphBuffer, x: i32, y: i32, n: i32) -> String {
    (x..x + n).map(|x| cell(buf, x, y).glyph).collect()
}

fn painted(c: &Ctx, view: &StageView) -> GlyphBuffer {
    let mut buf = blank();
    paint(c, view, SCREEN, &mut buf);
    buf
}

/// The scene in pixels.
const SCENE_PX: PxRect = PxRect::new(0, 0, 400, 192);
/// The lord's and the brigand's sprites as `stage` has them: twice their
/// size, feet at 188, middles at 112 and 288.
const LORD: (PxRect, PxRect) = (PxRect::new(0, 0, 48, 72), PxRect::new(64, 44, 96, 144));
const BRIGAND: (PxRect, PxRect) = (PxRect::new(0, 0, 64, 64), PxRect::new(208, 28, 160, 160));

/// The size of the one-bit picture the converter makes of a `w` × `h`
/// PNG enlarged as `job` says.
fn converted(w: u32, h: u32, job: &Job<'_>) -> (i32, i32) {
    let pixels = vec![128; (w * h * 4) as usize];
    let image = Rgba::new(w, h, &pixels).unwrap();
    let file = convert(&image, job.scale, &job.kind.methods()).unwrap();
    let bitmap = Bitmap::parse(&file).unwrap();
    (i32::from(bitmap.width()), i32::from(bitmap.height()))
}

#[test]
fn the_plates_go_along_the_bottom_and_the_pictures_above_them() {
    assert_eq!(scene(SCREEN), Rect::new(0, 0, 50, 12));
    assert_eq!(px_rect(scene(SCREEN)), SCENE_PX);
    assert_eq!(feet_y(SCENE_PX), 188);
    // The fighters stand as far from the middle as each other.
    assert_eq!(middles(SCENE_PX), [112, 288]);
    assert!(HORIZON_Y < feet_y(SCENE_PX));
}

#[test]
fn every_picture_is_drawn_at_the_size_the_converter_made_it() {
    let c = pd_ctx();
    let (id, _) = images(&c);
    // The backdrop: the screen's width.
    let backs = ["battlebacks/fields.png".to_owned()];
    let back = &jobs(&backs, |_| None).unwrap()[0];
    assert_eq!(back.scale, BACKDROP_SCALE);
    let (w, h) = BACKDROP_PX;
    let sprite = backdrop_sprite(id, SCENE_PX).unwrap();
    assert_eq!((sprite.dest.w, sprite.dest.h), converted(w, h, back));
    assert_eq!(sprite.dest.w, SCREEN.w * CELL_W);
    // A fighter: twice its sidecar's scale, rounded as the converter
    // rounds (75 × 2.5 = 187.5 is 188).
    let files = ["battlers/x.ron".to_owned(), "battlers/x.png".to_owned()];
    let sidecar = |_: &str| Some("(image: \"x.png\", scale: (5, 4))".to_owned());
    let job = jobs(&files, sidecar).unwrap().into_iter().next().unwrap();
    for (w, h) in [(64, 64), (75, 72)] {
        let shown = size_px(&picture(id, (w, h), Facing::Left, (5, 4)));
        assert_eq!(shown, converted(w, h, &job), "{w} × {h}");
    }
    assert_eq!(size_px(&picture(id, (75, 1), Facing::Left, (5, 4))).0, 188);
}

#[test]
fn the_backdrop_shows_only_above_the_horizon_line() {
    let c = pd_ctx();
    let view = stage(&c);
    let back = view.backdrop.unwrap();
    let sprite = backdrop_sprite(back, SCENE_PX).unwrap();
    assert_eq!(sprite.src, PxRect::new(0, 0, 336, 248));
    assert_eq!(sprite.dest, PxRect::new(0, -BACKDROP_TOP_PX, 400, 295));
    assert_eq!(sprite.clip, PxRect::new(0, 0, 400, HORIZON_Y));
    assert_eq!((sprite.paint, sprite.layer), (Paint::Image, Layer::Under));
    let buf = painted(&c, &view);
    assert_eq!(buf.sprites()[0], sprite);
    // The horizon: a black line a pixel high across the screen.
    let line = Overlay::new(PxRect::new(0, 110, 400, 1), ink(Ink::Black), Layer::Under);
    assert!(buf.overlays().contains(&line));
}

#[test]
fn each_fighter_stands_on_the_ground_at_double_size_facing_the_other() {
    let c = pd_ctx();
    let view = stage(&c);
    let white = ink(Ink::White);
    let lord = fighter_sprite(&view, 0, SCENE_PX, white).unwrap();
    assert_eq!((lord.src, lord.dest, lord.clip), (LORD.0, LORD.1, LORD.1));
    let brigand = fighter_sprite(&view, 1, SCENE_PX, white).unwrap();
    assert_eq!((brigand.src, brigand.dest), BRIGAND);
    // Both already face the other: neither is mirrored. Swapped, both are.
    assert!(!lord.flip_x && !brigand.flip_x);
    let mut swapped = view.clone();
    swapped.left = 1;
    let flipped = |i| fighter_sprite(&swapped, i, SCENE_PX, white).unwrap();
    assert!(flipped(0).flip_x && flipped(1).flip_x);
    assert_eq!(flipped(1).dest.x, LORD.1.x + 48 - 80);
    for s in [lord, brigand] {
        assert_eq!(
            (s.paint, s.opacity, s.layer),
            (Paint::Image, 255, Layer::Under)
        );
    }
    // A fighter taller than the scene is cut at the screen's top.
    let mut tall = view.clone();
    let (image, _) = images(&c);
    tall.fighters[0].picture = Some(picture(image, (64, 96), Facing::Right, (1, 1)));
    let s = fighter_sprite(&tall, 0, SCENE_PX, white).unwrap();
    assert_eq!((s.dest.y, s.clip.y, s.clip.h), (188 - 192, 0, 188));
}

#[test]
fn the_motions_are_the_pcs_in_pixels() {
    let c = pd_ctx();
    let mut view = stage(&c);
    // Hand to hand the lord runs up until the two overlap by 24 pixels:
    // 176 apart, less half of 96 + 160, plus 24.
    assert_eq!(reach_px(&view, 0), 176 - 128 + OVERLAP_PX);
    view.fighters[0].pose.lunge = 1.0;
    let lord = fighter_sprite(&view, 0, SCENE_PX, ink(Ink::White)).unwrap();
    assert_eq!(lord.dest.x, LORD.1.x + 72);
    // The brigand on the right goes left to reach the lord.
    view.fighters[0].pose.lunge = 0.0;
    view.fighters[1].pose.lunge = 1.0;
    let brigand = fighter_sprite(&view, 1, SCENE_PX, ink(Ink::White)).unwrap();
    assert_eq!(brigand.dest.x, BRIGAND.1.x - 72);
    // From afar it only steps forward.
    view.melee = false;
    assert_eq!(reach_px(&view, 1), STEP_PX);
    // A falling fighter sinks and fades.
    view.fighters[1].pose = Pose {
        fade: 1.0,
        ..Pose::default()
    };
    let gone = fighter_sprite(&view, 1, SCENE_PX, ink(Ink::White)).unwrap();
    assert_eq!(gone.dest.y, BRIGAND.1.y + 12);
    assert_eq!(gone.opacity, dim_opacity(1.0));
}

/// A test picture `w` × `h` (in screen pixels) of a PNG `source` big:
/// white, see-through but for a figure inset 4 pixels from its sides and
/// top, with the fighters' rim made as the converter's table makes it.
fn figure((w, h): (u16, u16), source: (u16, u16)) -> Vec<u8> {
    let stride = Bitmap::stride(w);
    let mut mask = vec![0u8; stride * usize::from(h)];
    for y in 4..usize::from(h) {
        for x in 4..usize::from(w) - 4 {
            mask[y * stride + x / 8] |= 0x80 >> (x % 8);
        }
    }
    let white = vec![0xff; mask.len()];
    let file = encode((w, h), source, &[&white], Some(&mask)).unwrap();
    with_rim(&file, FIGHTER_RIM).unwrap()
}

/// Test pictures: the backdrop black all over, the two fighters white
/// figures.
struct Pictures(Vec<(ImageId, Vec<u8>)>);

impl Pictures {
    fn of(c: &Ctx) -> Self {
        let (lord, brigand) = images(c);
        let back = c.game.content.battlers.backdrops.default.unwrap();
        let dark = vec![0; Bitmap::plane_bytes(400, 295)];
        Self(vec![
            (
                back,
                encode((400, 295), (336, 248), &[&dark], None).unwrap(),
            ),
            (lord, figure((96, 144), (48, 72))),
            (brigand, figure((160, 160), (64, 64))),
        ])
    }
}

impl Images for Pictures {
    fn image(&self, id: ImageId) -> Option<Image<'_>> {
        let (_, file) = self.0.iter().find(|(i, _)| *i == id)?;
        Image::from_bitmap(&Bitmap::parse(file).ok()?, 0)
    }
}

/// `buf` as the Playdate shows it, with the test pictures.
fn shown(c: &Ctx, buf: &GlyphBuffer) -> Box<Frame> {
    let mut frame = Box::new([0; FRAME_BYTES]);
    render(buf, &font(), &Pictures::of(c), &mut frame);
    frame
}

#[test]
fn one_frame_of_a_strike() {
    let c = pd_ctx();
    let view = strike(&c);
    let buf = painted(&c, &view);
    // The striker last, in front of its target; the target a black shape.
    // (The result's box cut the backdrop's sprite in pieces round it.)
    let sprites = buf.sprites();
    let [.., target, striker] = sprites.as_slice() else {
        panic!("{sprites:?}");
    };
    let (lord, brigand) = images(&c);
    assert_eq!((target.image, striker.image), (brigand, lord));
    assert_eq!(striker.dest.x, LORD.1.x + 72);
    assert_eq!(target.paint, Paint::Solid(ink(Ink::Black)));
    // The result over the brigand's head, in a white box on row 0 with a
    // black edge a pixel wide round it.
    let at = find(&buf, 0, "HIT -8").unwrap();
    assert_eq!(at, 288 / 8 - 3);
    assert_eq!(cell(&buf, at - 1, 0).bg, ink(Ink::White));
    let edge = Overlay::new(
        PxRect::new(8 * at - 9, 16, 66, 1),
        ink(Ink::Black),
        Layer::Over,
    );
    assert!(buf.overlays().contains(&edge), "{:?}", buf.overlays());
    // The plates: the striker's double, its name the bar; the HP and the
    // numbers as on the PC.
    assert_eq!(
        (cell(&buf, 0, 12).glyph, cell(&buf, 25, 12).glyph),
        ('╔', '┌')
    );
    assert_eq!(cell(&buf, 0, 14).glyph, '╚');
    assert!(is_bar(&buf, 12, "Ellery"));
    // The bar is the name and a cell either side, no more.
    let bar = |x| cell(&buf, x, 12).bg == ink(Ink::Black);
    assert!(bar(2) && bar(9) && !bar(10));
    assert!(!is_bar(&buf, 12, "Brigand"));
    assert!(row(&buf, 12).contains("HP 13/20"), "{}", row(&buf, 12));
    assert_eq!(text(&buf, 0, 13, 25), "║ 8 dmg  Hit 96  Crit 3 ║");
    assert_eq!(text(&buf, 25, 13, 25), "│ 8 dmg  Hit 96  Crit 3 │");

    // In pixels: the backdrop above the horizon, the line, white ground.
    let frame = shown(&c, &buf);
    let white = |x, y| is_white(&frame, x, y);
    assert!(!white(4, 50) && !white(4, 110) && white(4, 150));
    // The lord (a white figure) has its white rim against the backdrop,
    // two pixels above its head, then the backdrop.
    let (lord_x, head) = (LORD.1.x + 72 + 48, LORD.1.y + 4);
    assert!(white(lord_x, head) && white(lord_x, head - 1) && white(lord_x, head - 2));
    assert!(!white(lord_x, head - 3));
    // The struck brigand (a white figure too) is a black shape, its rim
    // with it, on the ground as against the backdrop.
    let (brigand_x, top) = (BRIGAND.1.x + 80, BRIGAND.1.y + 4);
    assert!(!white(brigand_x, 150) && !white(brigand_x, top));
    assert!(!white(brigand_x, top - 1) && !white(brigand_x, 186));
    assert!(white(brigand_x, 188 + 1));
}

#[test]
fn a_critical_flips_the_whole_picture_but_not_the_plates() {
    let c = pd_ctx();
    let mut view = stage(&c);
    view.veil = true;
    let buf = painted(&c, &view);
    let veil = Overlay::new(SCENE_PX, ink(Ink::Invert), Layer::Over);
    assert!(buf.overlays().contains(&veil));
    let frame = shown(&c, &buf);
    // The ground is black and the backdrop white, the plates as they were.
    assert!(!is_white(&frame, 4, 150) && is_white(&frame, 4, 50));
    assert!(is_white(&frame, 23 * 8 + 4, 13 * 16 + 8));
}

#[test]
fn a_banner_and_no_counter() {
    let c = pd_ctx();
    let mut view = stage(&c);
    view.banner = Some("Sunder".to_owned());
    view.result = Some((1, "MISS".to_owned()));
    view.fighters[1].numbers = None;
    let buf = painted(&c, &view);
    // The art's name at the top in the middle; the result a row lower.
    assert_eq!(find(&buf, 0, "Sunder"), Some(25 - 4 + 1));
    assert_eq!(find(&buf, 1, "MISS"), Some(288 / 8 - 3 + 1));
    assert!(find(&buf, 0, "MISS").is_none());
    assert_eq!(text(&buf, 25, 13, 25).trim(), "│ no counter            │");
    // Each box's black edge a pixel outside its cells, cut at the screen's
    // top: the banner's 8 cells from cell 21 on row 0, the result's 6 from
    // cell 33 on row 1.
    let black = ink(Ink::Black);
    let edge = |x, y, w, h| Overlay::new(PxRect::new(x, y, w, h), black, Layer::Over);
    let edges: Vec<Overlay> = buf
        .overlays()
        .into_iter()
        .filter(|o| o.layer == Layer::Over)
        .collect();
    let expected = [
        edge(167, 16, 66, 1),
        edge(167, 0, 1, 16),
        edge(232, 0, 1, 16),
        edge(263, 15, 50, 1),
        edge(263, 32, 50, 1),
        edge(263, 16, 1, 16),
        edge(312, 16, 1, 16),
    ];
    assert_eq!(edges, expected);
}

#[test]
fn the_result_goes_on_the_row_over_the_targets_head() {
    let c = pd_ctx();
    let view = stage(&c);
    let cells = scene(SCREEN);
    // The lord's head at 44 (188 − 144): row 1; the brigand's at 28: row 0.
    assert_eq!(result_row(&view, 0, cells, 0), 1);
    assert_eq!(result_row(&view, 1, cells, 0), 0);
    // Never above the row it is given.
    assert_eq!(result_row(&view, 1, cells, 1), 1);
    // No picture: the row over the ground.
    let mut bare = view.clone();
    bare.fighters[0].picture = None;
    assert_eq!(result_row(&bare, 0, cells, 0), 188 / 16 - 1);
}

#[test]
fn a_long_banner_is_shown_across_the_screen() {
    let c = pd_ctx();
    let mut view = stage(&c);
    let name = "A Very Long Name For An Art, Thirty-Eight";
    view.banner = Some(name.to_owned());
    let buf = painted(&c, &view);
    assert_eq!(find(&buf, 0, name), Some(25 - 21 + 1));
}

#[test]
fn a_long_name_and_big_numbers_are_cut_to_the_plate() {
    let c = pd_ctx();
    let mut view = stage(&c);
    view.fighters[0].name = "Bartholomew the Long".to_owned();
    view.fighters[0].max = 100;
    view.fighters[0].hp = 100;
    view.fighters[0].numbers = Some(Numbers {
        damage: 12,
        heals: false,
        hit: 100,
        crit: 10,
    });
    // Numbers of 22 cells with two spaces would touch the frame: one.
    view.fighters[1].numbers = Some(Numbers {
        damage: 12,
        heals: false,
        hit: 96,
        crit: 3,
    });
    let buf = painted(&c, &view);
    assert_eq!(text(&buf, 0, 12, 25), "┌─ Bartho…  HP 100/100 ─┐");
    // Two spaces between the numbers don't fit: one does, up to the frame.
    assert_eq!(text(&buf, 0, 13, 25), "│ 12 dmg Hit 100 Crit 10│");
    assert_eq!(text(&buf, 25, 13, 25), "│ 12 dmg Hit 96 Crit 3  │");
}
