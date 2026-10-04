//! The third part of `cargo xtask test-tileset`: writes
//! `assets/tilesets/test_auto.png` and `test_auto.ron`, the public test
//! tileset with **layers and looks** (ticket 0437, ADR-0052).
//!
//! It is shaped like the bought tileset, which never enters this
//! repository: 16×16 tiles, a tile for every terrain, pictures drawn
//! between tiles for each mix of their corners, a picture on a tile that
//! turns by its neighbours, and a second look for maps indoors. So the
//! tests, and a copy of the public repository, run the same code as the
//! game with its bought art.
//!
//! Each terrain's tile is its `bg` colour with its two glyphs in its `fg`
//! colour. A corner layer's pictures are a 2-pixel line, in the glyph
//! colour of the layer's first terrain, along the edges of the corners
//! that are in the layer: on the map it runs round the layer's tiles. It
//! is generated from our own data and the font atlas, not art, and the
//! same data always gives the same files.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use trpg_content::bundle::display_path;

use crate::font_atlas::encode_png;
use crate::test_tileset::{Canvas, Sources, TerrainLook};
use crate::test_units;
use crate::tileset_ron::{self, At, LayerText, LookText};

/// The tileset's id.
pub const ID: &str = "test_auto";
/// The image's path in the bundle.
pub const PNG_PATH: &str = "tilesets/test_auto.png";
/// The tileset file's path in the bundle.
pub const RON_PATH: &str = "tilesets/test_auto.ron";
/// A tile's side, in pixels: the bought tiles', and the glyph skin's.
pub const TILE: u32 = 16;
/// Tiles per row of the image: one for each mix of four corners.
pub const COLUMNS: u32 = 16;
/// How thick a corner layer's line is, in pixels.
pub const LINE: u32 = 2;

/// The corner layers, each the terrains in it: outdoors the water (and
/// what lies on it) and the woods; indoors the floor. A layer's line is
/// the glyph colour of its first terrain.
pub const WET: [&str; 4] = ["water", "sea", "bridge", "ice"];
/// See [`WET`].
pub const WOODS: [&str; 2] = ["forest", "thicket"];
/// See [`WET`].
pub const FLOORS: [&str; 2] = ["floor", "door"];
/// The corner layers in the image's order, one row each.
pub const CORNER_LAYERS: [&[&str]; 3] = [&WET, &WOODS, &FLOORS];

/// The terrain with a frame round its tile.
pub const FRAMED: &str = "fort";
/// The terrain with rails along its tile: above and below, or left and
/// right where the water is to its left and right.
pub const RAILED: &str = "bridge";
/// The terrains the rails turn for.
pub const RAILED_BESIDE: [&str; 3] = ["water", "sea", "ice"];
/// The mixes of sides (up, right, down, left) the rails turn for: water
/// both left and right.
pub const TURNED: [u8; 4] = [0b0101, 0b1101, 0b0111, 0b1111];
/// The pictures that go on a tile, in the image's last row.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Single {
    /// A 1-pixel frame.
    Frame,
    /// Rails along the top and the bottom.
    Rails,
    /// Rails along the left and the right.
    RailsTurned,
}
/// The single pictures in the image's order.
pub const SINGLES: [Single; 3] = [Single::Frame, Single::Rails, Single::RailsTurned];

/// Whether pixel `(x, y)` of the picture for `mix` (which of the corners
/// top-left, top-right, bottom-left, bottom-right are in the layer) is on
/// the line: in a corner that is in, within [`LINE`] pixels of the
/// picture's middle where the corner beside it, or the one across, is
/// out.
pub fn on_line(mix: u8, x: u32, y: u32) -> bool {
    let half = TILE / 2;
    let (right, below) = (x >= half, y >= half);
    let is_in = |right: bool, below: bool| {
        let corner = u8::from(right) + 2 * u8::from(below);
        mix & (8 >> corner) != 0
    };
    let near = |v: u32| {
        if v >= half {
            v - half < LINE
        } else {
            half - 1 - v < LINE
        }
    };
    let (near_x, near_y) = (near(x), near(y));
    is_in(right, below)
        && ((near_x && !is_in(!right, below))
            || (near_y && !is_in(right, !below))
            || (near_x && near_y && !is_in(!right, !below)))
}

/// Whether pixel `(x, y)` of `single` is drawn.
pub fn on_single(single: Single, x: u32, y: u32) -> bool {
    let edge = |v: u32, thick: u32| v < thick || v >= TILE - thick;
    match single {
        Single::Frame => edge(x, 1) || edge(y, 1),
        Single::Rails => edge(y, LINE),
        Single::RailsTurned => edge(x, LINE),
    }
}

/// Rows the terrain tiles take.
fn terrain_rows(src: &Sources) -> u32 {
    u32::try_from(src.terrain.len())
        .unwrap_or(u32::MAX)
        .div_ceil(COLUMNS)
}

/// `(column, row)` of terrain tile `index`.
fn terrain_cell(index: usize) -> (u32, u32) {
    let i = u32::try_from(index).unwrap_or(u32::MAX);
    (i % COLUMNS, i / COLUMNS)
}

/// The row of corner layer `layer` of [`CORNER_LAYERS`].
fn layer_row(src: &Sources, layer: usize) -> u32 {
    terrain_rows(src) + u32::try_from(layer).unwrap_or(u32::MAX)
}

/// The row of the single pictures.
fn singles_row(src: &Sources) -> u32 {
    layer_row(src, CORNER_LAYERS.len())
}

/// The look of terrain `id`.
fn terrain<'a>(src: &'a Sources, id: &str) -> Result<&'a TerrainLook, String> {
    let found = src.terrain.iter().find(|(name, ..)| name == id);
    found.ok_or_else(|| format!("terrain.ron has no terrain \"{id}\""))
}

/// The glyph colour of terrain `id`, solid.
fn line_color(src: &Sources, id: &str) -> Result<[u8; 4], String> {
    let (_, _, fg, _) = terrain(src, id)?;
    Ok([fg[0], fg[1], fg[2], 255])
}

/// The tileset image: width, height and RGBA8 pixels. The terrain tiles
/// in `terrain.ron`'s order; then a row for each of [`CORNER_LAYERS`],
/// the picture of mix `m` in column `m`; then the [`SINGLES`].
pub fn image(src: &Sources) -> Result<(u32, u32, Vec<u8>), String> {
    let (width, height) = (COLUMNS * TILE, (singles_row(src) + 1) * TILE);
    let mut canvas = Canvas::new(width, height);
    let pixels = || (0..TILE).flat_map(|y| (0..TILE).map(move |x| (x, y)));
    for (i, (_, glyphs, fg, bg)) in src.terrain.iter().enumerate() {
        let (cx, cy) = terrain_cell(i);
        let (left, top) = (cx * TILE, cy * TILE);
        for (x, y) in pixels() {
            canvas.set(left + x, top + y, [bg[0], bg[1], bg[2], 255]);
        }
        // Two glyphs fill a 16 × 16 tile exactly.
        canvas.stamp_at(src, *glyphs, *fg, (left, top));
    }
    for (layer, of) in CORNER_LAYERS.iter().enumerate() {
        let color = line_color(src, of[0])?;
        let top = layer_row(src, layer) * TILE;
        for mix in 1..15u8 {
            let left = u32::from(mix) * TILE;
            for (x, y) in pixels().filter(|&(x, y)| on_line(mix, x, y)) {
                canvas.set(left + x, top + y, color);
            }
        }
    }
    let top = singles_row(src) * TILE;
    for (column, single) in (0..).zip(SINGLES) {
        let of = if single == Single::Frame {
            FRAMED
        } else {
            RAILED
        };
        let color = line_color(src, of)?;
        for (x, y) in pixels().filter(|&(x, y)| on_single(single, x, y)) {
            canvas.set(column * TILE + x, top + y, color);
        }
    }
    Ok((width, height, canvas.rgba))
}

/// `names` as owned strings.
fn names(names: &[&str]) -> Vec<String> {
    names.iter().map(|&n| n.to_owned()).collect()
}

/// Each terrain's own tile.
fn terrain_tiles(src: &Sources) -> Vec<(String, At)> {
    let tile = |(i, (id, ..)): (usize, &TerrainLook)| (id.clone(), terrain_cell(i));
    src.terrain.iter().enumerate().map(tile).collect()
}

/// Corner layer `layer` of [`CORNER_LAYERS`]: a picture for every mix but
/// none and all.
fn corner_layer(src: &Sources, layer: usize) -> LayerText {
    let row = layer_row(src, layer);
    LayerText::Corners {
        of: names(CORNER_LAYERS[layer]),
        tiles: (1..15u8).map(|mix| (mix, (u32::from(mix), row))).collect(),
    }
}

/// The tileset's own look (a map outdoors), then its indoor one: the
/// same tiles, with the floor's line in place of the water's and the
/// woods', and nothing on the fort or the bridge.
pub fn looks(src: &Sources) -> (LookText, LookText) {
    let row = singles_row(src);
    let column = |single: Single| {
        let at = SINGLES.iter().position(|&s| s == single).unwrap_or(0);
        u32::try_from(at).unwrap_or(0)
    };
    let turned = (column(Single::RailsTurned), row);
    let outdoor = LookText {
        terrain: terrain_tiles(src),
        layers: vec![
            corner_layer(src, 0),
            corner_layer(src, 1),
            LayerText::Tiles {
                of: names(&[FRAMED]),
                at: (column(Single::Frame), row),
                beside: vec![],
                sides: vec![],
            },
            LayerText::Tiles {
                of: names(&[RAILED]),
                at: (column(Single::Rails), row),
                beside: names(&RAILED_BESIDE),
                sides: TURNED.iter().map(|&mix| (mix, turned)).collect(),
            },
        ],
    };
    let indoor = LookText {
        terrain: terrain_tiles(src),
        layers: vec![corner_layer(src, 2)],
    };
    (outdoor, indoor)
}

/// The tileset file for [`image`], with `sheets`' unit pictures (the test
/// unit sheets').
pub fn ron(src: &Sources, sheets: &[(String, [char; 2])]) -> String {
    let (outdoor, indoor) = looks(src);
    let mut out = String::from(
        "// Generated by `cargo xtask test-tileset` from terrain.ron, classes.ron,\n\
         // the palette and the font atlas. Do not edit; see assets/tilesets/README.md.\n\
         (\n",
    );
    // Writing to a `String` can't fail.
    let _ = writeln!(out, "    id: \"{ID}\",\n    image: \"{PNG_PATH}\",");
    let _ = writeln!(out, "    tile_px: ({TILE}, {TILE}),");
    out.push_str(&tileset_ron::look(&outdoor, "    "));
    out.push_str(&tileset_ron::looks(&[("indoor".to_owned(), indoor)]));
    out.push_str(&test_units::units(sheets));
    out.push_str(")\n");
    out
}

/// Writes the tileset under repo root `root`.
pub fn run(root: &Path) -> Result<String, String> {
    let src = Sources::embedded()?;
    let (width, height, rgba) = image(&src)?;
    let png = encode_png(width, height, &rgba)?;
    let text = ron(&src, &test_units::sheets(&src));
    let png_shown = display_path(PNG_PATH);
    let ron_shown = display_path(RON_PATH);
    for (shown, bytes) in [(&png_shown, png), (&ron_shown, text.into_bytes())] {
        let path = root.join(shown);
        if let Some(dir) = path.parent() {
            fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
        }
        fs::write(&path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))?;
    }
    Ok(format!(
        "test-tileset: wrote {png_shown} ({width}×{height}) and {ron_shown}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font_atlas::decode_png;

    fn sources() -> Sources {
        Sources::embedded().unwrap()
    }

    /// The picture of `mix` as text, a `#` for each pixel on the line.
    fn drawn(mix: u8) -> Vec<String> {
        (0..TILE)
            .map(|y| {
                (0..TILE)
                    .map(|x| if on_line(mix, x, y) { '#' } else { '.' })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn a_corner_pictures_line_runs_along_the_corners_that_are_in() {
        // Only the bottom-right corner: an L round its top-left.
        let only = drawn(0b0001);
        assert_eq!(only[..8], vec!["................".to_owned(); 8]);
        assert_eq!(only[8], "........########");
        assert_eq!(only[9], "........########");
        assert_eq!(only[10], "........##......");
        assert_eq!(only[15], "........##......");
        // The two top corners: a line along their bottom.
        let top = drawn(0b1100);
        assert_eq!(top[5], "................");
        assert_eq!(top[6], "################");
        assert_eq!(top[7], "################");
        assert_eq!(top[8], "................");
        // The two left corners: a line down their right.
        let left = drawn(0b1010);
        assert!(left.iter().all(|row| row == "......##........"));
        // All but the top-left: a dot in each of the three by the middle,
        // lines only where a corner beside is out.
        let three = drawn(0b0111);
        assert_eq!(three[0], "........##......");
        assert_eq!(three[7], "........##......");
        assert_eq!(three[8], "##########......");
        assert_eq!(three[9], "##########......");
        assert_eq!(three[10], "................");
        // Two corners across from each other: each its own L.
        let across = drawn(0b1001);
        assert_eq!(across[0], "......##........");
        assert_eq!(across[6], "########........");
        assert_eq!(across[7], "########........");
        assert_eq!(across[8], "........########");
        assert_eq!(across[15], "........##......");
        // None in, or all in: no line at all.
        for mix in [0, 15] {
            assert!(drawn(mix).iter().all(|row| !row.contains('#')), "{mix}");
        }
    }

    #[test]
    fn a_single_picture_is_a_frame_or_rails() {
        let at = |single, x, y| on_single(single, x, y);
        // A 1-pixel frame.
        assert!(at(Single::Frame, 0, 7) && at(Single::Frame, 15, 7));
        assert!(at(Single::Frame, 7, 0) && at(Single::Frame, 7, 15));
        assert!(!at(Single::Frame, 1, 1) && !at(Single::Frame, 14, 14));
        // Rails 2 pixels thick, above and below.
        assert!(at(Single::Rails, 7, 0) && at(Single::Rails, 7, 1));
        assert!(at(Single::Rails, 7, 14) && at(Single::Rails, 0, 15));
        assert!(!at(Single::Rails, 7, 2) && !at(Single::Rails, 0, 13));
        // Turned: left and right.
        assert!(at(Single::RailsTurned, 0, 7) && at(Single::RailsTurned, 1, 7));
        assert!(at(Single::RailsTurned, 14, 7) && at(Single::RailsTurned, 15, 0));
        assert!(!at(Single::RailsTurned, 2, 7) && !at(Single::RailsTurned, 13, 0));
    }

    /// Pixel `(x, y)` of an image `width` wide.
    fn px(rgba: &[u8], width: u32, x: u32, y: u32) -> [u8; 4] {
        let at = usize::try_from((y * width + x) * 4).unwrap();
        [rgba[at], rgba[at + 1], rgba[at + 2], rgba[at + 3]]
    }

    #[test]
    fn the_image_has_the_tiles_then_a_row_a_layer_then_the_singles() {
        let src = sources();
        // 16 terrains: one row; the layers on rows 1 to 3, the singles on
        // row 4.
        assert_eq!(terrain_rows(&src), 1);
        assert_eq!((terrain_cell(0), terrain_cell(15)), ((0, 0), (15, 0)));
        assert_eq!(terrain_cell(16), (0, 1));
        assert_eq!((layer_row(&src, 0), layer_row(&src, 2)), (1, 3));
        assert_eq!(singles_row(&src), 4);
        let (w, h, rgba) = image(&src).unwrap();
        assert_eq!((w, h), (256, 80));
        let solid = |c: &[u8; 3]| [c[0], c[1], c[2], 255];
        // A terrain tile: its background, with glyph pixels somewhere.
        let (_, _, fg, bg) = &src.terrain[1];
        assert_eq!(px(&rgba, w, 16, 0), solid(bg));
        let inked = (0..16)
            .flat_map(|y| (16..32).map(move |x| (x, y)))
            .filter(|&(x, y)| px(&rgba, w, x, y) == solid(fg))
            .count();
        assert!(inked > 0, "no glyph pixels in {:?}", src.terrain[1]);
        // The water layer's picture of mix 3 (the two bottom corners): a
        // line of the water's glyph colour under the middle, clear above.
        let water = line_color(&src, "water").unwrap();
        assert_eq!(px(&rgba, w, 3 * 16 + 4, 16 + 8), water);
        assert_eq!(px(&rgba, w, 3 * 16 + 4, 16 + 7), [0; 4]);
        // Mixes 0 and 15 have no picture.
        for column in [0, 15] {
            let clear = (0..16).all(|i| px(&rgba, w, column * 16 + i, 16 + i) == [0; 4]);
            assert!(clear, "{column}");
        }
        // The woods and the floors: their own rows and colours.
        let woods = line_color(&src, "forest").unwrap();
        let floors = line_color(&src, "floor").unwrap();
        assert_eq!(px(&rgba, w, 3 * 16 + 4, 32 + 8), woods);
        assert_eq!(px(&rgba, w, 3 * 16 + 4, 48 + 8), floors);
        assert_ne!(water, woods);
        // The singles: the fort's frame, the bridge's rails, then turned.
        let fort = line_color(&src, "fort").unwrap();
        let bridge = line_color(&src, "bridge").unwrap();
        assert_eq!(px(&rgba, w, 0, 64 + 7), fort);
        assert_eq!(px(&rgba, w, 1, 64 + 7), [0; 4]);
        assert_eq!(px(&rgba, w, 16 + 7, 64), bridge);
        assert_eq!(px(&rgba, w, 16, 64 + 7), [0; 4]);
        assert_eq!(px(&rgba, w, 32, 64 + 7), bridge);
        assert_eq!(px(&rgba, w, 32 + 7, 64), [0; 4]);
    }

    #[test]
    fn a_terrain_the_data_lacks_is_named() {
        let mut src = sources();
        src.terrain.retain(|(id, ..)| id != "forest");
        assert_eq!(
            image(&src).unwrap_err(),
            "terrain.ron has no terrain \"forest\""
        );
        assert!(terrain(&sources(), "forest").is_ok());
    }

    #[test]
    fn the_file_names_the_layers_the_other_look_and_the_unit_sheets() {
        let src = sources();
        let text = ron(&src, &test_units::sheets(&src));
        assert!(text.contains("    id: \"test_auto\",\n    image: \"tilesets/test_auto.png\",\n"));
        assert!(
            text.contains("    tile_px: (16, 16),\n    terrain: {\n        \"plain\": (0, 0),\n")
        );
        assert!(text.contains(
            "    layers: [\n        Corners(of: [\"water\", \"sea\", \"bridge\", \"ice\"], tiles: [\n            \
             (corners: \"...#\", at: (1, 1)),\n"
        ));
        assert!(text.contains("            (corners: \"###.\", at: (14, 1)),\n        ]),\n"));
        assert!(!text.contains("corners: \"####\"") && !text.contains("corners: \"....\""));
        assert!(text.contains("        Corners(of: [\"forest\", \"thicket\"], tiles: [\n"));
        assert!(text.contains("            (corners: \"...#\", at: (1, 2)),\n"));
        assert!(text.contains("        Tiles(of: [\"fort\"], at: (0, 4)),\n"));
        assert!(text.contains(
            "        Tiles(of: [\"bridge\"], at: (1, 4), beside: [\"water\", \"sea\", \"ice\"], sides: [\n            \
             (sides: \".#.#\", at: (2, 4)),\n            (sides: \"##.#\", at: (2, 4)),\n            \
             (sides: \".###\", at: (2, 4)),\n            (sides: \"####\", at: (2, 4)),\n        ]),\n    ],\n"
        ));
        assert!(text.contains(
            "    looks: {\n        \"indoor\": (\n            terrain: {\n                \"plain\": (0, 0),\n"
        ));
        assert!(text.contains(
            "            layers: [\n                Corners(of: [\"floor\", \"door\"], tiles: [\n                    \
             (corners: \"...#\", at: (1, 3)),\n"
        ));
        assert!(text.contains(
            "                ]),\n            ],\n        ),\n    },\n    unit_px: (16, 20),\n"
        ));
        assert!(text.contains(
            "            \"guard\": (image: \"tilesets/test_units/guard.png\", frame: (1, 0)),\n"
        ));
        assert!(text.ends_with("    ),\n)\n"));
        // Every terrain has its tile, in both looks.
        for (id, ..) in &src.terrain {
            assert_eq!(text.matches(&format!("\"{id}\": (")).count(), 2, "{id}");
        }
    }

    #[test]
    fn the_turned_rails_are_for_water_both_left_and_right() {
        // Sides are up, right, down, left.
        for mix in 0..16u8 {
            let both = mix & 0b0100 != 0 && mix & 0b0001 != 0;
            assert_eq!(TURNED.contains(&mix), both, "{mix:04b}");
        }
    }

    #[test]
    fn committed_files_are_what_the_tool_makes() {
        let src = sources();
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let png = fs::read(root.join(display_path(PNG_PATH))).unwrap();
        assert_eq!(
            decode_png(&png).unwrap(),
            image(&src).unwrap(),
            "rerun `cargo xtask test-tileset`"
        );
        let text = fs::read_to_string(root.join(display_path(RON_PATH))).unwrap();
        assert_eq!(
            text.replace("\r\n", "\n"),
            ron(&src, &test_units::sheets(&src)),
            "rerun `cargo xtask test-tileset`"
        );
    }

    #[test]
    fn run_writes_both_files_and_names_them() {
        let root = std::env::temp_dir().join(format!("xtask-test-auto-{}", std::process::id()));
        let summary = run(&root).unwrap();
        assert_eq!(
            summary,
            "test-tileset: wrote assets/tilesets/test_auto.png (256×80) and \
             assets/tilesets/test_auto.ron"
        );
        let src = sources();
        let written = fs::read(root.join("assets/tilesets/test_auto.png")).unwrap();
        assert_eq!(decode_png(&written).unwrap(), image(&src).unwrap());
        let text = fs::read_to_string(root.join("assets/tilesets/test_auto.ron")).unwrap();
        assert_eq!(text, ron(&src, &test_units::sheets(&src)));
        // A root that can't hold directories fails with the path.
        let file = root.join("file");
        fs::write(&file, "x").unwrap();
        let err = run(&file).unwrap_err();
        assert!(err.starts_with("creating "), "{err}");
        // The image's own path taken by a directory fails the write.
        let blocked = root.join("blocked");
        fs::create_dir_all(blocked.join("assets/tilesets/test_auto.png")).unwrap();
        let err = run(&blocked).unwrap_err();
        assert!(err.starts_with("writing "), "{err}");
        fs::remove_dir_all(&root).unwrap();
    }
}
