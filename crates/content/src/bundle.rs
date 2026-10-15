//! The asset bundle: every file under the repo's `assets/` directory, embedded
//! in the binary at compile time (ADR-0005). This is the only way `content`
//! reads data; there is no filesystem I/O.
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
use crate::prelude::*;

use core::cell::Cell;

use include_dir::{Dir, File, include_dir};

static ASSETS: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets");

#[cfg(feature = "private-assets")]
static PRIVATE: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets-private/game");

#[cfg(any(debug_assertions, feature = "test-fixtures"))]
static TEST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets-test");

#[cfg(all(
    feature = "private-assets",
    any(debug_assertions, feature = "test-fixtures")
))]
static PRIVATE_TEST: Dir<'static> = include_dir!("$CARGO_MANIFEST_DIR/../../assets-private/test");

/// Whether this build embeds the test files (`assets-test/`, ADR-0057): a
/// debug build, or one with the `test-fixtures` feature. `false` in every
/// build that ships.
pub const TEST_FILES: bool = cfg!(any(debug_assertions, feature = "test-fixtures"));

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

/// One embedded directory.
struct Layer {
    /// The prefix its files are shown with.
    display_root: &'static str,
    /// Its files.
    dir: &'static Dir<'static>,
    /// Whether it holds test files (ADR-0057). The test layers come last.
    test: bool,
}

/// The embedded directories. A file is read from the first one that has it.
/// The story's come first, then the test files'.
static LAYERS: &[Layer] = &[
    #[cfg(feature = "private-assets")]
    Layer {
        display_root: PRIVATE_DISPLAY_ROOT,
        dir: &PRIVATE,
        test: false,
    },
    Layer {
        display_root: DISPLAY_ROOT,
        dir: &ASSETS,
        test: false,
    },
    #[cfg(all(
        feature = "private-assets",
        any(debug_assertions, feature = "test-fixtures")
    ))]
    Layer {
        display_root: PRIVATE_TEST_DISPLAY_ROOT,
        dir: &PRIVATE_TEST,
        test: true,
    },
    #[cfg(any(debug_assertions, feature = "test-fixtures"))]
    Layer {
        display_root: TEST_DISPLAY_ROOT,
        dir: &TEST,
        test: true,
    },
];

thread_local! {
    /// Whether this thread is inside [`story_only`].
    static STORY_ONLY: Cell<bool> = const { Cell::new(false) };
}

/// Runs `f` with the test files hidden from this thread: every function
/// here answers as it does in a build that ships. For the tests that check
/// what such a build holds; nothing in the game calls it.
pub fn story_only<T>(f: impl FnOnce() -> T) -> T {
    /// Puts the old value back, also when `f` panics.
    struct Restore(bool);
    impl Drop for Restore {
        fn drop(&mut self) {
            STORY_ONLY.set(self.0);
        }
    }
    let _restore = Restore(STORY_ONLY.replace(true));
    f()
}

/// `layers` without its test layers (they are last).
fn story_of(layers: &'static [Layer]) -> &'static [Layer] {
    let story = layers.iter().take_while(|layer| !layer.test).count();
    &layers[..story]
}

/// `layers`' test layers (they are last).
fn tests_of(layers: &'static [Layer]) -> &'static [Layer] {
    &layers[story_of(layers).len()..]
}

/// The layers this thread reads: all of them, or inside [`story_only`]
/// those that aren't test files.
fn layers() -> &'static [Layer] {
    if STORY_ONLY.get() {
        story_of(LAYERS)
    } else {
        LAYERS
    }
}

/// The file at `path` and the layer it is read from: the first layer that
/// has it.
fn find(layers: &'static [Layer], path: &str) -> Option<(&'static Layer, &'static File<'static>)> {
    layers
        .iter()
        .find_map(|layer| Some((layer, layer.dir.get_file(path)?)))
}

/// The paths (sorted, each once) of the files of every layer inside `dir`:
/// directly inside it, or with `deep` also in any directory below it.
fn list(layers: &'static [Layer], dir: &str, deep: bool) -> Vec<&'static str> {
    let mut paths = Vec::new();
    for layer in layers {
        let root = if dir.is_empty() {
            Some(layer.dir)
        } else {
            layer.dir.get_dir(dir)
        };
        let mut pending: Vec<&'static Dir<'static>> = root.into_iter().collect();
        while let Some(dir) = pending.pop() {
            paths.extend(dir.files().filter_map(|f| f.path().to_str()));
            if deep {
                pending.extend(dir.dirs());
            }
        }
    }
    paths.sort_unstable();
    paths.dedup();
    paths
}

/// The human-facing name of `path` in `layers`: under the root of the layer
/// it is read from, or under `assets/` when no layer has it.
fn display_in(layers: &'static [Layer], path: &str) -> String {
    let root = find(layers, path).map_or(DISPLAY_ROOT, |(layer, _)| layer.display_root);
    format!("{root}/{path}")
}

/// Returns the UTF-8 contents of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), or `None` if it is missing or not valid UTF-8.
pub fn file(path: &str) -> Option<&'static str> {
    find(layers(), path)?.1.contents_utf8()
}

/// The test files' own file at `path`, with its human-facing name, when
/// the story has a file there too: the entries a data table gains in a
/// build with the test files (ADR-0057). [`file()`] reads the story's.
/// `None` without test files, and for a path only one of the two has.
pub fn test_part(path: &str) -> Option<(String, &'static str)> {
    part_in(layers(), path)
}

/// [`test_part`] in `layers`.
fn part_in(layers: &'static [Layer], path: &str) -> Option<(String, &'static str)> {
    find(story_of(layers), path)?;
    let (layer, file) = find(tests_of(layers), path)?;
    let display = format!("{}/{path}", layer.display_root);
    Some((display, file.contents_utf8()?))
}

/// Returns the raw bytes of the embedded file at `path` (relative to
/// `assets/`, `/`-separated), e.g. an image, or `None` if it is missing.
pub fn bytes(path: &str) -> Option<&'static [u8]> {
    find(layers(), path).map(|(_, file)| file.contents())
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file directly inside directory `dir`. A missing directory yields an empty
/// list. Use `""` for the bundle root.
pub fn files_in(dir: &str) -> Vec<&'static str> {
    list(layers(), dir, false)
}

/// Lists the paths (relative to `assets/`, `/`-separated, sorted) of every
/// file inside directory `dir` or any directory below it. A missing
/// directory yields an empty list. Use `""` for the whole bundle.
pub fn files_under(dir: &str) -> Vec<&'static str> {
    list(layers(), dir, true)
}

/// The human-facing name of an asset path, e.g. `assets/data/palette.ron`
/// (`assets-private/game/...` for a file read from the private assets,
/// `assets-test/...` for a test file).
pub fn display_path(path: &str) -> String {
    display_in(layers(), path)
}

/// The human-facing name of the test file at `path`, there or not: where a
/// tool that makes a test file writes it.
pub fn test_display_path(path: &str) -> String {
    format!("{TEST_DISPLAY_ROOT}/{path}")
}

/// Whether `shown` (a human-facing name, from [`display_path`]) names a
/// file of the private assets: bought art (ADR-0040), the game's or the
/// test files'.
pub fn is_private(shown: &str) -> bool {
    [PRIVATE_DISPLAY_ROOT, PRIVATE_TEST_DISPLAY_ROOT]
        .iter()
        .any(|root| {
            shown
                .strip_prefix(root)
                .is_some_and(|rest| rest.starts_with('/'))
        })
}

#[cfg(test)]
mod tests {
    use include_dir::DirEntry;

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

    /// `assets/` alone, as `include_dir` embeds it: what the `no_std`
    /// bundle's table must hold.
    static ASSETS_ONLY: &[Layer] = &[Layer {
        display_root: DISPLAY_ROOT,
        dir: &ASSETS,
        test: false,
    }];

    /// The `no_std` bundle (`build.rs`'s table) and `include_dir` list the
    /// same files of `assets/` with the same bytes, and answer every
    /// function the same, so the two can't drift (ADR-0070 §3).
    #[test]
    fn the_no_std_table_holds_the_same_files_as_include_dir() {
        use crate::bundle_table as table;

        let all = list(ASSETS_ONLY, "", true);
        assert!(all.len() > 100, "{} files", all.len());
        assert_eq!(table::files_under(""), all);
        for path in &all {
            let (_, embedded) = find(ASSETS_ONLY, path).unwrap();
            assert_eq!(table::bytes(path), Some(embedded.contents()), "{path}");
            assert_eq!(table::file(path), embedded.contents_utf8(), "{path}");
            assert_eq!(table::display_path(path), display_in(ASSETS_ONLY, path));
        }
        for dir in ["", "data", "audio", "audio/sfx", "lang", "no_such_dir"] {
            assert_eq!(table::files_in(dir), list(ASSETS_ONLY, dir, false), "{dir}");
            assert_eq!(
                table::files_under(dir),
                list(ASSETS_ONLY, dir, true),
                "{dir}"
            );
        }
        assert_eq!(table::bytes("data/nope.ron"), None);
        assert_eq!(table::file("data/nope.ron"), None);
        assert_eq!(table::test_part("data/characters.ron"), None);
        const { assert!(!table::TEST_FILES) };
        assert_eq!(table::story_only(|| 7), 7);
        let roots = [
            (table::DISPLAY_ROOT, DISPLAY_ROOT),
            (table::PRIVATE_DISPLAY_ROOT, PRIVATE_DISPLAY_ROOT),
            (table::TEST_DISPLAY_ROOT, TEST_DISPLAY_ROOT),
            (table::PRIVATE_TEST_DISPLAY_ROOT, PRIVATE_TEST_DISPLAY_ROOT),
        ];
        assert!(roots.iter().all(|(table, here)| table == here));
        assert_eq!(
            table::test_display_path("x/y.png"),
            test_display_path("x/y.png")
        );
        for shown in [
            "assets-private/game/x.png",
            "assets-private/test/x",
            "assets/x",
        ] {
            assert_eq!(table::is_private(shown), is_private(shown), "{shown}");
        }
    }

    #[test]
    fn display_path_prefixes_assets() {
        assert_eq!(display_path("data/palette.ron"), "assets/data/palette.ron");
    }

    /// Without the `private-assets` feature (every gate), `assets/` and the
    /// test files are the only layers: what a test reads never depends on
    /// the machine it runs on.
    #[cfg(not(feature = "private-assets"))]
    #[test]
    fn gates_read_only_the_public_assets_and_the_test_files() {
        let roots: Vec<&str> = LAYERS.iter().map(|layer| layer.display_root).collect();
        assert_eq!(roots, [DISPLAY_ROOT, TEST_DISPLAY_ROOT]);
        assert_eq!(story_of(LAYERS).len(), 1);
        assert_eq!(TEST_FILES, !tests_of(LAYERS).is_empty());
    }

    /// The data tables a build with the test files adds entries to: the
    /// only paths the story's files and the test files may share.
    const TABLES: [&str; 3] = ["data/characters.ron", "data/names.ron", "data/supports.ron"];

    /// A test file never stands in for a story file: but for the data
    /// tables, which are merged by id, no path is in both.
    #[test]
    fn only_the_data_tables_are_in_both_the_story_and_the_test_files() {
        let story = list(story_of(LAYERS), "", true);
        let shared: Vec<&str> = list(tests_of(LAYERS), "", true)
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

    const fn entry(path: &'static str, contents: &'static str) -> DirEntry<'static> {
        DirEntry::File(File::new(path, contents.as_bytes()))
    }

    /// A stand-in for `assets-private/game/`.
    static PRIVATE_FIXTURE: Dir<'static> = Dir::new(
        "",
        &[
            DirEntry::Dir(Dir::new("data", &[entry("data/a.ron", "private a")])),
            DirEntry::Dir(Dir::new(
                "portraits",
                &[entry("portraits/new.portrait", "private only")],
            )),
            DirEntry::Dir(Dir::new(
                "deep",
                &[DirEntry::Dir(Dir::new(
                    "deep/x",
                    &[entry("deep/x/y.txt", "private y")],
                ))],
            )),
        ],
    );

    /// A stand-in for `assets/`.
    static PUBLIC_FIXTURE: Dir<'static> = Dir::new(
        "",
        &[
            entry("top.txt", "public top"),
            DirEntry::Dir(Dir::new(
                "data",
                &[
                    entry("data/a.ron", "public a"),
                    entry("data/b.ron", "public b"),
                ],
            )),
            DirEntry::Dir(Dir::new(
                "deep",
                &[DirEntry::Dir(Dir::new(
                    "deep/x",
                    &[entry("deep/x/z.txt", "public z")],
                ))],
            )),
        ],
    );

    /// The private stand-in laid over the public one, as a build with the
    /// `private-assets` feature has them.
    static MERGED: &[Layer] = &[
        Layer {
            display_root: PRIVATE_DISPLAY_ROOT,
            dir: &PRIVATE_FIXTURE,
            test: false,
        },
        Layer {
            display_root: DISPLAY_ROOT,
            dir: &PUBLIC_FIXTURE,
            test: false,
        },
    ];

    /// A stand-in for `assets-private/test/`.
    static PRIVATE_TEST_FIXTURE: Dir<'static> = Dir::new(
        "",
        &[DirEntry::Dir(Dir::new(
            "portraits",
            &[entry("portraits/t.png", "bought t")],
        ))],
    );

    /// A stand-in for `assets-test/`.
    static TEST_FIXTURE: Dir<'static> = Dir::new(
        "",
        &[
            DirEntry::Dir(Dir::new(
                "data",
                &[entry("data/b.ron", "test b"), entry("data/t.ron", "test t")],
            )),
            DirEntry::Dir(Dir::new(
                "portraits",
                &[entry("portraits/t.png", "placeholder t")],
            )),
        ],
    );

    /// Every layer, as a debug build with the `private-assets` feature has
    /// them.
    static WITH_TESTS: &[Layer] = &[
        Layer {
            display_root: PRIVATE_DISPLAY_ROOT,
            dir: &PRIVATE_FIXTURE,
            test: false,
        },
        Layer {
            display_root: DISPLAY_ROOT,
            dir: &PUBLIC_FIXTURE,
            test: false,
        },
        Layer {
            display_root: PRIVATE_TEST_DISPLAY_ROOT,
            dir: &PRIVATE_TEST_FIXTURE,
            test: true,
        },
        Layer {
            display_root: TEST_DISPLAY_ROOT,
            dir: &TEST_FIXTURE,
            test: true,
        },
    ];

    fn roots(layers: &'static [Layer]) -> Vec<&'static str> {
        layers.iter().map(|layer| layer.display_root).collect()
    }

    #[test]
    fn the_test_layers_are_the_last_ones() {
        assert_eq!(
            roots(story_of(WITH_TESTS)),
            [PRIVATE_DISPLAY_ROOT, DISPLAY_ROOT]
        );
        assert_eq!(
            roots(tests_of(WITH_TESTS)),
            [PRIVATE_TEST_DISPLAY_ROOT, TEST_DISPLAY_ROOT]
        );
        assert_eq!(roots(story_of(MERGED)), roots(MERGED));
        assert!(tests_of(MERGED).is_empty());
    }

    #[test]
    fn a_test_file_is_added_after_the_story_files() {
        assert_eq!(text(WITH_TESTS, "data/t.ron"), Some("test t"));
        assert_eq!(
            display_in(WITH_TESTS, "data/t.ron"),
            "assets-test/data/t.ron"
        );
        assert_eq!(text(story_of(WITH_TESTS), "data/t.ron"), None);
        // The story's file is the one read where both have one.
        assert_eq!(text(WITH_TESTS, "data/b.ron"), Some("public b"));
        assert_eq!(
            list(WITH_TESTS, "data", false),
            ["data/a.ron", "data/b.ron", "data/t.ron"]
        );
    }

    #[test]
    fn a_test_part_is_the_test_files_own_file_where_the_story_has_one_too() {
        assert_eq!(
            part_in(WITH_TESTS, "data/b.ron"),
            Some(("assets-test/data/b.ron".to_owned(), "test b"))
        );
        // Only the test files have it: an ordinary file, read whole.
        assert_eq!(part_in(WITH_TESTS, "data/t.ron"), None);
        // Only the story has it.
        assert_eq!(part_in(WITH_TESTS, "data/a.ron"), None);
        assert_eq!(part_in(WITH_TESTS, "data/nope.ron"), None);
        // No test layers.
        assert_eq!(part_in(story_of(WITH_TESTS), "data/b.ron"), None);
    }

    #[test]
    fn a_bought_test_file_replaces_the_placeholder_and_is_private() {
        assert_eq!(text(WITH_TESTS, "portraits/t.png"), Some("bought t"));
        assert_eq!(
            display_in(WITH_TESTS, "portraits/t.png"),
            "assets-private/test/portraits/t.png"
        );
        assert!(is_private(&display_in(WITH_TESTS, "portraits/t.png")));
        assert!(is_private(&display_in(WITH_TESTS, "data/a.ron")));
        assert!(is_private(&display_in(
            WITH_TESTS,
            "portraits/new.portrait"
        )));
        assert!(!is_private(&display_in(WITH_TESTS, "data/b.ron")));
        assert!(!is_private(&display_in(WITH_TESTS, "data/t.ron")));
        assert!(!is_private(&display_in(WITH_TESTS, "data/nope.ron")));
        // Without the bought test files the placeholder is back.
        assert_eq!(
            text(&WITH_TESTS[3..], "portraits/t.png"),
            Some("placeholder t")
        );
        assert!(!is_private(&display_in(
            &WITH_TESTS[3..],
            "portraits/t.png"
        )));
        // Only a name under one of the two private folders.
        assert!(is_private("assets-private/game/x.png"));
        assert!(is_private("assets-private/test/x.png"));
        assert!(!is_private("assets-private/games/x.png"));
        assert!(!is_private("assets-private/game"));
        assert!(!is_private("assets-private/library/x.png"));
        assert!(!is_private("x/assets-private/game/x.png"));
    }

    fn text(layers: &'static [Layer], path: &str) -> Option<&'static str> {
        find(layers, path)?.1.contents_utf8()
    }

    #[test]
    fn a_private_file_replaces_the_public_file_at_the_same_path() {
        assert_eq!(text(MERGED, "data/a.ron"), Some("private a"));
        assert_eq!(
            display_in(MERGED, "data/a.ron"),
            "assets-private/game/data/a.ron"
        );
        // Without the private layer the public file is back.
        assert_eq!(text(&MERGED[1..], "data/a.ron"), Some("public a"));
    }

    #[test]
    fn a_private_only_file_is_added() {
        assert_eq!(text(MERGED, "portraits/new.portrait"), Some("private only"));
        assert_eq!(list(MERGED, "portraits", false), ["portraits/new.portrait"]);
        assert_eq!(text(&MERGED[1..], "portraits/new.portrait"), None);
        assert!(list(&MERGED[1..], "portraits", false).is_empty());
    }

    #[test]
    fn a_public_only_file_is_still_read() {
        assert_eq!(text(MERGED, "data/b.ron"), Some("public b"));
        assert_eq!(display_in(MERGED, "data/b.ron"), "assets/data/b.ron");
    }

    #[test]
    fn a_file_in_no_layer_is_missing_and_shown_under_assets() {
        assert!(find(MERGED, "data/nope.ron").is_none());
        assert_eq!(display_in(MERGED, "data/nope.ron"), "assets/data/nope.ron");
    }

    #[test]
    fn lists_merge_the_layers_sorted_with_each_path_once() {
        assert_eq!(list(MERGED, "data", false), ["data/a.ron", "data/b.ron"]);
        assert_eq!(list(MERGED, "", false), ["top.txt"]);
        // `deep` has no files of its own, only in the directory below.
        assert!(list(MERGED, "deep", false).is_empty());
        assert_eq!(list(MERGED, "deep", true), ["deep/x/y.txt", "deep/x/z.txt"]);
        assert_eq!(
            list(MERGED, "", true),
            [
                "data/a.ron",
                "data/b.ron",
                "deep/x/y.txt",
                "deep/x/z.txt",
                "portraits/new.portrait",
                "top.txt",
            ]
        );
        assert!(list(MERGED, "no_such_dir", true).is_empty());
    }
}
