//! The Playdate's one-bit pictures (ticket 1040, ADR-0085): which pictures
//! the Playdate draws one-bit, at what size and in which styles, as
//! `docs/design/playdate.md` (*One-bit pictures*, ticket 0062) decided.
//!
//! `build.rs` includes this file and converts every PNG of these kinds
//! into the Playdate's bundle, a one-bit picture at the PNG's own path
//! (`shuyi_assets::bitmap`), so screens and sidecars keep naming the same
//! image; `cargo xtask playdate-images` converts the same pictures into
//! files to look at, and the bought ones into the private checkout.
//!
//! The numbers marked *tunable* are starting values: the tickets that put
//! each kind on screen (1041 the combat picture, 1042 the portraits, 1044
//! the title) may change them.

use std::collections::BTreeMap;

use serde::Deserialize;
use shuyi_assets::bitmap::convert::{Lines, Method, Rgba, convert};
use shuyi_assets::bitmap::{Bitmap, encode};

/// A kind of picture and how the Playdate's version of it is made.
#[derive(Debug, PartialEq)]
pub struct Kind {
    /// What the pictures are, for messages.
    pub name: &'static str,
    /// The folder of `assets/` (and of `assets-private/game/`) whose PNGs,
    /// at any depth, are of this kind.
    pub folder: &'static str,
    /// How much bigger the Playdate shows a picture than its PNG: it is
    /// enlarged to that size before it is dithered.
    pub scale: Scale,
    /// The ways it is made, each a style of its one-bit picture; the first
    /// is the one shown unless a setting picks another.
    pub styles: &'static [Style],
    /// How its PNG is lightened before it is converted, if it is.
    pub lift: Option<Lift>,
    /// How wide a white rim it gets round its figure, in the picture's
    /// pixels ([`with_rim`]); 0 for none.
    pub rim: u16,
}

/// A lightening of a picture before it is converted: each brightness `b`
/// (0 to 1) becomes `scale × b + floor`, so its darkest is `floor` grey.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Lift {
    /// What each brightness is multiplied by.
    pub scale: f32,
    /// What is added to it.
    pub floor: f32,
}

impl Lift {
    /// `rgba`'s pixels lightened: each colour channel the same way, which
    /// lifts their brightness (a sum of the channels) the same way; the
    /// alpha kept.
    pub fn apply(self, rgba: &[u8]) -> Vec<u8> {
        let channel = |v: u8| {
            let lifted = (self.scale * f32::from(v) + self.floor * 255.0).round();
            // Clamped to a byte first.
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let byte = lifted.clamp(0.0, 255.0) as u8;
            byte
        };
        rgba.chunks(4)
            .flat_map(|p| {
                p.iter()
                    .enumerate()
                    .map(move |(i, &v)| if i == 3 { v } else { channel(v) })
            })
            .collect()
    }
}

impl Kind {
    /// Each style's method, in order.
    pub fn methods(&self) -> Vec<Method> {
        self.styles.iter().map(|style| style.method).collect()
    }
}

/// One way a kind of picture is made.
#[derive(Debug, PartialEq)]
pub struct Style {
    /// Its name in file names, e.g. `a-lines`.
    pub file: &'static str,
    /// How it is made.
    pub method: Method,
}

/// A style of `method`, named `file`.
const fn style(file: &'static str, method: Method) -> Style {
    Style { file, method }
}

/// How much bigger than its PNG a picture is shown.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scale {
    /// The first number divided by the second.
    Times(u32, u32),
    /// Twice the `scale` of the battler sidecar that names it, as the
    /// combat scene doubles every fighter (`assets/battlers/README.md`).
    Battler,
}

/// Style A (A1): the brightness stretched so the darkest 3% is black and
/// the lightest 3% white, then the 4 × 4 Bayer dither.
pub const STYLE_A: Method = Method::BAYER;

/// The art's own dark lines: an art pixel clearly darker than the
/// brightest one around it, or very dark (*tunable*).
const LINES: Lines = Lines {
    contrast: 0.3,
    dark: 0.15,
};

/// The lighter picture under the line styles: brightness curve 0.8.
const LIGHTER: f32 = 0.8;

/// Style A+ lines: A on the lighter picture, its dark lines on top.
pub const STYLE_A_LINES: Method = Method {
    curve: LIGHTER,
    lines: Some(LINES),
    ..STYLE_A
};

/// Style F+ lines: A+ lines with each grey first rounded to one of 5
/// levels.
pub const STYLE_F_LINES: Method = Method {
    levels: 5,
    ..STYLE_A_LINES
};

/// Style G: as F+ lines with 4 levels, bolder and flatter.
pub const STYLE_G: Method = Method {
    levels: 4,
    ..STYLE_A_LINES
};

/// A title backdrop, lightened (*tunable*: how much) and dithered (1044's
/// to judge; a battle backdrop is [`BATTLE_BACKDROP`]).
pub const BACKDROP: Method = Method {
    curve: LIGHTER,
    ..STYLE_A
};

/// A battle backdrop, lightened as the mockup Nick chose in 0062 had it
/// (ticket 1041): each brightness `b` becomes `0.6 b + 0.47`, so the
/// darkest is a middle grey and the backdrop stays a light pattern behind
/// the fighters (*tunable*).
pub const BATTLE_LIFT: Lift = Lift {
    scale: 0.6,
    floor: 120.0 / 255.0,
};

/// A battle backdrop after [`BATTLE_LIFT`]: dithered as it is, with no
/// stretch, which would make its darkest black again.
pub const BATTLE_BACKDROP: Method = Method {
    stretch: 0.0,
    ..STYLE_A
};

/// A title or battle backdrop (336 × 248) enlarged to the screen's width,
/// 400 pixels (*tunable*: how the screen that shows it lays it out).
const SCREEN_WIDE: Scale = Scale::Times(25, 21);

/// The white rim round a fighter, in screen pixels
/// (`docs/design/playdate.md`, *The combat picture*).
pub const FIGHTER_RIM: u16 = 2;

/// Every kind of picture the Playdate draws one-bit. A PNG of `assets/`
/// outside these folders stays a PNG in its bundle (the font's atlas,
/// the title's logo, the map's pictures), and a sprite of it is drawn as
/// a grey box until its own ticket gives it a method.
pub const KINDS: &[Kind] = &[
    // Q1: style A by default, and the Playdate's portrait style setting
    // (1042) picks one of the four, in this order. The bust at 2×.
    Kind {
        name: "portrait",
        folder: "portraits",
        scale: Scale::Times(2, 1),
        styles: &[
            style("a", STYLE_A),
            style("a-lines", STYLE_A_LINES),
            style("f-lines", STYLE_F_LINES),
            style("g", STYLE_G),
        ],
        lift: None,
        rim: 0,
    },
    // Q2: the fighters in style A only, at the PC's double size; one at
    // 1.25× is enlarged smoothly first. A white rim of 2 pixels keeps the
    // tops of their heads apart from the backdrop (ticket 1041).
    Kind {
        name: "combat picture",
        folder: "battlers",
        scale: Scale::Battler,
        styles: &[style("a", STYLE_A)],
        lift: None,
        rim: FIGHTER_RIM,
    },
    // Q2: the battle backdrop, lightened and dithered.
    Kind {
        name: "battle backdrop",
        folder: "battlebacks",
        scale: SCREEN_WIDE,
        styles: &[style("a", BATTLE_BACKDROP)],
        lift: Some(BATTLE_LIFT),
        rim: 0,
    },
    // Q3 and 1044: the title's backdrops, dithered like the card's.
    Kind {
        name: "title backdrop",
        folder: "title",
        scale: SCREEN_WIDE,
        styles: &[style("a", BACKDROP)],
        lift: None,
        rim: 0,
    },
];

/// The title's pictures that are not backdrops: its logo and the Game
/// Over words, a lettering of solid blocks that 1044 makes one-bit its
/// own way (white letters, a black shadow and rim), not by dithering.
const NOT_BACKDROPS: [&str; 2] = ["title/logo.png", "title/game_over.png"];

/// The kind of the picture at `path` (relative to `assets/`), or `None`
/// if the Playdate keeps it as it is.
pub fn kind_of(path: &str) -> Option<&'static Kind> {
    let png = path
        .rsplit_once('.')
        .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("png"));
    if !png || NOT_BACKDROPS.contains(&path) {
        return None;
    }
    KINDS.iter().find(|kind| {
        path.strip_prefix(kind.folder)
            .is_some_and(|rest| rest.starts_with('/'))
    })
}

/// The touched-up one-bit picture of the PNG at `path` (ticket 1042): a
/// `<stem>.1bit` file beside it among `files` (sorted), which the
/// Playdate uses as it is instead of converting the PNG. `None` if there
/// is none, or `path` is no picture the Playdate draws one-bit.
pub fn touched_up(path: &str, files: &[String]) -> Option<String> {
    kind_of(path)?;
    let (stem, _) = path.rsplit_once('.')?;
    let file = format!("{stem}.{}", shuyi_assets::bitmap::EXTENSION);
    files.binary_search(&file).is_ok().then_some(file)
}

/// The touched-up pictures among `files` (sorted): each PNG's path and
/// its [`touched_up`] file's.
pub fn touch_ups(files: &[String]) -> BTreeMap<String, String> {
    files
        .iter()
        .filter_map(|path| Some((path.clone(), touched_up(path, files)?)))
        .collect()
}

/// The one-bit picture file of the `width` × `height` picture of `rgba`
/// pixels, made as `job` says: lightened if its kind is, converted by its
/// kind's methods, then given its kind's rim.
pub fn make(width: u32, height: u32, rgba: &[u8], job: &Job<'_>) -> Result<Vec<u8>, String> {
    let lifted = job.kind.lift.map(|lift| lift.apply(rgba));
    let pixels = lifted.as_deref().unwrap_or(rgba);
    let image = Rgba::new(width, height, pixels).ok_or("decoded to the wrong number of pixels")?;
    let file = convert(&image, job.scale, &job.kind.methods())?;
    with_rim(&file, job.kind.rim)
}

/// One-bit picture `file` with a white rim `width` pixels wide round what
/// its mask shows: each see-through pixel that near a shown one (by a
/// round brush, as 0062's mockups drew it) is shown, white in every
/// style. The rim stops at the picture's edge, which keeps its size.
pub fn with_rim(file: &[u8], width: u16) -> Result<Vec<u8>, String> {
    let bitmap = Bitmap::parse(file)?;
    let Some(mask) = bitmap.mask().filter(|_| width > 0) else {
        return Ok(file.to_vec());
    };
    let (w, h) = (usize::from(bitmap.width()), usize::from(bitmap.height()));
    let stride = Bitmap::stride(bitmap.width());
    let index = |x: usize, y: usize| (y * stride + x / 8, 7 - x % 8);
    let shown = |x: usize, y: usize| {
        let (byte, bit) = index(x, y);
        mask[byte] >> bit & 1 == 1
    };
    let r = usize::from(width);
    let near = |x: usize, y: usize| {
        (y.saturating_sub(r)..=(y + r).min(h - 1)).any(|ny| {
            (x.saturating_sub(r)..=(x + r).min(w - 1)).any(|nx| {
                let (dx, dy) = (x.abs_diff(nx), y.abs_diff(ny));
                dx * dx + dy * dy <= r * r + r && shown(nx, ny)
            })
        })
    };
    let mut grown = mask.to_vec();
    for y in 0..h {
        for x in (0..w).filter(|&x| near(x, y)) {
            let (byte, bit) = index(x, y);
            grown[byte] |= 1 << bit;
        }
    }
    let styles: Vec<Vec<u8>> = (0..bitmap.styles())
        .filter_map(|n| bitmap.style(n))
        .map(|rows| {
            rows.iter()
                .zip(mask.iter().zip(&grown))
                .map(|(&row, (&old, &new))| row | (new & !old))
                .collect()
        })
        .collect();
    let planes: Vec<&[u8]> = styles.iter().map(Vec::as_slice).collect();
    let size = (bitmap.width(), bitmap.height());
    Ok(encode(size, bitmap.source_size(), &planes, Some(&grown))?)
}

/// A picture to convert: its path, how much it is enlarged and its kind.
#[derive(Debug, PartialEq)]
pub struct Job<'a> {
    /// Relative to `assets/` (or `assets-private/game/`).
    pub path: &'a str,
    /// Enlarged the first number divided by the second times.
    pub scale: (u32, u32),
    /// Its kind: its styles.
    pub kind: &'static Kind,
}

/// The pictures among `files` (paths relative to a folder laid out as
/// `assets/`) the Playdate draws one-bit, in `files`' order. `read`
/// gives a file's text (the battler sidecars that set a fighter's scale).
pub fn jobs(
    files: &[String],
    read: impl Fn(&str) -> Option<String>,
) -> Result<Vec<Job<'_>>, String> {
    let scales = battler_scales(files, read)?;
    files
        .iter()
        .filter_map(|path| Some((path.as_str(), kind_of(path)?)))
        .map(|(path, kind)| {
            let scale = match kind.scale {
                Scale::Times(num, den) => (num, den),
                Scale::Battler => {
                    let (num, den) = scales.get(path).copied().unwrap_or((1, 1));
                    (2 * num, den)
                }
            };
            Ok(Job { path, scale, kind })
        })
        .collect()
}

/// What a battler sidecar says about its picture.
#[derive(Deserialize)]
struct Sidecar {
    image: String,
    #[serde(default = "same_size")]
    scale: (u32, u32),
}

const fn same_size() -> (u32, u32) {
    (1, 1)
}

/// The scale of each battler picture (by path) that a sidecar among
/// `files` names. Two sidecars may share a picture only at one scale: the
/// Playdate's picture is made at one size.
fn battler_scales(
    files: &[String],
    read: impl Fn(&str) -> Option<String>,
) -> Result<BTreeMap<String, (u32, u32)>, String> {
    let mut scales: BTreeMap<String, (u32, u32)> = BTreeMap::new();
    for path in files {
        let Some(name) = path.strip_prefix("battlers/") else {
            continue;
        };
        let ron = name
            .rsplit_once('.')
            .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("ron"));
        if !ron || name.contains('/') {
            continue;
        }
        let text = read(path).ok_or_else(|| format!("{path}: can't be read"))?;
        let sidecar: Sidecar = shuyi_assets::parse_ron(path, &text).map_err(|e| e.to_string())?;
        let image = format!("battlers/{}", sidecar.image);
        match scales.insert(image.clone(), sidecar.scale) {
            Some(other) if other != sidecar.scale => {
                return Err(format!(
                    "{path}: {image} is shown at scale {:?} here and {other:?} by another \
                     sidecar; the Playdate's picture has one size",
                    sidecar.scale
                ));
            }
            _ => {}
        }
    }
    Ok(scales)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn paths(list: &[&str]) -> Vec<String> {
        list.iter().map(|p| (*p).to_string()).collect()
    }

    #[test]
    fn each_folder_is_its_kind() {
        let name = |path: &str| kind_of(path).map(|k| k.name);
        assert_eq!(name("portraits/lead_f/happy.png"), Some("portrait"));
        assert_eq!(name("battlers/brigand.png"), Some("combat picture"));
        assert_eq!(name("battlebacks/fields.png"), Some("battle backdrop"));
        assert_eq!(name("title/fields_b_dusk.PNG"), Some("title backdrop"));
        assert_eq!(name("title/logo.png"), None);
        assert_eq!(name("title/game_over.png"), None);
        assert_eq!(name("title/title.ron"), None);
        assert_eq!(name("battlers/brigand.ron"), None);
        assert_eq!(name("portraitsx/a.png"), None);
        assert_eq!(name("fonts/atlas.png"), None);
        assert_eq!(name("images/effect_marks.png"), None);
    }

    #[test]
    fn portraits_come_in_the_four_styles_in_order() {
        let portrait = kind_of("portraits/a/b.png").unwrap();
        let methods = [STYLE_A, STYLE_A_LINES, STYLE_F_LINES, STYLE_G];
        assert_eq!(portrait.methods(), methods);
        let files: Vec<&str> = portrait.styles.iter().map(|s| s.file).collect();
        assert_eq!(files, ["a", "a-lines", "f-lines", "g"]);
        assert_eq!(STYLE_A.lines, None);
        assert_eq!((STYLE_A_LINES.curve, STYLE_A_LINES.levels), (0.8, 0));
        assert!(STYLE_A_LINES.lines.is_some());
        assert_eq!((STYLE_F_LINES.levels, STYLE_G.levels), (5, 4));
        assert_eq!(STYLE_G.lines, STYLE_A_LINES.lines);
        // Title backdrops: style A on a lighter picture.
        assert_eq!(
            BACKDROP,
            Method {
                curve: 0.8,
                ..STYLE_A
            }
        );
        assert_ne!(BACKDROP, STYLE_A);
        // Battle backdrops: lifted first, then dithered with no stretch.
        let battle = kind_of("battlebacks/fields.png").unwrap();
        assert_eq!(battle.methods(), [BATTLE_BACKDROP]);
        let unstretched = Method {
            stretch: 0.0,
            ..STYLE_A
        };
        assert_eq!(BATTLE_BACKDROP, unstretched);
        assert_eq!(battle.lift, Some(BATTLE_LIFT));
        assert_eq!(KINDS.iter().filter(|k| k.lift.is_some()).count(), 1);
    }

    #[test]
    fn a_lift_lightens_each_colour_and_keeps_the_alpha() {
        let lift = BATTLE_LIFT;
        // Black becomes a middle grey (120), white stays white (clamped).
        assert_eq!(
            lift.apply(&[0, 0, 0, 7, 255, 255, 255, 255]),
            [120, 120, 120, 7, 255, 255, 255, 255]
        );
        assert_eq!(lift.apply(&[100, 50, 10, 0]), [180, 150, 126, 0]);
    }

    #[test]
    fn a_touched_up_picture_is_the_1bit_file_beside_its_png() {
        let files = paths(&[
            "battlers/brigand.png",
            "portraits/a/happy.1bit",
            "portraits/a/happy.png",
            "portraits/a/sad.png",
            "title/logo.1bit",
            "title/logo.png",
        ]);
        assert_eq!(
            touched_up("portraits/a/happy.png", &files).as_deref(),
            Some("portraits/a/happy.1bit")
        );
        assert_eq!(touched_up("portraits/a/sad.png", &files), None);
        assert_eq!(touched_up("battlers/brigand.png", &files), None);
        // Only a picture the Playdate draws one-bit.
        assert_eq!(touched_up("title/logo.png", &files), None);
        assert_eq!(touched_up("portraits/a/happy.1bit", &files), None);
        let all = touch_ups(&files);
        assert_eq!(
            all.into_iter().collect::<Vec<_>>(),
            [(
                "portraits/a/happy.png".to_string(),
                "portraits/a/happy.1bit".to_string()
            )]
        );
    }

    #[test]
    fn a_fighter_is_shown_at_twice_its_sidecars_scale() {
        let files = paths(&[
            "battlers/README.md",
            "battlers/brigand.png",
            "battlers/brigand.ron",
            "battlers/default.png",
            "battlers/default.ron",
            "battlers/unused.png",
            "portraits/lead_f/happy.png",
            "title/logo.png",
        ]);
        let read = |path: &str| match path {
            "battlers/brigand.ron" => {
                Some("(id: \"brigand\", image: \"brigand.png\", faces: Left, scale: (5, 4))".into())
            }
            "battlers/default.ron" => Some("(id: \"default\", image: \"default.png\")".into()),
            _ => None,
        };
        let jobs = jobs(&files, read).unwrap();
        let scales: Vec<(&str, (u32, u32))> = jobs.iter().map(|j| (j.path, j.scale)).collect();
        assert_eq!(
            scales,
            [
                ("battlers/brigand.png", (10, 4)),
                ("battlers/default.png", (2, 1)),
                ("battlers/unused.png", (2, 1)),
                ("portraits/lead_f/happy.png", (2, 1)),
            ]
        );
        assert_eq!(jobs[3].kind.name, "portrait");
    }

    #[test]
    fn a_picture_shared_at_two_scales_or_a_bad_sidecar_is_an_error() {
        let files = paths(&["battlers/a.ron", "battlers/b.ron"]);
        let read = |path: &str| {
            Some(if path == "battlers/a.ron" {
                "(image: \"x.png\", scale: (5, 4))".to_string()
            } else {
                "(image: \"x.png\")".to_string()
            })
        };
        let error = jobs(&files, read).unwrap_err();
        assert!(
            error.starts_with("battlers/b.ron: battlers/x.png is shown"),
            "{error}"
        );
        let same = |_: &str| Some("(image: \"x.png\", scale: (5, 4))".to_string());
        assert!(jobs(&files, same).is_ok());
        let broken = |_: &str| Some("(image: ".to_string());
        assert!(
            jobs(&files, broken)
                .unwrap_err()
                .starts_with("battlers/a.ron")
        );
        assert_eq!(
            jobs(&files, |_| None).unwrap_err(),
            "battlers/a.ron: can't be read"
        );
        // Sidecars below the folder are no battler's.
        let deeper = paths(&["battlers/old/a.ron"]);
        assert_eq!(jobs(&deeper, |_| None), Ok(vec![]));
    }

    /// A 7 × 7 one-bit picture, black, showing only the pixels `shown`.
    fn dots(shown: &[(usize, usize)]) -> Vec<u8> {
        let mut mask = [0u8; 7];
        for &(x, y) in shown {
            mask[y] |= 0x80 >> x;
        }
        encode((7, 7), (7, 7), &[&[0; 7]], Some(&mask)).unwrap()
    }

    /// Each row of `file`'s first style: `#` black, `.` white, ` ` see-through.
    fn drawn(file: &[u8]) -> Vec<String> {
        let bitmap = Bitmap::parse(file).unwrap();
        (0..bitmap.height())
            .map(|y| {
                (0..bitmap.width())
                    .map(|x| match bitmap.pixel(0, x, y) {
                        Some(false) => '#',
                        Some(true) => '.',
                        None => ' ',
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn a_rim_is_a_round_brush_of_white_round_the_figure() {
        let file = with_rim(&dots(&[(3, 3)]), 2).unwrap();
        let rows = [
            "       ", "  ...  ", " ..... ", " ..#.. ", " ..... ", "  ...  ", "       ",
        ];
        assert_eq!(drawn(&file), rows);
        // It stops at the picture's edge, which keeps its size.
        let corner = with_rim(&dots(&[(0, 0)]), 2).unwrap();
        assert_eq!(drawn(&corner)[..3], ["#..    ", "...    ", "..     "]);
        assert_eq!(bitmap_size(&corner), (7, 7));
        // No rim, or no mask: the file as it was.
        assert_eq!(with_rim(&dots(&[(3, 3)]), 0).unwrap(), dots(&[(3, 3)]));
        let solid = encode((7, 1), (7, 1), &[&[0]], None).unwrap();
        assert_eq!(with_rim(&solid, 2).unwrap(), solid);
        assert!(with_rim(b"not a picture", 2).is_err());
    }

    fn bitmap_size(file: &[u8]) -> (u16, u16) {
        let bitmap = Bitmap::parse(file).unwrap();
        (bitmap.width(), bitmap.height())
    }

    #[test]
    fn only_fighters_get_a_rim() {
        let rim = |path: &str| kind_of(path).unwrap().rim;
        assert_eq!(rim("battlers/brigand.png"), FIGHTER_RIM);
        assert_eq!(FIGHTER_RIM, 2);
        for path in ["portraits/a/b.png", "battlebacks/fields.png", "title/x.png"] {
            assert_eq!(rim(path), 0, "{path}");
        }
    }
}
