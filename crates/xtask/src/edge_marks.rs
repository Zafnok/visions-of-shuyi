//! `cargo xtask edge-marks`: writes `assets/images/edge_marks.png`, the
//! arrows a sprite map skin puts at the edges of the map's view where the
//! map goes on past it (ticket 0449; `docs/design/look-and-feel.md`,
//! *Map edges*).
//!
//! 42×13: a **right**, a **left**, a **down** and an **up** arrow, each a
//! triangle 6 deep and 11 across in the palette's `cursor` colour with a
//! 1-pixel edge in its `black`: the right and the left arrow in boxes of
//! 8×13, the down and the up arrow in boxes of 13×8 along the top. It is
//! our own picture, made from the palette: rerun the command after
//! changing those colours.

use std::fs;
use std::path::Path;

use trpg_content::bundle::display_path;
use trpg_content::image::EDGE_MARKS_PATH;

use shuyi_cli::codec::encode_png;

/// How far an arrow reaches out from its base, in pixels.
pub const DEPTH: u32 = 6;
/// How wide its base is, in pixels.
pub const BASE: u32 = 11;
/// The image's size: four boxes of the arrow and its edge side by side.
pub const SIZE: (u32, u32) = (2 * (DEPTH + 2) + 2 * (BASE + 2), BASE + 2);
/// The palette names of the arrows' and the edge's colours.
pub const COLORS: [&str; 2] = ["cursor", "black"];

/// Which way an arrow points, in the image's order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Way {
    Right,
    Left,
    Down,
    Up,
}

impl Way {
    const ALL: [Way; 4] = [Way::Right, Way::Left, Way::Down, Way::Up];

    /// Its box in the image: left x, width, height.
    fn sides(self) -> (u32, u32, u32) {
        let (deep, wide) = (DEPTH + 2, BASE + 2);
        match self {
            Way::Right => (0, deep, wide),
            Way::Left => (deep, deep, wide),
            Way::Down => (2 * deep, wide, deep),
            Way::Up => (2 * deep + wide, wide, deep),
        }
    }

    /// Whether pixel `(x, y)` of its box is on the arrow.
    fn on(self, x: i64, y: i64) -> bool {
        let deep = i64::from(DEPTH) + 1;
        // Along the base (`along`), and out from it (`out`, 1 at the base).
        let (along, out) = match self {
            Way::Right => (y, x),
            Way::Left => (y, deep - x),
            Way::Down => (x, y),
            Way::Up => (x, deep - y),
        };
        let row = along - 1;
        let half = (i64::from(BASE) - 1) / 2;
        (0..i64::from(BASE)).contains(&row) && out >= 1 && out <= deep - (row - half).abs() - 1
    }
}

/// The colours the image is made from: the arrows' and the edge's.
pub fn embedded_colors() -> Result<[[u8; 3]; 2], String> {
    let palette = trpg_content::palette::load().map_err(|errors| {
        let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
        lines.join("\n")
    })?;
    let color = |name: &str| {
        palette
            .get(name)
            .ok_or_else(|| format!("no palette colour {name}"))
    };
    Ok([color(COLORS[0])?, color(COLORS[1])?])
}

/// The colour of pixel `(x, y)` of the image.
fn pixel([arrow, edge]: [[u8; 3]; 2], x: u32, y: u32) -> [u8; 4] {
    let boxed = Way::ALL.into_iter().find_map(|way| {
        let (left, w, h) = way.sides();
        let inside = (left..left + w).contains(&x) && y < h;
        inside.then(|| (way, i64::from(x - left), i64::from(y)))
    });
    let Some((way, bx, by)) = boxed else {
        return [0; 4];
    };
    let solid = |[r, g, b]: [u8; 3]| [r, g, b, 255];
    if way.on(bx, by) {
        return solid(arrow);
    }
    let beside = |v: i64| v - 1..=v + 1;
    let near = beside(by).any(|ny| beside(bx).any(|nx| way.on(nx, ny)));
    if near { solid(edge) } else { [0; 4] }
}

/// The image's RGBA8 pixels, row-major.
pub fn pixels(colors: [[u8; 3]; 2]) -> Vec<u8> {
    (0..SIZE.1)
        .flat_map(|y| (0..SIZE.0).flat_map(move |x| pixel(colors, x, y)))
        .collect()
}

/// Runs the command: writes the image under repo root `root`.
pub fn run(root: &Path) -> Result<String, String> {
    let colors = embedded_colors()?;
    let shown = display_path(EDGE_MARKS_PATH);
    let path = root.join(&shown);
    let png = encode_png(SIZE.0, SIZE.1, &pixels(colors))?;
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    fs::write(&path, png).map_err(|e| format!("writing {}: {e}", path.display()))?;
    Ok(format!("edge-marks: wrote {shown} ({}×{})", SIZE.0, SIZE.1))
}

#[cfg(test)]
mod tests {
    use super::*;
    use shuyi_cli::codec::decode_png;

    const COLORS: [[u8; 3]; 2] = [[1, 2, 3], [7, 8, 9]];

    /// The image as text: `#` arrow, `e` edge, `.` clear.
    fn drawn() -> Vec<String> {
        let rows = (0..SIZE.1).map(|y| {
            let row = (0..SIZE.0).map(|x| match pixel(COLORS, x, y) {
                [1, 2, 3, 255] => '#',
                [7, 8, 9, 255] => 'e',
                [0, 0, 0, 0] => '.',
                other => panic!("({x}, {y}): {other:?}"),
            });
            row.collect()
        });
        rows.collect()
    }

    #[test]
    fn right_left_down_and_up_each_with_a_one_pixel_edge() {
        assert_eq!(
            drawn(),
            [
                "eee..........eeeeeeeeeeeeeeee.....eee.....",
                "e#ee........ee#ee###########e....ee#ee....",
                "e##ee......ee##eee#########ee...ee###ee...",
                "e###ee....ee###e.ee#######ee...ee#####ee..",
                "e####ee..ee####e..ee#####ee...ee#######ee.",
                "e#####eeee#####e...ee###ee...ee#########ee",
                "e######ee######e....ee#ee....e###########e",
                "e#####eeee#####e.....eee.....eeeeeeeeeeeee",
                "e####ee..ee####e..........................",
                "e###ee....ee###e..........................",
                "e##ee......ee##e..........................",
                "e#ee........ee#e..........................",
                "eee..........eee..........................",
            ]
        );
    }

    #[test]
    fn pixels_are_row_major() {
        let rgba = pixels(COLORS);
        assert_eq!(rgba.len(), 42 * 13 * 4);
        let at = |x: usize, y: usize| &rgba[(y * 42 + x) * 4..][..4];
        assert_eq!(at(1, 6), [1, 2, 3, 255]);
        assert_eq!(at(7, 6), [7, 8, 9, 255]);
        assert_eq!(at(20, 12), [0, 0, 0, 0]);
    }

    #[test]
    fn the_colours_are_the_palettes() {
        let palette = trpg_content::palette::load().unwrap();
        let [arrow, edge] = embedded_colors().unwrap();
        assert_eq!(arrow, palette.get("cursor").unwrap());
        assert_eq!(edge, palette.get("black").unwrap());
    }

    #[test]
    fn committed_image_is_what_the_tool_makes() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let committed = fs::read(root.join(display_path(EDGE_MARKS_PATH))).unwrap();
        assert_eq!(
            decode_png(&committed).unwrap(),
            (SIZE.0, SIZE.1, pixels(embedded_colors().unwrap())),
            "rerun `cargo xtask edge-marks`"
        );
    }

    #[test]
    fn run_writes_the_image_and_names_it() {
        let root = std::env::temp_dir().join(format!("xtask-edge-marks-{}", std::process::id()));
        let summary = run(&root).unwrap();
        assert_eq!(
            summary,
            "edge-marks: wrote assets/images/edge_marks.png (42×13)"
        );
        let written = fs::read(root.join("assets/images/edge_marks.png")).unwrap();
        let colors = embedded_colors().unwrap();
        assert_eq!(decode_png(&written).unwrap(), (42, 13, pixels(colors)));
        // A root that can't hold directories fails with the path.
        let file = root.join("file");
        fs::write(&file, "x").unwrap();
        let err = run(&file).unwrap_err();
        assert!(err.starts_with("creating "), "{err}");
        // The image's own path taken by a directory fails the write.
        let blocked = root.join("blocked");
        fs::create_dir_all(blocked.join("assets/images/edge_marks.png")).unwrap();
        let err = run(&blocked).unwrap_err();
        assert!(err.starts_with("writing "), "{err}");
        fs::remove_dir_all(&root).unwrap();
    }
}
