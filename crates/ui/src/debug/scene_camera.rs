//! The scene camera (debug builds, ticket 0228): the test map as a
//! backdrop (ADR-0048) behind a window of the console, panned by the pixel
//! and zoomed in whole steps, with a caption and a text box over it. For
//! checking by eye that a renderer keeps the scene sharp and seamless.
//!
//! The screen is the engine's (`shuyi_ui::debug::camera`, ticket 1154);
//! this game gives it the test map as its map skin paints it, where the
//! window and the box go, and what to say without the map.

use crate::prelude::*;
use alloc::rc::Rc;

use trpg_content::bundle::display_path;
use trpg_core::Pos;

use shuyi_ui::debug::CameraLayout;
pub use shuyi_ui::debug::SceneCameraScreen;

use crate::cinematic::MapPicture;
use crate::glyph_buffer::{GlyphBuffer, Rect};
use crate::map_view::{CursorStyle, CursorView, MapScene};
use crate::screen::Ctx;

/// The map shown: `assets-test/maps/test_small.map`.
const MAP: &str = "test_small";
/// The window the scene shows through: 384 × 224 console pixels, so the
/// test map in glyphs (224 × 128) is smaller than it at 1× and bigger from
/// 2× up; and the text box drawn over its bottom-right corner.
const LAYOUT: CameraLayout = CameraLayout {
    window: Rect::new(26, 4, 48, 14),
    text_box: Rect::new(62, 15, 24, 5),
};

/// The camera at 1× over the middle of the test map, as `ctx`'s map skin
/// paints it.
pub fn scene_camera(ctx: &Ctx) -> SceneCameraScreen {
    let missing = format!("No {}", display_path(&format!("maps/{MAP}.map")));
    SceneCameraScreen::new(ctx.console, map_picture(ctx), LAYOUT, missing)
}

/// The whole test map with a path and the cursor on it (pixel-placed
/// rectangles, to see them scale), painted by `ctx`'s map skin into a
/// buffer just big enough. `None` without the map.
fn map_picture(ctx: &Ctx) -> Option<Rc<GlyphBuffer>> {
    let map = ctx.game.content.maps.get(MAP)?;
    let tiles = &map.map.tiles;
    let size = (i32::from(tiles.width()), i32::from(tiles.height()));
    let mut scene = MapScene::new(Pos::new(0, 0), size);
    scene.set_terrain(|pos| tiles.get(pos).copied());
    scene.look.clone_from(&map.look);
    scene.path = [(2, 3), (3, 3), (4, 3), (4, 4), (5, 4)]
        .map(|(x, y)| Pos::new(x, y))
        .to_vec();
    scene.cursor = Some(CursorView {
        pos: Pos::new(7, 3),
        brightness: 1.0,
        style: CursorStyle::Corners,
    });
    MapPicture::paint(ctx, ctx.game.map_skin.as_ref(), &scene).map(|picture| picture.0.scene)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::color::UiColor;
    use crate::console::{ConsoleSizes, ConsoleSpec};
    use crate::glyph_buffer::Cell;
    use crate::input::Action;
    use crate::map_view::skin_named;
    use crate::screen::tests::ctx;
    use crate::screen::{FrameInput, Screen, Shuyi, Transition};

    const WINDOW: Rect = LAYOUT.window;
    const TEXT_BOX: Rect = LAYOUT.text_box;
    const LEFT: i32 = 2;

    fn update(screen: &mut SceneCameraScreen, ctx: &mut Ctx, input: &FrameInput) -> Transition {
        Screen::<Shuyi>::update(screen, ctx, input)
    }

    /// Steps the zoom to `zoom` with Confirm, from whatever it is.
    fn zoom_to(screen: &mut SceneCameraScreen, ctx: &mut Ctx, zoom: u8) {
        while screen.zoom() != zoom {
            update(screen, ctx, &frame(&[Action::Confirm], 0.0, &[]));
        }
    }

    /// Pans by `(dx, dy)` scene pixels, holding the cursor keys that long.
    fn pan(screen: &mut SceneCameraScreen, ctx: &mut Ctx, dx: f32, dy: f32) {
        use Action::{CursorDown, CursorLeft, CursorRight, CursorUp};
        let (x, y) = (
            if dx < 0.0 { CursorLeft } else { CursorRight },
            if dy < 0.0 { CursorUp } else { CursorDown },
        );
        update(screen, ctx, &frame(&[], dx.abs() / 60.0, &[x]));
        update(screen, ctx, &frame(&[], dy.abs() / 60.0, &[y]));
    }

    fn draw(ctx: &Ctx, screen: &SceneCameraScreen) -> GlyphBuffer {
        let p = &ctx.palette;
        let mut buf = GlyphBuffer::with_spec(
            ConsoleSpec::PC,
            Cell::new('x', p.get(UiColor::Text), p.get(UiColor::PanelBg)),
        );
        Screen::<Shuyi>::draw(screen, ctx, &mut buf);
        buf
    }

    /// One frame of `dt` seconds with `pressed` pressed and `held` down.
    fn frame(pressed: &[Action], dt: f32, held: &[Action]) -> FrameInput {
        FrameInput::new(pressed.to_vec(), dt, held.to_vec())
    }

    fn near(a: (f32, f32), b: (f32, f32)) -> bool {
        (a.0 - b.0).abs() < 1e-3 && (a.1 - b.1).abs() < 1e-3
    }

    #[test]
    fn the_sizes_are_the_windows_and_the_buffers_pixels() {
        assert_eq!(scene_camera(&ctx()).window_px(), (384, 224));
        // The text box overlaps the window's corner and sticks out of it.
        let overlap = WINDOW.intersect(&TEXT_BOX).unwrap();
        assert_eq!(overlap, Rect::new(62, 15, 12, 3));
    }

    #[test]
    fn the_scene_is_the_whole_test_map_as_the_skin_paints_it() {
        let mut ctx = ctx();
        let screen = scene_camera(&ctx);
        assert_eq!(Screen::<Shuyi>::name(&screen), "scene_camera");
        // 14 × 8 tiles of two cells by one.
        let scene = screen.picture().unwrap();
        assert_eq!((scene.width(), scene.height()), (28, 8));
        assert_ne!(scene.get(0, 0).unwrap().glyph, ' ');
        assert!(scene.backdrop().is_none());
        // The path and the cursor are rectangles on it.
        assert!(!scene.overlays().is_empty());
        assert!(near(screen.centre(), (112.0, 64.0)));
        assert_eq!(screen.zoom(), 1);
        // The test tileset's tiles are 24 px: 336 × 192, in whole cells.
        ctx.game.map_skin = skin_named(&ctx.game.content, "sprite").unwrap();
        let screen = scene_camera(&ctx);
        let scene = screen.picture().unwrap();
        assert_eq!((scene.width(), scene.height()), (42, 12));
        assert!(!scene.sprites().is_empty());
        assert!(near(screen.centre(), (168.0, 96.0)));
    }

    #[test]
    fn the_scene_shows_only_through_the_windows_see_through_cells() {
        let ctx = ctx();
        let screen = scene_camera(&ctx);
        let buf = draw(&ctx, &screen);
        let backdrop = buf.backdrop().unwrap();
        assert_eq!(backdrop.clip(), WINDOW);
        assert_eq!(backdrop.zoom(), 1);
        // A scene smaller than the window is centred in it.
        assert!(near(backdrop.origin_px(), (-80.0, -48.0)));
        assert_eq!(backdrop.scene(), &**screen.picture().unwrap());
        for y in 0..i32::from(ConsoleSpec::PC.rows) {
            for x in 0..i32::from(ConsoleSpec::PC.cols) {
                let hole = WINDOW.contains(x, y) && !TEXT_BOX.contains(x, y);
                assert_eq!(buf.get(x, y).unwrap().see_through, hole, "({x}, {y})");
            }
        }
        // The caption's glyphs are over the scene; the box's cells are
        // solid, with their own background.
        let hi = ctx.palette.get(UiColor::TextHighlight);
        assert_eq!(buf.get(27, 4), Some(&Cell::see_through('T', hi)));
        let text = ctx.palette.get(UiColor::Text);
        let panel_bg = ctx.palette.get(UiColor::PanelBg);
        assert_eq!(buf.get(64, 16), Some(&Cell::new('A', text, panel_bg)));
        let row: String = (LEFT..40).map(|x| buf.get(x, 2).unwrap().glyph).collect();
        assert_eq!(row.trim(), "zoom 1x   window starts at -80.0,-48.0");
    }

    #[test]
    fn confirm_steps_the_zoom_through_the_whole_steps_and_round() {
        let mut ctx = ctx();
        let mut screen = scene_camera(&ctx);
        let mut zooms = Vec::new();
        for _ in 0..5 {
            let stay = update(&mut screen, &mut ctx, &frame(&[Action::Confirm], 0.0, &[]));
            assert!(matches!(stay, Transition::None));
            zooms.push(screen.zoom());
            assert_eq!(
                draw(&ctx, &screen).backdrop().unwrap().zoom(),
                screen.zoom()
            );
        }
        assert_eq!(zooms, [2, 3, 4, 1, 2]);
        let sounds = ctx.audio.take();
        let cues: Vec<_> = sounds.iter().map(|s| s.cue()).collect();
        assert_eq!(cues, [Some("menu_select"); 5]);
        // Two presses in one frame are two steps.
        update(
            &mut screen,
            &mut ctx,
            &frame(&[Action::Confirm, Action::Confirm], 0.0, &[]),
        );
        assert_eq!(screen.zoom(), 4);
    }

    #[test]
    fn held_cursor_keys_pan_by_the_pixel_and_stop_at_the_edges() {
        use Action::{CursorDown, CursorLeft, CursorRight, CursorUp};
        let mut ctx = ctx();
        let mut screen = scene_camera(&ctx);
        // At 1× the scene is smaller than the window: nothing to pan.
        update(
            &mut screen,
            &mut ctx,
            &frame(&[], 1.0, &[CursorRight, CursorDown]),
        );
        assert!(near(screen.origin_px(), (-80.0, -48.0)));
        // At 4× the window shows 96 × 56 of 224 × 128, from the middle.
        zoom_to(&mut screen, &mut ctx, 4);
        assert!(near(screen.origin_px(), (64.0, 36.0)));
        // 60 px a second: a tenth of a second is 6 px, half a frame less.
        update(&mut screen, &mut ctx, &frame(&[], 0.1, &[CursorRight]));
        assert!(near(screen.origin_px(), (70.0, 36.0)));
        update(&mut screen, &mut ctx, &frame(&[], 0.0125, &[CursorDown]));
        assert!(near(screen.origin_px(), (70.0, 36.75)));
        update(
            &mut screen,
            &mut ctx,
            &frame(&[], 0.1, &[CursorLeft, CursorUp]),
        );
        assert!(near(screen.origin_px(), (64.0, 30.75)));
        // Opposite keys cancel; a pressed key that isn't held does nothing.
        update(
            &mut screen,
            &mut ctx,
            &frame(&[CursorRight], 0.1, &[CursorLeft, CursorRight]),
        );
        assert!(near(screen.origin_px(), (64.0, 30.75)));
        // Not held: no pan, however long the frame.
        update(&mut screen, &mut ctx, &frame(&[], 5.0, &[]));
        assert!(near(screen.origin_px(), (64.0, 30.75)));
        // Each edge stops the window, and the centre with it: one step
        // back moves at once.
        update(
            &mut screen,
            &mut ctx,
            &frame(&[], 60.0, &[CursorRight, CursorDown]),
        );
        assert!(near(screen.origin_px(), (128.0, 72.0)));
        assert!(near(screen.centre(), (176.0, 100.0)));
        update(
            &mut screen,
            &mut ctx,
            &frame(&[], 0.1, &[CursorLeft, CursorUp]),
        );
        assert!(near(screen.origin_px(), (122.0, 66.0)));
        update(
            &mut screen,
            &mut ctx,
            &frame(&[], 60.0, &[CursorLeft, CursorUp]),
        );
        assert!(near(screen.origin_px(), (0.0, 0.0)));
        assert!(near(screen.centre(), (48.0, 28.0)));
        // A bad frame time moves nothing.
        for dt in [f32::NAN, f32::INFINITY, -1.0] {
            update(
                &mut screen,
                &mut ctx,
                &frame(&[], dt, &[CursorRight, CursorDown]),
            );
            assert!(near(screen.origin_px(), (0.0, 0.0)), "{dt}");
        }
        // The frame shows where the window is.
        let buf = draw(&ctx, &screen);
        assert!(near(buf.backdrop().unwrap().origin_px(), (0.0, 0.0)));
    }

    #[test]
    fn zooming_keeps_the_point_where_the_window_can_show_it() {
        let mut ctx = ctx();
        let mut screen = scene_camera(&ctx);
        zoom_to(&mut screen, &mut ctx, 4);
        pan(&mut screen, &mut ctx, -20.0, 10.0);
        assert!(near(screen.centre(), (92.0, 74.0)));
        // 4× → 1×: the whole scene fits, so the centre is its middle.
        update(&mut screen, &mut ctx, &frame(&[Action::Confirm], 0.0, &[]));
        assert!(near(screen.origin_px(), (-80.0, -48.0)));
        assert!(near(screen.centre(), (112.0, 64.0)));
        // → 2× → 3×: 128 × 74.67 shown, still from the middle.
        update(
            &mut screen,
            &mut ctx,
            &frame(&[Action::Confirm; 2], 0.0, &[]),
        );
        assert!(near(screen.centre(), (112.0, 64.0)));
        // 3× → 4× near the top-left corner: the point is kept.
        pan(&mut screen, &mut ctx, -60.0, -20.0);
        assert!(near(screen.centre(), (64.0, 44.0)));
        update(&mut screen, &mut ctx, &frame(&[Action::Confirm], 0.0, &[]));
        assert!(near(screen.centre(), (64.0, 44.0)));
        assert!(near(screen.origin_px(), (16.0, 16.0)));
        // At 3× the window's middle can't be nearer the edge than half of
        // what it shows: 64 and 37.33.
        zoom_to(&mut screen, &mut ctx, 3);
        pan(&mut screen, &mut ctx, -500.0, -500.0);
        assert!(near(screen.centre(), (64.0, 112.0 / 3.0)));
        assert!(near(screen.origin_px(), (0.0, 0.0)));
    }

    #[test]
    fn cancel_closes_with_the_cancel_sound() {
        let mut ctx = ctx();
        let mut screen = scene_camera(&ctx);
        let pop = update(
            &mut screen,
            &mut ctx,
            &frame(&[Action::Cancel, Action::Confirm], 0.0, &[]),
        );
        assert!(matches!(pop, Transition::Pop));
        assert_eq!(screen.zoom(), 1);
        let sounds = ctx.audio.take();
        let cues: Vec<_> = sounds.iter().map(|s| s.cue()).collect();
        assert_eq!(cues, [Some("menu_cancel")]);
    }

    #[test]
    fn says_so_when_the_map_is_missing() {
        let mut ctx = ctx();
        ctx.game.content.maps.clear();
        let mut screen = scene_camera(&ctx);
        assert!(screen.picture().is_none());
        update(
            &mut screen,
            &mut ctx,
            &frame(&[Action::Confirm], 0.5, &[Action::CursorRight]),
        );
        let buf = draw(&ctx, &screen);
        assert!(buf.backdrop().is_none());
        let row: String = (0..40).map(|x| buf.get(x, 2).unwrap().glyph).collect();
        assert_eq!(row.trim(), "No assets-test/maps/test_small.map");
        assert!(!buf.get(WINDOW.x, WINDOW.y).unwrap().see_through);
    }
}
