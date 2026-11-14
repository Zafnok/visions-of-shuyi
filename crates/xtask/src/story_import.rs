//! `cargo xtask story-import <n> [--dry-run]` (ticket 0152, ADR-0122):
//! takes a pull request of the writer's mirror that Nick approved into the
//! private assets repository. The mirror's files are byte copies of the
//! assets repository's at a pin (the sync's commit names it in an
//! `Assets-Commit:` trailer), so the writer's diff, with full blob ids,
//! applies three-way onto the assets repository's `main` however far it
//! moved since. Works on a scratch index in the checkout
//! `assets-private/`: its working tree and branch are never touched. Then
//! pushes a branch there and opens its pull request; the `story-import`
//! skill moves the game's pin after it merges (`content-change`).

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use serde_json::Value;

/// The writer's mirror.
pub const MIRROR: &str = "Zafnok/visions-of-shuyi-story";
/// The private assets repository (ADR-0040, ADR-0117).
pub const ASSETS: &str = "Zafnok/visions-of-shuyi-assets";
/// Whose approval takes a pull request in: the mirror's owner, Nick.
pub const OWNER: &str = "Zafnok";
/// What the mirror holds, read by the sync too.
pub const FILES: &str = "tools/story-mirror/files.txt";
/// The checkout of the assets repository, in the game's folder.
const CHECKOUT: &str = "assets-private/";

pub const USAGE: &str = "usage: cargo xtask story-import <pull request> [--dry-run]\n\n\
takes a pull request of the writer's mirror (Zafnok/visions-of-shuyi-story)\n\
that Nick approved into the private assets repository: applies its change\n\
three-way onto origin/main in assets-private/, with the writer as the\n\
commit's author, pushes the branch story/<n>-<slug> and opens its pull\n\
request. --dry-run prints the change and pushes nothing. The story-import\n\
skill has the steps after it.";

/// One line of [`FILES`]: a mirror path, where it comes from, and whether
/// a writer's change there is taken back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Entry {
    pub mirror: String,
    pub from: String,
    pub import: bool,
}

/// The lines of [`FILES`]'s text.
pub fn read_files(text: &str) -> Result<Vec<Entry>, String> {
    let mut out = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let words: Vec<&str> = line.split_whitespace().collect();
        let entry = match words.as_slice() {
            [] => continue,
            [first, ..] if first.starts_with('#') => continue,
            [mirror, from] => (mirror, from, false),
            [mirror, from, "import"] => (mirror, from, true),
            _ => {
                return Err(format!(
                    "{FILES}:{}: expected <mirror path> <from> [import]",
                    i + 1
                ));
            }
        };
        let (mirror, from, import) = entry;
        if import && !from.starts_with(CHECKOUT) {
            return Err(format!(
                "{FILES}:{}: only a path in {CHECKOUT} can be taken back",
                i + 1
            ));
        }
        out.push(Entry {
            mirror: (*mirror).to_owned(),
            from: (*from).to_owned(),
            import,
        });
    }
    Ok(out)
}

/// How specifically `pattern` (a mirror path of [`FILES`]) holds `path`:
/// `None` when it doesn't; a longer pattern beats a shorter one, and a file
/// named whole beats both. With the path `path` has under `from`.
fn holds(pattern: &str, from: &str, path: &str) -> Option<(usize, String)> {
    if let Some((dir, ext)) = pattern.split_once('*') {
        let name = path.strip_prefix(dir)?;
        let from_dir = from.split_once('*')?.0;
        let ok = !name.contains('/') && name.ends_with(ext) && name.len() > ext.len();
        ok.then(|| (dir.len(), format!("{from_dir}{name}")))
    } else if pattern.ends_with('/') {
        let rest = path.strip_prefix(pattern)?;
        Some((pattern.len(), format!("{from}{rest}")))
    } else {
        (path == pattern).then(|| (usize::MAX, from.to_owned()))
    }
}

/// Where a writer's change to the mirror's `path` goes in the assets
/// repository: `Ok` the path there; `Err(true)` when the mirror holds it but
/// doesn't take it back (a proposal for Nick), `Err(false)` when the mirror
/// doesn't hold it at all.
pub fn assets_path(entries: &[Entry], path: &str) -> Result<String, bool> {
    let best = entries
        .iter()
        .filter_map(|e| holds(&e.mirror, &e.from, path).map(|(n, to)| (n, e, to)))
        .max_by_key(|(n, _, _)| *n);
    match best {
        Some((_, entry, to)) if entry.import => Ok(to[CHECKOUT.len()..].to_owned()),
        Some(_) => Err(true),
        None => Err(false),
    }
}

/// Each of `paths` in the assets repository, or why the import refuses.
pub fn check_paths(entries: &[Entry], paths: &[String]) -> Result<Vec<(String, String)>, String> {
    let (mut kept, mut proposals, mut strangers) = (Vec::new(), Vec::new(), Vec::new());
    for path in paths {
        match assets_path(entries, path) {
            Ok(to) => kept.push((path.clone(), to)),
            Err(true) => proposals.push(path.as_str()),
            Err(false) => strangers.push(path.as_str()),
        }
    }
    let mut why = String::new();
    if !proposals.is_empty() {
        let _ = writeln!(
            why,
            "these files are decisions, not the writer's to change: a proposal for Nick, \
             who decides it. Ask the writer to take them out of the pull request (and \
             say what they propose in it): {}",
            proposals.join(", ")
        );
    }
    if !strangers.is_empty() {
        let _ = writeln!(
            why,
            "these files aren't any the mirror holds ({FILES}): {}",
            strangers.join(", ")
        );
    }
    if why.is_empty() {
        Ok(kept)
    } else {
        Err(why.trim_end().to_owned())
    }
}

/// The value of the trailer `key` in a commit message.
pub fn trailer<'a>(message: &'a str, key: &str) -> Option<&'a str> {
    message.lines().rev().find_map(|line| {
        let value = line.trim().strip_prefix(key)?.strip_prefix(':')?.trim();
        (!value.is_empty()).then_some(value)
    })
}

/// A mirror pull request the import may take.
#[derive(Debug, PartialEq, Eq)]
pub struct Pull {
    pub number: u64,
    pub title: String,
    pub login: String,
    pub files: Vec<String>,
}

/// `gh pr view --json number,title,state,author,latestReviews,files`'s
/// answer, as a pull request the import may take, or why not: it must be
/// open and approved by [`OWNER`]'s latest review.
pub fn check_pull(json: &str) -> Result<Pull, String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("gh's answer: {e}"))?;
    let text = |key: &str| v[key].as_str().unwrap_or_default().to_owned();
    let number = v["number"].as_u64().ok_or("gh's answer has no number")?;
    if text("state") != "OPEN" {
        return Err(format!(
            "pull request {number} is {}, not open",
            text("state")
        ));
    }
    let reviews = v["latestReviews"].as_array().cloned().unwrap_or_default();
    let owner = reviews
        .iter()
        .filter(|r| r["author"]["login"].as_str() == Some(OWNER))
        .filter_map(|r| r["state"].as_str())
        .next_back();
    if owner != Some("APPROVED") {
        let state = owner.map_or("no review".to_owned(), str::to_lowercase);
        return Err(format!(
            "pull request {number} isn't approved by {OWNER} ({state}): Nick approves it on \
             GitHub first"
        ));
    }
    let files = v["files"]
        .as_array()
        .map(|f| {
            f.iter()
                .filter_map(|f| f["path"].as_str().map(str::to_owned))
                .collect()
        })
        .unwrap_or_default();
    Ok(Pull {
        number,
        title: text("title"),
        login: v["author"]["login"].as_str().unwrap_or_default().to_owned(),
        files,
    })
}

/// The writer as a commit's author: their name (or login) and GitHub's
/// noreply address, from `gh api users/<login>`'s answer.
pub fn author(json: &str) -> Result<(String, String), String> {
    let v: Value = serde_json::from_str(json).map_err(|e| format!("gh's answer: {e}"))?;
    let login = v["login"].as_str().ok_or("gh's answer has no login")?;
    let id = v["id"].as_u64().ok_or("gh's answer has no id")?;
    let name = v["name"]
        .as_str()
        .filter(|n| !n.is_empty())
        .unwrap_or(login);
    Ok((
        name.to_owned(),
        format!("{id}+{login}@users.noreply.github.com"),
    ))
}

/// `story/<n>-<slug>`: the branch for pull request `n` titled `title`.
pub fn branch(n: u64, title: &str) -> String {
    let mut slug = String::new();
    for c in title.chars().flat_map(char::to_lowercase) {
        if c.is_ascii_alphanumeric() {
            slug.push(c);
        } else if !slug.is_empty() && !slug.ends_with('-') {
            slug.push('-');
        }
    }
    let slug: String = slug.chars().take(40).collect();
    format!("story/{n}-{}", slug.trim_end_matches('-'))
}

/// `patch` (a `git diff` of the mirror) with every mirror path as its
/// path in the assets repository, `paths` mapping one to the other.
pub fn map_patch(patch: &str, paths: &[(String, String)]) -> Result<String, String> {
    let to = |path: &str| {
        paths
            .iter()
            .find(|(from, _)| from == path)
            .map(|(_, to)| to.clone())
            .ok_or_else(|| format!("the diff names {path}, which the check didn't see"))
    };
    let mut out = String::new();
    // `---` and `+++` name files only in a file's header, before its first
    // hunk: in a hunk they are lines of the script.
    let mut header = false;
    for line in patch.split_inclusive('\n') {
        let body = line.trim_end_matches('\n');
        if let Some(rest) = body.strip_prefix("diff --git a/") {
            let (a, b) = rest.split_once(" b/").ok_or("a path with \" b/\" in it")?;
            let _ = writeln!(out, "diff --git a/{} b/{}", to(a)?, to(b)?);
            header = true;
        } else if let (true, Some(path)) = (header, body.strip_prefix("--- a/")) {
            let _ = writeln!(out, "--- a/{}", to(path)?);
        } else if let (true, Some(path)) = (header, body.strip_prefix("+++ b/")) {
            let _ = writeln!(out, "+++ b/{}", to(path)?);
        } else {
            header &= !body.starts_with("@@") && !body.starts_with("GIT binary patch");
            out.push_str(line);
        }
    }
    Ok(out)
}

/// Runs `git` in `dir` with `args` and `env`; its stdout, or its stderr.
fn git(dir: &Path, args: &[&str], env: &[(&str, &str)]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .envs(env.iter().copied())
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let err = String::from_utf8_lossy(&output.stderr);
        Err(format!("git {}: {}", args.join(" "), err.trim()))
    }
}

/// What [`apply`] made: the commit, on top of the assets repository's
/// `origin/main`, and its diff.
pub struct Applied {
    pub commit: String,
    pub diff: String,
}

/// A file deleted on drop.
struct Scratch(PathBuf);

impl Drop for Scratch {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

/// Fetches pull request `pull` of the mirror at `mirror_url` into the
/// assets repository's checkout `assets`, and commits its change on top of
/// `origin/main` with `author` (name, email): three-way, on a scratch
/// index. The commit is on no branch yet.
pub fn apply(
    assets: &Path,
    mirror_url: &str,
    pull: &Pull,
    author: &(String, String),
    entries: &[Entry],
) -> Result<Applied, String> {
    let n = pull.number;
    let (main, head) = (
        "refs/story-mirror/main",
        format!("refs/story-mirror/pull/{n}"),
    );
    git(assets, &["fetch", "-q", "origin", "main"], &[])?;
    let refspecs = [
        format!("+refs/heads/main:{main}"),
        format!("+refs/pull/{n}/head:{head}"),
    ];
    git(
        assets,
        &["fetch", "-q", mirror_url, &refspecs[0], &refspecs[1]],
        &[],
    )?;
    let base = git(assets, &["merge-base", main, &head], &[])?
        .trim()
        .to_owned();
    let message = git(assets, &["log", "-1", "--format=%B", &base], &[])?;
    let pin = trailer(&message, "Assets-Commit").ok_or_else(|| {
        format!("pull request {n} doesn't start from a commit the sync made (no Assets-Commit trailer on {base})")
    })?;
    let names = git(
        assets,
        &["diff", "--no-renames", "--name-only", &base, &head],
        &[],
    )?;
    let names: Vec<String> = names.lines().map(str::to_owned).collect();
    if names.is_empty() {
        return Err(format!("pull request {n} changes nothing"));
    }
    let paths = check_paths(entries, &names)?;
    let args = [
        "diff",
        "--no-renames",
        "--full-index",
        "--binary",
        &base,
        &head,
    ];
    let patch = map_patch(&git(assets, &args, &[])?, &paths)?;

    let tag = format!("story-import-{}-{n}", std::process::id());
    let patch_file = Scratch(std::env::temp_dir().join(format!("{tag}.patch")));
    fs::write(&patch_file.0, &patch).map_err(|e| format!("can't write the patch: {e}"))?;
    let index = Scratch(std::env::temp_dir().join(format!("{tag}.index")));
    let index_path = index.0.to_string_lossy().into_owned();
    let env = [("GIT_INDEX_FILE", index_path.as_str())];
    git(assets, &["read-tree", "origin/main"], &env)?;
    let patch_path = patch_file.0.to_string_lossy().into_owned();
    if let Err(e) = git(assets, &["apply", "--cached", "--3way", &patch_path], &env) {
        let conflicts = git(assets, &["diff", "--name-only", "--diff-filter=U"], &env)
            .unwrap_or_default()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(", ");
        return Err(if conflicts.is_empty() {
            format!("the change doesn't apply onto origin/main: {e}")
        } else {
            format!(
                "the change conflicts with origin/main in {conflicts}: the writer's lines and \
                 a later change touch the same place. Ask the writer to update the pull \
                 request from the mirror's main (it has the later change) and approve it again"
            )
        });
    }
    let tree = git(assets, &["write-tree"], &env)?.trim().to_owned();
    let mut msg = format!(
        "Story: {}\n\n{MIRROR}#{n}, from the mirror at {base}",
        pull.title
    );
    let _ = write!(msg, " (assets {pin}).\n\nStory-PR: {n}\n");
    let author_env = [
        ("GIT_AUTHOR_NAME", author.0.as_str()),
        ("GIT_AUTHOR_EMAIL", author.1.as_str()),
    ];
    let commit = Command::new("git")
        .arg("-C")
        .arg(assets)
        .args(["commit-tree", &tree, "-p", "origin/main", "-m", &msg])
        .envs(author_env)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if !commit.status.success() {
        return Err(format!(
            "git commit-tree: {}",
            String::from_utf8_lossy(&commit.stderr).trim()
        ));
    }
    let commit = String::from_utf8_lossy(&commit.stdout).trim().to_owned();
    let diff = git(
        assets,
        &["diff", "--stat", "-p", "origin/main", &commit],
        &[],
    )?;
    Ok(Applied { commit, diff })
}

/// Runs `gh` with `args`: its stdout, or its stderr as the error.
pub fn gh(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|e| format!("cannot run gh: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let said = String::from_utf8_lossy(&output.stderr);
        Err(format!("gh {}: {}", args.join(" "), said.trim()))
    }
}

/// The pull request's body in the assets repository.
fn body(n: u64, login: &str) -> String {
    format!(
        "The writer's change from {MIRROR}#{n} (by {login}), approved by Nick on the mirror, \
         taken in by `cargo xtask story-import` (ticket 0152).\n\nMerges first; then the \
         game's pull request moves `assets-private.rev` here (the `story-import` skill), and \
         its merge closes the mirror's pull request.\n"
    )
}

/// The command: `args` after `story-import`, run in the game's folder
/// `root` with `gh` answering GitHub's questions. Prints as it goes.
pub fn command(root: &Path, args: &[String], gh: &dyn Fn(&[&str]) -> Result<String, String>) -> u8 {
    let dry_run = args.iter().any(|a| a == "--dry-run");
    let numbers: Vec<&String> = args.iter().filter(|a| *a != "--dry-run").collect();
    let n = match numbers.as_slice() {
        [n] if n.parse::<u64>().is_ok() => n.as_str(),
        _ => {
            eprintln!("{USAGE}");
            return 2;
        }
    };
    match run(root, n, dry_run, gh) {
        Ok(()) => 0,
        Err(e) => {
            eprintln!("story-import: {e}");
            1
        }
    }
}

fn run(
    root: &Path,
    n: &str,
    dry_run: bool,
    gh: &dyn Fn(&[&str]) -> Result<String, String>,
) -> Result<(), String> {
    let fields = "number,title,state,author,latestReviews,files";
    let pull = check_pull(&gh(&["pr", "view", n, "-R", MIRROR, "--json", fields])?)?;
    let entries = read_files(
        &fs::read_to_string(root.join(FILES)).map_err(|e| format!("can't read {FILES}: {e}"))?,
    )?;
    check_paths(&entries, &pull.files)?;
    let assets = root.join(CHECKOUT);
    if !assets.join(".git").exists() {
        return Err("no assets-private/ here: run `cargo shuyi private-assets` first".to_owned());
    }
    let author = author(&gh(&["api", &format!("users/{}", pull.login)])?)?;
    let url = format!("https://github.com/{MIRROR}.git");
    let applied = apply(&assets, &url, &pull, &author, &entries)?;
    println!("{}", applied.diff);
    if dry_run {
        println!("dry run: commit {} made, nothing pushed", applied.commit);
        return Ok(());
    }
    let branch = branch(pull.number, &pull.title);
    let refspec = format!("{}:refs/heads/{branch}", applied.commit);
    git(&assets, &["push", "-q", "origin", &refspec], &[])?;
    let title = format!("[story#{}] {}", pull.number, pull.title);
    let body = body(pull.number, &pull.login);
    let args = [
        "pr", "create", "-R", ASSETS, "--base", "main", "--head", &branch, "--title", &title,
        "--body", &body,
    ];
    println!("{}", gh(&args)?.trim());
    println!(
        "next: the story-import skill (the gates on the branch's commit, the game's pull \
         request, the pin after the merge)"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn entries() -> Vec<Entry> {
        let text = fs::read_to_string(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../..")
                .join(FILES),
        );
        read_files(&text.unwrap()).unwrap()
    }

    #[test]
    fn the_files_list_reads() {
        let entries = entries();
        let first = &entries[0];
        assert_eq!(first.mirror, "dialogue/*.dlg");
        assert_eq!(first.from, "assets-private/content/dialogue/*.dlg");
        assert!(first.import);
        assert!(entries.iter().any(|e| e.mirror == "README.md" && !e.import));
        assert!(read_files("a b c d").unwrap_err().contains(":1: expected"));
        assert!(read_files("# a\n\na b maybe").is_err());
        assert!(
            read_files("a docs/a import")
                .unwrap_err()
                .contains("only a path")
        );
    }

    #[test]
    fn mirror_paths_map_to_the_assets_repository() {
        let entries = entries();
        let map = |p: &str| assets_path(&entries, p);
        assert_eq!(
            map("dialogue/ch01.dlg"),
            Ok("content/dialogue/ch01.dlg".to_owned())
        );
        assert_eq!(map("story/outline.md"), Ok("story/outline.md".to_owned()));
        assert_eq!(
            map("story/chapters/ch02.md"),
            Ok("story/chapters/ch02.md".to_owned())
        );
        for proposal in [
            "story/design/audio.md",
            "design/audio.md",
            // The format's copy, spelled so that xtask doesn't read as a
            // reader of `assets/dialogue/README.md` (changes.yml's lines).
            concat!("dialogue/", "README", ".md"),
            "README.md",
        ] {
            assert_eq!(map(proposal), Err(true), "{proposal}");
        }
        for stranger in [
            "dialogue/sub/x.dlg",
            "dialogue/.dlg",
            "dialogue/x.txt",
            "notes.md",
        ] {
            assert_eq!(map(stranger), Err(false), "{stranger}");
        }
        let paths = ["dialogue/ch01.dlg", "design/audio.md", "x.md"].map(str::to_owned);
        let why = check_paths(&entries, &paths).unwrap_err();
        assert!(
            why.contains("a proposal for Nick, who decides it."),
            "{why}"
        );
        assert!(why.contains("in it): design/audio.md\n"), "{why}");
        assert!(
            why.ends_with("holds (tools/story-mirror/files.txt): x.md"),
            "{why}"
        );
    }

    #[test]
    fn trailers_are_read_from_the_end() {
        let message = "Game 1234567: x\n\nGame-Commit: abc\nAssets-Commit:  def \n";
        assert_eq!(trailer(message, "Assets-Commit"), Some("def"));
        assert_eq!(trailer(message, "Game-Commit"), Some("abc"));
        assert_eq!(trailer(message, "Story-PR"), None);
        assert_eq!(trailer("Assets-Commit:\n", "Assets-Commit"), None);
    }

    /// `gh pr view`'s answer for a pull request reviewed by `reviews`
    /// (login, state) changing `files`.
    fn pull_json(state: &str, reviews: &[(&str, &str)], files: &[&str]) -> String {
        let reviews: Vec<Value> = reviews
            .iter()
            .map(|(login, state)| serde_json::json!({ "author": { "login": login }, "state": state }))
            .collect();
        let files: Vec<Value> = files
            .iter()
            .map(|p| serde_json::json!({ "path": p }))
            .collect();
        serde_json::json!({
            "number": 7, "title": "Warmer camp scene", "state": state,
            "author": { "login": "writer" }, "latestReviews": reviews, "files": files,
        })
        .to_string()
    }

    #[test]
    fn only_an_open_pull_request_nick_approved_is_taken() {
        let ok = check_pull(&pull_json(
            "OPEN",
            &[("writer", "COMMENTED"), (OWNER, "APPROVED")],
            &["dialogue/ch01.dlg"],
        ));
        assert_eq!(
            ok,
            Ok(Pull {
                number: 7,
                title: "Warmer camp scene".to_owned(),
                login: "writer".to_owned(),
                files: vec!["dialogue/ch01.dlg".to_owned()],
            })
        );
        let none = check_pull(&pull_json("OPEN", &[], &[])).unwrap_err();
        assert!(
            none.contains("isn't approved by Zafnok (no review)"),
            "{none}"
        );
        let changes =
            check_pull(&pull_json("OPEN", &[(OWNER, "CHANGES_REQUESTED")], &[])).unwrap_err();
        assert!(changes.contains("(changes_requested)"), "{changes}");
        let other = check_pull(&pull_json("OPEN", &[("someone", "APPROVED")], &[])).unwrap_err();
        assert!(other.contains("(no review)"), "{other}");
        let closed = check_pull(&pull_json("MERGED", &[(OWNER, "APPROVED")], &[])).unwrap_err();
        assert_eq!(closed, "pull request 7 is MERGED, not open");
        assert!(check_pull("not json").is_err());
    }

    #[test]
    fn an_approved_change_to_a_decision_is_refused_by_the_command() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        for file in ["design/audio.md", "story/design/setting-and-tone.md"] {
            let json = pull_json("OPEN", &[(OWNER, "APPROVED")], &["dialogue/ch01.dlg", file]);
            let gh = |args: &[&str]| {
                assert_eq!(args[..2], ["pr", "view"]);
                Ok(json.clone())
            };
            let err = run(&root, "7", true, &gh).unwrap_err();
            assert!(err.contains(&format!("in it): {file}")), "{err}");
        }
        let gh = |_: &[&str]| Ok(pull_json("OPEN", &[], &["dialogue/ch01.dlg"]));
        assert!(
            run(&root, "7", true, &gh)
                .unwrap_err()
                .contains("isn't approved")
        );
        assert_eq!(command(&root, &["x".to_owned()], &gh), 2);
        assert_eq!(command(&root, &["7".to_owned()], &gh), 1);
    }

    #[test]
    fn the_author_is_the_writer_at_githubs_noreply_address() {
        let json = r#"{"login":"inkwell","id":42,"name":"Ada Ink"}"#;
        assert_eq!(
            author(json),
            Ok((
                "Ada Ink".to_owned(),
                "42+inkwell@users.noreply.github.com".to_owned()
            ))
        );
        let json = r#"{"login":"inkwell","id":42,"name":null}"#;
        assert_eq!(author(json).unwrap().0, "inkwell");
        let said = body(12, "inkwell");
        assert!(
            said.contains("Zafnok/visions-of-shuyi-story#12 (by inkwell)"),
            "{said}"
        );
        assert_eq!(
            branch(12, "Warmer camp: Joss's lines!"),
            "story/12-warmer-camp-joss-s-lines"
        );
    }

    #[test]
    fn the_patch_s_paths_are_the_assets_repository_s() {
        let patch = "diff --git a/dialogue/ch01.dlg b/dialogue/ch01.dlg\nindex 1..2 100644\n\
                     --- a/dialogue/ch01.dlg\n+++ b/dialogue/ch01.dlg\n@@ -1 +1 @@\n--- a/x\n+b\n\
                     diff --git a/story/new.md b/story/new.md\n--- /dev/null\n+++ b/story/new.md\n";
        let paths = [
            (
                "dialogue/ch01.dlg".to_owned(),
                "content/dialogue/ch01.dlg".to_owned(),
            ),
            ("story/new.md".to_owned(), "story/new.md".to_owned()),
        ];
        let mapped = map_patch(patch, &paths).unwrap();
        assert!(mapped.starts_with(
            "diff --git a/content/dialogue/ch01.dlg b/content/dialogue/ch01.dlg\nindex 1..2 100644\n\
             --- a/content/dialogue/ch01.dlg\n+++ b/content/dialogue/ch01.dlg\n"
        ));
        assert!(mapped.contains("--- /dev/null\n+++ b/story/new.md\n"));
        let stranger = "diff --git a/x.md b/x.md\n--- a/x.md\n";
        assert!(map_patch(stranger, &paths).is_err());
    }

    /// `git` in `dir`, which must succeed.
    fn sh(dir: &Path, args: &[&str]) -> String {
        git(dir, args, &[]).unwrap()
    }

    /// `dir` as a new repository on `main`, with an author and LF files.
    fn init(dir: &Path) {
        sh(dir, &["init", "-q", "-b", "main"]);
        sh(dir, &["config", "user.email", "test@example.com"]);
        sh(dir, &["config", "user.name", "Test"]);
        sh(dir, &["config", "core.autocrlf", "false"]);
    }

    fn write(dir: &Path, path: &str, text: &str) {
        let file = dir.join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, text).unwrap();
    }

    /// The script the sync copied, in both throwaway repositories.
    const SCRIPT: &str = "one\ntwo\nthree\nfour\nfive\n";

    /// Throwaway repositories under `root`: the assets repository's
    /// `origin` with [`SCRIPT`], its clone `assets`, and a `mirror` whose
    /// sync commit names that pin and whose pull request 7 changes line 2.
    fn throwaway(root: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let _ = fs::remove_dir_all(root);
        let (origin, assets, mirror) = (
            root.join("origin"),
            root.join("assets"),
            root.join("mirror"),
        );
        for dir in [&origin, &mirror] {
            fs::create_dir_all(dir).unwrap();
        }
        init(&origin);
        write(&origin, "content/dialogue/ch01.dlg", SCRIPT);
        sh(&origin, &["add", "."]);
        sh(&origin, &["commit", "-q", "-m", "first"]);
        let pin = sh(&origin, &["rev-parse", "HEAD"]).trim().to_owned();
        let from = origin.to_str().unwrap();
        sh(
            root,
            &["clone", "-q", "-c", "core.autocrlf=false", from, "assets"],
        );
        init(&assets);

        init(&mirror);
        write(&mirror, "dialogue/ch01.dlg", SCRIPT);
        sh(&mirror, &["add", "."]);
        let sync = format!("Game 1234567: x\n\nGame-Commit: 1234567\nAssets-Commit: {pin}\n");
        sh(&mirror, &["commit", "-q", "-m", &sync]);
        sh(&mirror, &["switch", "-q", "-c", "writer"]);
        write(
            &mirror,
            "dialogue/ch01.dlg",
            "one\nTWO, warmer\nthree\nfour\nfive\n",
        );
        sh(&mirror, &["commit", "-q", "-am", "warmer"]);
        sh(&mirror, &["update-ref", "refs/pull/7/head", "HEAD"]);
        sh(&mirror, &["switch", "-q", "main"]);
        (origin, assets, mirror)
    }

    /// The writer changed line 2 of a script the sync copied; the assets
    /// repository then changed line 5. The import holds both; a change of
    /// the writer's line on the assets side is a conflict, named.
    #[test]
    fn an_approved_change_applies_three_way_onto_a_file_that_moved_on() {
        let root = std::env::temp_dir().join(format!("trpg-story-import-{}", std::process::id()));
        let (origin, assets, mirror) = throwaway(&root);
        let run = sh;
        let script = SCRIPT;

        // The file moves on since the pin.
        write(
            &origin,
            "content/dialogue/ch01.dlg",
            "one\ntwo\nthree\nfour\nFIVE\n",
        );
        run(&origin, &["commit", "-q", "-am", "later"]);

        let pull = Pull {
            number: 7,
            title: "Warmer".to_owned(),
            login: "writer".to_owned(),
            files: vec![],
        };
        let who = (
            "Ada Ink".to_owned(),
            "42+inkwell@users.noreply.github.com".to_owned(),
        );
        let url = mirror.to_string_lossy().into_owned();
        let applied = apply(&assets, &url, &pull, &who, &entries()).unwrap();
        let show = |what: &str| run(&assets, &["show", "-s", what, &applied.commit]);
        assert_eq!(
            run(
                &assets,
                &[
                    "show",
                    &format!("{}:content/dialogue/ch01.dlg", applied.commit)
                ]
            ),
            "one\nTWO, warmer\nthree\nfour\nFIVE\n"
        );
        assert_eq!(
            show("--format=%an <%ae>").trim(),
            "Ada Ink <42+inkwell@users.noreply.github.com>"
        );
        let message = show("--format=%B");
        assert!(message.starts_with("Story: Warmer\n"), "{message}");
        assert_eq!(trailer(&message, "Story-PR"), Some("7"));
        let parent = run(&assets, &["rev-parse", "origin/main"]);
        assert_eq!(
            run(&assets, &["rev-parse", &format!("{}^", applied.commit)]),
            parent
        );
        assert!(
            applied.diff.contains("content/dialogue/ch01.dlg"),
            "{}",
            applied.diff
        );
        // The checkout itself is as it was.
        assert_eq!(run(&assets, &["status", "--porcelain"]), "");
        assert_eq!(
            fs::read_to_string(assets.join("content/dialogue/ch01.dlg")).unwrap(),
            script
        );

        // The same line changed on both sides: a conflict, named.
        write(
            &origin,
            "content/dialogue/ch01.dlg",
            "one\nTWO, colder\nthree\nfour\nFIVE\n",
        );
        run(&origin, &["commit", "-q", "-am", "clash"]);
        let err = apply(&assets, &url, &pull, &who, &entries()).err().unwrap();
        assert!(
            err.contains("conflicts with origin/main in content/dialogue/ch01.dlg"),
            "{err}"
        );

        // A pull request that doesn't start from a sync commit.
        run(&mirror, &["switch", "-q", "--orphan", "stray"]);
        write(&mirror, "dialogue/ch01.dlg", "x\n");
        run(&mirror, &["add", "."]);
        run(&mirror, &["commit", "-q", "-m", "stray"]);
        run(&mirror, &["update-ref", "refs/pull/7/head", "HEAD"]);
        assert!(apply(&assets, &url, &pull, &who, &entries()).is_err());
        fs::remove_dir_all(&root).unwrap();
    }
}
