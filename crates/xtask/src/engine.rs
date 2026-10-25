//! Shuyi Engine's revision and its local checkout (ADR-0063 §8, ticket
//! 1131).
//!
//! The game depends on the engine's repository by a git revision, which is
//! written in three places: `shuyi-engine.rev`, the `rev` of each engine
//! crate in the root `Cargo.toml`'s `[workspace.dependencies]`, and each
//! engine package's source in `Cargo.lock`. `cargo xtask engine-pin` moves
//! all three, and `--check` (in CI) fails when they disagree or when the
//! pin ledger, `docs/engine/pins.md`, has no section for the pin
//! (ADR-0096). `--report` runs the engine watcher (`trpg-engine-watch`)
//! here: what the `engine-pin` issue says.
//!
//! `cargo xtask engine` clones the engine into `shuyi-engine/` and writes
//! `.cargo/engine-patch.toml`, a `[patch]` that `.cargo/config.toml`
//! includes when it is there: the game then builds on the checkout, so one
//! working copy edits both. Both are git-ignored, so CI never uses the
//! patch. While it is on, `Cargo.lock` names the checkout instead of the
//! revision; `cargo xtask engine --off` takes it away before a commit.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;
use std::process::Command;

use trpg_engine_watch::{Run, fetch, ledger, markdown, render};

/// The engine's repository.
pub const URL: &str = "https://github.com/Zafnok/shuyi-engine";
/// The file that names the pinned commit.
pub const REV_FILE: &str = "shuyi-engine.rev";
/// The checkout's folder, from the repository's root.
pub const CHECKOUT: &str = "shuyi-engine";
/// The `[patch]` that points the build at the checkout.
pub const PATCH_FILE: &str = ".cargo/engine-patch.toml";
/// The engine's crates the game uses, with their folders in its repository.
pub const CRATES: [(&str, &str); 6] = [
    ("shuyi-core", "crates/core"),
    ("shuyi-assets", "crates/assets"),
    ("shuyi-ui", "crates/ui"),
    ("shuyi-platform", "crates/platform"),
    ("shuyi-app", "crates/app"),
    ("shuyi-cli", "crates/cli"),
];

pub const USAGE: &str = "usage: cargo xtask engine [--off]\n       \
cargo xtask engine-pin [<commit> | --check]\n\n\
engine        clone Shuyi Engine at the pinned commit into shuyi-engine/ (a checkout that\n              \
is there already is fetched and left where it is) and build the game on it\n\
engine --off  build the game on the pinned commit again; do this before committing\n\
engine-pin    pin the commit the checkout is at (it must be on the engine's main), or <commit>\n\
engine-pin --check  fail when shuyi-engine.rev, Cargo.toml and Cargo.lock disagree, or when\n              \
docs/engine/pins.md lacks this pin's section or a decision (ADR-0096)\n\
engine-pin --report the engine PRs between the pin and the engine's main, sorted, and whether\n              \
a move is due: what the engine-pin issue says (ADR-0096)";

/// The commit `text` names: 40 lower-case hex digits.
pub fn parse_rev(text: &str) -> Result<String, String> {
    let rev = text.trim();
    let is_hex = |c: char| c.is_ascii_digit() || ('a'..='f').contains(&c);
    if rev.len() == 40 && rev.chars().all(is_hex) {
        Ok(rev.to_owned())
    } else {
        Err(format!("expected a 40-character commit id, found `{rev}`"))
    }
}

/// Whether `line` declares an engine crate from the engine's repository.
fn is_engine_line(line: &str) -> bool {
    let crate_name = line.split([' ', '=', '.']).next().unwrap_or("");
    CRATES.iter().any(|(name, _)| *name == crate_name) && line.contains(&format!("\"{URL}\""))
}

/// The `rev = "…"` of each engine crate `manifest` declares, by crate.
pub fn manifest_revs(manifest: &str) -> Vec<(String, String)> {
    manifest
        .lines()
        .filter(|line| is_engine_line(line))
        .filter_map(|line| {
            let name = line.split([' ', '=', '.']).next()?;
            let rev = line.split("rev = \"").nth(1)?.split('"').next()?;
            Some((name.to_owned(), rev.to_owned()))
        })
        .collect()
}

/// `manifest` with every engine crate's `rev` set to `rev`. Fails unless it
/// declares each of [`CRATES`] once, with a `rev`.
pub fn set_manifest_rev(manifest: &str, rev: &str) -> Result<String, String> {
    let found = manifest_revs(manifest);
    for (name, _) in CRATES {
        let count = found.iter().filter(|(n, _)| n == name).count();
        if count != 1 {
            return Err(format!(
                "Cargo.toml: expected one `{name} = {{ git = \"{URL}\", rev = … }}`, found {count}"
            ));
        }
    }
    let mut out = String::with_capacity(manifest.len());
    for line in manifest.split_inclusive('\n') {
        match line.split_once("rev = \"").filter(|_| is_engine_line(line)) {
            Some((before, after)) => {
                let rest = after.split_once('"').map_or("", |(_, rest)| rest);
                // Writing to a `String` can't fail.
                let _ = write!(out, "{before}rev = \"{rev}\"{rest}");
            }
            None => out.push_str(line),
        }
    }
    Ok(out)
}

/// Each engine package in `lock` with its source (`None` for a path, which
/// is what the local checkout's patch makes).
pub fn lock_sources(lock: &str) -> Vec<(String, Option<String>)> {
    lock.split("[[package]]")
        .filter_map(|block| {
            let field = |key: &str| {
                block.lines().find_map(|line| {
                    let value = line.strip_prefix(key)?.strip_prefix(" = \"")?;
                    value.strip_suffix('"').map(str::to_owned)
                })
            };
            let name = field("name")?;
            CRATES
                .iter()
                .any(|(n, _)| *n == name)
                .then(|| (name, field("source")))
        })
        .collect()
}

/// What disagrees between the pin file's text, the root manifest and the
/// lock file. With the local checkout on (`patched`), the lock names the
/// checkout and isn't checked.
pub fn problems(rev_text: &str, manifest: &str, lock: &str, patched: bool) -> Vec<String> {
    let rev = match parse_rev(rev_text) {
        Ok(rev) => rev,
        Err(e) => return vec![format!("{REV_FILE}: {e}")],
    };
    let mut out = Vec::new();
    let revs = manifest_revs(manifest);
    for (name, _) in CRATES {
        match revs.iter().find(|(n, _)| n == name) {
            None => out.push(format!("Cargo.toml: no `{name}` from {URL}")),
            Some((_, r)) if *r != rev => out.push(format!(
                "Cargo.toml: `{name}` is at {r}, {REV_FILE} names {rev}: run `cargo xtask engine-pin {rev}`"
            )),
            Some(_) => {}
        }
    }
    if patched {
        return out;
    }
    let want = format!("git+{URL}?rev={rev}#{rev}");
    let sources = lock_sources(lock);
    for (name, _) in CRATES {
        match sources.iter().find(|(n, _)| n == name) {
            None => out.push(format!("Cargo.lock: no `{name}`")),
            Some((_, None)) => out.push(format!(
                "Cargo.lock: `{name}` is the local checkout's: run `cargo xtask engine --off` and commit the lock it writes"
            )),
            Some((_, Some(source))) if *source != want => out.push(format!(
                "Cargo.lock: `{name}` is from {source}, not {want}: run `cargo xtask engine-pin {rev}`"
            )),
            Some(_) => {}
        }
    }
    out
}

/// The `[patch]` that builds the game on the checkout.
pub fn patch_file() -> String {
    let mut out = format!(
        "# Written by `cargo xtask engine` (ticket 1131): build the game on the\n\
         # checkout in {CHECKOUT}/. `cargo xtask engine --off` removes it.\n\
         [patch.\"{URL}\"]\n"
    );
    for (name, dir) in CRATES {
        let _ = writeln!(out, "{name} = {{ path = \"{CHECKOUT}/{dir}\" }}");
    }
    out
}

/// Runs `git <args>` in `dir`: its output, or what it printed as the error.
/// It never stops to ask for a password on the terminal.
fn git(dir: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .current_dir(dir)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("spawn git: {e}"))?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }
    Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
}

/// Lets Cargo write `Cargo.lock` again for the manifests as they are.
fn relock(root: &Path) -> Result<(), String> {
    let cargo = std::env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let output = Command::new(cargo)
        .args(["metadata", "--format-version", "1", "--quiet"])
        .current_dir(root)
        .output()
        .map_err(|e| format!("spawn cargo: {e}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

fn read(root: &Path, file: &str) -> Result<String, String> {
    fs::read_to_string(root.join(file)).map_err(|e| format!("{file}: {e}"))
}

/// What is wrong with the pin ledger, `docs/engine/pins.md` (ADR-0096 §6):
/// its last section isn't `rev_text`'s commit, a row has no decision, or
/// an `adopt → NNNN` names no ticket.
fn ledger_problems(root: &Path, rev_text: &str) -> Vec<String> {
    let found = fetch::ledger(root).and_then(|sections| {
        let ids = fetch::ticket_ids(root)?;
        Ok(ledger::problems(&sections, rev_text, &ids))
    });
    found.unwrap_or_else(|e| vec![e])
}

/// `engine-pin --report`: the engine watcher's report (ADR-0096), as the
/// `engine-pin` issue would say it.
///
/// Not covered by mutation testing (`#[mutants::skip]`): it only joins the
/// watcher's `gh` calls to its renderer, which `trpg-engine-watch` tests.
#[mutants::skip]
pub fn watch_report(root: &Path) -> Result<String, String> {
    let watch = fetch::watch(root, fetch::now())?;
    let run = Run {
        trigger: "manual".to_owned(),
        ..Run::default()
    };
    let rendered = render(&watch.report, &watch.ledger, &watch.open_ids, &run);
    Ok(markdown(&watch.report, rendered.as_ref()))
}

/// `engine-pin --check`.
pub fn check(root: &Path) -> Result<String, Vec<String>> {
    let files = (|| {
        Ok((
            read(root, REV_FILE)?,
            read(root, "Cargo.toml")?,
            read(root, "Cargo.lock")?,
        ))
    })();
    let (rev, manifest, lock) = files.map_err(|e: String| vec![e])?;
    let patched = root.join(PATCH_FILE).exists();
    let mut found = problems(&rev, &manifest, &lock, patched);
    found.extend(ledger_problems(root, &rev));
    if !found.is_empty() {
        return Err(found);
    }
    let rev = rev.trim();
    Ok(if patched {
        format!(
            "engine-pin: OK, at {rev}, and {} has its section (the local checkout is on: Cargo.lock not checked)",
            ledger::PATH
        )
    } else {
        format!(
            "engine-pin: OK, at {rev}, and {} has its section",
            ledger::PATH
        )
    })
}

/// `engine-pin [<commit>]`: the commit, or the one the checkout is at.
pub fn pin(root: &Path, commit: Option<&str>) -> Result<String, String> {
    if root.join(PATCH_FILE).exists() {
        return Err(
            "the local checkout is on, so Cargo.lock can't name the pin: run `cargo xtask engine --off` first"
                .to_owned(),
        );
    }
    let rev = if let Some(commit) = commit {
        parse_rev(commit)?
    } else {
        let checkout = root.join(CHECKOUT);
        if !checkout.join(".git").exists() {
            return Err(format!(
                "{CHECKOUT}/ isn't there: give the commit, or run `cargo xtask engine` first"
            ));
        }
        let changes = git(&checkout, &["status", "--porcelain"])?;
        if !changes.is_empty() {
            return Err(format!(
                "{CHECKOUT}/ has changes that aren't committed:\n{changes}\ncommit and push them, then run this again"
            ));
        }
        git(&checkout, &["fetch", "--quiet", "origin"])?;
        let head = git(&checkout, &["rev-parse", "HEAD"])?;
        // A squash merge leaves the PR branch's commits off `main`, and the
        // branch may be deleted: only a commit on `main` stays fetchable.
        let on_main = &[
            "branch",
            "--remotes",
            "--list",
            "origin/main",
            "--contains",
            &head,
        ];
        if git(&checkout, on_main)?.is_empty() {
            return Err(format!(
                "commit {head} isn't on the engine's main: merge its PR, then \
                 `git -C {CHECKOUT} switch main` and `git -C {CHECKOUT} pull --ff-only`, and run this again"
            ));
        }
        head
    };
    let manifest = set_manifest_rev(&read(root, "Cargo.toml")?, &rev)?;
    let old = (read(root, REV_FILE)?, read(root, "Cargo.toml")?);
    let write = |rev_text: &str, manifest: &str| {
        fs::write(root.join(REV_FILE), rev_text).map_err(|e| format!("{REV_FILE}: {e}"))?;
        fs::write(root.join("Cargo.toml"), manifest).map_err(|e| format!("Cargo.toml: {e}"))
    };
    write(&format!("{rev}\n"), &manifest)?;
    if let Err(e) = relock(root) {
        write(&old.0, &old.1)?;
        return Err(format!(
            "Cargo can't fetch {rev} from {URL}; nothing changed\n  cargo said: {e}"
        ));
    }
    Ok(format!(
        "{REV_FILE}, Cargo.toml and Cargo.lock now name {rev}: commit them with your change"
    ))
}

/// `engine [--off]`.
pub fn checkout(root: &Path, off: bool) -> Result<String, String> {
    let patch = root.join(PATCH_FILE);
    if off {
        if patch.exists() {
            fs::remove_file(&patch).map_err(|e| format!("{PATCH_FILE}: {e}"))?;
        }
        relock(root)?;
        return Ok(format!(
            "the game builds on the pinned commit again (Cargo.lock names it); {CHECKOUT}/ is kept"
        ));
    }
    let rev = parse_rev(&read(root, REV_FILE)?).map_err(|e| format!("{REV_FILE}: {e}"))?;
    let dir = root.join(CHECKOUT);
    let at = if dir.join(".git").exists() {
        git(&dir, &["fetch", "--quiet", "origin"])?;
        let head = git(&dir, &["rev-parse", "HEAD"])?;
        if head == rev {
            format!("{CHECKOUT}/ is at the pinned commit {rev}")
        } else {
            format!("{CHECKOUT}/ is at {head}, left as it is (the pin is {rev})")
        }
    } else {
        if fs::read_dir(&dir).is_ok_and(|mut entries| entries.next().is_some()) {
            return Err(format!(
                "{CHECKOUT}/ has files but isn't a git checkout: move it away first"
            ));
        }
        git(
            root,
            &[
                "clone",
                "--quiet",
                "--config",
                "core.longpaths=true",
                URL,
                CHECKOUT,
            ],
        )?;
        git(&dir, &["checkout", "--quiet", "--detach", &rev])?;
        format!("{CHECKOUT}/ is a new clone at the pinned commit {rev}")
    };
    fs::write(&patch, patch_file()).map_err(|e| format!("{PATCH_FILE}: {e}"))?;
    relock(root)?;
    Ok(format!(
        "{at}; the game builds on it now. Cargo.lock names the checkout while this is on: \
         `cargo xtask engine --off` before committing the game"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    const REV: &str = "5aa0f666216b05db22b90776a60e608abc3c31ad";
    const OTHER: &str = "0123456789abcdef0123456789abcdef01234567";

    fn manifest(rev: &str) -> String {
        let mut out = String::from("[workspace.dependencies]\nserde = \"1\"\n");
        for (name, _) in CRATES {
            let _ = writeln!(
                out,
                "{name} = {{ git = \"{URL}\", rev = \"{rev}\", default-features = false }}"
            );
        }
        out
    }

    fn lock(source: Option<&str>) -> String {
        let mut out = String::from(
            "version = 4\n\n[[package]]\nname = \"serde\"\nversion = \"1.0.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\n",
        );
        for (name, _) in CRATES {
            let _ = write!(
                out,
                "\n[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\n"
            );
            if let Some(source) = source {
                let _ = writeln!(out, "source = \"{source}\"");
            }
            out.push_str("dependencies = [\n \"serde\",\n]\n");
        }
        out
    }

    fn git_source(rev: &str) -> String {
        format!("git+{URL}?rev={rev}#{rev}")
    }

    #[test]
    fn a_rev_is_forty_lower_case_hex_digits() {
        assert_eq!(parse_rev(&format!(" {REV}\n")), Ok(REV.to_owned()));
        assert!(parse_rev(&REV[..39]).is_err());
        assert!(parse_rev(&REV.to_uppercase()).is_err());
        assert!(parse_rev(&format!("{}g", &REV[..39])).is_err());
        assert!(parse_rev("").is_err());
    }

    #[test]
    fn the_manifest_names_each_crate_at_its_rev() {
        let revs = manifest_revs(&manifest(REV));
        let names: Vec<&str> = revs.iter().map(|(n, _)| n.as_str()).collect();
        assert_eq!(names, CRATES.map(|(n, _)| n));
        assert!(revs.iter().all(|(_, r)| r == REV));
        // A crate of the same name from somewhere else isn't the engine's.
        assert_eq!(
            manifest_revs("shuyi-core = { path = \"x\", rev = \"y\" }\n"),
            [] as [(std::string::String, std::string::String); 0]
        );
        assert_eq!(
            manifest_revs("# shuyi-core = { git = \"https://github.com/Zafnok/shuyi-engine\" }\n"),
            [] as [(std::string::String, std::string::String); 0]
        );
    }

    #[test]
    fn setting_the_rev_changes_only_the_engine_lines() {
        let old = format!(
            "{}other = {{ git = \"x\", rev = \"{REV}\" }}\n",
            manifest(REV)
        );
        let new = set_manifest_rev(&old, OTHER).unwrap();
        assert!(manifest_revs(&new).iter().all(|(_, r)| r == OTHER));
        assert!(new.contains(&format!("other = {{ git = \"x\", rev = \"{REV}\" }}\n")));
        assert!(new.contains(", default-features = false }\n"));
        assert_eq!(new.len(), old.len());
        assert_eq!(set_manifest_rev(&new, REV).unwrap(), old);
    }

    #[test]
    fn setting_the_rev_needs_each_crate_once() {
        let missing = manifest(REV).replace("shuyi-cli = ", "unrelated = ");
        let err = set_manifest_rev(&missing, OTHER).unwrap_err();
        assert!(err.contains("`shuyi-cli"), "{err}");
        assert!(err.contains("found 0"), "{err}");
        let twice = format!("{}{}", manifest(REV), manifest(REV));
        assert!(
            set_manifest_rev(&twice, OTHER)
                .unwrap_err()
                .contains("found 2")
        );
    }

    #[test]
    fn the_lock_gives_each_engine_package_its_source() {
        let sources = lock_sources(&lock(Some(&git_source(REV))));
        assert_eq!(sources.len(), CRATES.len());
        assert!(
            sources
                .iter()
                .all(|(_, s)| s.as_deref() == Some(git_source(REV).as_str()))
        );
        assert!(lock_sources(&lock(None)).iter().all(|(_, s)| s.is_none()));
    }

    #[test]
    fn all_three_agreeing_is_no_problem() {
        let found = problems(
            &format!("{REV}\n"),
            &manifest(REV),
            &lock(Some(&git_source(REV))),
            false,
        );
        assert_eq!(found, Vec::<String>::new());
    }

    #[test]
    fn a_bad_rev_file_is_the_only_problem_said() {
        let found = problems("main\n", &manifest(REV), "", false);
        assert_eq!(found.len(), 1);
        assert!(found[0].starts_with("shuyi-engine.rev: "), "{found:?}");
    }

    #[test]
    fn the_manifest_must_be_at_the_rev_file_s_commit() {
        let found = problems(REV, &manifest(OTHER), &lock(Some(&git_source(REV))), false);
        assert_eq!(found.len(), CRATES.len(), "{found:?}");
        assert!(found[0].contains(&format!("is at {OTHER}")), "{found:?}");
        let gone = manifest(REV).replace("shuyi-ui = ", "unrelated = ");
        let found = problems(REV, &gone, &lock(Some(&git_source(REV))), false);
        assert_eq!(found, [format!("Cargo.toml: no `shuyi-ui` from {URL}")]);
    }

    #[test]
    fn the_lock_must_be_at_the_rev_and_not_the_checkout() {
        let found = problems(REV, &manifest(REV), &lock(Some(&git_source(OTHER))), false);
        assert_eq!(found.len(), CRATES.len(), "{found:?}");
        assert!(found[0].contains("engine-pin"), "{found:?}");
        let found = problems(REV, &manifest(REV), &lock(None), false);
        assert_eq!(found.len(), CRATES.len(), "{found:?}");
        assert!(found[0].contains("engine --off"), "{found:?}");
        let found = problems(REV, &manifest(REV), "version = 4\n", false);
        assert_eq!(found.len(), CRATES.len(), "{found:?}");
        assert!(found[0].starts_with("Cargo.lock: no `"), "{found:?}");
    }

    #[test]
    fn with_the_checkout_on_the_lock_is_not_checked() {
        assert_eq!(
            problems(REV, &manifest(REV), &lock(None), true),
            [] as [std::string::String; 0]
        );
        assert_eq!(
            problems(REV, &manifest(OTHER), &lock(None), true).len(),
            CRATES.len()
        );
    }

    #[test]
    fn the_patch_points_each_crate_at_the_checkout() {
        let patch = patch_file();
        assert!(patch.contains(&format!("[patch.\"{URL}\"]\n")));
        for (name, dir) in CRATES {
            assert!(
                patch.contains(&format!("{name} = {{ path = \"shuyi-engine/{dir}\" }}\n")),
                "{patch}"
            );
        }
    }

    #[test]
    fn the_checkout_and_its_patch_are_never_committed() {
        let root = crate::repo_root();
        let ignore = fs::read_to_string(root.join(".gitignore")).unwrap();
        assert!(ignore.lines().any(|l| l == format!("/{CHECKOUT}/")));
        assert!(ignore.lines().any(|l| l == format!("/{PATCH_FILE}")));
        // `.cargo/config.toml` takes the patch only when it is there.
        let config = fs::read_to_string(root.join(".cargo/config.toml")).unwrap();
        assert!(config.contains("include = [{ path = \"engine-patch.toml\", optional = true }]"));
        // And the checkout is a workspace of its own, not one of the game's.
        let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
        assert!(manifest.contains(&format!("exclude = [\"{CHECKOUT}\"]")));
    }

    #[test]
    fn the_real_repo_agrees_with_itself() {
        let root = crate::repo_root();
        let rev = fs::read_to_string(root.join(REV_FILE)).unwrap();
        let manifest = fs::read_to_string(root.join("Cargo.toml")).unwrap();
        let lock = fs::read_to_string(root.join("Cargo.lock")).unwrap();
        // The lock names the checkout while it is on, on a developer's machine.
        let patched = root.join(PATCH_FILE).exists();
        assert_eq!(
            problems(&rev, &manifest, &lock, patched),
            Vec::<String>::new()
        );
        assert!(check(&root).is_ok());
    }

    #[test]
    fn the_check_fails_on_each_ledger_problem() {
        let dir = std::env::temp_dir().join(format!("engine-pin-ledger-{}", std::process::id()));
        for folder in ["docs/engine", "tickets/open", "tickets/done"] {
            fs::create_dir_all(dir.join(folder)).unwrap();
        }
        fs::write(dir.join(REV_FILE), format!("{REV}\n")).unwrap();
        fs::write(dir.join("Cargo.toml"), manifest(REV)).unwrap();
        fs::write(dir.join("Cargo.lock"), lock(Some(&git_source(REV)))).unwrap();
        fs::write(dir.join("tickets/done/1191-x.md"), "").unwrap();
        let check_with = |ledger: &str| {
            fs::write(dir.join(ledger::PATH), ledger).unwrap();
            check(&dir)
        };
        let good =
            "## 2026-10-25: baseline → 5aa0f666\n| #78 | 2511 | A prelude | adopt → 1191 |\n";
        assert!(check_with(good).unwrap().contains("has its section"));
        let stale = check_with(&good.replace("5aa0f666", "01234567")).unwrap_err();
        assert!(
            stale[0].contains("the last section pins 01234567"),
            "{stale:?}"
        );
        let undecided = check_with(&good.replace("adopt → 1191", "")).unwrap_err();
        assert!(
            undecided[0].contains("the row of #78 has no decision"),
            "{undecided:?}"
        );
        let unknown = check_with(&good.replace("1191", "1192")).unwrap_err();
        assert!(
            unknown[0].contains("`adopt → 1192` names no ticket"),
            "{unknown:?}"
        );
        fs::remove_file(dir.join(ledger::PATH)).unwrap();
        assert!(check(&dir).unwrap_err()[0].contains("pins.md"));
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn pinning_is_refused_while_the_checkout_is_on() {
        let dir = std::env::temp_dir().join(format!("engine-pin-{}", std::process::id()));
        fs::create_dir_all(dir.join(".cargo")).unwrap();
        fs::write(dir.join(PATCH_FILE), patch_file()).unwrap();
        let err = pin(&dir, Some(REV)).unwrap_err();
        assert!(err.contains("engine --off"), "{err}");
        fs::remove_file(dir.join(PATCH_FILE)).unwrap();
        // Without the checkout or a commit there is nothing to pin.
        let err = pin(&dir, None).unwrap_err();
        assert!(err.contains("isn't there"), "{err}");
        assert!(
            pin(&dir, Some("main"))
                .unwrap_err()
                .contains("40-character")
        );
        let _ = fs::remove_dir_all(&dir);
    }
}
