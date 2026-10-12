//! The combat stage's look: where the pictures and the plates go.

use trpg_content::image::ImageInfo;
use trpg_core::Faction;

use super::*;
use crate::screen::tests::ctx;
use crate::screens::battle::stage::view::Numbers;

const WHITE: Rgb = Rgb::new(250, 250, 250);

/// A picture `w` × `h` px facing `faces` at `scale`, of the default image.
fn picture(c: &Ctx, (w, h): (u32, u32), faces: Facing, scale: (u32, u32)) -> Battler {
    Battler {
        id: "t".to_owned(),
        image: c.content.battlers.battlers["default"].image,
        size: ImageInfo {
            width: w,
            height: h,
        },
        faces,
        scale,
    }
}

fn fighter(name: &str, faction: Faction, picture: Option<Battler>) -> FighterView {
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
        picture,
        pose: Pose::default(),
    }
}

/// The lord (48×72, facing right) attacks a brigand (64×64 at 1.25×,
/// facing left), hand to hand, the lord on the left.
fn stage(c: &Ctx) -> StageView {
    let lord = picture(c, (48, 72), Facing::Right, (1, 1));
    let brigand = picture(c, (64, 64), Facing::Left, (5, 4));
    StageView {
        backdrop: c.content.battlers.backdrops.default,
        banner: None,
        fighters: [
            fighter("Ellery", Faction::Player, Some(lord)),
            fighter("Brigand", Faction::Enemy, Some(brigand)),
        ],
        left: 0,
        melee: true,
        striker: None,
        result: None,
        veil: false,
    }
}

fn painted(c: &Ctx, view: &StageView) -> GlyphBuffer {
    let blank = Cell::new(
        '#',
        c.palette.get(UiColor::Text),
        c.palette.get(UiColor::Black),
    );
    let mut buf = GlyphBuffer::new(100, 32, blank);
    paint(c, view, &mut buf);
    buf
}

fn text(buf: &GlyphBuffer, x: i32, y: i32, n: i32) -> String {
    (x..x + n).map(|x| buf.get(x, y).unwrap().glyph).collect()
}

fn row(buf: &GlyphBuffer, y: i32) -> String {
    text(buf, STAGE.x, y, STAGE.w)
}

/// The lord's and the brigand's sprites as `stage` has them.
const LORD: (PxRect, PxRect) = (PxRect::new(0, 0, 48, 72), PxRect::new(202, 132, 96, 144));
const BRIGAND: (PxRect, PxRect) = (PxRect::new(0, 0, 64, 64), PxRect::new(470, 116, 160, 160));
/// The backdrop's three pieces: the middle, the left edge, the right edge.
const BACK: [(PxRect, PxRect); 3] = [
    (PxRect::new(0, 40, 336, 192), PxRect::new(64, 0, 672, 384)),
    (PxRect::new(0, 40, 32, 192), PxRect::new(0, 0, 64, 384)),
    (PxRect::new(304, 40, 32, 192), PxRect::new(736, 0, 64, 384)),
];

#[test]
fn the_stage_covers_the_map_and_the_side_panel_and_the_backdrop_fits() {
    assert_eq!(STAGE, Rect::new(0, 0, 100, 30));
    assert_eq!(SCENE, Rect::new(0, 0, 100, 24));
    assert_eq!(scene_px(), PxRect::new(0, 0, 800, 384));
    assert_eq!(PLATE, (50, 6));
    assert_eq!(SCENE.h + PLATE.1, STAGE.h);
    assert_eq!(PLATE.0 * 2, STAGE.w);
    // The backdrop's window and its two mirrored edges at double size are
    // the scene, inside the file.
    let (w, h) = trpg_content::battler::BACKDROP_PX;
    assert_eq!((BACKDROP_SRC.w + 2 * EDGE_PX) * int(ZOOM), scene_px().w);
    assert_eq!(BACKDROP_SRC.h * int(ZOOM), scene_px().h);
    assert_eq!((BACKDROP_SRC.x, BACKDROP_SRC.w), (0, int(w)));
    assert!(BACKDROP_SRC.y + BACKDROP_SRC.h <= int(h));
    // The fighters stand as far from the middle as each other.
    assert_eq!(MIDDLES[0] + MIDDLES[1], scene_px().w);
    // The biggest picture allowed stands on the ground inside the scene.
    let (_, max_h) = trpg_content::battler::MAX_SCALED_PX;
    assert!(int(max_h * ZOOM) > FEET_Y, "the tallest pictures are cut");
    assert!(FEET_Y < scene_px().h);
}

#[test]
fn the_backdrop_is_its_window_and_its_edges_mirrored_beside_it() {
    let c = ctx();
    let image = c.content.battlers.backdrops.default.unwrap();
    let sprites = backdrop_sprites(image);
    let places: Vec<(PxRect, PxRect)> = sprites.iter().map(|s| (s.src, s.dest)).collect();
    assert_eq!(places, BACK);
    let flips: Vec<bool> = sprites.iter().map(|s| s.flip_x).collect();
    assert_eq!(flips, [false, true, true]);
    for s in sprites {
        assert_eq!((s.image, s.layer, s.opacity), (image, Layer::Under, 255));
        assert_eq!((s.clip, s.paint), (s.dest, Paint::Image));
    }
}

#[test]
fn each_picture_stands_on_the_ground_at_double_size_facing_the_other() {
    let c = ctx();
    let view = stage(&c);
    let lord = fighter_sprite(&view, 0, WHITE).unwrap();
    // 48×72 doubled, centred on 250, feet at 276.
    assert_eq!((lord.src, lord.dest), LORD);
    assert_eq!(lord.clip, lord.dest);
    assert_eq!(
        (lord.flip_x, lord.opacity, lord.paint, lord.layer),
        (false, 255, Paint::Image, Layer::Under)
    );
    // 64×64 at 1.25× doubled is 160×160, centred on 550; it looks left
    // as drawn, so it isn't mirrored on the right.
    let brigand = fighter_sprite(&view, 1, WHITE).unwrap();
    assert_eq!((brigand.src, brigand.dest), BRIGAND);
    assert!(!brigand.flip_x);
    // Swapped: each takes the other's place and is mirrored.
    let swapped = StageView { left: 1, ..view };
    let lord = fighter_sprite(&swapped, 0, WHITE).unwrap();
    assert_eq!(lord.dest, PxRect::new(502, 132, 96, 144));
    assert!(lord.flip_x);
    let brigand = fighter_sprite(&swapped, 1, WHITE).unwrap();
    assert_eq!(brigand.dest, PxRect::new(170, 116, 160, 160));
    assert!(brigand.flip_x);
    assert_eq!(fighter_sprite(&swapped, 2, WHITE), None);
}

#[test]
fn a_fighter_without_a_picture_has_no_sprite() {
    let c = ctx();
    let mut view = stage(&c);
    view.fighters[1].picture = None;
    assert_eq!(fighter_sprite(&view, 1, WHITE), None);
    assert!(fighter_sprite(&view, 0, WHITE).is_some());
    // The other's reach counts its own width only: 300 - 48 + 24.
    assert_eq!(reach_px(&view, 0), 276);
    // And its result sits high in the scene.
    assert_eq!(result_row(&view, 1), 16);
}

#[test]
fn a_run_up_reaches_until_the_pictures_overlap_or_only_steps() {
    let c = ctx();
    let mut view = stage(&c);
    // 300 between the middles, half of 96 + 160 between the edges, 24 in.
    assert_eq!(reach_px(&view, 0), 196);
    assert_eq!(reach_px(&view, 1), 196);
    view.melee = false;
    assert_eq!((reach_px(&view, 0), reach_px(&view, 1)), (STEP_PX, STEP_PX));
    // Pictures so wide they already overlap: a step at least.
    view.melee = true;
    let wide = picture(&c, (240, 72), Facing::Right, (1, 1));
    view.fighters[0].picture = Some(wide.clone());
    view.fighters[1].picture = Some(wide);
    assert_eq!(reach_px(&view, 0), STEP_PX);
    // A little narrower: 300 - 292 + 24 is a step; a pixel less wide,
    // one pixel more.
    view.fighters[1].picture = Some(picture(&c, (52, 72), Facing::Right, (1, 1)));
    assert_eq!(reach_px(&view, 0), 32);
    view.fighters[1].picture = Some(picture(&c, (51, 72), Facing::Right, (1, 1)));
    assert_eq!(reach_px(&view, 0), 33);
}

#[test]
fn a_pose_moves_the_picture_toward_or_away_from_the_opponent() {
    let pose = |lunge, dodge, shake, fade| Pose {
        lunge,
        dodge,
        shake,
        flash: false,
        fade,
    };
    assert_eq!(offset_px(&Pose::default(), 152), (0, 0));
    assert_eq!(offset_px(&pose(1.0, 0.0, 0.0, 0.0), 152), (152, 0));
    assert_eq!(offset_px(&pose(0.75, 0.0, 0.0, 0.0), 152), (114, 0));
    // Leaning back is its own, short distance.
    assert_eq!(offset_px(&pose(-1.0, 0.0, 0.0, 0.0), 152), (-16, 0));
    assert_eq!(offset_px(&pose(-0.5, 0.0, 0.0, 0.0), 152), (-8, 0));
    assert_eq!(offset_px(&pose(0.0, 1.0, 0.0, 0.0), 152), (-28, 0));
    assert_eq!(offset_px(&pose(0.0, 0.5, 0.0, 0.0), 152), (-14, 0));
    // Shaken away, then toward; twice as far for a critical.
    assert_eq!(offset_px(&pose(0.0, 0.0, 1.0, 0.0), 152), (-5, 0));
    assert_eq!(offset_px(&pose(0.0, 0.0, -1.0, 0.0), 152), (5, 0));
    assert_eq!(offset_px(&pose(0.0, 0.0, 2.0, 0.0), 152), (-10, 0));
    assert_eq!(offset_px(&pose(0.0, 0.0, 0.0, 0.5), 152), (0, 6));
    assert_eq!(offset_px(&pose(0.0, 0.0, 0.0, 1.0), 152), (0, 12));
    assert_eq!(offset_px(&pose(1.0, 1.0, 1.0, 1.0), 100), (67, 12));
}

#[test]
fn the_left_fighter_moves_right_and_the_right_one_left() {
    let c = ctx();
    let mut view = stage(&c);
    view.fighters[0].pose.lunge = 1.0;
    view.fighters[1].pose.lunge = 1.0;
    let x = |view: &StageView, i| fighter_sprite(view, i, WHITE).unwrap().dest.x;
    assert_eq!((x(&view, 0), x(&view, 1)), (202 + 196, 470 - 196));
    view.melee = false;
    assert_eq!((x(&view, 0), x(&view, 1)), (202 + 32, 470 - 32));
    view.left = 1;
    assert_eq!((x(&view, 0), x(&view, 1)), (502 - 32, 170 + 32));
}

#[test]
fn a_hit_picture_is_a_white_shape_and_a_falling_one_fades_and_sinks() {
    let c = ctx();
    let mut view = stage(&c);
    view.fighters[1].pose.flash = true;
    let hit = fighter_sprite(&view, 1, WHITE).unwrap();
    assert_eq!(hit.paint, Paint::Solid(WHITE));
    assert_eq!(hit.opacity, 255);
    view.fighters[1].pose = Pose {
        fade: 0.5,
        ..Pose::default()
    };
    let falling = fighter_sprite(&view, 1, WHITE).unwrap();
    assert_eq!(falling.opacity, 128);
    assert_eq!(falling.dest, PxRect::new(470, 122, 160, 160));
    assert_eq!(falling.paint, Paint::Image);
    view.fighters[1].pose.fade = 1.0;
    assert_eq!(fighter_sprite(&view, 1, WHITE).unwrap().opacity, 0);
}

#[test]
fn a_picture_is_cut_at_the_scenes_edges() {
    let c = ctx();
    let mut view = stage(&c);
    // Thrown far past the left edge: what is left of it inside.
    view.fighters[0].pose.dodge = 8.0;
    let lord = fighter_sprite(&view, 0, WHITE).unwrap();
    assert_eq!(lord.dest, PxRect::new(-22, 132, 96, 144));
    assert_eq!(lord.clip, PxRect::new(0, 132, 74, 144));
    view.fighters[0].pose.dodge = 20.0;
    assert_eq!(fighter_sprite(&view, 0, WHITE), None);
}

#[test]
fn the_result_sits_over_its_targets_head() {
    let c = ctx();
    let mut view = stage(&c);
    // The lord's top is at 132 px: row 8; the label one above.
    assert_eq!(result_row(&view, 0), 7);
    // The brigand's at 116 px: row 7.
    assert_eq!(result_row(&view, 1), 6);
    // A picture as tall as the scene has room for: never above row 1.
    view.fighters[0].picture = Some(picture(&c, (48, 130), Facing::Right, (1, 1)));
    assert_eq!(result_row(&view, 0), 1);
    view.fighters[0].picture = Some(picture(&c, (48, 144), Facing::Right, (1, 1)));
    assert_eq!(result_row(&view, 0), 1);
    view.fighters[0].picture = Some(picture(&c, (48, 114), Facing::Right, (1, 1)));
    assert_eq!(result_row(&view, 0), 2);
}

#[test]
fn the_backdrop_then_the_target_then_the_striker_are_drawn() {
    let c = ctx();
    let mut view = stage(&c);
    let images = |view: &StageView| -> Vec<(PxRect, PxRect)> {
        painted(&c, view)
            .sprites()
            .iter()
            .map(|s| (s.src, s.dest))
            .collect()
    };
    let with = |fighters: [(PxRect, PxRect); 2]| [&BACK[..], &fighters[..]].concat();
    assert_eq!(images(&view), with([LORD, BRIGAND]));
    view.striker = Some(1);
    assert_eq!(images(&view), with([LORD, BRIGAND]));
    view.striker = Some(0);
    assert_eq!(images(&view), with([BRIGAND, LORD]));
    // Every sprite lies under the glyphs.
    let sprites = painted(&c, &view).sprites();
    assert!(sprites.iter().all(|s| s.layer == Layer::Under));
    // No backdrop: the pictures on the panel colour, and no veil.
    view.backdrop = None;
    view.veil = true;
    assert_eq!(images(&view), [BRIGAND, LORD]);
}

#[test]
fn a_critical_veils_the_stage_in_white_over_the_pictures() {
    let c = ctx();
    let mut view = stage(&c);
    assert_eq!(painted(&c, &view).sprites().len(), 5);
    view.veil = true;
    let sprites = painted(&c, &view).sprites();
    assert_eq!(sprites.len(), 8);
    let veil: Vec<(PxRect, PxRect)> = sprites[5..].iter().map(|s| (s.src, s.dest)).collect();
    assert_eq!(veil, BACK);
    for (piece, under) in sprites[5..].iter().zip(&sprites[..3]) {
        assert_eq!(piece.paint, Paint::Solid(c.palette.get(UiColor::White)));
        assert_eq!(piece.opacity, VEIL_OPACITY);
        assert_eq!(piece.flip_x, under.flip_x);
        assert_eq!((under.paint, under.opacity), (Paint::Image, 255));
    }
}

#[test]
fn the_plates_show_names_hp_and_numbers_left_and_right() {
    let c = ctx();
    let mut view = stage(&c);
    view.fighters[1].numbers = None;
    let buf = painted(&c, &view);
    let p = |u| c.palette.get(u);
    assert_eq!(text(&buf, 2, 25, 6), "Ellery");
    assert_eq!(text(&buf, 52, 25, 7), "Brigand");
    assert_eq!(buf.get(2, 25).unwrap().fg, p(UiColor::Player));
    assert_eq!(buf.get(52, 25).unwrap().fg, p(UiColor::Enemy));
    assert_eq!(buf.get(2, 25).unwrap().bg, p(UiColor::PanelBg));
    // 13 of 20 HP: 13 of the bar's 20 cells, in the middle colour.
    assert_eq!(text(&buf, 2, 26, 31), "HP 13 █████████████░░░░░░░ /20 ");
    assert_eq!(text(&buf, 52, 26, 5), "HP 13");
    assert_eq!(buf.get(8, 26).unwrap().fg, p(UiColor::HpMid));
    assert_eq!(buf.get(21, 26).unwrap().fg, p(UiColor::TextDim));
    assert_eq!(text(&buf, 2, 28, 22), "8 dmg  Hit 96  Crit 3 ");
    assert_eq!(text(&buf, 52, 28, 11), "no counter ");
    // Each plate is a box in its side's colour, under the scene.
    assert_eq!(buf.get(0, 24).unwrap().glyph, '┌');
    assert_eq!(buf.get(49, 29).unwrap().glyph, '┘');
    assert_eq!(buf.get(50, 24).unwrap().glyph, '┌');
    assert_eq!(buf.get(99, 29).unwrap().glyph, '┘');
    assert_eq!(buf.get(0, 24).unwrap().fg, p(UiColor::Player));
    assert_eq!(buf.get(50, 24).unwrap().fg, p(UiColor::Enemy));
    // The scene's cells are blank over the pictures; the rows under the
    // stage are untouched.
    assert_eq!(row(&buf, 0), " ".repeat(100));
    assert_eq!(row(&buf, 23), " ".repeat(100));
    assert_eq!(row(&buf, 30), "#".repeat(100));
    // Swapped sides: the defender's plate on the left.
    view.left = 1;
    let buf = painted(&c, &view);
    assert_eq!(text(&buf, 2, 25, 7), "Brigand");
    assert_eq!(text(&buf, 52, 25, 6), "Ellery");
}

#[test]
fn a_lit_name_is_inverted_and_a_heal_reads_as_one() {
    let c = ctx();
    let mut view = stage(&c);
    view.fighters[0].lit = true;
    view.fighters[0].numbers = Some(Numbers {
        damage: 5,
        heals: true,
        hit: 100,
        crit: 0,
    });
    let buf = painted(&c, &view);
    let p = |u| c.palette.get(u);
    let name = buf.get(2, 25).unwrap();
    assert_eq!(
        (name.fg, name.bg),
        (p(UiColor::PanelBg), p(UiColor::Player))
    );
    let other = buf.get(52, 25).unwrap();
    assert_eq!(
        (other.fg, other.bg),
        (p(UiColor::Enemy), p(UiColor::PanelBg))
    );
    assert_eq!(text(&buf, 2, 28, 24), "+5 hp  Hit 100  Crit 0  ");
}

#[test]
fn long_names_and_numbers_stay_inside_their_plate() {
    let c = ctx();
    let mut view = stage(&c);
    view.fighters[0].name = "N".repeat(60);
    let buf = painted(&c, &view);
    // 46 cells inside the border and its margin, then the border.
    assert_eq!(text(&buf, 2, 25, 48), format!("{} │", "N".repeat(46)));
}

#[test]
fn the_result_and_the_banner_are_labels_over_the_scene() {
    let c = ctx();
    let mut view = stage(&c);
    view.result = Some((1, "HIT -8".to_owned()));
    view.banner = Some("Pinning Shot".to_owned());
    let buf = painted(&c, &view);
    // Over the brigand: centred on its middle (550 px, column 68), one
    // row above its head.
    assert_eq!(text(&buf, 63, 6, 10), "  HIT -8  ");
    let cell = buf.get(65, 6).unwrap();
    let p = |u| c.palette.get(u);
    assert_eq!(
        (cell.fg, cell.bg),
        (p(UiColor::TextHighlight), p(UiColor::PanelBg))
    );
    // The banner in the middle of the top row.
    assert_eq!(text(&buf, 42, 0, 16), "  Pinning Shot  ");
    // The labels cut the backdrop under them into more pieces.
    assert!(buf.sprites().len() > 5);
    // Over the lord when he is the target; over the right fighter when
    // the sides are swapped.
    view.result = Some((0, "MISS".to_owned()));
    let buf = painted(&c, &view);
    assert_eq!(text(&buf, 27, 7, 8), "  MISS  ");
    view.left = 1;
    let buf = painted(&c, &view);
    assert_eq!(text(&buf, 64, 7, 8), "  MISS  ");
}

#[test]
fn a_long_label_stays_inside_the_scene() {
    let c = ctx();
    let mut view = stage(&c);
    view.result = Some((1, "R".repeat(120)));
    let buf = painted(&c, &view);
    assert_eq!(row(&buf, 6), format!(" {} ", "R".repeat(98)));
    // A label near an edge is pushed back in.
    view.result = Some((1, "R".repeat(70)));
    let buf = painted(&c, &view);
    assert_eq!(
        row(&buf, 6),
        format!("{} {} ", " ".repeat(28), "R".repeat(70))
    );
    view.left = 1;
    let buf = painted(&c, &view);
    assert_eq!(
        row(&buf, 6),
        format!(" {} {}", "R".repeat(70), " ".repeat(28))
    );
}
