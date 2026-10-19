//! `cargo xtask playdate-images`: the Playdate's one-bit pictures (ticket
//! 1040, ADR-0085) as files to look at and to work from. The Playdate's
//! build makes its own when it is built (`crates/content/build.rs`); this
//! makes the same ones, by the same table (`trpg_content::playdate_images`):
//!
//! - from `assets/` into `target/playdate/images/`, never committed;
//! - from the bought art in `assets-private/game/`, if it is checked out,
//!   into the private checkout's own `generated/playdate-images/`, never
//!   into this repository (ADR-0040).
//!
//! Each picture is a `.1bit` file at its PNG's path and, to look at, a
//! black and white PNG of each style beside it (`neutral.a-lines.png`).
//! A touched-up picture (ticket 1042), a `<stem>.1bit` beside the PNG, is
//! copied as it is instead of converted, as the Playdate's build embeds it
//! (ticket 1061): what this shows is what ships.
//!
//! `frame-png --playdate` draws its pictures the same way ([`Pictures`]).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use shuyi_assets::bitmap::convert::{Rgba, convert, png_pixels};
use shuyi_assets::bitmap::{Bitmap, EXTENSION};
use trpg_content::playdate_images::{Job, jobs, touch_ups};
use trpg_ui::ImageId;
use trpg_ui::onebit::{Image, Images};

use shuyi_cli::codec::encode_png;

/// Where the pictures of `assets/` go, under the repository's root.
pub const OUT: &str = "target/playdate/images";

/// The bought art, under the repository's root (ADR-0040).
const PRIVATE: &str = "assets-private/game";

/// Where the bought art's pictures go, under the repository's root: the
/// private checkout's own folder of generated files.
pub const PRIVATE_OUT: &str = "assets-private/generated/playdate-images";

pub const USAGE: &str = "usage: cargo xtask playdate-images\n\n\
Converts the pictures the Playdate draws one-bit (crates/content/src/playdate_images.rs)\n\
from assets/ into target/playdate/images/, and the bought ones in assets-private/game/\n\
(if checked out) into assets-private/generated/playdate-images/: a .1bit file at each\n\
PNG's path and a PNG of each style to look at. The Playdate's build converts its own.";

/// The one-bit picture file of PNG file `png`, made as `job` says.
pub fn convert_png(png: &[u8], job: &Job<'_>) -> Result<Vec<u8>, String> {
    let (width, height, rgba) = png_pixels(png)?;
    let image = Rgba::new(width, height, &rgba).ok_or("decoded to the wrong number of pixels")?;
    convert(&image, job.scale, &job.kind.methods())
}

/// The names of the style files of the picture at `path` (one per style,
/// `<stem>.<style>.png`) and its `.1bit` file's name.
fn file_names(job: &Job<'_>) -> (String, Vec<String>) {
    let stem = job.path.rsplit_once('.').map_or(job.path, |(stem, _)| stem);
    let styles = job
        .kind
        .styles
        .iter()
        .map(|style| format!("{stem}.{}.png", style.file))
        .collect();
    (format!("{stem}.{EXTENSION}"), styles)
}

/// Style `n` of `bitmap` as a black and white PNG, see-through where the
/// mask is clear.
pub fn style_png(bitmap: &Bitmap<'_>, n: usize) -> Result<Vec<u8>, String> {
    let (width, height) = (bitmap.width(), bitmap.height());
    let mut rgba = Vec::with_capacity(usize::from(width) * usize::from(height) * 4);
    for y in 0..height {
        for x in 0..width {
            rgba.extend(match bitmap.pixel(n, x, y) {
                Some(true) => [255; 4],
                Some(false) => [0, 0, 0, 255],
                None => [0; 4],
            });
        }
    }
    encode_png(u32::from(width), u32::from(height), &rgba)
}

/// Every file's path under `dir`, `/`-separated and sorted.
fn files_under(dir: &Path) -> Result<Vec<String>, String> {
    fn walk(dir: &Path, prefix: &str, out: &mut Vec<String>) -> Result<(), String> {
        let entries = fs::read_dir(dir).map_err(|e| format!("reading {}: {e}", dir.display()))?;
        for entry in entries {
            let entry = entry.map_err(|e| format!("reading {}: {e}", dir.display()))?;
            let name = entry.file_name().to_string_lossy().into_owned();
            let path = format!("{prefix}{name}");
            if entry.path().is_dir() {
                walk(&entry.path(), &format!("{path}/"), out)?;
            } else {
                out.push(path);
            }
        }
        Ok(())
    }
    let mut out = Vec::new();
    walk(dir, "", &mut out)?;
    out.sort_unstable();
    Ok(out)
}

/// Converts the pictures of the folder `from` (laid out as `assets/`)
/// into `to`, copying the touched-up ones; returns how many.
pub fn convert_folder(from: &Path, to: &Path) -> Result<usize, String> {
    let files = files_under(from)?;
    let read = |path: &str| fs::read_to_string(from.join(path)).ok();
    let jobs = jobs(&files, read).map_err(|e| format!("{}/{e}", from.display()))?;
    let touched = touch_ups(&files);
    for job in &jobs {
        let file = if let Some(touched) = touched.get(job.path) {
            let shown = from.join(touched);
            fs::read(&shown).map_err(|e| format!("reading {}: {e}", shown.display()))?
        } else {
            let shown = from.join(job.path);
            let png = fs::read(&shown).map_err(|e| format!("reading {}: {e}", shown.display()))?;
            convert_png(&png, job).map_err(|e| format!("{}: {e}", shown.display()))?
        };
        let shown = from.join(job.path);
        let bitmap = Bitmap::parse(&file).map_err(|e| format!("{}: {e}", shown.display()))?;
        let (bits, styles) = file_names(job);
        let mut outputs = vec![(bits, file.clone())];
        // A touched-up picture may hold fewer styles than its kind.
        for (n, name) in styles.into_iter().take(bitmap.styles()).enumerate() {
            outputs.push((name, style_png(&bitmap, n)?));
        }
        for (name, bytes) in outputs {
            let out = to.join(name);
            if let Some(dir) = out.parent() {
                fs::create_dir_all(dir).map_err(|e| format!("making {}: {e}", dir.display()))?;
            }
            fs::write(&out, bytes).map_err(|e| format!("writing {}: {e}", out.display()))?;
        }
    }
    Ok(jobs.len())
}

/// Runs the command under the repository's `root`; returns what to print.
pub fn run(root: &Path) -> Result<String, String> {
    let out = root.join(OUT);
    let public = convert_folder(&root.join("assets"), &out)?;
    let summary = format!("playdate-images: {public} pictures of assets/ into {OUT}/");
    let private = root.join(PRIVATE);
    let bought = if private.is_dir() {
        let count = convert_folder(&private, &root.join(PRIVATE_OUT))?;
        format!("playdate-images: {count} bought pictures of {PRIVATE}/ into {PRIVATE_OUT}/")
    } else {
        format!("playdate-images: no {PRIVATE}/, so no bought pictures")
    };
    Ok(format!(
        "{summary}
{bought}"
    ))
}

/// The one-bit pictures of the game's bundle, made as the Playdate's
/// build makes its own, by image path: for `frame-png --playdate`, whose
/// bundle is the PC's, its PNGs (the bought ones too in a build with the
/// `private-assets` feature).
pub struct Pictures(BTreeMap<String, Vec<u8>>);

impl Pictures {
    /// Converts every picture of the game's bundle the Playdate draws
    /// one-bit.
    pub fn of_the_bundle() -> Result<Self, String> {
        let bundle = trpg_content::bundle::install();
        let files: Vec<String> = bundle
            .files_under("")
            .into_iter()
            .map(String::from)
            .collect();
        let read = |path: &str| bundle.file(path).map(String::from);
        let mut pictures = BTreeMap::new();
        for job in jobs(&files, read)? {
            let png = bundle.bytes(job.path).unwrap_or_default();
            let file = convert_png(png, &job).map_err(|e| format!("{}: {e}", job.path))?;
            pictures.insert(job.path.to_string(), file);
        }
        Ok(Self(pictures))
    }
}

impl Images for Pictures {
    fn image(&self, id: ImageId) -> Option<Image<'_>> {
        let bitmap = Bitmap::parse(self.0.get(id.path())?).ok()?;
        Image::from_bitmap(&bitmap, 0)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    /// The repository root a test runs in.
    fn repo() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
    }

    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "xtask-playdate-images-{name}-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        dir
    }

    /// The command converts the pictures the Playdate's build does, each a
    /// `.1bit` file equal to the one in the Playdate's bundle, and a PNG
    /// of each style.
    #[test]
    fn the_command_makes_what_the_playdates_build_makes() {
        let out = scratch("assets");
        let count = convert_folder(&repo().join("assets"), &out).unwrap();
        assert!(count >= 20, "{count}");
        let face = fs::read(out.join("portraits/lead_f/happy.1bit")).unwrap();
        let bitmap = Bitmap::parse(&face).unwrap();
        assert_eq!(
            (bitmap.width(), bitmap.height(), bitmap.styles()),
            (64, 64, 4)
        );
        for style in ["a", "a-lines", "f-lines", "g"] {
            let png = fs::read(out.join(format!("portraits/lead_f/happy.{style}.png"))).unwrap();
            let (w, h, _) = png_pixels(&png).unwrap();
            assert_eq!((w, h), (64, 64), "{style}");
        }
        assert!(out.join("title/fields_b_dusk.a.png").is_file());
        assert!(out.join("battlers/brigand.1bit").is_file());
        assert!(!out.join("title/logo.1bit").exists());
        let pictures = Pictures::of_the_bundle().unwrap();
        assert_eq!(pictures.0.get("portraits/lead_f/happy.png"), Some(&face));
        // The renderer finds it by its image id, in style A.
        let table = trpg_content::image::load().unwrap();
        let id = table.id("portraits/lead_f/happy.png").unwrap();
        assert_eq!(pictures.image(id), Image::from_bitmap(&bitmap, 0));
        assert!(pictures.image(id).is_some());
        let logo = table.id("title/logo.png").unwrap();
        assert_eq!(pictures.image(logo), None);
        let _ = fs::remove_dir_all(&out);
    }

    #[test]
    fn a_style_png_is_black_white_and_see_through() {
        let file =
            shuyi_assets::bitmap::encode((2, 2), (1, 1), &[&[0x80, 0x40]], Some(&[0xc0, 0x40]))
                .unwrap();
        let bitmap = Bitmap::parse(&file).unwrap();
        let png = style_png(&bitmap, 0).unwrap();
        let (w, h, rgba) = png_pixels(&png).unwrap();
        assert_eq!((w, h), (2, 2));
        assert_eq!(rgba, [[255; 4], [0, 0, 0, 255], [0; 4], [255; 4]].concat());
    }

    /// A `<stem>.1bit` beside a PNG is the picture, copied as it is, and
    /// its styles are drawn from it, not from the PNG.
    #[test]
    fn a_touched_up_picture_is_copied_not_converted() {
        let from = scratch("touched-from");
        let face = repo().join("assets/portraits/lead_m/neutral.png");
        let portraits = from.join("portraits/x");
        fs::create_dir_all(&portraits).unwrap();
        fs::copy(&face, portraits.join("neutral.png")).unwrap();
        fs::copy(&face, portraits.join("sad.png")).unwrap();
        let touched =
            shuyi_assets::bitmap::encode((2, 2), (1, 1), &[&[0x80, 0x40]], Some(&[0xc0, 0x40]))
                .unwrap();
        fs::write(portraits.join("neutral.1bit"), &touched).unwrap();
        let to = scratch("touched-to");
        assert_eq!(convert_folder(&from, &to).unwrap(), 2);
        assert_eq!(
            fs::read(to.join("portraits/x/neutral.1bit")).unwrap(),
            touched
        );
        let (w, h, _) =
            png_pixels(&fs::read(to.join("portraits/x/neutral.a.png")).unwrap()).unwrap();
        assert_eq!((w, h), (2, 2));
        assert!(!to.join("portraits/x/neutral.a-lines.png").exists());
        // The one beside no touch-up is converted.
        let sad = Bitmap::parse(&fs::read(to.join("portraits/x/sad.1bit")).unwrap())
            .map(|b| (b.width(), b.height()))
            .unwrap();
        assert_eq!(sad, (64, 64));
        let _ = fs::remove_dir_all(&from);
        let _ = fs::remove_dir_all(&to);
    }

    #[test]
    fn the_bought_art_is_converted_only_when_checked_out() {
        let root = scratch("root");
        let face = repo().join("assets/portraits/lead_m/neutral.png");
        let portraits = root.join("assets/portraits/x");
        fs::create_dir_all(&portraits).unwrap();
        fs::copy(&face, portraits.join("neutral.png")).unwrap();
        let summary = run(&root).unwrap();
        assert!(summary.contains("1 pictures of assets/"), "{summary}");
        assert!(summary.contains("no assets-private/game/"), "{summary}");
        assert!(root.join(OUT).join("portraits/x/neutral.1bit").is_file());
        let bought = root.join(PRIVATE).join("portraits/y");
        fs::create_dir_all(&bought).unwrap();
        fs::copy(&face, bought.join("sad.png")).unwrap();
        let summary = run(&root).unwrap();
        assert!(summary.contains("1 bought pictures"), "{summary}");
        assert!(
            root.join(PRIVATE_OUT)
                .join("portraits/y/sad.1bit")
                .is_file()
        );
        assert!(!root.join(OUT).join("portraits/y").exists());
        let _ = fs::remove_dir_all(&root);
    }
}
