//! The title's look on the Playdate (ADR-0038: a look's tests read cells,
//! items and pixels): where the backdrop, the logo, the menu and the help
//! go on the 400 × 240 screen, and the title's picture through the
//! one-bit renderer. What the title shows is tested on its view
//! (`title/tests.rs`, `tests/it/playdate.rs`).

use std::rc::Rc;

use shuyi_assets::bitmap::{Bitmap, encode};
use trpg_content::playdate_images::{self as images, lettering};

use super::*;
use crate::cinematic::CinematicPlayer;
use crate::console::{ConsoleSizes, ConsoleSpec};
use crate::map_view::playdate::tests::font;
use crate::onebit::{FRAME_BYTES, Frame, Image, Images, is_white, render};
use crate::screens::playdate::tests::{blank, find, is_bar, pd_ctx, rows};
use crate::screens::title::view::CinematicView;
use crate::widgets::{MenuItemView, MenuView};

/// The screen's pixels above its bottom row.
const PICTURE: PxRect = PxRect::new(0, 0, 400, 224);

fn backdrop(c: &Ctx) -> ImageId {
    c.game.content.title.backdrops[0]
}

fn logo(c: &Ctx) -> (ImageId, (u32, u32)) {
    c.game.content.title.logo.unwrap()
}

fn item(label: &str, focused: bool, enabled: bool) -> MenuItemView {
    MenuItemView {
        label: label.into(),
        suffix: None,
        note: None,
        enabled,
        focused,
    }
}

/// The title with its menu as on the Playdate (no Quit), New Game focused
/// and Load Game greyed out.
fn view(c: &Ctx) -> TitleView {
    let labels = ["New Game", "Load Game", "Options", "Profile", "Credits"];
    let items = labels
        .iter()
        .enumerate()
        .map(|(i, l)| item(l, i == 0, i != 1))
        .collect();
    TitleView {
        cinematic: None,
        picture: FULL,
        menu_opacity: FULL,
        backdrop: Some(backdrop(c)),
        sparks: Some(1234),
        prompt: None,
        menu: Some(MenuView {
            items,
            first_shown: 0,
            shown: 5,
        }),
        notice: None,
        note: None,
        help: Some("✛ move · Ⓐ select".to_owned()),
        debug_hint: None,
    }
}

fn painted(c: &Ctx, view: &TitleView) -> GlyphBuffer {
    let mut buf = blank();
    paint(c, view, &mut buf);
    buf
}

/// Test pictures: the backdrop black all over; the logo as the converter
/// makes a lettering of one letter pixel, the PNG's (4, 0).
struct Pictures(Vec<(ImageId, Vec<u8>)>);

impl Pictures {
    fn of(c: &Ctx) -> Self {
        let dark = vec![0; Bitmap::plane_bytes(400, 295)];
        let back = encode((400, 295), (336, 248), &[&dark], None).unwrap();
        let mut rgba = vec![0; 52 * 31 * 4];
        rgba[4 * 4..4 * 4 + 4].copy_from_slice(&[0xf0, 0xdc, 0xa0, 255]);
        let block = images::LETTER_BLOCK;
        let letters = lettering(52, 31, &rgba, block).unwrap();
        Self(vec![(backdrop(c), back), (logo(c).0, letters)])
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
    let map = crate::onebit::playdate_map().unwrap();
    render(buf, &font(), &Pictures::of(c), &map, &mut frame);
    frame
}

#[test]
fn the_logo_and_the_backdrop_are_drawn_at_the_sizes_the_converter_makes() {
    let c = pd_ctx();
    assert_eq!(LETTER_BLOCK, i32::try_from(images::LETTER_BLOCK).unwrap());
    // The logo: its 52 × 31 blocks and a block of rim round them, 3 pixels
    // a block, centred at the top.
    let (image, size) = logo(&c);
    assert_eq!(size, (52, 31), "the shipped logo");
    let sprite = logo_sprite(image, size, 400);
    assert_eq!(sprite.src, PxRect::new(0, 0, 52, 31));
    assert_eq!(sprite.dest, PxRect::new(119, 2, 162, 99));
    let made = lettering(52, 31, &vec![0; 52 * 31 * 4], images::LETTER_BLOCK).unwrap();
    let made = Bitmap::parse(&made).unwrap();
    assert_eq!((made.width(), made.height()), (162, 99));
    // The backdrop: 336 × 248 enlarged to the screen's width, its middle
    // rows above the bottom row.
    let sprite = backdrop_sprite(backdrop(&c), PICTURE);
    assert_eq!(sprite.src, PxRect::new(0, 0, 336, 248));
    assert_eq!(sprite.dest, PxRect::new(0, -35, 400, 295));
    let files = ["title/fields_b_day.png".to_owned()];
    let job = &images::jobs(&files, |_| None).unwrap()[0];
    assert_eq!(job.scale, BACKDROP_SCALE);
    assert_eq!(
        (enlarged(336, job.scale), enlarged(248, job.scale)),
        (400, 295)
    );
}

#[test]
fn around_a_hole_is_up_to_four_rectangles() {
    let area = PxRect::new(0, 0, 400, 224);
    assert_eq!(around(area, None), [area]);
    let hole = PxRect::new(100, 112, 200, 112);
    assert_eq!(
        around(area, Some(hole)),
        [
            PxRect::new(0, 0, 400, 112),
            PxRect::new(0, 112, 100, 112),
            PxRect::new(300, 112, 100, 112),
        ]
    );
    // A hole in the middle: above, below, left and right of it.
    let middle = PxRect::new(100, 50, 200, 50);
    assert_eq!(
        around(area, Some(middle)),
        [
            PxRect::new(0, 0, 400, 50),
            PxRect::new(0, 100, 400, 124),
            PxRect::new(0, 50, 100, 50),
            PxRect::new(300, 50, 100, 50),
        ]
    );
    // An area away from the screen's corner.
    let inset = PxRect::new(10, 20, 100, 100);
    assert_eq!(
        around(inset, Some(PxRect::new(30, 50, 40, 20))),
        [
            PxRect::new(10, 20, 100, 30),
            PxRect::new(10, 70, 100, 50),
            PxRect::new(10, 50, 20, 20),
            PxRect::new(70, 50, 40, 20),
        ]
    );
    // A hole past the area leaves it whole.
    let below = PxRect::new(0, 224, 400, 16);
    assert_eq!(around(area, Some(below)), [area]);
}

/// One background shot: the title's picture alone, as the cinematic's
/// logo shot paints it.
#[test]
fn the_titles_picture_is_the_backdrop_and_the_logo_above_a_white_bottom_row() {
    let c = pd_ctx();
    let mut buf = blank();
    paint_logo(&c, Some(backdrop(&c)), None, &mut buf);
    let (image, size) = logo(&c);
    let mut back = backdrop_sprite(backdrop(&c), PICTURE);
    back.clip = PICTURE;
    assert_eq!(buf.sprites(), [back, logo_sprite(image, size, 400)]);
    assert!(rows(&buf).iter().all(String::is_empty), "no text");
    let frame = shown(&c, &buf);
    // The backdrop (black here) down to the bottom row, which is white.
    assert!(!is_white(&frame, 10, 100) && !is_white(&frame, 399, 223));
    assert!(is_white(&frame, 10, 224) && is_white(&frame, 399, 239));
    // The logo's letter pixel: a white block 3 pixels square at block
    // (5, 1), a black rim round it, over the backdrop.
    let (x, y) = (119 + 5 * 3, 2 + 3);
    assert!(is_white(&frame, x, y) && is_white(&frame, x + 2, y + 2));
    assert!(!is_white(&frame, x - 1, y) && !is_white(&frame, x + 3, y + 3));
    // On no backdrop, the rim is black on the white ground.
    let mut bare = blank();
    paint_logo(&c, None, None, &mut bare);
    let frame = shown(&c, &bare);
    assert!(!is_white(&frame, x - 1, y) && !is_white(&frame, x, y - 1));
    assert!(is_white(&frame, x, y) && is_white(&frame, x - 4, y));
}

/// The menu: a window over the backdrop, its bottom on the row over the
/// help line.
#[test]
fn the_menu_is_a_window_over_the_backdrop_above_the_help_line() {
    let c = pd_ctx();
    let buf = painted(&c, &view(&c));
    let rows = rows(&buf);
    assert_eq!(rows[14], "✛ move · Ⓐ select");
    let window = Rect::new(18, 7, 13, 7);
    assert!(rows[7].starts_with("╔══"), "{}", rows[7]);
    assert_eq!(rows[13].chars().next(), Some('╚'));
    for (y, label) in (8..).zip(["New Game", "Load Game", "Options", "Profile", "Credits"]) {
        assert_eq!(find(&buf, y, label), Some(window.x + 2), "{label}");
    }
    assert!(is_bar(&buf, 8, "New Game"));
    assert!(!is_bar(&buf, 9, "Load Game"));
    assert_ne!(buf.get(20, 9).map(|c| c.fg), buf.get(20, 10).map(|c| c.fg));
    // Neither picture is drawn over the window.
    let hole = px_rect(window);
    for sprite in buf.sprites() {
        assert_eq!(sprite.clip.intersect(&hole), None, "{sprite:?}");
    }
    let frame = shown(&c, &buf);
    // Inside the window white; beside it the backdrop.
    let (x, y) = (window.x * 8 + 92, 12 * 16 + 8);
    assert!(is_white(&frame, x, y), "inside the window");
    assert!(!is_white(&frame, window.x * 8 - 4, y), "beside it");
    // A notice takes the help line's place.
    let mut v = view(&c);
    v.notice = Some("The suspended battle can't be read".to_owned());
    assert_eq!(rows_of(&c, &v)[14], "The suspended battle can't be read");
}

fn rows_of(c: &Ctx, view: &TitleView) -> Vec<String> {
    rows(&painted(c, view))
}

#[test]
fn the_prompt_is_a_window_of_its_own_in_the_menus_place() {
    let c = pd_ctx();
    let mut v = view(&c);
    v.menu = None;
    v.help = None;
    v.prompt = Some("Press any button".to_owned());
    let buf = painted(&c, &v);
    let rows = rows(&buf);
    assert_eq!(rows[12], "║ Press any button ║");
    assert_eq!(find(&buf, 12, "Press"), Some(17));
    assert_eq!(rows[14], "");
    // Too long for the screen: the window as wide as it, the line cut
    // inside its frame.
    v.prompt = Some("Press any button ".repeat(4));
    let rows = rows_of(&c, &v);
    assert!(
        rows[12].starts_with("║ Press") && rows[12].ends_with("… ║"),
        "{}",
        rows[12]
    );
    let hole = px_rect(Rect::new(15, 11, 20, 3));
    for sprite in buf.sprites() {
        assert_eq!(sprite.clip.intersect(&hole), None, "{sprite:?}");
    }
}

/// One bit has no fades: the menu shows while it is more than half solid;
/// the cinematic once it is more than half bright.
#[test]
fn the_menu_and_the_cinematic_cut_in_halfway_through_their_fades() {
    let c = pd_ctx();
    let mut v = view(&c);
    v.menu_opacity = 128;
    assert!(rows_of(&c, &v)[8].contains("New Game"));
    v.menu_opacity = 127;
    let buf = painted(&c, &v);
    assert!(rows(&buf).iter().all(String::is_empty), "{:?}", rows(&buf));
    assert_eq!(buf.sprites()[0].clip, PICTURE, "the backdrop whole");
    // The cinematic's first shot, the map, over the title's picture.
    let player = CinematicPlayer::new(&c, "title", None).unwrap();
    let shot = |brightness| CinematicView {
        player: Rc::new(player.clone()),
        t_ms: 70_000,
        brightness,
    };
    v.cinematic = Some(shot(128));
    let buf = painted(&c, &v);
    let (image, _) = logo(&c);
    assert!(buf.sprites().iter().all(|s| s.image != image), "no logo");
    // The moment the view says: 70 s in, the brigand's attack.
    let mut fight = blank();
    player.draw(&c, 70.0, 1.0, &mut fight);
    assert_eq!(rows(&buf), rows(&fight));
    assert_eq!(buf.sprites(), fight.sprites());
    v.cinematic = Some(shot(127));
    let buf = painted(&c, &v);
    assert!(buf.sprites().iter().any(|s| s.image == image), "the logo");
    // Going to black (ticket 0843): the title's picture while it is more
    // than half solid, then black, the cinematic not yet bright enough.
    v.menu_opacity = 0;
    for cinematic in [None, Some(shot(127))] {
        v.cinematic = cinematic;
        v.picture = 128;
        let buf = painted(&c, &v);
        assert!(buf.sprites().iter().any(|s| s.image == image), "the logo");
        v.picture = 127;
        let buf = painted(&c, &v);
        assert_eq!(buf.sprites().len(), 0);
        let ink = pd::inks(&c);
        for y in 0..i32::from(buf.height()) {
            for x in 0..i32::from(buf.width()) {
                let cell = buf.get(x, y).unwrap();
                assert_eq!((cell.glyph, cell.bg), (' ', ink.text), "({x}, {y})");
            }
        }
    }
}

/// The cinematic on the Playdate's skins: the map at half the file's zoom,
/// the fight on the combat picture, the logo the Playdate's.
#[test]
fn the_cinematics_shots_are_painted_in_the_playdates_look() {
    let mut c = pd_ctx();
    c.console = ConsoleSpec::PLAYDATE;
    let player = CinematicPlayer::new(&c, "title", None).unwrap();
    let screen = Rect::new(0, 0, 50, 15);
    let draw = |t: f32| {
        let mut buf = blank();
        player.draw(&c, t, 1.0, &mut buf);
        buf
    };
    // The first shot: 3× in the file.
    let frame = player.frame(&c, 1.0, screen);
    assert!(
        matches!(frame, crate::cinematic::Frame::MapPan { zoom: 2, .. }),
        "{frame:?}"
    );
    assert!(draw(1.0).backdrop().is_some(), "the map");
    // The brigand's attack (69.95 s on): the plates along the bottom.
    let fight = draw(70.0);
    let corner = fight.get(0, 12).map(|c| c.glyph);
    assert!(matches!(corner, Some('┌' | '╔')), "{corner:?}");
    // The logo: the Playdate's sprite, on no backdrop here.
    let (image, size) = logo(&c);
    assert_eq!(draw(118.0).sprites(), [logo_sprite(image, size, 400)]);
}
