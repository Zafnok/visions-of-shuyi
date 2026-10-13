//! The title screen's art (`assets/title/`, ticket 0811, ADR-0057): the
//! logo, a picture of the game's name, and the backdrops one of which is
//! behind it each launch. `content` reads only each image's size; the
//! title screen draws them as sprites (ADR-0038). Format:
//! `assets/title/README.md`.

use serde::Deserialize;

use crate::battler::BACKDROP_PX;
use crate::bundle;
use crate::error::ContentError;
use crate::image::{ImageId, ImageTable};
use crate::ron_loader::parse_ron;

/// Directory of the title art inside the asset bundle.
pub const TITLE_DIR: &str = "title";
/// The title art's file inside the asset bundle.
pub const TITLE_PATH: &str = "title/title.ron";
/// The biggest a logo may be, in its own pixels, width × height. One of
/// its pixels is drawn 8 console pixels big (a cell's width), so this is
/// the console's width by the rows above the menu.
pub const LOGO_MAX_PX: (u32, u32) = (100, 34);

/// The title screen's pictures.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TitleArt {
    /// The game's name as a picture, with its size in its own pixels;
    /// `None` only in the empty default.
    pub logo: Option<(ImageId, (u32, u32))>,
    /// The pictures one of which is behind the logo, in file order. May
    /// be empty: the logo is then on the plain background.
    pub backdrops: Vec<ImageId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct TitleFile {
    logo: String,
    #[serde(default)]
    backdrops: Vec<String>,
}

/// Loads the embedded title art file, checking each image against
/// `images`. Reports every problem.
pub fn load(images: &ImageTable) -> Result<TitleArt, Vec<ContentError>> {
    let file = bundle::display_path(TITLE_PATH);
    let source = bundle::file(TITLE_PATH)
        .ok_or_else(|| vec![ContentError::new(&file, "file not found in asset bundle")])?;
    from_source(&file, source, images)
}

/// Parses title art `source` (errors attributed to `file`), checking each
/// image against `images`: the logo must be there and at most
/// [`LOGO_MAX_PX`]; each backdrop must be there, [`BACKDROP_PX`] big and
/// named once. Reports every problem.
pub fn from_source(
    file: &str,
    source: &str,
    images: &ImageTable,
) -> Result<TitleArt, Vec<ContentError>> {
    let raw: TitleFile = parse_ron(file, source).map_err(|e| vec![e])?;
    let mut errors = Vec::new();
    let mut check = |what: String, result: Result<(ImageId, (u32, u32)), String>| {
        if let Err(problem) = &result {
            errors.push(ContentError::new(file, format!("{what}: {problem}")));
        }
        result.ok()
    };
    let logo = check("logo".to_owned(), check_logo(&raw.logo, images));
    let mut backdrops = Vec::new();
    for (i, relative) in raw.backdrops.iter().enumerate() {
        let result = if raw.backdrops[..i].contains(relative) {
            Err(format!("{relative} is named twice"))
        } else {
            check_backdrop(relative, images)
        };
        if let Some((image, _)) = check(format!("backdrop {}", i + 1), result) {
            backdrops.push(image);
        }
    }
    if errors.is_empty() {
        Ok(TitleArt { logo, backdrops })
    } else {
        Err(errors)
    }
}

/// The image at `relative` (to the title directory) with its size, or
/// that it is missing, in plain words.
fn find(relative: &str, images: &ImageTable) -> Result<(ImageId, (u32, u32)), String> {
    let path = format!("{TITLE_DIR}/{relative}");
    let found = images.id(&path).and_then(|id| Some((id, images.info(id)?)));
    let (id, size) = found.ok_or_else(|| {
        format!(
            "{} is not a PNG image in the assets",
            bundle::display_path(&path)
        )
    })?;
    Ok((id, (size.width, size.height)))
}

/// The logo at `relative`, or what is wrong with it: it must be in
/// `images` and at most [`LOGO_MAX_PX`] big.
fn check_logo(relative: &str, images: &ImageTable) -> Result<(ImageId, (u32, u32)), String> {
    let (id, (w, h)) = find(relative, images)?;
    let (max_w, max_h) = LOGO_MAX_PX;
    if w > max_w || h > max_h {
        return Err(format!(
            "{relative} is {w}×{h} px; a logo is at most {max_w}×{max_h} px"
        ));
    }
    Ok((id, (w, h)))
}

/// The backdrop at `relative`, or what is wrong with it: it must be in
/// `images` and be [`BACKDROP_PX`] big.
fn check_backdrop(relative: &str, images: &ImageTable) -> Result<(ImageId, (u32, u32)), String> {
    let (id, (w, h)) = find(relative, images)?;
    if (w, h) != BACKDROP_PX {
        let (bw, bh) = BACKDROP_PX;
        return Err(format!(
            "{relative} is {w}×{h} px; a backdrop is {bw}×{bh} px"
        ));
    }
    Ok((id, (w, h)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::image::png_header;

    const FILE: &str = "assets/title/title.ron";

    /// A table with a logo, an oversized logo in each direction, two
    /// backdrops and a picture of another size.
    fn images() -> ImageTable {
        let files: [(&'static str, u32, u32); 8] = [
            ("title/logo.png", 100, 34),
            ("title/wide.png", 101, 34),
            ("title/tall.png", 100, 35),
            ("title/a.png", 336, 248),
            ("title/b.png", 336, 248),
            ("title/narrow.png", 335, 248),
            ("title/short.png", 336, 247),
            // Not under `title/`: the file can't name it.
            ("images/c.png", 336, 248),
        ];
        let pngs: Vec<_> = files
            .iter()
            .map(|&(path, w, h)| (path, png_header(w, h)))
            .collect();
        ImageTable::from_files(pngs.iter().map(|(path, png)| (*path, &png[..]))).unwrap()
    }

    fn messages(source: &str) -> Vec<String> {
        from_source(FILE, source, &images())
            .unwrap_err()
            .iter()
            .map(ToString::to_string)
            .collect()
    }

    #[test]
    fn the_embedded_title_art_loads() {
        let images = ImageTable::load().unwrap();
        let art = load(&images).unwrap();
        let (logo, size) = art.logo.unwrap();
        assert_eq!(logo.path(), "title/logo.png");
        assert_eq!(Some(size), images.info(logo).map(|i| (i.width, i.height)));
        assert!(art.backdrops.len() >= 2, "the title rotates its backdrops");
        assert!(art.backdrops.iter().all(|b| b.path().starts_with("title/")));
    }

    #[test]
    fn a_logo_and_backdrops_in_file_order() {
        let images = images();
        let art = from_source(
            FILE,
            r#"(logo: "logo.png", backdrops: ["b.png", "a.png"])"#,
            &images,
        )
        .unwrap();
        assert_eq!(
            art.logo,
            Some((images.id("title/logo.png").unwrap(), (100, 34)))
        );
        let paths: Vec<_> = art.backdrops.iter().map(|b| b.path()).collect();
        assert_eq!(paths, ["title/b.png", "title/a.png"]);
        // Backdrops are optional.
        let art = from_source(FILE, r#"(logo: "logo.png")"#, &images).unwrap();
        assert_eq!(art.backdrops, []);
    }

    #[test]
    fn every_problem_is_reported() {
        assert_eq!(
            messages(
                r#"(logo: "wide.png",
                    backdrops: ["a.png", "narrow.png", "a.png", "c.png", "short.png"])"#
            ),
            [
                format!("{FILE}: logo: wide.png is 101×34 px; a logo is at most 100×34 px"),
                format!("{FILE}: backdrop 2: narrow.png is 335×248 px; a backdrop is 336×248 px"),
                format!("{FILE}: backdrop 3: a.png is named twice"),
                format!("{FILE}: backdrop 4: assets/title/c.png is not a PNG image in the assets"),
                format!("{FILE}: backdrop 5: short.png is 336×247 px; a backdrop is 336×248 px"),
            ]
        );
        assert_eq!(
            messages(r#"(logo: "tall.png")"#),
            [format!(
                "{FILE}: logo: tall.png is 100×35 px; a logo is at most 100×34 px"
            )]
        );
        assert_eq!(
            messages(r#"(logo: "none.png")"#),
            [format!(
                "{FILE}: logo: assets/title/none.png is not a PNG image in the assets"
            )]
        );
    }

    #[test]
    fn syntax_errors_and_unknown_fields_are_reported() {
        assert_eq!(messages("(").len(), 1);
        assert!(messages(r#"(logo: "logo.png", extra: 1)"#)[0].contains("extra"));
        assert!(messages("()")[0].contains("logo"));
    }
}
