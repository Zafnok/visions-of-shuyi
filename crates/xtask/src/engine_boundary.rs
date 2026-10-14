//! `cargo xtask check-engine-boundary`: fails when something of the game
//! leaks into the engine (ADR-0063 §4, ticket 1102).
//!
//! Two checks:
//!
//! 1. **Dependencies.** For every workspace package whose manifest is
//!    under `engine/` (read from `cargo metadata`), a path dependency must
//!    also be under `engine/`, and any other dependency must be on the
//!    list in `engine/README.md`'s fenced `engine-deps` block.
//! 2. **Text.** No file under `engine/` may name a game crate (`trpg_`,
//!    `trpg-`, `visions-of-shuyi`), a game asset folder (`assets-private`,
//!    `assets-test/`, a path starting with `assets/`), or an id from the
//!    game's data: class, character, terrain, item, spell, skill and art
//!    ids, and sound and music cue ids. Ids are matched as whole words,
//!    ignoring case; short ids and the common words in [`ALLOW`] are
//!    skipped.
//!
//! The rules are pure functions over the metadata JSON and file text; [`run`]
//! is the thin part that runs cargo, loads the data and walks the folder.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::check_keys::{files_under, relative};

/// The engine's folder, relative to the repository root.
const ENGINE_DIR: &str = "engine";
/// The README that holds the list of allowed third-party crates.
const README: &str = "engine/README.md";
/// The fence that opens that list.
const DEPS_FENCE: &str = "```engine-deps";

/// Strings no engine file may contain: the game's crates and binary, and
/// the game's private and test asset folders.
const BANNED: [&str; 5] = [
    "trpg_",
    "trpg-",
    "visions-of-shuyi",
    "assets-private",
    "assets-test/",
];

/// Ids this long or shorter are skipped: `axe`, `sea`, `ice` and the like
/// are too short to be the game's alone.
const MAX_SKIPPED_LEN: usize = 3;

/// Ids from the game's data that are also ordinary words an engine's code
/// or docs use for their own meaning. Each is here for the reason beside it.
const ALLOW: &[&str] = &[
    // Programming words: an engine says these about its own code.
    "block",   // a code or text block; also a sound cue
    "fire",    // fire an event or a trigger; also a spell
    "floor",   // `f32::floor`; also a terrain
    "force",   // force a redraw; also a spell
    "lead",    // the leading edge, to lead with; also the player's character
    "plain",   // plain text, plain data; also a terrain
    "resolve", // resolve a path or a conflict; also a skill
    "title",   // a window or screen title; also a music cue
    "miss",    // a cache miss; also a sound cue
    "bridge",  // a JS bridge to the browser; also a terrain
    "guard",   // a scope guard; also a class
    "peak",    // a peak level or meter; also a terrain
    // Map words a tactics-kit sample or test names its own tiles with.
    "road",     // also a terrain
    "forest",   // also a terrain
    "mountain", // also a terrain
    "water",    // also a terrain
    "door",     // also a terrain
    "wall",     // also a terrain
    // Words a sample's own abilities, items or music may use.
    "heal",    // also a spell and a sound cue
    "potion",  // also an item
    "village", // also a music cue
    "defeat",  // also a music cue
];

/// Runs both checks on the repository at `repo_root` and returns every
/// finding as `file:line: message` (empty = the boundary holds).
pub fn run(repo_root: &Path) -> Vec<String> {
    let mut errors = Vec::new();
    match fs::read_to_string(repo_root.join(README))
        .map_err(|e| format!("{README}: cannot read: {e}"))
        .and_then(|readme| allowed_deps(&readme))
    {
        Ok(allowed) => match cargo_metadata(repo_root).and_then(|m| check_deps(&m, &allowed)) {
            Ok(problems) => {
                for problem in problems {
                    let manifest = fs::read_to_string(repo_root.join(&problem.manifest));
                    let line = manifest.map_or(1, |text| dep_line(&text, &problem.dep));
                    errors.push(format!("{}:{line}: {}", problem.manifest, problem.message));
                }
            }
            Err(e) => errors.push(e),
        },
        Err(e) => errors.push(e),
    }
    let ids = match game_ids() {
        Ok(ids) => ids,
        Err(e) => {
            errors.push(e);
            BTreeSet::new()
        }
    };
    for file in files_under(&repo_root.join(ENGINE_DIR), &mut errors) {
        let rel = relative(repo_root, &file);
        match fs::read(&file) {
            // Not UTF-8: an image or a sound, which names nothing.
            Ok(bytes) => {
                if let Ok(text) = String::from_utf8(bytes) {
                    errors.extend(scan_file(&rel, &text, &ids));
                }
            }
            Err(e) => errors.push(format!("{rel}: cannot read: {e}")),
        }
    }
    errors
}

/// `cargo metadata` for the workspace, as JSON text.
fn cargo_metadata(repo_root: &Path) -> Result<String, String> {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".to_owned());
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--no-deps"])
        .current_dir(repo_root)
        .output()
        .map_err(|e| format!("cargo metadata: cannot run: {e}"))?;
    if !output.status.success() {
        return Err(format!(
            "cargo metadata failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout).map_err(|e| format!("cargo metadata: not UTF-8: {e}"))
}

/// Every id in the game's data that the text check looks for.
fn game_ids() -> Result<BTreeSet<String>, String> {
    let content =
        trpg_content::load_embedded().map_err(|e| format!("the game's data does not load: {e}"))?;
    let mut ids = Vec::new();
    ids.extend(content.classes.classes.keys().map(|id| id.0.clone()));
    ids.extend(content.characters.characters.keys().map(|id| id.0.clone()));
    ids.extend(content.characters.generics.keys().cloned());
    ids.extend(
        content
            .terrain
            .display
            .terrains
            .iter()
            .map(|t| t.id.clone()),
    );
    ids.extend(content.items.items.keys().map(|id| id.0.clone()));
    ids.extend(content.spells.spells.keys().map(|id| id.0.clone()));
    ids.extend(content.skills.skills.keys().map(|id| id.0.clone()));
    ids.extend(content.arts.arts.keys().map(|id| id.0.clone()));
    ids.extend(content.audio.sounds.keys().cloned());
    ids.extend(content.audio.music.keys().cloned());
    Ok(id_set(ids))
}

/// The ids worth looking for, lowercased: longer than
/// [`MAX_SKIPPED_LEN`] and not in [`ALLOW`].
pub fn id_set(ids: impl IntoIterator<Item = String>) -> BTreeSet<String> {
    ids.into_iter()
        .map(|id| id.to_ascii_lowercase())
        .filter(|id| id.len() > MAX_SKIPPED_LEN && !ALLOW.contains(&id.as_str()))
        .collect()
}

/// The crate names in the README's `engine-deps` block, one per line
/// (blank lines and `#` comments skipped).
pub fn allowed_deps(readme: &str) -> Result<BTreeSet<String>, String> {
    let mut lines = readme.lines().skip_while(|line| line.trim() != DEPS_FENCE);
    if lines.next().is_none() {
        return Err(format!("{README}: no {DEPS_FENCE} block"));
    }
    Ok(lines
        .map(str::trim)
        .take_while(|line| !line.starts_with("```"))
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect())
}

/// A dependency an engine package may not have.
#[derive(Debug, PartialEq, Eq)]
pub struct DepProblem {
    /// The package's manifest, relative to the workspace root, with `/`.
    pub manifest: String,
    /// The dependency's name, to find its line in the manifest.
    pub dep: String,
    /// What is wrong.
    pub message: String,
}

/// Checks the dependencies of every package under `engine/` in
/// `metadata` (`cargo metadata --format-version 1` JSON) against
/// `allowed`.
pub fn check_deps(metadata: &str, allowed: &BTreeSet<String>) -> Result<Vec<DepProblem>, String> {
    let json: Value =
        serde_json::from_str(metadata).map_err(|e| format!("cargo metadata: bad JSON: {e}"))?;
    let root = slashes(
        json["workspace_root"]
            .as_str()
            .ok_or("cargo metadata: no workspace_root")?,
    );
    let engine = format!("{root}/{ENGINE_DIR}/");
    let packages = json["packages"]
        .as_array()
        .ok_or("cargo metadata: no packages")?;
    let mut problems = Vec::new();
    for package in packages {
        let manifest = slashes(package["manifest_path"].as_str().unwrap_or_default());
        let Some(rel) = manifest.strip_prefix(&engine) else {
            continue;
        };
        let manifest = format!("{ENGINE_DIR}/{rel}");
        let name = package["name"].as_str().unwrap_or_default();
        for dep in package["dependencies"].as_array().into_iter().flatten() {
            let dep_name = dep["name"].as_str().unwrap_or_default();
            let message = match dep["path"].as_str() {
                Some(path) if !format!("{}/", slashes(path)).starts_with(&engine) => format!(
                    "{name} depends on {dep_name}, a crate outside {ENGINE_DIR}/ (ADR-0063 §2)"
                ),
                Some(_) => continue,
                None if allowed.contains(dep_name) => continue,
                None => format!(
                    "{name} depends on {dep_name}, which is not in {README}'s engine-deps list"
                ),
            };
            problems.push(DepProblem {
                manifest: manifest.clone(),
                dep: dep_name.to_owned(),
                message,
            });
        }
    }
    Ok(problems)
}

/// `path` with `\` turned into `/` and no trailing `/`.
fn slashes(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_owned()
}

/// The 1-based line of `manifest` that declares `dep` (1 if none does).
pub fn dep_line(manifest: &str, dep: &str) -> usize {
    manifest
        .lines()
        .position(|line| {
            line.trim_start()
                .strip_prefix(dep)
                .is_some_and(|rest| rest.trim_start().starts_with(['=', '.']))
        })
        .map_or(1, |i| i + 1)
}

/// Every finding in one engine file (`rel` is its repo-relative path).
pub fn scan_file(rel: &str, text: &str, ids: &BTreeSet<String>) -> Vec<String> {
    let mut errors = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let at = i + 1;
        for banned in BANNED {
            if line.contains(banned) {
                errors.push(format!("{rel}:{at}: names `{banned}`, which is the game's"));
            }
        }
        if names_game_assets(line) {
            errors.push(format!("{rel}:{at}: names the game's `assets/` folder"));
        }
        for word in line.split(|c: char| !(c.is_ascii_alphanumeric() || c == '_')) {
            if ids.contains(&word.to_ascii_lowercase()) {
                errors.push(format!(
                    "{rel}:{at}: names `{word}`, an id from the game's data"
                ));
            }
        }
    }
    errors
}

/// Whether `line` has a path starting with `assets/` (or climbing up to
/// one with `../`), rather than an `assets/` folder of the engine's own
/// (`examples/demo/assets/`).
fn names_game_assets(line: &str) -> bool {
    line.match_indices("assets/").any(|(i, _)| {
        let before = &line[..i];
        before.ends_with("../")
            || !before.ends_with(|c: char| {
                c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '/' | '.')
            })
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|s| (*s).to_owned()).collect()
    }

    /// `cargo metadata` JSON for a workspace at `D:\repo` (Windows paths,
    /// as on Nick's machine) with one engine package and one game package.
    fn metadata(engine_deps: &str) -> String {
        format!(
            r#"{{
  "workspace_root": "D:\\repo",
  "packages": [
    {{
      "name": "shuyi-ui",
      "manifest_path": "D:\\repo\\engine\\crates\\ui\\Cargo.toml",
      "dependencies": [{engine_deps}]
    }},
    {{
      "name": "game-ui",
      "manifest_path": "D:\\repo\\crates\\ui\\Cargo.toml",
      "dependencies": [{{ "name": "png", "source": "registry+https://github.com/rust-lang/crates.io-index" }}]
    }}
  ]
}}"#
        )
    }

    const SERDE: &str =
        r#"{ "name": "serde", "source": "registry+https://github.com/rust-lang/crates.io-index" }"#;
    const ENGINE_CORE: &str =
        r#"{ "name": "shuyi-core", "source": null, "path": "D:\\repo\\engine\\crates\\core" }"#;
    const GAME_CORE: &str =
        r#"{ "name": "game-core", "source": null, "path": "D:\\repo\\crates\\core" }"#;

    #[test]
    fn allowed_deps_reads_the_fenced_block() {
        let readme = "# E\n\n```\nnot this\n```\n\n```engine-deps\nserde\n\n# why\n  ron  \n```\nthiserror\n";
        assert_eq!(allowed_deps(readme), Ok(set(&["serde", "ron"])));
    }

    #[test]
    fn allowed_deps_without_the_block_is_an_error() {
        let error = allowed_deps("# E\n```\nserde\n```\n").unwrap_err();
        assert!(error.contains("no ```engine-deps block"), "{error}");
    }

    #[test]
    fn the_real_readme_lists_the_starting_crates() {
        let readme = include_str!("../../../engine/README.md");
        let allowed = allowed_deps(readme).unwrap_or_default();
        for name in ["serde", "macroquad", "quad-storage", "insta"] {
            assert!(allowed.contains(name), "{name}");
        }
    }

    #[test]
    fn engine_package_on_listed_and_engine_crates_passes() {
        let json = metadata(&format!("{SERDE}, {ENGINE_CORE}"));
        assert_eq!(check_deps(&json, &set(&["serde"])), Ok(vec![]));
    }

    #[test]
    fn engine_package_on_a_game_crate_fails() {
        let json = metadata(GAME_CORE);
        assert_eq!(
            check_deps(&json, &set(&["serde"])),
            Ok(vec![DepProblem {
                manifest: "engine/crates/ui/Cargo.toml".to_owned(),
                dep: "game-core".to_owned(),
                message: "shuyi-ui depends on game-core, a crate outside engine/ (ADR-0063 §2)"
                    .to_owned(),
            }])
        );
    }

    #[test]
    fn engine_package_on_an_unlisted_crate_fails() {
        let json = metadata(SERDE);
        let problems = check_deps(&json, &set(&["ron"])).unwrap_or_default();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].dep, "serde");
        assert_eq!(
            problems[0].message,
            "shuyi-ui depends on serde, which is not in engine/README.md's engine-deps list"
        );
    }

    #[test]
    fn a_path_dependency_beside_engine_is_outside_it() {
        // `D:\repo\engine-old` starts with `D:\repo\engine` but is not in it.
        let json = metadata(r#"{ "name": "old", "path": "D:\\repo\\engine-old\\x" }"#);
        assert_eq!(check_deps(&json, &set(&[])).unwrap_or_default().len(), 1);
    }

    #[test]
    fn game_packages_are_not_checked() {
        let json = metadata("");
        assert_eq!(check_deps(&json, &set(&[])), Ok(vec![]));
    }

    #[test]
    fn check_deps_reports_bad_metadata() {
        assert!(check_deps("not json", &set(&[])).is_err());
        assert!(check_deps(r#"{"packages": []}"#, &set(&[])).is_err());
        assert!(check_deps(r#"{"workspace_root": "/r"}"#, &set(&[])).is_err());
    }

    #[test]
    fn check_deps_reads_unix_paths() {
        let json = r#"{"workspace_root": "/r/", "packages": [{"name": "e",
            "manifest_path": "/r/engine/crates/e/Cargo.toml",
            "dependencies": [{"name": "g", "path": "/r/crates/g"}, {"name": "c", "path": "/r/engine/crates/c"}]}]}"#;
        let problems = check_deps(json, &set(&[])).unwrap_or_default();
        assert_eq!(problems.len(), 1);
        assert_eq!(problems[0].manifest, "engine/crates/e/Cargo.toml");
        assert_eq!(problems[0].dep, "g");
    }

    #[test]
    fn dep_line_finds_the_declaration() {
        let manifest = "[package]\nname = \"serde-ish\"\n\n[dependencies]\nserde = \"1\"\nron.workspace = true\n";
        assert_eq!(dep_line(manifest, "serde"), 5);
        assert_eq!(dep_line(manifest, "ron"), 6);
        assert_eq!(dep_line(manifest, "png"), 1);
    }

    #[test]
    fn a_file_naming_a_game_crate_fails_with_its_line() {
        let text = "//! A frame.\nuse trpg_core::Pos;\n";
        assert_eq!(
            scan_file("engine/crates/ui/src/lib.rs", text, &set(&[])),
            vec!["engine/crates/ui/src/lib.rs:2: names `trpg_`, which is the game's"]
        );
    }

    #[test]
    fn every_banned_string_fails() {
        for banned in BANNED {
            let text = format!("see {banned}x\n");
            assert_eq!(scan_file("f", &text, &set(&[])).len(), 1, "{banned}");
        }
    }

    #[test]
    fn a_path_into_the_games_assets_fails() {
        for line in [
            "assets/data/classes.ron",
            "load(\"assets/x\")",
            "`assets/`",
            "../../assets/fonts",
        ] {
            assert_eq!(
                scan_file("f", line, &set(&[])),
                vec!["f:1: names the game's `assets/` folder"],
                "{line}"
            );
        }
    }

    #[test]
    fn an_assets_folder_of_the_engines_own_passes() {
        for line in [
            "examples/demo/assets/font.png",
            "./assets/x",
            "my_assets/x",
            "test-assets/x",
            "the assets folder",
        ] {
            assert_eq!(
                scan_file("f", line, &set(&[])),
                Vec::<String>::new(),
                "{line}"
            );
        }
    }

    #[test]
    fn a_file_naming_a_class_id_fails() {
        let ids = id_set(["Swordsman".to_owned(), "iron_sword".to_owned()]);
        let text = "// fine\nlet class = \"swordsman\";\nlet Swordsman = iron_sword;\n";
        assert_eq!(
            scan_file("f", text, &ids),
            vec![
                "f:2: names `swordsman`, an id from the game's data",
                "f:3: names `Swordsman`, an id from the game's data",
                "f:3: names `iron_sword`, an id from the game's data",
            ]
        );
    }

    #[test]
    fn ids_inside_longer_words_pass() {
        let ids = id_set(["mage".to_owned()]);
        assert_eq!(
            scan_file("f", "image magee archmage", &ids),
            Vec::<String>::new()
        );
    }

    #[test]
    fn a_clean_file_passes() {
        let ids = id_set(["swordsman".to_owned()]);
        let text = "//! The frame.\npub struct Frame { cells: Vec<Cell> }\n";
        assert_eq!(scan_file("f", text, &ids), Vec::<String>::new());
    }

    #[test]
    fn id_set_skips_short_and_allowed_ids() {
        let ids = id_set(["axe", "mage", "fire", "Heal", "ARCHER"].map(str::to_owned));
        assert_eq!(ids, set(&["mage", "archer"]));
    }

    #[test]
    fn game_ids_has_every_kind_of_id() {
        let ids = game_ids().unwrap_or_default();
        for id in [
            "swordsman",     // class
            "retainer",      // character
            "red_brigand",   // generic unit
            "thicket",       // terrain
            "iron_sword",    // item
            "cinder_oath",   // spell
            "keen_edge",     // skill
            "flowing_cut",   // art
            "menu_move",     // sound cue
            "dungeon_tense", // music cue
        ] {
            assert!(ids.contains(id), "{id}");
        }
        assert!(!ids.contains("fire"), "allowed words are left out");
    }

    #[test]
    fn run_reports_findings_in_engine_files() {
        let root =
            std::env::temp_dir().join(format!("xtask-engine-boundary-{}", std::process::id()));
        let engine = root.join(ENGINE_DIR);
        fs::create_dir_all(&engine).unwrap();
        fs::write(engine.join("README.md"), "```engine-deps\nserde\n```\n").unwrap();
        fs::write(engine.join("lib.rs"), "use trpg_core::Pos;\n").unwrap();
        fs::write(engine.join("pic.png"), [0xff, 0xfe, 0x00]).unwrap();
        let errors = run(&root);
        let _ = fs::remove_dir_all(&root);
        assert!(
            errors.contains(&"engine/lib.rs:1: names `trpg_`, which is the game's".to_owned()),
            "{errors:?}"
        );
        assert!(!errors.iter().any(|e| e.contains("pic.png")), "{errors:?}");
    }

    #[test]
    fn allow_has_only_lowercase_ids_longer_than_the_skip() {
        for word in ALLOW {
            assert!(word.len() > MAX_SKIPPED_LEN, "{word} is skipped anyway");
            assert_eq!(*word, word.to_ascii_lowercase(), "{word}");
        }
    }
}
