//! `cargo xtask ui-import`: cuts the bought UI kit's pictures the windows
//! are framed with (ticket 1380, `docs/design/look-and-feel.md`, *Windows
//! (the picture look)*) into `assets-private/game/ui/`, where the game's
//! frames file (`assets/ui/frames.ron`) names them.
//!
//! Which picture is which is the source list `assets-src/ui/pictures.ron`:
//! a picture is copied as it is, recoloured (its cool pixels moved to a
//! hue, its gold kept: the map menu's royal blue), or cut down to a plate
//! (two end caps and one plain column between them, which the game
//! stretches to a title's width).
//!
//! The output is bought art (ADR-0040): it goes to the private assets
//! checkout, never into this repository.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use serde::Deserialize;
use shuyi_cli::codec::{decode_png, encode_png};

use crate::battler_import::{to_hsv, to_rgb};

/// The source list, from the repository's root.
pub const SOURCES: &str = "assets-src/ui/pictures.ron";
/// Where the pictures go, from the repository's root.
pub const OUT_DIR: &str = "assets-private/game/ui";
/// What a source's path is under, from the repository's root.
pub const PRIVATE_DIR: &str = "assets-private";

/// `--help`.
pub const USAGE: &str = "usage: cargo xtask ui-import [--list]\n\
    cuts the UI kit's pictures named in assets-src/ui/pictures.ron into assets-private/game/ui/";

/// The source list's file.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sources {
    /// Each picture written.
    pub pictures: Vec<Picture>,
}

/// One picture written: its name in `game/ui/`, the file it comes from,
/// and what is done to it on the way.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Picture {
    /// Its file name in `game/ui/`.
    pub out: String,
    /// The bought file, under `assets-private/`.
    pub from: String,
    /// Its cool pixels moved to a hue.
    #[serde(default)]
    pub recolour: Option<Recolour>,
    /// Cut to a plate's caps and one column.
    #[serde(default)]
    pub plate: Option<PlateCut>,
}

/// A recolour (1379's mockup's): every pixel that isn't clear and isn't
/// warm (a hue from 14.4 to 61.2 degrees at saturation over 0.25: the
/// gold) takes `hue`, its saturation raised to at least 0.35 and then
/// multiplied by `sat`, its brightness multiplied by `val` (both kept to 1
/// at most).
#[derive(Debug, Clone, Copy, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Recolour {
    /// The new hue, in degrees.
    pub hue: f32,
    /// What the saturation is multiplied by.
    pub sat: f32,
    /// What the brightness is multiplied by.
    pub val: f32,
}

/// A plate cut down: its `left` and `right` columns kept as end caps, and
/// the one column `column` between them as its plain body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlateCut {
    /// The left cap's width.
    pub left: u32,
    /// The body's column.
    pub column: u32,
    /// The right cap's width.
    pub right: u32,
}

/// Whether `rgb` is the kit's gold, kept by a [`Recolour`]: a hue from
/// 14.4 to 61.2 degrees, saturated over a quarter (its brightest channel
/// more than four times as far from its darkest as from black).
fn warm(rgb: [u8; 3]) -> bool {
    let [red, green, blue] = rgb;
    let (max, min) = (red.max(green).max(blue), red.min(green).min(blue));
    let strong = 4 * u16::from(max - min) > u16::from(max);
    strong && (14.4..=61.2).contains(&to_hsv(rgb).0)
}

/// Recolours the RGBA8 `pixels` by `r`.
pub fn recolour(pixels: &mut [u8], r: Recolour) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let [red, green, blue, alpha] = *pixel;
        if alpha == 0 || warm([red, green, blue]) {
            continue;
        }
        let (_, sat, val) = to_hsv([red, green, blue]);
        let [red, green, blue] = to_rgb(
            r.hue,
            (sat.max(0.35) * r.sat).min(1.0),
            (val * r.val).min(1.0),
        );
        *pixel = [red, green, blue, alpha];
    }
}

/// The `w × h` RGBA8 `pixels` cut down by `cut`: `left + 1 + right` wide.
pub fn cut_plate(w: u32, h: u32, pixels: &[u8], cut: PlateCut) -> Result<(u32, Vec<u8>), String> {
    if cut.left + cut.right > w || cut.column >= w {
        return Err(format!(
            "a plate of caps {} and {} with its body at column {} doesn't fit a picture {w} wide",
            cut.left, cut.right, cut.column
        ));
    }
    let columns: Vec<u32> = (0..cut.left)
        .chain([cut.column])
        .chain(w - cut.right..w)
        .collect();
    let width = u32::try_from(columns.len()).map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(columns.len() * h as usize * 4);
    for y in 0..h {
        for &x in &columns {
            let at = ((y * w + x) * 4) as usize;
            out.extend_from_slice(&pixels[at..at + 4]);
        }
    }
    Ok((width, out))
}

/// Reads the source list.
pub fn read_sources(root: &Path) -> Result<Sources, String> {
    let path = root.join(SOURCES);
    let text = fs::read_to_string(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
    ron::from_str(&text).map_err(|e| format!("{SOURCES}: {e}"))
}

/// `--list`: one line per picture.
pub fn list(sources: &Sources) -> String {
    let mut out = String::new();
    for s in &sources.pictures {
        let how = match (s.recolour, s.plate) {
            (Some(r), _) => format!(" (recoloured to hue {})", r.hue),
            (None, Some(_)) => " (cut to a plate)".to_owned(),
            (None, None) => String::new(),
        };
        let _ = writeln!(out, "{} <- {}{how}", s.out, s.from);
    }
    out
}

/// Writes every picture of the source list into `game/ui/`.
pub fn run(root: &Path) -> Result<String, String> {
    let sources = read_sources(root)?;
    let private = root.join(PRIVATE_DIR);
    let out_dir = root.join(OUT_DIR);
    if !private.join("library").is_dir() || !private.join("game").is_dir() {
        return Err(format!(
            "{PRIVATE_DIR}/library/ or {PRIVATE_DIR}/game/ is missing: run `cargo shuyi private-assets --library` first"
        ));
    }
    fs::create_dir_all(&out_dir).map_err(|e| format!("creating {}: {e}", out_dir.display()))?;
    for s in &sources.pictures {
        let path = private.join(&s.from);
        let bytes = fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        let (mut w, h, mut pixels) = decode_png(&bytes).map_err(|e| format!("{}: {e}", s.from))?;
        if let Some(r) = s.recolour {
            recolour(&mut pixels, r);
        }
        if let Some(cut) = s.plate {
            (w, pixels) = cut_plate(w, h, &pixels, cut).map_err(|e| format!("{}: {e}", s.out))?;
        }
        let out = out_dir.join(&s.out);
        fs::write(&out, encode_png(w, h, &pixels)?)
            .map_err(|e| format!("writing {}: {e}", out.display()))?;
    }
    Ok(format!(
        "ui-import: wrote {} pictures to {OUT_DIR}/; commit and push in assets-private/, then run \
         `cargo shuyi private-assets --pin`",
        sources.pictures.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const GOLD: [u8; 4] = [196, 162, 115, 255];
    const PLUM: [u8; 4] = [58, 44, 54, 255];
    const CLEAR: [u8; 4] = [0, 0, 0, 0];

    #[test]
    fn a_recolour_moves_the_plum_and_keeps_the_gold_and_the_clear() {
        let mut pixels = [GOLD, PLUM, CLEAR].concat();
        recolour(
            &mut pixels,
            Recolour {
                hue: 218.0,
                sat: 1.6,
                val: 1.5,
            },
        );
        assert_eq!(&pixels[0..4], &GOLD);
        assert_eq!(&pixels[8..12], &CLEAR);
        let (hue, sat, val) = to_hsv([pixels[4], pixels[5], pixels[6]]);
        assert!((hue - 218.0).abs() < 2.0, "hue {hue}");
        // Saturation 0.35 at least, times 1.6; brightness 58/255 × 1.5.
        assert!((sat - 0.56).abs() < 0.03, "sat {sat}");
        assert!((val - 87.0 / 255.0).abs() < 0.01, "val {val}");
        assert_eq!(pixels[7], 255);
        // A greyish yellow isn't gold (too little colour), and a strong
        // blue isn't (another hue): both move.
        let (grey, blue) = ([120, 115, 100, 255], [40, 60, 200, 255]);
        let mut pixels = [grey, blue].concat();
        recolour(
            &mut pixels,
            Recolour {
                hue: 0.0,
                sat: 1.0,
                val: 1.0,
            },
        );
        assert_ne!(&pixels[0..4], &grey);
        assert_ne!(&pixels[4..8], &blue);
        // Gold at exactly a quarter's saturation (50 of 200) isn't gold
        // enough; a step more is.
        assert!(!warm([200, 175, 150]));
        assert!(warm([200, 175, 149]));
    }

    #[test]
    fn a_plate_keeps_its_caps_and_one_column() {
        // 5 wide, 2 tall: columns are 0..5, rows multiply by 10.
        let px = |x: u8, y: u8| [x + 10 * y, 0, 0, 255];
        let pixels: Vec<u8> = (0..2)
            .flat_map(|y| (0..5).flat_map(move |x| px(x, y)))
            .collect();
        let cut = PlateCut {
            left: 1,
            column: 2,
            right: 1,
        };
        let (w, out) = cut_plate(5, 2, &pixels, cut).unwrap();
        assert_eq!(w, 3);
        let reds: Vec<u8> = out.as_chunks::<4>().0.iter().map(|p| p[0]).collect();
        assert_eq!(reds, [0, 2, 4, 10, 12, 14]);
        let wide = PlateCut {
            left: 3,
            column: 2,
            right: 3,
        };
        assert!(cut_plate(5, 2, &pixels, wide).is_err());
        let past = PlateCut {
            left: 1,
            column: 5,
            right: 1,
        };
        assert!(cut_plate(5, 2, &pixels, past).is_err());
        // Caps of 2 and 3 fill 5 columns exactly: that fits.
        let full = PlateCut {
            left: 2,
            column: 2,
            right: 3,
        };
        assert_eq!(cut_plate(5, 2, &pixels, full).map(|(w, _)| w), Ok(6));
        // Caps of 1 and 5 are 6 columns: too many for 5.
        let lopsided = PlateCut {
            left: 1,
            column: 2,
            right: 5,
        };
        assert!(cut_plate(5, 2, &pixels, lopsided).is_err());
    }

    #[test]
    fn the_source_list_reads_and_lists() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let sources = read_sources(&root).unwrap();
        let listed = list(&sources);
        for name in ["panel.png", "menu.png", "map_menu.png", "plate.png"] {
            assert!(listed.contains(name), "{listed}");
        }
        assert!(listed.contains("recoloured to hue 218"));
        assert!(listed.contains("cut to a plate"));
    }

    #[test]
    fn a_missing_checkout_is_an_error() {
        let root = std::env::temp_dir().join("trpg-ui-import-missing");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("assets-src/ui")).unwrap();
        fs::write(root.join(SOURCES), "(pictures: [])").unwrap();
        let err = run(&root).unwrap_err();
        assert!(err.contains("private-assets --library"), "{err}");
        // The bought art's folder alone isn't enough: the library too.
        fs::create_dir_all(root.join("assets-private/game")).unwrap();
        let err = run(&root).unwrap_err();
        assert!(err.contains("private-assets --library"), "{err}");
        // Nor the library alone.
        fs::remove_dir_all(root.join("assets-private/game")).unwrap();
        fs::create_dir_all(root.join("assets-private/library")).unwrap();
        let err = run(&root).unwrap_err();
        assert!(err.contains("private-assets --library"), "{err}");
        // Both: an empty list writes nothing, and says so.
        fs::create_dir_all(root.join("assets-private/game")).unwrap();
        assert!(run(&root).unwrap().contains("wrote 0 pictures"));
        let _ = fs::remove_dir_all(&root);
    }
}
