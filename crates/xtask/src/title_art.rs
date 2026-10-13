//! `cargo xtask title-art`: writes the title screen's pictures (ticket
//! 0811, ADR-0059; `assets/title/README.md`).
//!
//! - The **logo**, `assets/title/logo.png`, from `assets-src/title/logo.txt`
//!   and the palette: our own picture, one character a pixel.
//! - The public **placeholder backdrops**, `assets/title/<name>.png`: three
//!   flat bands of palette colours each, 336×248.
//! - With `--bought`: the bought backdrops, copied from
//!   `assets-private/library/` to `assets-private/game/title/` under the
//!   placeholders' names (ADR-0040: never into this repository).
//!
//! The same sources always give the same pixels: rerun the command after
//! changing the logo's text or the palette.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use trpg_content::PaletteDef;
use trpg_content::battler::BACKDROP_PX;
use trpg_content::image::png_size;
use trpg_content::title::{LOGO_MAX_PX, TITLE_DIR};

use crate::font_atlas::encode_png;

/// Shown for `--help` and after a bad argument.
pub const USAGE: &str = "usage: cargo xtask title-art [--bought]\n\n\
Writes assets/title/logo.png from assets-src/title/logo.txt and the palette, and the\n\
placeholder backdrops beside it.\n\
--bought  instead copy the bought backdrops from assets-private/library/ to\n\
          assets-private/game/title/ (run `cargo xtask private-assets --library` first;\n\
          afterwards commit and push in assets-private/, then run\n\
          `cargo xtask private-assets --pin`)";

/// The logo's source, relative to the repo root.
pub const LOGO_SOURCE: &str = "assets-src/title/logo.txt";
/// The logo's file name in the title directory.
pub const LOGO_FILE: &str = "logo.png";
/// The line of the logo's source that ends its colours and starts its rows.
pub const ROWS_MARK: &str = "---";
/// The sorted bought packs, relative to the repo root.
pub const LIBRARY_DIR: &str = "assets-private/library";
/// The private assets the game is built with, relative to the repo root.
pub const GAME_DIR: &str = "assets-private/game";

/// One backdrop: its file name, the palette names of its placeholder's
/// sky, far and ground bands, and the bought picture, relative to
/// [`LIBRARY_DIR`].
pub type Backdrop = (&'static str, [&'static str; 3], &'static str);

/// The title's backdrops (`docs/design/look-and-feel.md`, *Title screen*).
pub const BACKDROPS: [Backdrop; 7] = [
    (
        "fields_a_dusk.png",
        ["attack_range", "peak_bg", "thicket"],
        "tiny-tales/battle-backgrounds/Battleback_FieldsA_Dusk.png",
    ),
    (
        "fields_a_night.png",
        ["sea", "water_bg", "forest_bg"],
        "tiny-tales/battle-backgrounds/Battleback_FieldsA_Night.png",
    ),
    (
        "fields_b_day.png",
        ["water", "thicket", "forest"],
        "tiny-tales/battle-backgrounds/Battleback_FieldsB_Day.png",
    ),
    (
        "fields_b_dusk.png",
        ["effect", "thicket_bg", "thicket"],
        "tiny-tales/battle-backgrounds/Battleback_FieldsB_Dusk.png",
    ),
    (
        "fields_b_night.png",
        ["move_range", "thicket_bg", "heal_range"],
        "tiny-tales/battle-backgrounds/Battleback_FieldsB_Night.png",
    ),
    (
        "meadows_dusk.png",
        ["fire_bg", "mountain_bg", "grass"],
        "tiny-tales/battle-backgrounds/Battleback_Meadows_Dusk.png",
    ),
    (
        "meadows_night.png",
        ["water_bg", "ice_bg", "heal_range"],
        "tiny-tales/battle-backgrounds/Battleback_Meadows_Night.png",
    ),
];

/// The first row of a placeholder's far band and of its ground band.
pub const BANDS: [u32; 2] = [84, 120];

/// A picture: width, height and RGBA8 pixels, row-major.
pub type Picture = (u32, u32, Vec<u8>);

/// The embedded palette.
fn palette() -> Result<PaletteDef, String> {
    PaletteDef::load().map_err(|errors| {
        let lines: Vec<String> = errors.iter().map(ToString::to_string).collect();
        lines.join("\n")
    })
}

/// The logo `source` describes, in `palette`'s colours. Before
/// [`ROWS_MARK`], each line is a character and the palette name it stands
/// for; after it, each line is a row of pixels, `.` for a clear one.
/// Blank lines and lines starting with `#` are skipped. Errors name the
/// line (from 1).
pub fn parse_logo(source: &str, palette: &PaletteDef) -> Result<Picture, String> {
    let mut colors: BTreeMap<char, [u8; 3]> = BTreeMap::new();
    let mut rows: Vec<&str> = Vec::new();
    let mut in_rows = false;
    let mut rgba = Vec::new();
    for (n, line) in source.lines().enumerate() {
        let n = n + 1;
        if line.trim().is_empty() || line.starts_with('#') {
            continue;
        }
        if line == ROWS_MARK {
            in_rows = true;
        } else if in_rows {
            if rows
                .first()
                .is_some_and(|r| r.chars().count() != line.chars().count())
            {
                return Err(format!("line {n}: every row must be as wide as the first"));
            }
            for key in line.chars() {
                let pixel = match (key, colors.get(&key)) {
                    ('.', _) => [0; 4],
                    (_, Some(&[r, g, b])) => [r, g, b, 255],
                    (_, None) => return Err(format!("line {n}: no colour for '{key}'")),
                };
                rgba.extend(pixel);
            }
            rows.push(line);
        } else {
            let mut parts = line.split_whitespace();
            let (key, name) = (parts.next().unwrap_or_default(), parts.next());
            let mut chars = key.chars();
            let (Some(key), None, Some(name), None) =
                (chars.next(), chars.next(), name, parts.next())
            else {
                return Err(format!("line {n}: expected a character and a palette name"));
            };
            let color = palette
                .get(name)
                .ok_or_else(|| format!("line {n}: no palette colour {name}"))?;
            if key == '.' || colors.insert(key, color).is_some() {
                return Err(format!("line {n}: '{key}' is taken"));
            }
        }
    }
    let width = rows.first().map_or(0, |r| r.chars().count());
    let size = u32::try_from(width)
        .ok()
        .zip(u32::try_from(rows.len()).ok());
    match size {
        Some((w, h)) if w > 0 && w <= LOGO_MAX_PX.0 && h <= LOGO_MAX_PX.1 => Ok((w, h, rgba)),
        _ => Err(format!(
            "the logo is {width}×{} px; it must be 1×1 to {}×{} px",
            rows.len(),
            LOGO_MAX_PX.0,
            LOGO_MAX_PX.1
        )),
    }
}

/// A placeholder backdrop: `colors` as the sky, far and ground bands.
pub fn placeholder(colors: [[u8; 3]; 3]) -> Picture {
    let (width, height) = BACKDROP_PX;
    let band = |y: u32| usize::from(y >= BANDS[0]) + usize::from(y >= BANDS[1]);
    let rgba = (0..height)
        .flat_map(|y| {
            let [r, g, b] = colors[band(y)];
            (0..width).flat_map(move |_| [r, g, b, 255])
        })
        .collect();
    (width, height, rgba)
}

/// Every public file the command writes: its name in the title directory
/// and its picture. `logo_source` is the text of [`LOGO_SOURCE`].
pub fn pictures(logo_source: &str) -> Result<Vec<(&'static str, Picture)>, String> {
    let palette = palette()?;
    let logo = parse_logo(logo_source, &palette).map_err(|e| format!("{LOGO_SOURCE}: {e}"))?;
    let mut out = vec![(LOGO_FILE, logo)];
    for (name, bands, _) in BACKDROPS {
        let color = |band: &str| {
            palette
                .get(band)
                .ok_or_else(|| format!("no palette colour {band}"))
        };
        let colors = [color(bands[0])?, color(bands[1])?, color(bands[2])?];
        out.push((name, placeholder(colors)));
    }
    Ok(out)
}

/// Writes `bytes` to `path`, making its directory.
fn write(path: &Path, bytes: &[u8]) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).map_err(|e| format!("creating {}: {e}", dir.display()))?;
    }
    fs::write(path, bytes).map_err(|e| format!("writing {}: {e}", path.display()))
}

/// Runs the command without `--bought`: writes the logo and the
/// placeholders under repo root `root`.
pub fn run(root: &Path) -> Result<String, String> {
    let source = root.join(LOGO_SOURCE);
    let source =
        fs::read_to_string(&source).map_err(|e| format!("reading {}: {e}", source.display()))?;
    let mut lines = Vec::new();
    for (name, (w, h, rgba)) in pictures(&source)? {
        let shown = format!("assets/{TITLE_DIR}/{name}");
        write(&root.join(&shown), &encode_png(w, h, &rgba)?)?;
        lines.push(format!("title-art: wrote {shown} ({w}×{h})"));
    }
    Ok(lines.join("\n"))
}

/// Runs the command with `--bought`: copies each of `backdrops`' bought
/// pictures into the private assets under repo root `root`.
pub fn run_bought(root: &Path, backdrops: &[Backdrop]) -> Result<String, String> {
    let library = root.join(LIBRARY_DIR);
    let game = root.join(GAME_DIR);
    if !library.is_dir() || !game.is_dir() {
        return Err(format!(
            "{LIBRARY_DIR}/ or {GAME_DIR}/ is missing: run `cargo xtask private-assets --library` first"
        ));
    }
    let mut files = Vec::new();
    for (name, _, source) in backdrops {
        let path = library.join(source);
        let bytes = fs::read(&path).map_err(|e| format!("reading {}: {e}", path.display()))?;
        if png_size(&bytes) != Some(BACKDROP_PX) {
            return Err(format!(
                "{source} is not a {}×{} px backdrop",
                BACKDROP_PX.0, BACKDROP_PX.1
            ));
        }
        files.push((format!("{TITLE_DIR}/{name}"), bytes));
    }
    for (relative, bytes) in &files {
        write(&game.join(relative), bytes)?;
    }
    Ok(format!(
        "title-art: copied {} backdrops to {GAME_DIR}/{TITLE_DIR}/",
        files.len()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::font_atlas::decode_png;

    fn repo() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn temp(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("xtask-title-art-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    fn logo(source: &str) -> Result<Picture, String> {
        parse_logo(source, &PaletteDef::load().unwrap())
    }

    #[test]
    fn a_logo_is_its_rows_in_the_palettes_colours() {
        let palette = PaletteDef::load().unwrap();
        let solid = |name: &str| {
            let [r, g, b] = palette.get(name).unwrap();
            [r, g, b, 255]
        };
        let (width, height, rgba) =
            logo("# ours\n\nc cursor\ns  sea\n---\nc.s\n\n# two rows\n.cc\n").unwrap();
        assert_eq!((width, height), (3, 2));
        let (pale, dark, clear) = (solid("cursor"), solid("sea"), [0; 4]);
        assert_ne!(pale, dark);
        assert_eq!(rgba, [pale, clear, dark, clear, pale, pale].concat());
    }

    #[test]
    fn logo_errors_name_the_line() {
        let err = |source: &str| logo(source).unwrap_err();
        assert_eq!(
            err("c cursor\n---\ncc\nc\n"),
            "line 4: every row must be as wide as the first"
        );
        assert_eq!(err("c cursor\n---\ncx\n"), "line 3: no colour for 'x'");
        assert_eq!(
            err("c\n---\nc\n"),
            "line 1: expected a character and a palette name"
        );
        assert_eq!(
            err("cc cursor\n"),
            "line 1: expected a character and a palette name"
        );
        assert_eq!(
            err("c cursor sea\n"),
            "line 1: expected a character and a palette name"
        );
        assert_eq!(err("c nope\n"), "line 1: no palette colour nope");
        assert_eq!(err("c cursor\nc sea\n"), "line 2: 'c' is taken");
        assert_eq!(err(". sea\n"), "line 1: '.' is taken");
        assert_eq!(
            err("c cursor\n---\n"),
            "the logo is 0×0 px; it must be 1×1 to 100×34 px"
        );
        assert_eq!(
            err("c cursor\n"),
            "the logo is 0×0 px; it must be 1×1 to 100×34 px"
        );
    }

    #[test]
    fn a_logo_may_be_as_big_as_the_limit_and_no_bigger() {
        let rows = |w: usize, h: usize| {
            format!(
                "c cursor\n---\n{}",
                format!("{}\n", "c".repeat(w)).repeat(h)
            )
        };
        assert_eq!(logo(&rows(100, 34)).map(|(w, h, _)| (w, h)), Ok((100, 34)));
        assert_eq!(logo(&rows(1, 1)).map(|(w, h, _)| (w, h)), Ok((1, 1)));
        assert_eq!(
            logo(&rows(101, 34)).unwrap_err(),
            "the logo is 101×34 px; it must be 1×1 to 100×34 px"
        );
        assert_eq!(
            logo(&rows(100, 35)).unwrap_err(),
            "the logo is 100×35 px; it must be 1×1 to 100×34 px"
        );
    }

    #[test]
    fn a_placeholder_is_three_bands() {
        let (w, h, rgba) = placeholder([[1, 1, 1], [2, 2, 2], [3, 3, 3]]);
        assert_eq!((w, h), (336, 248));
        assert_eq!(rgba.len(), 336 * 248 * 4);
        let at = |x: usize, y: usize| rgba[(y * 336 + x) * 4..][..4].to_vec();
        assert_eq!(at(0, 0), [1, 1, 1, 255]);
        assert_eq!(at(335, 83), [1, 1, 1, 255]);
        assert_eq!(at(0, 84), [2, 2, 2, 255]);
        assert_eq!(at(335, 119), [2, 2, 2, 255]);
        assert_eq!(at(0, 120), [3, 3, 3, 255]);
        assert_eq!(at(335, 247), [3, 3, 3, 255]);
    }

    #[test]
    fn the_backdrops_have_their_own_names_and_looks() {
        for (i, a) in BACKDROPS.iter().enumerate() {
            for b in &BACKDROPS[..i] {
                assert_ne!(a.0, b.0);
                assert_ne!(a.1, b.1, "{} and {} would look the same", a.0, b.0);
                assert_ne!(a.2, b.2);
            }
        }
    }

    #[test]
    fn committed_pictures_are_what_the_tool_makes() {
        let source = fs::read_to_string(repo().join(LOGO_SOURCE)).unwrap();
        let made = pictures(&source).unwrap();
        assert_eq!(made.len(), 8);
        for (name, picture) in made {
            let committed = fs::read(repo().join(format!("assets/title/{name}"))).unwrap();
            assert!(
                decode_png(&committed).unwrap() == picture,
                "{name}: rerun `cargo xtask title-art`"
            );
        }
    }

    #[test]
    fn run_writes_the_pictures_and_names_them() {
        let root = temp("run");
        assert!(run(&root).unwrap_err().starts_with("reading "));
        fs::create_dir_all(root.join("assets-src/title")).unwrap();
        fs::write(root.join(LOGO_SOURCE), "c cursor\n---\ncc\n").unwrap();
        let summary = run(&root).unwrap();
        let lines: Vec<_> = summary.lines().collect();
        assert_eq!(lines.len(), 8);
        assert_eq!(lines[0], "title-art: wrote assets/title/logo.png (2×1)");
        assert_eq!(
            lines[1],
            "title-art: wrote assets/title/fields_a_dusk.png (336×248)"
        );
        let written = fs::read(root.join("assets/title/logo.png")).unwrap();
        assert_eq!(decode_png(&written).unwrap().0, 2);
        let written = fs::read(root.join("assets/title/meadows_night.png")).unwrap();
        assert_eq!(png_size(&written), Some((336, 248)));
        // A bad logo fails with its file and writes nothing new.
        fs::write(root.join(LOGO_SOURCE), "c cursor\n---\nx\n").unwrap();
        assert_eq!(
            run(&root).unwrap_err(),
            "assets-src/title/logo.txt: line 3: no colour for 'x'"
        );
        fs::remove_dir_all(&root).unwrap();
    }

    #[test]
    fn bought_backdrops_are_copied_under_the_placeholders_names() {
        const ONE: [Backdrop; 1] = [("a.png", ["sea"; 3], "tt/a_bought.png")];
        let root = temp("bought");
        assert!(run_bought(&root, &ONE).unwrap_err().contains("is missing"));
        fs::create_dir_all(root.join(LIBRARY_DIR).join("tt")).unwrap();
        assert!(run_bought(&root, &ONE).unwrap_err().contains("is missing"));
        fs::create_dir_all(root.join(GAME_DIR)).unwrap();
        assert!(run_bought(&root, &ONE).unwrap_err().starts_with("reading "));
        let (w, h, rgba) = placeholder([[9, 9, 9]; 3]);
        let png = encode_png(w, h, &rgba).unwrap();
        let source = root.join(LIBRARY_DIR).join("tt/a_bought.png");
        fs::write(&source, encode_png(2, 1, &[0; 8]).unwrap()).unwrap();
        assert_eq!(
            run_bought(&root, &ONE).unwrap_err(),
            "tt/a_bought.png is not a 336×248 px backdrop"
        );
        assert!(!root.join(GAME_DIR).join("title").exists());
        fs::write(&source, &png).unwrap();
        assert_eq!(
            run_bought(&root, &ONE).unwrap(),
            "title-art: copied 1 backdrops to assets-private/game/title/"
        );
        assert_eq!(
            fs::read(root.join(GAME_DIR).join("title/a.png")).unwrap(),
            png
        );
        fs::remove_dir_all(&root).unwrap();
    }
}
