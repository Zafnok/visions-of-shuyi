//! `cargo xtask icon preview <out-dir>` and `cargo xtask icon build <option>`
//! (ticket 0902): the game's icon, drawn from a glyph of the font atlas on a
//! coloured square, so it is reproducible and in the game's own style.
//!
//! `preview` writes every option at 32×32 and 256×256 for Nick to pick from.
//! `build` writes the picked option to `assets/icon/`: `icon.ico` (16, 32,
//! 48, 256; the exe's icon, ticket 0902) and `icon-{16,32,64}.png` (the
//! window's icon, read by `app`).

use std::fs;
use std::path::Path;

use trpg_content::FontAtlasDef;

use crate::font_atlas::{decode_png, encode_png};

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
/// Sizes of the PNG files `app` hands to the window.
const WINDOW_SIZES: [u32; 3] = [16, 32, 64];

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
            for size in WINDOW_SIZES {
                let png = encode_png(size, size, &atlas.draw(o, size)?)?;
                fs::write(out.join(format!("icon-{size}.png")), png).map_err(|e| e.to_string())?;
            }
            Ok(format!(
                "wrote option {n} ({}) to {}",
                o.name,
                out.display()
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
    use super::*;

    fn root() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    #[test]
    fn every_option_draws_at_every_size() {
        let atlas = Atlas::load(root()).unwrap();
        for o in OPTIONS {
            for size in ICO_SIZES.into_iter().chain(WINDOW_SIZES) {
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

    #[test]
    fn bad_arguments_are_refused() {
        assert!(run(&["build".to_owned(), "9".to_owned()], root()).is_err());
        assert!(run(&[], root()).is_err());
    }
}
