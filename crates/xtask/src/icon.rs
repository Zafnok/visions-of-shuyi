//! `cargo xtask icon preview <out-dir>` and `cargo xtask icon build <option>`
//! (ticket 0902): the game's icon, drawn from a glyph of the font atlas on a
//! coloured square, so it is reproducible and in the game's own style.
//!
//! `preview` writes every option at 32×32 and 256×256 for Nick to pick from.
//! `build` writes the picked option: `assets/icon/icon.ico` (16, 32, 48, 256;
//! the exe's icon, ticket 0902) and, in `assets-src/icon/`, what `shuyi.ron`'s
//! `app` reads for the window (ticket 1201): `icon.png`, and the sizes the
//! engine's downscale would blur, drawn in `icon-{16,32}.png`. `cargo shuyi
//! app-info` makes the window's icons from those.

use std::fs;
use std::path::Path;

use trpg_content::FontAtlasDef;

use shuyi_cli::codec::{decode_png, encode_png};

/// One design: a glyph in a colour on a background with a border.
struct Option_ {
    name: &'static str,
    glyph: char,
    fg: [u8; 3],
    bg: [u8; 3],
    border: [u8; 3],
}

const OPTIONS: &[Option_] = &[
    Option_ {
        name: "hero",
        glyph: '@',
        fg: [0xf2, 0xc8, 0x4b],
        bg: [0x14, 0x1e, 0x3c],
        border: [0x4a, 0x62, 0xa8],
    },
    Option_ {
        name: "sword",
        glyph: '\u{2020}',
        fg: [0xf0, 0xf0, 0xf0],
        bg: [0x7a, 0x14, 0x1e],
        border: [0xd2, 0x5a, 0x4b],
    },
    Option_ {
        name: "gem",
        glyph: '\u{2666}',
        fg: [0x7a, 0xe6, 0xd2],
        bg: [0x1c, 0x12, 0x2e],
        border: [0x8a, 0x5a, 0xc8],
    },
];

/// Sizes of the pictures in the `.ico`.
const ICO_SIZES: [u32; 4] = [16, 32, 48, 256];
/// The side of `shuyi.ron`'s `icon`. 64 is a whole fraction of it, so the
/// engine's downscale gives the 64 px window icon these same pixels.
const SOURCE_SIZE: u32 = 512;
/// Sizes of the window's icons drawn for `shuyi.ron`'s `icon_sizes`.
const DRAWN_SIZES: [u32; 2] = [16, 32];

pub fn run(args: &[String], root: &Path) -> Result<String, String> {
    let atlas = Atlas::load(root)?;
    match args {
        [cmd, dir] if cmd == "preview" => {
            let dir = Path::new(dir);
            fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            for (i, o) in OPTIONS.iter().enumerate() {
                for size in [32, 256] {
                    let png = encode_png(size, size, &atlas.draw(o, size)?)?;
                    let name = format!("option-{}-{}-{size}.png", i + 1, o.name);
                    fs::write(dir.join(name), png).map_err(|e| e.to_string())?;
                }
            }
            Ok(format!(
                "wrote {} previews to {}",
                OPTIONS.len(),
                dir.display()
            ))
        }
        [cmd, n] if cmd == "build" => {
            let o = n
                .parse::<usize>()
                .ok()
                .and_then(|n| n.checked_sub(1))
                .and_then(|i| OPTIONS.get(i))
                .ok_or_else(|| format!("option must be 1 to {}", OPTIONS.len()))?;
            let out = root.join("assets/icon");
            fs::create_dir_all(&out).map_err(|e| e.to_string())?;
            let mut pictures = Vec::new();
            for size in ICO_SIZES {
                pictures.push((size, encode_png(size, size, &atlas.draw(o, size)?)?));
            }
            fs::write(out.join("icon.ico"), ico(&pictures)).map_err(|e| e.to_string())?;
            let source = root.join("assets-src/icon");
            fs::create_dir_all(&source).map_err(|e| e.to_string())?;
            let png = encode_png(SOURCE_SIZE, SOURCE_SIZE, &atlas.draw(o, SOURCE_SIZE)?)?;
            fs::write(source.join("icon.png"), png).map_err(|e| e.to_string())?;
            for size in DRAWN_SIZES {
                let png = encode_png(size, size, &atlas.draw(o, size)?)?;
                fs::write(source.join(format!("icon-{size}.png")), png)
                    .map_err(|e| e.to_string())?;
            }
            Ok(format!(
                "wrote option {n} ({}) to {} and {}: now run `cargo shuyi app-info`",
                o.name,
                out.display(),
                source.display()
            ))
        }
        _ => Err("usage: cargo xtask icon preview <out-dir> | build <option>".to_owned()),
    }
}

/// An `.ico` file holding PNG-encoded pictures (Windows Vista and later).
fn ico(pictures: &[(u32, Vec<u8>)]) -> Vec<u8> {
    let mut out = vec![0, 0, 1, 0];
    out.extend(u16::try_from(pictures.len()).unwrap_or(0).to_le_bytes());
    let mut offset = 6 + 16 * pictures.len();
    for (size, png) in pictures {
        // A width or height of 256 is stored as 0.
        let dim = u8::try_from(*size).unwrap_or(0);
        out.extend([dim, dim, 0, 0, 1, 0, 32, 0]);
        out.extend(u32::try_from(png.len()).unwrap_or(0).to_le_bytes());
        out.extend(u32::try_from(offset).unwrap_or(0).to_le_bytes());
        offset += png.len();
    }
    for (_, png) in pictures {
        out.extend(png);
    }
    out
}

struct Atlas {
    def: FontAtlasDef,
    width: u32,
    pixels: Vec<u8>,
}

impl Atlas {
    fn load(root: &Path) -> Result<Self, String> {
        let ron =
            fs::read_to_string(root.join("assets/fonts/atlas.ron")).map_err(|e| e.to_string())?;
        let def: FontAtlasDef = ron::from_str(&ron).map_err(|e| e.to_string())?;
        let png = fs::read(root.join("assets/fonts/atlas.png")).map_err(|e| e.to_string())?;
        let (width, _, pixels) = decode_png(&png)?;
        Ok(Self { def, width, pixels })
    }

    /// Whether the atlas pixel at (`x`, `y`) is lit.
    fn lit(&self, x: u32, y: u32) -> bool {
        let i = ((y * self.width + x) * 4) as usize;
        self.pixels[i + 3] > 127 && self.pixels[i] > 127
    }

    /// The icon `o` as `size`×`size` RGBA pixels.
    fn draw(&self, o: &Option_, size: u32) -> Result<Vec<u8>, String> {
        let r = self
            .def
            .glyph_rect(o.glyph)
            .ok_or_else(|| format!("the font has no {:?}", o.glyph))?;
        // The biggest whole-pixel scale that leaves a margin.
        let scale = (size * 3 / 4 / r.h).max(1);
        let (gw, gh) = (r.w * scale, r.h * scale);
        let (gx, gy) = (size.saturating_sub(gw) / 2, size.saturating_sub(gh) / 2);
        let border = (size / 16).max(1);
        let mut out = Vec::with_capacity((size * size * 4) as usize);
        for y in 0..size {
            for x in 0..size {
                let edge = x < border || y < border || x >= size - border || y >= size - border;
                let on_glyph = x >= gx
                    && y >= gy
                    && x < gx + gw
                    && y < gy + gh
                    && self.lit(r.x + (x - gx) / scale, r.y + (y - gy) / scale);
                let c = if on_glyph {
                    o.fg
                } else if edge {
                    o.border
                } else {
                    o.bg
                };
                out.extend([c[0], c[1], c[2], 255]);
            }
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use shuyi_cli::Project;
    use shuyi_cli::app_info;
    use shuyi_cli::project::AppPart;

    use super::*;

    fn root() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    #[test]
    fn every_option_draws_at_every_size() {
        let atlas = Atlas::load(root()).unwrap();
        for o in OPTIONS {
            for size in ICO_SIZES.into_iter().chain(DRAWN_SIZES) {
                let px = atlas.draw(o, size).unwrap();
                assert_eq!(px.len(), (size * size * 4) as usize, "{}", o.name);
                assert!(px.chunks(4).any(|p| p[..3] == o.fg), "{} {size}", o.name);
            }
        }
    }

    #[test]
    fn ico_header_points_at_the_pictures() {
        let bytes = ico(&[(16, vec![1, 2, 3]), (256, vec![4, 5])]);
        assert_eq!(&bytes[..6], &[0, 0, 1, 0, 2, 0]);
        // Second entry: size 256 stored as 0, 2 bytes at offset 6+32+3.
        assert_eq!(bytes[22], 0);
        assert_eq!(u32::from_le_bytes(bytes[30..34].try_into().unwrap()), 2);
        assert_eq!(u32::from_le_bytes(bytes[34..38].try_into().unwrap()), 41);
        assert_eq!(&bytes[41..], &[4, 5]);
    }

    /// A 4×4 atlas of two 2×4 cells, `'a'` the second: its glyph is lit at
    /// (0, 0) and (1, 3). The first cell tests what counts as lit.
    fn tiny() -> Atlas {
        let mut pixels = vec![0; 4 * 4 * 4];
        let mut set = |x: usize, y: usize, rgba: [u8; 4]| {
            pixels[(y * 4 + x) * 4..][..4].copy_from_slice(&rgba);
        };
        set(0, 0, [255, 255, 255, 127]);
        set(1, 0, [127, 255, 255, 255]);
        set(2, 0, [128, 0, 0, 128]);
        set(3, 3, [255, 255, 255, 255]);
        Atlas {
            def: FontAtlasDef {
                cell_w: 2,
                cell_h: 4,
                columns: 2,
                glyphs: [('b', 0), ('a', 1)].into_iter().collect(),
            },
            width: 4,
            pixels,
        }
    }

    #[test]
    fn lit_needs_a_bright_enough_opaque_pixel() {
        let atlas = tiny();
        assert!(!atlas.lit(0, 0), "alpha 127");
        assert!(!atlas.lit(1, 0), "red 127");
        assert!(atlas.lit(2, 0), "128 and 128");
        assert!(atlas.lit(3, 3));
        assert!(!atlas.lit(3, 2));
    }

    #[test]
    fn the_glyph_is_scaled_and_centered_inside_the_border() {
        let o = Option_ {
            name: "t",
            glyph: 'a',
            fg: [1, 2, 3],
            bg: [4, 5, 6],
            border: [7, 8, 9],
        };
        let px = tiny().draw(&o, 16).unwrap();
        let at = |x: usize, y: usize| px[(y * 16 + x) * 4..][..3].to_vec();
        // Scale 3: the glyph is 6×12 at (5, 2); its lit pixels are 3×3 blocks.
        for (x, y) in [(5, 2), (7, 4), (8, 11), (10, 13)] {
            assert_eq!(at(x, y), o.fg, "{x},{y}");
        }
        for (x, y) in [(8, 2), (5, 5), (7, 11), (8, 10), (11, 13), (10, 14), (4, 2)] {
            assert_eq!(at(x, y), o.bg, "{x},{y}");
        }
        for (x, y) in [(0, 5), (15, 5), (5, 0), (5, 15)] {
            assert_eq!(at(x, y), o.border, "{x},{y}");
        }
        assert_eq!(at(1, 1), o.bg);
        assert_eq!(at(14, 14), o.bg);
        // The border is a sixteenth of the size: two pixels at 32.
        let px = tiny().draw(&o, 32).unwrap();
        let at = |x: usize, y: usize| px[(y * 32 + x) * 4..][..3].to_vec();
        assert_eq!(at(1, 5), o.border);
        assert_eq!(at(2, 5), o.bg);
        assert_eq!(at(30, 5), o.border);
        assert_eq!(at(29, 5), o.bg);
    }

    #[test]
    fn a_glyph_the_font_lacks_is_an_error() {
        let o = Option_ {
            name: "t",
            glyph: 'z',
            fg: [0; 3],
            bg: [0; 3],
            border: [0; 3],
        };
        assert!(tiny().draw(&o, 16).is_err());
    }

    #[test]
    fn preview_and_build_write_their_files() {
        let dir = std::env::temp_dir().join(format!("icon-test-{}", std::process::id()));
        let fonts = dir.join("assets/fonts");
        fs::create_dir_all(&fonts).unwrap();
        for f in ["atlas.png", "atlas.ron"] {
            fs::copy(root().join("assets/fonts").join(f), fonts.join(f)).unwrap();
        }
        let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let previews = dir.join("previews");
        let out = run(&args(&["preview", previews.to_str().unwrap()]), &dir).unwrap();
        assert!(out.contains("3 previews"), "{out}");
        assert!(previews.join("option-2-sword-256.png").is_file());
        assert!(run(&args(&["bogus", previews.to_str().unwrap()]), &dir).is_err());
        assert!(run(&args(&["bogus", "1"]), &dir).is_err());
        assert!(run(&args(&["build", "0"]), &dir).is_err());
        assert!(run(&args(&["build", "4"]), &dir).is_err());
        let out = run(&args(&["build", "2"]), &dir).unwrap();
        assert!(out.contains("option 2 (sword)"), "{out}");
        let ico = fs::read(dir.join("assets/icon/icon.ico")).unwrap();
        assert_eq!(&ico[..6], &[0, 0, 1, 0, 4, 0]);
        assert!(dir.join("assets-src/icon/icon.png").is_file());
        for size in DRAWN_SIZES {
            assert!(
                dir.join(format!("assets-src/icon/icon-{size}.png"))
                    .is_file()
            );
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    /// `shuyi.ron`'s `app`, as the engine's commands read it.
    fn app_part() -> AppPart {
        Project::load(root()).unwrap().config.app.unwrap()
    }

    /// Acceptance (1201): the window's icons are the hero drawn at each size,
    /// the 64 px one by the engine's downscale of the 512 px source.
    #[test]
    fn the_windows_icons_are_drawn_at_each_size() {
        let part = app_part();
        assert_eq!(
            part.icon_sizes.keys().copied().collect::<Vec<_>>(),
            DRAWN_SIZES
        );
        assert_eq!(
            app_info::stale(&part, root(), "cargo shuyi"),
            Vec::<String>::new()
        );
        let atlas = Atlas::load(root()).unwrap();
        for size in [16, 32, 64] {
            let png = fs::read(root().join(format!("crates/app/icon/{size}.png"))).unwrap();
            let (w, h, pixels) = decode_png(&png).unwrap();
            assert_eq!((w, h), (size, size));
            assert!(pixels == atlas.draw(&OPTIONS[0], size).unwrap(), "{size}");
        }
    }

    /// Acceptance (1201): `app-info` refuses a drawn picture that is missing
    /// or the wrong size, and `--check` a window icon that is out of date.
    #[test]
    fn app_info_refuses_bad_or_stale_icons() {
        let dir = std::env::temp_dir().join(format!("icon-refuse-{}", std::process::id()));
        let fonts = dir.join("assets/fonts");
        fs::create_dir_all(&fonts).unwrap();
        for f in ["atlas.png", "atlas.ron"] {
            fs::copy(root().join("assets/fonts").join(f), fonts.join(f)).unwrap();
        }
        run(&["build".to_owned(), "1".to_owned()], &dir).unwrap();
        let part = app_part();
        let app_info = |check| app_info::run(&part, &dir, "cargo shuyi", check);
        app_info(false).unwrap();
        app_info(true).unwrap();

        let drawn = dir.join("assets-src/icon/icon-16.png");
        let good = fs::read(&drawn).unwrap();
        fs::write(&drawn, encode_png(17, 17, &[0; 17 * 17 * 4]).unwrap()).unwrap();
        let err = app_info(false).unwrap_err();
        assert!(
            err.contains("app.icon_sizes") && err.contains("17 × 17"),
            "{err}"
        );
        fs::remove_file(&drawn).unwrap();
        let err = app_info(false).unwrap_err();
        assert!(
            err.contains("app.icon_sizes") && err.contains("can't be read"),
            "{err}"
        );
        fs::write(&drawn, good).unwrap();

        let window = dir.join("crates/app/icon/32.png");
        fs::write(&window, encode_png(32, 32, &[0; 32 * 32 * 4]).unwrap()).unwrap();
        let err = app_info(true).unwrap_err();
        assert!(err.contains("crates/app/icon/32.png"), "{err}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn bad_arguments_are_refused() {
        assert!(run(&["build".to_owned(), "9".to_owned()], root()).is_err());
        assert!(run(&[], root()).is_err());
    }
}
