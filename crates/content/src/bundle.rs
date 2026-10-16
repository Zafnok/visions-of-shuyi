//! The asset bundle: every file under the repo's `assets/` directory, embedded
//! in the binary at compile time (ADR-0005). This is the only way `content`
//! reads data; there is no filesystem I/O.
//!
//! The layering is Shuyi Engine's (`shuyi_assets::bundle`, ticket 1103);
//! this module gives it the game's folders as layers ([`layers`]), installs
//! them once ([`install`]), and keeps the functions every loader calls,
//! which read the installed bundle.
//!
//! With the `private-assets` feature the bought art in `assets-private/game/`
//! is embedded too and laid over `assets/` (ADR-0040): a private file
//! replaces the public file at the same path, and a private-only file is
//! added. Tests and gates build without the feature, so they always read the
//! public placeholders.
//!
//! A debug build, and a build with the `test-fixtures` feature, also embeds
//! the **test files** in `assets-test/` (ADR-0057): the characters, chapter,
//! battles and pictures of the tests and the debug tools. They are added
//! after the story's files ([`TEST_FILES`]); a build that ships has none.
//!
//! Without `std` (the Playdate, ADR-0070 §3) `include_dir` does not build:
//! `assets/` is the table of files `build.rs` writes, and there are no test
//! files and no private layer.
use crate::prelude::*;

#[cfg(feature = "std")]
use include_dir::{Dir, include_dir};
pub use shuyi_assets::bundle::story_only;
use shuyi_assets::bundle::{self as engine, Bundle, Layer};

#[cfg(feature = "std")]
static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets");

/// Every file under `assets/`: its path (relative to `assets/`,
/// `/`-separated) and contents, sorted by path. The bundle without `std`;
/// with it, only the test that keeps the two the same reads it.
#[cfg(any(not(feature = "std"), test))]
static TABLE: &[(&str, &[u8])] = include!(concat!(env!("OUT_DIR"), "/bundle_files.rs"));

#[cfg(all(feature = "std", feature = "private-assets"))]
static PRIVATE: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets-private/game");

#[cfg(all(feature = "std", any(debug_assertions, feature = "test-fixtures")))]
static TEST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets-test");

#[cfg(all(
    feature = "std",
    feature = "private-assets",
    any(debug_assertions, feature = "test-fixtures")
))]
static PRIVATE_TEST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets-private/test");

/// Whether this build embeds the test files (`assets-test/`, ADR-0057): a
/// debug build, or one with the `test-fixtures` feature, with `std`.
/// `false` in every build that ships.
pub const TEST_FILES: bool = cfg!(all(
    feature = "std",
    any(debug_assertions, feature = "test-fixtures")
));

/// Prefix used when showing an asset path to a human (`assets/data/x.ron`).
pub const DISPLAY_ROOT: &str = "assets";

/// The same prefix for a file that comes from the private assets
/// (`assets-private/game/portraits/x.png`).
pub const PRIVATE_DISPLAY_ROOT: &str = "assets-private/game";

/// The same prefix for a test file (`assets-test/maps/test_small.map`).
pub const TEST_DISPLAY_ROOT: &str = "assets-test";

/// The same prefix for a test file that is bought art
/// (`assets-private/test/portraits/test_lord/neutral.png`).
pub const PRIVATE_TEST_DISPLAY_ROOT: &str = "assets-private/test";

/// The game's embedded directories, in the order a file is read from them:
/// the bought art, `assets/`, then the test files' bought art and the test
/// files, each when this build has it.
pub fn layers() -> Vec<Layer> {
    vec![
        #[cfg(all(feature = "std", feature = "private-assets"))]
        Layer::dir(PRIVATE_DISPLAY_ROOT, &PRIVATE).private(),
        #[cfg(feature = "std")]
        Layer::dir(DISPLAY_ROOT, &ASSETS),
        #[cfg(not(feature = "std"))]
        Layer::table(DISPLAY_ROOT, TABLE),
        #[cfg(all(
            feature = "std",
            feature = "private-assets",
            any(debug_assertions, feature = "test-fixtures")
        ))]
        Layer::dir(PRIVATE_TEST_DISPLAY_ROOT, &PRIVATE_TEST)
            .private()
            .test(),
        #[cfg(all(feature = "std", any(debug_assertions, feature = "test-fixtures")))]
        Layer::dir(TEST_DISPLAY_ROOT, &TEST).test(),
    ]
}

/// The game's bundle, the process-wide one: installed from [`layers`] the
/// first time anything reads a file ([`crate::load_embedded`] does first).
pub fn install() -> &'static Bundle {
    engine::get_or_install(|| Bundle::new(layers()))
}

/// Returns the UTF-8 contents of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), or `None` if it is missing or not valid UTF-8.
pub fn file(path: &str) -> Option<&'static str> {
    install().file(path)
}

/// The test files' own file at `path`, with its human-facing name, when
/// the story has a file there too: the entries a data table gains in a
/// build with the test files (ADR-0057). [`file()`] reads the story's.
/// `None` without test files, and for a path only one of the two has.
pub fn test_part(path: &str) -> Option<(String, &'static str)> {
    install().test_part(path)
}

/// Returns the raw bytes of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), e.g. an image, or `None` if it is missing.
pub fn bytes(path: &str) -> Option<&'static [u8]> {
    install().bytes(path)
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file directly inside directory `dir`. A missing directory yields an empty
/// list. Use `""` for the bundle root.
pub fn files_in(dir: &str) -> Vec<&'static str> {
    install().files_in(dir)
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file inside directory `dir` or any directory below it. A missing
/// directory yields an empty list. Use `""` for the whole bundle.
pub fn files_under(dir: &str) -> Vec<&'static str> {
    install().files_under(dir)
}

/// The human-facing name of an asset path, e.g. `assets/data/palette.ron`
/// (`assets-private/game/...` for a file read from the private assets,
/// `assets-test/...` for a test file).
pub fn display_path(path: &str) -> String {
    install().display_path(path)
}

/// The human-facing name of the test file at `path`, there or not: where a
/// tool that makes a test file writes it.
pub fn test_display_path(path: &str) -> String {
    format!("{TEST_DISPLAY_ROOT}/{path}")
}

/// Whether `shown` (a human-facing name, from [`display_path`]) names a
/// file of the private assets: bought art (ADR-0040), the game's or the
/// test files'. Also in a build without them.
pub fn is_private(shown: &str) -> bool {
    [PRIVATE_DISPLAY_ROOT, PRIVATE_TEST_DISPLAY_ROOT]
        .iter()
        .any(|root| engine::shown_under(shown, root))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_existing_file() {
        let text = file("data/palette.ron");
        assert!(text.is_some_and(|t| t.contains("player")));
    }

    #[test]
    fn missing_file_is_none() {
        assert_eq!(file("data/does_not_exist.ron"), None);
    }

    #[test]
    fn reads_binary_file() {
        let png = bytes("fonts/atlas.png").unwrap_or_default();
        assert!(png.starts_with(b"\x89PNG"));
        assert_eq!(bytes("fonts/nope.png"), None);
        assert_eq!(
            bytes("data/palette.ron").map(<[u8]>::len),
            file("data/palette.ron").map(str::len)
        );
    }

    #[test]
    fn lists_files_in_directory() {
        let files = files_in("data");
        assert!(files.contains(&"data/palette.ron"));
        assert!(files.iter().all(|p| p.starts_with("data/")));
        let mut sorted = files.clone();
        sorted.sort_unstable();
        assert_eq!(files, sorted);
    }

    #[test]
    fn root_and_missing_directories() {
        assert!(files_in("").iter().all(|p| !p.contains('/')));
        assert!(files_in("no_such_dir").is_empty());
    }

    #[test]
    fn lists_files_under_a_directory_and_below() {
        let all = files_under("");
        for path in ["data/palette.ron", "fonts/atlas.png", "audio/sfx/heal.wav"] {
            assert!(all.contains(&path), "{path}");
        }
        let mut sorted = all.clone();
        sorted.sort_unstable();
        sorted.dedup();
        assert_eq!(all, sorted);
        let audio = files_under("audio");
        assert!(audio.contains(&"audio/audio.ron"));
        assert!(audio.contains(&"audio/sfx/heal.wav"));
        assert!(audio.iter().all(|p| p.starts_with("audio/")));
        // `data` holds one folder: the colour themes' palettes.
        let data = files_under("data");
        assert!(data.contains(&"data/palette.ron"));
        assert!(data.contains(&"data/palettes/moonlit.ron"));
        assert!(files_in("data").iter().all(|f| data.contains(f)));
        assert!(!files_in("data").contains(&"data/palettes/moonlit.ron"));
        assert!(files_under("no_such_dir").is_empty());
    }

    /// The `no_std` bundle (`build.rs`'s table) and `include_dir` list the
    /// same files of `assets/` with the same bytes, and answer every
    /// function the same, so the two can't drift (ADR-0070 §3).
    #[test]
    fn the_no_std_table_holds_the_same_files_as_include_dir() {
        let embedded = Bundle::new(vec![Layer::dir(DISPLAY_ROOT, &ASSETS)]);
        let table = Bundle::new(vec![Layer::table(DISPLAY_ROOT, TABLE)]);
        let all = embedded.files_under("");
        assert!(all.len() > 100, "{} files", all.len());
        assert_eq!(table.files_under(""), all);
        for path in &all {
            assert_eq!(table.bytes(path), embedded.bytes(path), "{path}");
            assert_eq!(table.file(path), embedded.file(path), "{path}");
            assert_eq!(table.display_path(path), embedded.display_path(path));
        }
        for dir in ["", "data", "audio", "audio/sfx", "lang", "no_such_dir"] {
            assert_eq!(table.files_in(dir), embedded.files_in(dir), "{dir}");
            assert_eq!(table.files_under(dir), embedded.files_under(dir), "{dir}");
        }
        assert_eq!(table.bytes("data/nope.ron"), None);
        assert_eq!(table.file("data/nope.ron"), None);
        assert_eq!(table.test_part("data/characters.ron"), None);
        assert_eq!(table.display_path("data/nope.ron"), "assets/data/nope.ron");
    }

    #[test]
    fn display_path_prefixes_assets() {
        assert_eq!(display_path("data/palette.ron"), "assets/data/palette.ron");
    }

    /// Only a name under one of the two private folders, whether this build
    /// has them or not.
    #[test]
    fn a_name_under_a_private_folder_is_private() {
        assert!(is_private("assets-private/game/x.png"));
        assert!(is_private("assets-private/test/x.png"));
        assert!(!is_private("assets-private/games/x.png"));
        assert!(!is_private("assets-private/game"));
        assert!(!is_private("assets-private/library/x.png"));
        assert!(!is_private("x/assets-private/game/x.png"));
        assert!(!is_private("assets/x.png"));
        assert!(!is_private("assets-test/x.png"));
    }

    fn names(layers: &[Layer]) -> Vec<&'static str> {
        layers.iter().map(|layer| layer.name).collect()
    }

    /// Without the `private-assets` feature (every gate), `assets/` and the
    /// test files are the only layers: what a test reads never depends on
    /// the machine it runs on.
    #[cfg(not(feature = "private-assets"))]
    #[test]
    fn gates_read_only_the_public_assets_and_the_test_files() {
        let layers = layers();
        assert_eq!(names(&layers), [DISPLAY_ROOT, TEST_DISPLAY_ROOT]);
        assert_eq!(layers.iter().filter(|layer| !layer.test).count(), 1);
        assert!(layers.iter().all(|layer| !layer.private));
        assert_eq!(TEST_FILES, layers.iter().any(|layer| layer.test));
    }

    /// The installed bundle is the game's layers, in their order.
    #[test]
    fn the_installed_bundle_is_the_games_layers() {
        assert_eq!(names(install().layers()), names(&layers()));
        assert!(core::ptr::eq(install(), install()));
    }

    /// The data tables a build with the test files adds entries to: the
    /// only paths the story's files and the test files may share.
    const TABLES: [&str; 4] = [
        "data/characters.ron",
        "data/lessons.ron",
        "data/names.ron",
        "data/supports.ron",
    ];

    /// A test file never stands in for a story file: but for the data
    /// tables, which are merged by id, no path is in both.
    #[test]
    fn only_the_data_tables_are_in_both_the_story_and_the_test_files() {
        let story = story_only(|| files_under(""));
        let tests = Bundle::new(layers().into_iter().filter(|l| l.test).collect());
        let shared: Vec<&str> = tests
            .files_under("")
            .into_iter()
            .filter(|path| story.contains(path))
            .collect();
        assert_eq!(shared, TABLES);
        for path in TABLES {
            let (shown, part) = test_part(path).unwrap();
            assert_eq!(shown, format!("assets-test/{path}"));
            assert_ne!(Some(part), file(path));
            assert_eq!(display_path(path), format!("assets/{path}"));
        }
        assert_eq!(test_part("maps/test_small.map"), None);
        assert_eq!(test_part("data/palette.ron"), None);
    }

    #[test]
    fn a_test_file_is_read_and_shown_under_its_own_root() {
        let map = "maps/test_small.map";
        assert!(file(map).is_some());
        assert!(bytes(map).is_some());
        assert_eq!(display_path(map), "assets-test/maps/test_small.map");
        assert!(files_in("maps").contains(&map));
        assert!(files_under("").contains(&map));
        assert!(!is_private(&display_path(map)));
        assert_eq!(test_display_path(map), display_path(map));
        assert_eq!(test_display_path("x/new.png"), "assets-test/x/new.png");
    }

    #[test]
    fn story_only_hides_the_test_files_from_this_thread_until_it_returns() {
        let map = "maps/test_small.map";
        let inside = story_only(|| {
            // A nested call leaves them hidden when it returns.
            story_only(|| ());
            (
                file(map),
                bytes(map),
                files_in("maps").contains(&map),
                files_under("").contains(&map),
                display_path(map),
                test_part(TABLES[0]),
                file("maps/ch01.map").is_some(),
            )
        });
        assert_eq!(
            inside,
            (
                None,
                None,
                false,
                false,
                "assets/maps/test_small.map".to_owned(),
                None,
                true
            )
        );
        assert!(file(map).is_some());
        // Also after a panic inside it.
        assert!(std::panic::catch_unwind(|| story_only(|| panic!("inside"))).is_err());
        assert!(file(map).is_some());
        // Another thread is never affected.
        let elsewhere = story_only(|| std::thread::spawn(move || file(map).is_some()).join());
        assert!(matches!(elsewhere, Ok(true)));
    }
}
