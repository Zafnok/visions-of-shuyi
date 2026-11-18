//! The repository's process tool (ADR-0129): the ticket lint, `pr-size`
//! and the other checks every commit, push and pull request runs. It
//! depends on no crate of the game and none of the engine's, so it builds
//! in seconds in a fresh worktree.
//!
//! The game's tool program is `trpg-tools` (`crates/tools`): `cargo shuyi`,
//! the bots, the voice, the importers and the generators. Each of its
//! commands still answers here by its old name ([`FORWARDED`]): xtask runs
//! it there, which builds the game.
//!
//! A CLI tool prints to stdout by design, so `print_stdout` is allowed here.
#![allow(clippy::print_stdout)]

mod clean_targets;
mod code_lines;
mod engine;
mod leaks;
mod mentions;
mod pr_size;
mod tickets;
#[cfg(test)]
mod workflow_guard;

use std::env;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};
use std::time::SystemTime;

use xtask::FORWARDED;

const USAGE: &str = "usage: cargo xtask <command>\n\n\
available commands:\n  \
ticket-lint [--pr-branch <name>]   check tickets/{open,done} against tickets/README.md\n    \
[--check-engine]                 and look up each engine#NNNN in the engine's tickets\n  \
check-mentions <text>              find what GitHub would read as a mention of a user, in a PR title\n  \
check-leaks                        find a tracked file that quotes the game's lines or map rows (ADR-0118)\n  \
engine [--off]                    build the game on Shuyi Engine checked out in shuyi-engine/\n  \
engine-pin [<commit> | --check [--offline]] move the engine's pinned commit, or check it and its ledger (ADR-0063 §8)\n  \
engine-pin --report                the engine PRs past the pin, sorted, and whether a move is due (ADR-0096)\n  \
clean-merged-targets [--dry-run]   delete target/ in worktrees whose PR has merged\n  \
code-lines [--base <ref>]          production Rust lines added and removed per crate (ADR-0092)\n  \
pr-size [--base <ref>] [--branch <name>] [--hook]\n                                     \
the PR's changed lines against the ~3,000 limit; warns, never fails (ADR-0101)\n\n\
the game's tool program, trpg-tools, runs these (they build the game; cargo xtask <command>\n\
still runs each, and `cargo run -p trpg-tools -- --help` lists them):\n  \
skill-pack, sfx, test-card, test-tileset, effect-marks, edge-marks, lettering, title-art,\n  \
map-sprite-import, battler-import, tileset-import, playtest, skirmish, portrait-import,\n  \
lines, check-script, story-import, voice, voice-test-clips, icon, playdate, playdate-images,\n  \
readme-pictures, and Shuyi's tools: shuyi (`cargo shuyi`), check-keys, check-text,\n  \
lang-status, font-atlas, frame-png, web, private-assets (ADR-0129)";

fn main() -> ExitCode {
    ExitCode::from(dispatch(env::args().skip(1)))
}

/// The actual command dispatch, as a plain exit code (0 success) rather than
/// `ExitCode` so it's directly comparable in tests.
fn dispatch(mut args: impl Iterator<Item = String>) -> u8 {
    match args.next().as_deref() {
        Some("ticket-lint") => ticket_lint(&args.collect::<Vec<_>>()),
        Some("check-mentions") => check_mentions(&args.collect::<Vec<_>>()),
        Some("check-leaks") => leaks::command(&repo_root(), &args.collect::<Vec<_>>()),
        Some("engine") => engine(&args.collect::<Vec<_>>()),
        Some("engine-pin") => engine_pin(&args.collect::<Vec<_>>()),
        Some("clean-merged-targets") => clean_merged_targets(&args.collect::<Vec<_>>()),
        Some("code-lines") => code_lines(&args.collect::<Vec<_>>()),
        Some("pr-size") => pr_size(&args.collect::<Vec<_>>(), || {
            std::io::read_to_string(std::io::stdin()).unwrap_or_default()
        }),
        Some(command) if FORWARDED.contains(&command) => forward(command, args),
        Some(command) => {
            eprintln!("unknown command: {command}");
            eprintln!("{USAGE}");
            2
        }
        None => {
            println!("{USAGE}");
            2
        }
    }
}

/// Runs `command` with `args` in the game's tool program, `trpg-tools`
/// (ADR-0129), and answers its exit code: `cargo xtask playtest …` keeps
/// working, and builds the game as it always did.
fn forward(command: &str, args: impl Iterator<Item = String>) -> u8 {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| "cargo".into());
    let release = cfg!(not(debug_assertions));
    let private_assets = cfg!(feature = "private-assets");
    let run = Command::new(cargo)
        .args(forward_args(command, args, release, private_assets))
        .current_dir(repo_root())
        .status();
    match run {
        Ok(status) => status
            .code()
            .and_then(|code| u8::try_from(code).ok())
            .unwrap_or(1),
        Err(e) => {
            eprintln!("{command}: can't run cargo for trpg-tools: {e}");
            1
        }
    }
}

/// The arguments to `cargo` that run `command` with `args` in
/// `trpg-tools`: in the profile xtask was built in, with the bought art when
/// xtask was built with it.
fn forward_args(
    command: &str,
    args: impl Iterator<Item = String>,
    release: bool,
    private_assets: bool,
) -> Vec<String> {
    let mut out: Vec<String> = ["run", "-q", "-p", "trpg-tools"].map(str::to_owned).into();
    if release {
        out.push("--release".to_owned());
    }
    if private_assets {
        out.extend(["--features", "private-assets"].map(str::to_owned));
    }
    out.extend(["--".to_owned(), command.to_owned()]);
    out.extend(args);
    out
}

fn ticket_lint(args: &[String]) -> u8 {
    let (pr_branch, check_engine) = match parse_ticket_lint_args(args) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };

    let repo_root = repo_root();
    let errors = tickets::run(&repo_root, pr_branch.as_deref(), check_engine);

    if errors.is_empty() {
        println!("ticket-lint: OK");
        return 0;
    }

    eprintln!("ticket-lint: {} error(s)", errors.len());
    for error in &errors {
        eprintln!("  {error}");
    }
    1
}

/// `check-mentions <text>`: no accidental mention of a GitHub user in a PR
/// title, which the release notes are generated from (ticket 0124).
fn check_mentions(args: &[String]) -> u8 {
    let [text] = args else {
        eprintln!("usage: cargo xtask check-mentions <text>");
        return 2;
    };
    let found = mentions::bare_mentions(text);
    if found.is_empty() {
        println!("check-mentions: OK");
        return 0;
    }
    eprintln!("check-mentions: {} bare mention(s)", found.len());
    for mention in &found {
        eprintln!("  {mention}");
    }
    eprintln!(
        "put it in backticks or drop the at sign: GitHub reads it as a mention of that user, \
         and the release notes list them as a contributor"
    );
    1
}

/// `engine [--off]`: the local checkout of Shuyi Engine (ticket 1131).
fn engine(args: &[String]) -> u8 {
    let off = match args {
        [] => false,
        [flag] if flag == "--off" => true,
        _ => {
            eprintln!("{}", engine::USAGE);
            return 2;
        }
    };
    report(engine::checkout(&repo_root(), off))
}

/// Prints what `engine-pin --check` found: its exit code.
fn check_report(found: Result<String, Vec<String>>) -> u8 {
    match found {
        Ok(said) => {
            println!("{said}");
            0
        }
        Err(problems) => {
            eprintln!("engine-pin: {} problem(s)", problems.len());
            for problem in &problems {
                eprintln!("  {problem}");
            }
            1
        }
    }
}

/// `engine-pin [<commit> | --check [--offline] | --report]`: the engine's pinned commit.
fn engine_pin(args: &[String]) -> u8 {
    match args {
        [flag] if flag == "--check" => check_report(engine::check_with_gains(&repo_root())),
        [flag, offline] if flag == "--check" && offline == "--offline" => {
            check_report(engine::check(&repo_root()))
        }
        [flag] if flag == "--report" => report(engine::watch_report(&repo_root())),
        [commit] if !commit.starts_with('-') => report(engine::pin(&repo_root(), Some(commit))),
        [] => report(engine::pin(&repo_root(), None)),
        _ => {
            eprintln!("{}", engine::USAGE);
            2
        }
    }
}

/// Prints what a command did, or why it failed.
fn report(outcome: Result<String, String>) -> u8 {
    match outcome {
        Ok(said) => {
            println!("{said}");
            0
        }
        Err(e) => {
            eprintln!("{e}");
            1
        }
    }
}

fn clean_merged_targets(args: &[String]) -> u8 {
    let dry_run = match clean_targets::parse_args(args) {
        Ok(dry_run) => dry_run,
        Err(e) => {
            eprintln!("{e}");
            return 2;
        }
    };
    let repo_root = repo_root();
    let tools = clean_targets::RealTools {
        repo_root: &repo_root,
    };
    let print = |line| println!("{line}");
    match clean_targets::run(&tools, &repo_root, SystemTime::now(), dry_run, print) {
        Ok(outcome) => u8::from(outcome.failed),
        Err(e) => {
            eprintln!("clean-merged-targets: {e}");
            1
        }
    }
}

/// `ticket-lint`'s arguments: the `--pr-branch` value, and whether
/// `--check-engine` was given.
fn parse_ticket_lint_args(args: &[String]) -> Result<(Option<String>, bool), String> {
    let mut pr_branch = None;
    let mut check_engine = false;
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--pr-branch" => match iter.next() {
                Some(branch) => pr_branch = Some(branch.clone()),
                None => return Err("--pr-branch requires a value".to_string()),
            },
            "--check-engine" => check_engine = true,
            other => return Err(format!("unknown argument to ticket-lint: {other}")),
        }
    }
    Ok((pr_branch, check_engine))
}

fn code_lines(args: &[String]) -> u8 {
    let base = match args {
        [] => "origin/main",
        [flag, base] if flag == "--base" => base.as_str(),
        [help] if help == "--help" => {
            println!("{}", code_lines::USAGE);
            return 0;
        }
        _ => {
            eprintln!("{}", code_lines::USAGE);
            return 2;
        }
    };
    match code_lines::run(&repo_root(), base) {
        Ok(report) => {
            print!("{report}");
            0
        }
        Err(e) => {
            eprintln!("code-lines: {e}");
            1
        }
    }
}

/// `pr-size`'s arguments: the base, the branch (if given), and `--hook`.
fn parse_pr_size_args(args: &[String]) -> Result<(String, Option<String>, bool), String> {
    let (mut base, mut branch, mut hook) = ("origin/main".to_owned(), None, false);
    let mut iter = args.iter();
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--base" => base.clone_from(iter.next().ok_or("--base requires a value")?),
            "--branch" => branch = Some(iter.next().ok_or("--branch requires a value")?.clone()),
            "--hook" => hook = true,
            other => return Err(format!("unknown argument to pr-size: {other}")),
        }
    }
    Ok((base, branch, hook))
}

/// `pr-size`; `hook_stdin` reads the hook's input, only with `--hook`.
fn pr_size(args: &[String], hook_stdin: impl FnOnce() -> String) -> u8 {
    if args.iter().any(|a| a == "--help") {
        println!("{}", pr_size::USAGE);
        return 0;
    }
    let (base, branch, hook) = match parse_pr_size_args(args) {
        Ok(parsed) => parsed,
        Err(e) => {
            eprintln!("pr-size: {e}\n{}", pr_size::USAGE);
            return 2;
        }
    };
    let root = repo_root();
    if hook {
        let answer = pr_size::hook_answer(&hook_stdin(), &root, &base, branch.as_deref());
        print!("{answer}");
        return 0;
    }
    let measured = pr_size::changed_files(&root, &base).and_then(|(files, from)| {
        let branch = branch.map_or_else(|| pr_size::current_branch(&root), Ok)?;
        Ok((files, from, branch))
    });
    let (files, from, branch) = match measured {
        Ok(measured) => measured,
        Err(e) => {
            eprintln!("pr-size: {e}");
            return 1;
        }
    };
    print!(
        "{}",
        pr_size::output(
            &files,
            &branch,
            &format!("{base} (from {from})"),
            false,
            env::var("GITHUB_ACTIONS").ok().as_deref(),
        )
    );
    0
}

/// `xtask` always runs via `cargo xtask`, so `CARGO_MANIFEST_DIR` (this
/// crate's directory, `<repo>/crates/xtask`) locates the repo root.
#[allow(clippy::expect_used)] // CARGO_MANIFEST_DIR is baked in at compile time; the two
// `parent()` calls can only fail if xtask's own manifest ever moves.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask is at <repo>/crates/xtask")
        .to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{BTreeMap, BTreeSet};

    fn args(items: &[&str]) -> Vec<String> {
        items.iter().map(|s| (*s).to_string()).collect()
    }

    /// The lines of `manifest`'s `[header]` table, without comments and
    /// blank lines.
    fn table<'a>(manifest: &'a str, header: &str) -> Vec<&'a str> {
        let lines = manifest.lines().skip_while(|l| l.trim() != header).skip(1);
        lines
            .take_while(|l| !l.starts_with('['))
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .collect()
    }

    /// The platform crate here, `trpg-playdate`, copies the workspace lints
    /// because one of them differs (`unsafe_code`, ADR-0070 §4); the copy
    /// must not drift.
    #[test]
    fn platform_lints_are_the_workspace_lints_except_unsafe_code() {
        let root = repo_root();
        let workspace = std::fs::read_to_string(root.join("Cargo.toml")).unwrap();
        let clippy = table(&workspace, "[workspace.lints.clippy]");
        assert!(clippy.len() > 5, "{clippy:?}");
        let rust: Vec<String> = table(&workspace, "[workspace.lints.rust]")
            .iter()
            .map(|l| l.replace("unsafe_code = \"forbid\"", "unsafe_code = \"deny\""))
            .collect();
        assert!(
            rust.contains(&"unsafe_code = \"deny\"".to_owned()),
            "{rust:?}"
        );
        // `shuyi-app`'s copy is checked by a test in the engine's
        // repository (ticket 1131).
        let manifest = std::fs::read_to_string(root.join("crates/playdate/Cargo.toml")).unwrap();
        assert_eq!(table(&manifest, "[lints.clippy]"), clippy);
        assert_eq!(table(&manifest, "[lints.rust]"), rust);
        // Every other crate inherits the workspace's `forbid`, the game's
        // binary too: its platform code is the engine's (ticket 1110).
        for other in ["app", "bots", "core", "content", "ui", "tools", "xtask"] {
            let manifest =
                std::fs::read_to_string(root.join(format!("crates/{other}/Cargo.toml"))).unwrap();
            assert_eq!(table(&manifest, "[lints]"), ["workspace = true"], "{other}");
        }
    }

    /// The playtest bots are dev tooling (ADR-0033): nothing the game is
    /// built from may depend on them, and they may not depend on the game's
    /// look (ADR-0038). Every way to a crate of this workspace is a
    /// manifest here, so reading the manifests covers indirect ones too.
    #[test]
    fn bots_stay_out_of_the_game() {
        // A manifest without its comments.
        let manifest = |name: &str| {
            let path = repo_root().join(format!("crates/{name}/Cargo.toml"));
            let text = std::fs::read_to_string(path).unwrap();
            let lines: Vec<&str> = text.lines().filter(|l| !l.starts_with('#')).collect();
            lines.join("\n")
        };
        for game in ["core", "content", "ui", "app"] {
            assert!(!manifest(game).contains("trpg-bots"), "{game}");
        }
        for bots in ["bots", "core", "content"] {
            let manifest = manifest(bots);
            assert!(!manifest.contains("trpg-ui"), "{bots}");
            assert!(!manifest.contains("trpg-app"), "{bots}");
        }
        assert!(manifest("tools").contains("trpg-bots"));
    }

    /// The names of the `.rs` files directly in `dir` (not in its subfolders).
    fn top_level_rs_files(dir: &std::path::Path) -> Vec<String> {
        let mut names: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.is_file() && path.extension().is_some_and(|e| e == "rs"))
            .map(|path| path.file_name().unwrap().to_string_lossy().into_owned())
            .collect();
        names.sort();
        names
    }

    /// The workflow and action files under `.github`.
    fn workflow_files() -> Vec<PathBuf> {
        let github = repo_root().join(".github");
        let mut files = Vec::new();
        for entry in std::fs::read_dir(github.join("workflows")).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_some_and(|e| e == "yml" || e == "yaml") {
                files.push(path);
            }
        }
        for entry in std::fs::read_dir(github.join("actions")).unwrap() {
            let action = entry.unwrap().path().join("action.yml");
            if action.is_file() {
                files.push(action);
            }
        }
        files.sort();
        files
    }

    /// GitHub runs no job of a workflow file that is not valid YAML, and only
    /// says so after the merge: an unquoted `run:` line ending in `::`
    /// stopped every Pages deploy (ticket 0118).
    #[test]
    fn every_workflow_file_parses() {
        let files = workflow_files();
        assert!(files.len() >= 9, "{files:?}");
        for file in files {
            let text = std::fs::read_to_string(&file).unwrap();
            if let Err(e) = serde_norway::from_str::<serde_norway::Value>(&text) {
                panic!("{} is not valid YAML: {e}", file.display());
            }
        }

        // The check sees the mistake.
        let broken = "steps:
  - run: cargo test private_assets::
";
        assert!(serde_norway::from_str::<serde_norway::Value>(broken).is_err());
    }

    /// The `.rs` files under `dir`, at any depth.
    fn rs_files_under(dir: &Path, out: &mut Vec<PathBuf>) {
        for entry in std::fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                rs_files_under(&path, out);
            } else if path.extension().is_some_and(|e| e == "rs") {
                out.push(path);
            }
        }
    }

    /// Which crates read each Markdown file of the repository, from the
    /// string literals that name it (`"../../docs/design/audio.md"`,
    /// `"dialogue/README.md"` in the embedded assets): the file's path
    /// from the root, to the packages whose sources name it.
    fn markdown_the_code_reads() -> BTreeMap<String, BTreeSet<String>> {
        let root = repo_root();
        let mut read: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for krate in std::fs::read_dir(root.join("crates")).unwrap() {
            let krate = krate.unwrap().path();
            let manifest = std::fs::read_to_string(krate.join("Cargo.toml")).unwrap();
            let package = manifest
                .lines()
                .find_map(|l| l.strip_prefix("name = \""))
                .and_then(|n| n.strip_suffix('"'))
                .expect("a package name")
                .to_string();
            let mut files = Vec::new();
            rs_files_under(&krate, &mut files);
            for file in files {
                let text = std::fs::read_to_string(&file).unwrap();
                // Every other piece between quotes is a literal; a comment
                // names a file without reading it.
                let code = text.lines().filter(|l| !l.trim_start().starts_with("//"));
                for literal in code.flat_map(|l| l.split('"').skip(1).step_by(2)) {
                    let markdown = Path::new(literal).extension().is_some_and(|e| e == "md");
                    if !markdown || !literal.contains('/') {
                        continue;
                    }
                    let path = literal.trim_start_matches("../");
                    for base in ["", "assets/", "assets-test/"] {
                        let path = format!("{base}{path}");
                        if root.join(&path).is_file() {
                            read.entry(path).or_default().insert(package.clone());
                        }
                    }
                }
            }
        }
        read
    }

    /// CI skips its build and tests on a pull request that changes only
    /// docs, Markdown among them (ADR-0014), so PR #464 changed only
    /// `docs/design/audio.md`, merged green and broke `main`. Each Markdown
    /// file the code reads has a `read by` line in `changes.yml` naming the
    /// crates that read it, whose tests CI's `doc-tests` job runs (ticket
    /// 0252, ADR-0098).
    #[test]
    fn every_markdown_file_the_code_reads_is_tested() {
        let changes =
            std::fs::read_to_string(repo_root().join(".github/workflows/changes.yml")).unwrap();
        let mut listed: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for line in changes.lines().filter(|l| l.contains("echo \"read by ")) {
            let (paths, rest) = line.trim().split_once(')').unwrap();
            let crates = rest
                .split_once("read by ")
                .unwrap()
                .1
                .split_once(':')
                .unwrap()
                .0;
            assert!(
                rest.contains(&format!("doc_crates=\"$doc_crates {crates}\"")),
                "{line}"
            );
            let crates: BTreeSet<String> = crates.split(' ').map(str::to_string).collect();
            for path in paths.split('|') {
                assert!(
                    listed.insert(path.to_string(), crates.clone()).is_none(),
                    "{path}"
                );
            }
        }
        let mut read = markdown_the_code_reads();
        // A file name in playdate_images' test of a made-up bundle, not read.
        assert!(read.remove("assets/battlers/README.md").is_some());
        assert_eq!(listed, read, "changes.yml's `read by` lines");
    }

    /// No workflow job that checks out the private files keeps a build of
    /// them in the Actions cache or an artifact, which a fork's pull request
    /// can read (ADR-0117 §4, ticket 0171).
    #[test]
    fn no_job_with_private_files_caches_or_uploads_a_build() {
        let dir = repo_root().join(".github/workflows");
        let mut problems = Vec::new();
        let mut guarded = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let path = entry.unwrap().path();
            if path.extension().is_none_or(|e| e != "yml") {
                continue;
            }
            let text = std::fs::read_to_string(&path).unwrap();
            let file = path.file_name().unwrap().to_string_lossy().into_owned();
            guarded += text.matches("Swatinem/rust-cache@").count();
            problems.extend(workflow_guard::check(&file, &text));
        }
        assert!(guarded >= 10, "{guarded}");
        assert!(
            problems.is_empty(),
            "{}",
            problems.join(
                "
"
            )
        );
        // Each upload the check allows is still there.
        for (file, job, upload) in workflow_guard::ALLOWED_UPLOADS {
            let text = std::fs::read_to_string(dir.join(file)).unwrap();
            assert!(
                text.contains(&format!(
                    "
  {job}:
"
                )),
                "{file}: {job}"
            );
            assert!(
                text.contains(&format!(
                    "path: {upload}
"
                )),
                "{file}: {upload}"
            );
        }
    }

    /// Cargo links every file directly under a crate's `tests/` as its own
    /// program (about 100 MB each), so each crate keeps one: new integration
    /// tests are modules of `tests/it/main.rs` (ticket 0114).
    #[test]
    fn each_crate_has_at_most_one_integration_test_program() {
        let crates = std::fs::read_dir(repo_root().join("crates")).unwrap();
        let mut checked = 0;
        for krate in crates {
            let tests = krate.unwrap().path().join("tests");
            if !tests.is_dir() {
                continue;
            }
            checked += 1;
            let files = top_level_rs_files(&tests);
            assert!(
                files.len() <= 1,
                "{} has {files:?}: put new integration tests in tests/it/ \
                 as a module listed in tests/it/main.rs",
                tests.display()
            );
        }
        assert!(checked >= 4, "{checked}");

        // The check sees a second file.
        let dir = std::env::temp_dir().join(format!("xtask-tests-dir-{}", std::process::id()));
        std::fs::create_dir_all(dir.join("it")).unwrap();
        std::fs::write(dir.join("it/main.rs"), "").unwrap();
        std::fs::write(dir.join("a.rs"), "").unwrap();
        std::fs::write(dir.join("notes.md"), "").unwrap();
        assert_eq!(top_level_rs_files(&dir), ["a.rs"]);
        std::fs::write(dir.join("b.rs"), "").unwrap();
        assert_eq!(top_level_rs_files(&dir), ["a.rs", "b.rs"]);
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn code_lines_reports_and_rejects_bad_arguments() {
        assert_eq!(
            dispatch(args(&["code-lines", "--base", "HEAD"]).into_iter()),
            0
        );
        assert_eq!(code_lines(&args(&["--help"])), 0);
        assert_eq!(code_lines(&args(&["--base"])), 2);
        assert_eq!(code_lines(&args(&["--base", "no-such-ref-1151"])), 1);
    }

    #[test]
    fn pr_size_reports_hooks_and_rejects_bad_arguments() {
        assert_eq!(
            dispatch(args(&["pr-size", "--base", "HEAD"]).into_iter()),
            0
        );
        let pr_size = |a: &[&str]| {
            pr_size(&args(a), || {
                r#"{"tool_input":{"command":"git push"}}"#.to_owned()
            })
        };
        assert_eq!(
            pr_size(&["--base", "HEAD", "--branch", "sonar/x", "--hook"]),
            0
        );
        assert_eq!(pr_size(&["--base", "no-such-ref-0135", "--hook"]), 0);
        assert_eq!(pr_size(&["--help"]), 0);
        assert_eq!(pr_size(&["--base"]), 2);
        assert_eq!(pr_size(&["--branch"]), 2);
        assert_eq!(pr_size(&["--size"]), 2);
        assert_eq!(pr_size(&["--base", "no-such-ref-0135"]), 1);
        assert_eq!(
            parse_pr_size_args(&args(&["--hook", "--branch", "t1-a", "--base", "HEAD^1"])),
            Ok(("HEAD^1".to_owned(), Some("t1-a".to_owned()), true))
        );
        assert_eq!(
            parse_pr_size_args(&[]),
            Ok(("origin/main".to_owned(), None, false))
        );
    }

    #[test]
    fn repo_root_points_at_the_workspace_root() {
        let root = repo_root();
        assert!(root.join("Cargo.toml").is_file());
        assert!(root.join("tickets/README.md").is_file());
    }

    #[test]
    fn parse_ticket_lint_args_with_no_args_is_none() {
        assert_eq!(parse_ticket_lint_args(&[]), Ok((None, false)));
    }

    #[test]
    fn parse_ticket_lint_args_reads_the_pr_branch_value() {
        assert_eq!(
            parse_ticket_lint_args(&args(&["--pr-branch", "t0106-x"])),
            Ok((Some("t0106-x".to_string()), false))
        );
    }

    #[test]
    fn parse_ticket_lint_args_reads_check_engine_in_any_order() {
        let want = Ok((Some("t0106-x".to_string()), true));
        assert_eq!(
            parse_ticket_lint_args(&args(&["--check-engine", "--pr-branch", "t0106-x"])),
            want
        );
        assert_eq!(
            parse_ticket_lint_args(&args(&["--pr-branch", "t0106-x", "--check-engine"])),
            want
        );
    }

    #[test]
    fn parse_ticket_lint_args_requires_a_pr_branch_value() {
        assert_eq!(
            parse_ticket_lint_args(&args(&["--pr-branch"])),
            Err("--pr-branch requires a value".to_string())
        );
    }

    #[test]
    fn parse_ticket_lint_args_rejects_unknown_flags() {
        assert_eq!(
            parse_ticket_lint_args(&args(&["--bogus"])),
            Err("unknown argument to ticket-lint: --bogus".to_string())
        );
    }

    #[test]
    fn ticket_lint_fails_fast_on_bad_args() {
        assert_eq!(ticket_lint(&args(&["--pr-branch"])), 2);
    }

    #[test]
    fn ticket_lint_succeeds_on_the_real_repo() {
        assert_eq!(ticket_lint(&[]), 0);
    }

    #[test]
    fn ticket_lint_fails_when_pr_branch_names_an_unknown_ticket() {
        // Ticket 9999 will never exist (see tickets/README.md's numbering),
        // unlike a real open/done ticket id, whose status changes as tickets
        // are worked — this exercises the wrapper's non-zero exit path
        // without depending on the repo's current ticket state.
        assert_eq!(ticket_lint(&args(&["--pr-branch", "t9999-ghost"])), 1);
    }

    #[test]
    fn check_mentions_fails_on_a_bare_mention() {
        let title = "[0710] Dialogue scripts choose the music: @music";
        assert_eq!(dispatch(args(&["check-mentions", title]).into_iter()), 1);
    }

    #[test]
    fn check_mentions_passes_on_backticks_and_an_email_address() {
        for title in [
            "[0710] Dialogue scripts choose the music: `@music`",
            "Mail noreply@anthropic.com about [0001]",
        ] {
            assert_eq!(dispatch(args(&["check-mentions", title]).into_iter()), 0);
        }
    }

    #[test]
    fn check_mentions_takes_exactly_one_argument() {
        assert_eq!(dispatch(args(&["check-mentions"]).into_iter()), 2);
        assert_eq!(dispatch(args(&["check-mentions", "a", "b"]).into_iter()), 2);
    }

    #[test]
    fn dispatch_with_no_command_prints_usage_and_fails() {
        assert_eq!(dispatch(std::iter::empty()), 2);
    }

    #[test]
    fn dispatch_with_unknown_command_fails() {
        assert_eq!(dispatch(args(&["bogus"]).into_iter()), 2);
    }

    #[test]
    fn engine_pin_check_passes_on_the_real_repo() {
        assert_eq!(check_report(engine::check(&repo_root())), 0);
        assert_eq!(check_report(Err(vec!["a problem".to_owned()])), 1);
        assert_eq!(engine_pin(&args(&["--bogus"])), 2);
        assert_eq!(engine_pin(&args(&["a", "b"])), 2);
        assert_eq!(engine(&args(&["--bogus"])), 2);
        assert_eq!(engine(&args(&["--off", "x"])), 2);
    }

    #[test]
    fn clean_merged_targets_rejects_unknown_args() {
        // Only the argument check: a real run needs `gh` and would delete
        // build folders.
        assert_eq!(clean_merged_targets(&args(&["--bogus"])), 2);
        assert_eq!(
            dispatch(args(&["clean-merged-targets", "--dry-run", "x"]).into_iter()),
            2
        );
    }

    #[test]
    fn dispatch_runs_ticket_lint() {
        assert_eq!(dispatch(args(&["ticket-lint"]).into_iter()), 0);
    }

    /// A moved command runs in `trpg-tools`, in xtask's profile, with the
    /// bought art when xtask has it (ADR-0129).
    #[test]
    fn a_moved_command_runs_in_the_game_s_tool_program() {
        let forwarded = |release, private| {
            forward_args(
                "playtest",
                args(&["quick", "--runs", "2"]).into_iter(),
                release,
                private,
            )
        };
        assert_eq!(
            forwarded(false, false),
            args(&[
                "run",
                "-q",
                "-p",
                "trpg-tools",
                "--",
                "playtest",
                "quick",
                "--runs",
                "2"
            ])
        );
        assert_eq!(
            forwarded(true, true),
            args(&[
                "run",
                "-q",
                "-p",
                "trpg-tools",
                "--release",
                "--features",
                "private-assets",
                "--",
                "playtest",
                "quick",
                "--runs",
                "2",
            ])
        );
    }

    /// xtask builds without the game (ADR-0129): no crate of the game and
    /// none of the engine's in its manifest, so none in its build.
    #[test]
    fn xtask_depends_on_no_crate_of_the_game_or_the_engine() {
        let manifest =
            std::fs::read_to_string(repo_root().join("crates/xtask/Cargo.toml")).unwrap();
        let dependencies = manifest
            .lines()
            .filter(|l| !l.starts_with('#'))
            .filter_map(|l| l.split_once('=').map(|(name, _)| name.trim()));
        for name in dependencies {
            let allowed = name == "trpg-engine-watch"
                || !(name.starts_with("trpg-") || name.starts_with("shuyi-"));
            assert!(allowed, "{name}");
        }
        assert!(manifest.contains("trpg-engine-watch"));
    }

    /// No command is both xtask's own and forwarded.
    #[test]
    fn the_forwarded_commands_are_not_xtask_s_own() {
        for own in [
            "ticket-lint",
            "check-mentions",
            "check-leaks",
            "engine",
            "engine-pin",
            "clean-merged-targets",
            "code-lines",
            "pr-size",
        ] {
            assert!(!FORWARDED.contains(&own), "{own}");
            assert!(USAGE.contains(own), "{own}");
        }
        for moved in FORWARDED {
            assert!(USAGE.contains(moved), "{moved}");
        }
    }
}
