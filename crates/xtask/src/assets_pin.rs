// engine-friction: engine#3445, moves (game 1402 deletes it)
//! `assets-pin` (tickets 0194 and 0195, ADR-0135): the pin of the private
//! repository.
//!
//! `assets-private.rev` is the **base**: it only ever moves forward, and
//! only to a commit on the private `main`; `private-land.yml` moves it
//! when it folds the landed changes. A change in flight adds its **own pin
//! file**, `assets-private.d/<branch>.rev`, naming the head of its private
//! branch (`--add`), so two changes never edit the same line. The
//! **effective pin**, what every build reads, is the base merged with each
//! pin file in file-name order (`resolve.sh`, run by CI's private-assets
//! action and by `--resolve`).
//!
//! `--check` reads the private repository's history from the checkout in
//! `assets-private/` (CI checks out its whole history for this).

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use crate::engine::parse_rev;

pub const USAGE: &str = "usage: cargo xtask assets-pin --check [--base <ref>] | --add | --resolve\n\n\
--check    fail when assets-private.rev moves back, sideways or off the private repository's main\n           \
from its value at <ref> (default origin/main), or when a pin file in assets-private.d/ isn't\n           \
its branch's or is already in the base (tickets 0194, 0195)\n\
--add      write the private checkout's branch head into assets-private.d/<branch>.rev, this\n           \
change's own pin (ADR-0135)\n\
--resolve  put assets-private/ at the effective pin, the base merged with every pin file";

/// The file that names the base pin, and the checkout it names a commit of.
const PIN_FILE: &str = "assets-private.rev";
const CHECKOUT: &str = "assets-private";
/// The folder of the changes' own pin files.
pub const PIN_DIR: &str = "assets-private.d";
/// The private repository's `main`, as the checkout knows it.
const MAIN: &str = "origin/main";
/// The script that merges the pin files, shared with CI.
const RESOLVE: &str = ".github/actions/private-assets/resolve.sh";

/// What the private repository's history says of commits and branches.
pub trait History {
    /// Whether `old` is `new` or one of its ancestors; `None` when either
    /// commit isn't in the history.
    fn is_ancestor(&self, old: &str, new: &str) -> Option<bool>;
    /// Whether `commit` is on the private repository's `main`.
    fn on_main(&self, commit: &str) -> bool;
    /// The commit the private branch `name` is at on `origin`, if there is
    /// one.
    fn branch_head(&self, name: &str) -> Option<String>;
}

/// The verdict on moving the base pin from `old` to `new`: the warnings to
/// print, or why the move is refused.
pub fn judge(old: &str, new: &str, history: &impl History) -> Result<Vec<String>, String> {
    if old == new {
        return Ok(Vec::new());
    }
    if history.is_ancestor(new, new).is_none() {
        return Err(format!(
            "{PIN_FILE} names {new}, which is not in the private repository's history"
        ));
    }
    if !history.on_main(new) {
        return Err(format!(
            "{PIN_FILE} names {new}, which is not on the private repository's main. A \
             change in flight never edits {PIN_FILE}: undo that, then give the change its \
             own pin file with `cargo xtask assets-pin --add` (ADR-0135, the \
             content-change skill)"
        ));
    }
    if !history.on_main(old) {
        // The base broke the rule above; its commit may be gone or squashed
        // into another, so ancestry can't tell forward from back.
        return Ok(vec![format!(
            "the base's pin {old} is not on the private repository's main, so the \
             direction of this move can't be checked: taken as a repair"
        )]);
    }
    match history.is_ancestor(old, new) {
        Some(true) => Ok(Vec::new()),
        _ => Err(format!(
            "{PIN_FILE} moves back: {new} does not come after the base's pin {old} in \
             the private repository's history, so main would lose the content between \
             them (ticket 0194). Undo the change to {PIN_FILE} and use a pin file \
             (`cargo xtask assets-pin --add`, ADR-0135)"
        )),
    }
}

/// The verdict on the pin file `assets-private.d/<name>.rev`, naming
/// `commit`, beside the base pin `base`: the warnings to print, or why it
/// is refused.
pub fn judge_pin(
    base: &str,
    name: &str,
    commit: &str,
    history: &impl History,
) -> Result<Vec<String>, String> {
    let file = format!("{PIN_DIR}/{name}.rev");
    match history.is_ancestor(commit, base) {
        None => {
            return Err(format!(
                "{file} names {commit}, which is not in the private repository's history: \
                 push its branch there first (the content-change skill)"
            ));
        }
        Some(true) => {
            return Err(format!(
                "{file} names {commit}, which the base pin {base} already holds: delete \
                 the file (private-land.yml folds a landed pin; this one was left behind)"
            ));
        }
        Some(false) => {}
    }
    match history.branch_head(name) {
        None if history.on_main(commit) => Ok(vec![format!(
            "{file}: its branch is gone and {commit} is on the private main, so it has \
             landed; private-land.yml folds it into {PIN_FILE}"
        )]),
        None => Err(format!(
            "{file}: there is no branch {name} in the private repository. A pin file is \
             named after its private branch, which private-land.yml finds the pull request \
             by: push the branch, or rename the file after it"
        )),
        Some(head) if head != commit => Ok(vec![format!(
            "{file} names {commit}, but the branch {name} is at {head}. CI tests the pinned \
             commit and private-land.yml refuses to land a branch that moved past it: run \
             `cargo xtask assets-pin --add` again and commit the file"
        )]),
        Some(_) => Ok(Vec::new()),
    }
}

/// The pin files in `root`, by file name: each one's branch (the file's
/// name without `.rev`) and the commit it names.
pub fn read_pins(root: &Path) -> Result<Vec<(String, String)>, String> {
    let dir = root.join(PIN_DIR);
    let Ok(entries) = std::fs::read_dir(&dir) else {
        return Ok(Vec::new());
    };
    let mut files: Vec<String> = entries
        .flatten()
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .collect();
    files.sort_unstable();
    files
        .into_iter()
        .map(|file| {
            let Some(name) = file.strip_suffix(".rev").filter(|n| !n.is_empty()) else {
                return Err(format!(
                    "{PIN_DIR}/ holds only <branch>.rev pin files, not `{file}` (ADR-0135)"
                ));
            };
            let text = std::fs::read_to_string(dir.join(&file))
                .map_err(|e| format!("read {PIN_DIR}/{file}: {e}"))?;
            let commit = parse_rev(&text).map_err(|e| format!("{PIN_DIR}/{file}: {e}"))?;
            Ok((name.to_owned(), commit))
        })
        .collect()
}

/// The history of a git checkout.
pub struct Git<'a>(pub &'a Path);

impl Git<'_> {
    fn command(&self, args: &[&str]) -> Command {
        let mut command = Command::new("git");
        command
            .args(args)
            .current_dir(self.0)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes");
        command
    }

    /// Whether `git <args>` succeeds in the checkout (`None` when git
    /// itself couldn't run), never asking for a password.
    fn succeeds(&self, args: &[&str]) -> Option<bool> {
        self.command(args)
            .output()
            .ok()
            .map(|out| out.status.success())
    }

    /// What `git <args>` prints, trimmed, or why it failed.
    fn read(&self, args: &[&str]) -> Result<String, String> {
        let out = self
            .command(args)
            .output()
            .map_err(|e| format!("spawn git: {e}"))?;
        if out.status.success() {
            Ok(String::from_utf8_lossy(&out.stdout).trim().to_owned())
        } else {
            Err(format!(
                "git {}: {}",
                args.join(" "),
                String::from_utf8_lossy(&out.stderr).trim()
            ))
        }
    }

    fn has(&self, commit: &str) -> bool {
        self.succeeds(&["cat-file", "-e", &format!("{commit}^{{commit}}")]) == Some(true)
    }
}

impl History for Git<'_> {
    fn is_ancestor(&self, old: &str, new: &str) -> Option<bool> {
        if !self.has(old) || !self.has(new) {
            return None;
        }
        self.succeeds(&["merge-base", "--is-ancestor", old, new])
    }

    fn on_main(&self, commit: &str) -> bool {
        self.has(commit)
            && self.succeeds(&["merge-base", "--is-ancestor", commit, MAIN]) == Some(true)
    }

    fn branch_head(&self, name: &str) -> Option<String> {
        self.read(&[
            "rev-parse",
            "--verify",
            "-q",
            &format!("refs/remotes/origin/{name}^{{commit}}"),
        ])
        .ok()
        .filter(|head| !head.is_empty())
    }
}

/// The base pin in `root`'s working tree.
fn read_base(root: &Path) -> Result<String, String> {
    std::fs::read_to_string(root.join(PIN_FILE))
        .map_err(|e| format!("read {PIN_FILE}: {e}"))
        .and_then(|text| parse_rev(&text))
}

/// The private checkout in `root`, or the command that makes it.
fn checkout(root: &Path) -> Result<PathBuf, String> {
    let checkout = root.join(CHECKOUT);
    if checkout.join(".git").exists() {
        Ok(checkout)
    } else {
        Err("this needs the private repository: run `cargo shuyi private-assets` first".to_owned())
    }
}

/// Checks the base pin in `root`'s working tree against its value at
/// `base`, and every pin file beside it: what to print, or why it fails.
pub fn check(root: &Path, base: &str) -> Result<String, String> {
    let new = read_base(root)?;
    let shown = Command::new("git")
        .args(["show", &format!("{base}:{PIN_FILE}")])
        .current_dir(root)
        .output()
        .map_err(|e| format!("spawn git: {e}"))?;
    if !shown.status.success() {
        return Err(format!(
            "read {PIN_FILE} at {base}: {}",
            String::from_utf8_lossy(&shown.stderr).trim()
        ));
    }
    let old = parse_rev(&String::from_utf8_lossy(&shown.stdout))?;
    let pins = read_pins(root)?;
    if old == new && pins.is_empty() {
        return Ok(format!(
            "{PIN_FILE} is unchanged from {base}, and no pin files"
        ));
    }
    let checkout = checkout(root)?;
    let git = Git(&checkout);
    let missing = !git.has(&old)
        || !git.has(&new)
        || !git.on_main(&new)
        || pins.iter().any(|(_, commit)| !git.has(commit));
    if missing || !pins.is_empty() {
        // A local checkout may be behind, and branches move; CI's has the
        // whole history and no key left to fetch with, so this fails there
        // and changes nothing.
        let _ = git.succeeds(&["fetch", "-q", "origin"]);
    }
    let mut said = Vec::new();
    if old == new {
        said.push(format!("{PIN_FILE} is unchanged from {base}"));
    } else {
        let warnings = judge(&old, &new, &git)?;
        said.push(format!("{PIN_FILE} moves forward: {old} -> {new}"));
        said.extend(warnings.into_iter().map(|w| format!("warning: {w}")));
    }
    for (name, commit) in &pins {
        let warnings = judge_pin(&new, name, commit, &git)?;
        said.push(format!("{PIN_DIR}/{name}.rev pins {commit}"));
        said.extend(warnings.into_iter().map(|w| format!("warning: {w}")));
    }
    Ok(said.join("\n"))
}

/// Writes the private checkout's branch head into its pin file,
/// `assets-private.d/<branch>.rev`, once that commit is pushed.
pub fn add(root: &Path) -> Result<String, String> {
    let checkout = checkout(root)?;
    let git = Git(&checkout);
    let changes = git.read(&["status", "--porcelain"])?;
    if !changes.is_empty() {
        return Err(format!(
            "{CHECKOUT}/ has changes that aren't committed:\n{changes}\ncommit and push \
             them, then run this again"
        ));
    }
    let Ok(branch) = git.read(&["symbolic-ref", "-q", "--short", "HEAD"]) else {
        return Err(format!(
            "{CHECKOUT}/ is on no branch: switch to this change's private branch \
             (`git -C {CHECKOUT} switch <branch>`), then run this again"
        ));
    };
    if branch == "main" || branch.contains('/') || branch.is_empty() {
        return Err(format!(
            "{CHECKOUT}/ is on `{branch}`: a change's pin is the head of its own private \
             branch, named t<NNNN>-<slug> (the content-change skill)"
        ));
    }
    git.read(&["fetch", "-q", "origin"])
        .map_err(|e| format!("fetch the private repository: {e}"))?;
    let head = git.read(&["rev-parse", "HEAD"])?;
    if git.branch_head(&branch).as_deref() != Some(head.as_str()) {
        return Err(format!(
            "{head} isn't pushed as the private branch {branch}: push it first \
             (`git -C {CHECKOUT} push origin HEAD:refs/heads/{branch}`), then run this again"
        ));
    }
    let dir = root.join(PIN_DIR);
    std::fs::create_dir_all(&dir).map_err(|e| format!("make {PIN_DIR}/: {e}"))?;
    std::fs::write(dir.join(format!("{branch}.rev")), format!("{head}\n"))
        .map_err(|e| format!("write {PIN_DIR}/{branch}.rev: {e}"))?;
    Ok(format!(
        "{PIN_DIR}/{branch}.rev now names {head}: commit it with your change, and leave \
         {PIN_FILE} alone (ADR-0135)"
    ))
}

/// Puts the private checkout at the effective pin and says which it is.
pub fn resolve(root: &Path) -> Result<String, String> {
    let base = read_base(root)?;
    let pins: Vec<String> = read_pins(root)?.into_iter().map(|(_, c)| c).collect();
    let checkout = checkout(root)?;
    let _ = Git(&checkout).succeeds(&["fetch", "-q", "origin"]);
    let effective = run_resolve(&root.join(RESOLVE), &checkout, &base, &pins)?;
    Ok(if pins.is_empty() {
        format!("{CHECKOUT}/ is at the base pin {effective} (no pin files)")
    } else {
        format!(
            "{CHECKOUT}/ is at the effective pin {effective}: {base} with {}",
            pins.join(", ")
        )
    })
}

/// Runs `resolve.sh` from `script` on `checkout`: the effective commit, or
/// why the merge failed (its own words go to stderr as it runs).
pub fn run_resolve(
    script: &Path,
    checkout: &Path,
    base: &str,
    pins: &[String],
) -> Result<String, String> {
    let slashes = |p: &Path| p.to_string_lossy().replace('\\', "/");
    let out = bash()
        .arg(slashes(script))
        .arg(slashes(checkout))
        .arg(base)
        .args(pins)
        .stdin(Stdio::null())
        .stderr(Stdio::inherit())
        .output()
        .map_err(|e| format!("spawn bash for {RESOLVE}: {e}"))?;
    if out.status.success() {
        let stdout = String::from_utf8_lossy(&out.stdout);
        Ok(stdout.lines().last().unwrap_or("").trim().to_owned())
    } else {
        Err(format!("{RESOLVE} failed: the pins don't merge (above)"))
    }
}

/// Bash: Git's own on Windows, where a bare `bash` may be WSL's.
fn bash() -> Command {
    if cfg!(windows) {
        let exec_path = Command::new("git").arg("--exec-path").output();
        if let Ok(out) = exec_path {
            let dir = PathBuf::from(String::from_utf8_lossy(&out.stdout).trim());
            // <Git>/mingw64/libexec/git-core -> <Git>/bin/bash.exe
            if let Some(git) = dir.ancestors().nth(3) {
                let bash = git.join("bin").join("bash.exe");
                if bash.is_file() {
                    return Command::new(bash);
                }
            }
        }
    }
    Command::new("bash")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line of commits `a` → `b` → `c` on main, and `s`, a branch commit
    /// after `a` on the branch `t-s`; `t-moved` is at `s` too, and `t-gone`
    /// isn't there.
    struct Line;

    impl History for Line {
        fn is_ancestor(&self, old: &str, new: &str) -> Option<bool> {
            let after = |c: &str| match c {
                "a" => Some(vec!["a", "b", "c", "s"]),
                "b" => Some(vec!["b", "c"]),
                "c" => Some(vec!["c"]),
                "s" => Some(vec!["s"]),
                _ => None,
            };
            after(new)?;
            Some(after(old)?.contains(&new))
        }

        fn on_main(&self, commit: &str) -> bool {
            matches!(commit, "a" | "b" | "c")
        }

        fn branch_head(&self, name: &str) -> Option<String> {
            matches!(name, "t-s" | "t-moved").then(|| "s".to_owned())
        }
    }

    #[test]
    fn an_unchanged_pin_and_a_forward_move_pass_quietly() {
        assert_eq!(judge("b", "b", &Line), Ok(vec![]));
        assert_eq!(judge("a", "c", &Line), Ok(vec![]));
    }

    #[test]
    fn a_move_back_fails_and_says_the_fix() {
        let e = judge("c", "a", &Line).unwrap_err();
        assert!(e.contains("moves back"), "{e}");
        assert!(e.contains("assets-pin --add"), "{e}");
    }

    #[test]
    fn a_move_sideways_fails() {
        assert!(judge("b", "s", &Line).is_err());
    }

    /// Ticket 0195: a change in flight pins its branch in its own file.
    #[test]
    fn a_base_on_a_branch_commit_fails_and_says_use_a_pin_file() {
        let e = judge("a", "s", &Line).unwrap_err();
        assert!(e.contains("not on the private repository's main"), "{e}");
        assert!(e.contains("assets-pin --add"), "{e}");
    }

    #[test]
    fn a_base_off_main_is_a_repair_that_warns() {
        let warnings = judge("s", "c", &Line).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("taken as a repair"), "{warnings:?}");
    }

    #[test]
    fn an_unknown_new_pin_fails() {
        let e = judge("a", "x", &Line).unwrap_err();
        assert!(e.contains("not in the private repository's history"), "{e}");
    }

    #[test]
    fn a_pin_file_at_its_branch_head_passes_quietly() {
        assert_eq!(judge_pin("c", "t-s", "s", &Line), Ok(vec![]));
    }

    #[test]
    fn a_pin_file_behind_its_branch_warns() {
        struct Moved;
        impl History for Moved {
            fn is_ancestor(&self, old: &str, new: &str) -> Option<bool> {
                Line.is_ancestor(old, new)
            }
            fn on_main(&self, commit: &str) -> bool {
                Line.on_main(commit)
            }
            fn branch_head(&self, _: &str) -> Option<String> {
                Some("later".to_owned())
            }
        }
        let warnings = judge_pin("c", "t-moved", "s", &Moved).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("--add` again"), "{warnings:?}");
    }

    #[test]
    fn a_pin_file_the_base_holds_is_a_leftover() {
        let e = judge_pin("c", "t-s", "b", &Line).unwrap_err();
        assert!(e.contains("already holds"), "{e}");
    }

    #[test]
    fn a_pin_file_with_no_branch_fails_unless_it_landed() {
        let e = judge_pin("c", "t-gone", "s", &Line).unwrap_err();
        assert!(e.contains("no branch t-gone"), "{e}");
        let warnings = judge_pin("a", "t-gone", "c", &Line).unwrap();
        assert!(warnings[0].contains("has landed"), "{warnings:?}");
    }

    #[test]
    fn a_pin_file_of_an_unknown_commit_fails() {
        let e = judge_pin("c", "t-s", "x", &Line).unwrap_err();
        assert!(e.contains("not in the private repository's history"), "{e}");
    }

    /// A scratch folder that is deleted when the test ends.
    struct Scratch(PathBuf);

    impl Scratch {
        fn new(name: &str) -> Scratch {
            let dir = std::env::temp_dir()
                .join(format!("xtask-assets-pin-{name}-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&dir);
            std::fs::create_dir_all(&dir).unwrap();
            Scratch(dir)
        }
    }

    impl Drop for Scratch {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }

    /// `git <args>` in `dir`, as a test user; its output.
    fn git(dir: &Path, args: &[&str]) -> String {
        let out = Command::new("git")
            .args([
                "-c",
                "user.name=t",
                "-c",
                "user.email=t@t",
                "-c",
                "commit.gpgsign=false",
                "-c",
                "init.defaultBranch=main",
            ])
            .args(args)
            .current_dir(dir)
            .output()
            .unwrap();
        assert!(out.status.success(), "{args:?}: {out:?}");
        String::from_utf8(out.stdout).unwrap().trim().to_owned()
    }

    /// Writes `text` to `path` in `dir` and commits it.
    fn commit_file(dir: &Path, path: &str, text: &str) -> String {
        std::fs::write(dir.join(path), text).unwrap();
        git(dir, &["add", "-A"]);
        git(dir, &["commit", "-qm", path]);
        git(dir, &["rev-parse", "HEAD"])
    }

    /// The real git calls, on a scratch repository shaped like [`Line`].
    #[test]
    fn git_reads_ancestry_main_and_branches() {
        let scratch = Scratch::new("history");
        let dir = &scratch.0;
        git(dir, &["init", "-q"]);
        let commit = |message: &str| {
            git(dir, &["commit", "-q", "--allow-empty", "-m", message]);
            git(dir, &["rev-parse", "HEAD"])
        };
        let a = commit("a");
        let b = commit("b");
        let c = commit("c");
        git(dir, &["update-ref", "refs/remotes/origin/main", &c]);
        git(dir, &["switch", "-q", "--detach", &a]);
        let s = commit("s");
        git(dir, &["update-ref", "refs/remotes/origin/t-s", &s]);
        let history = Git(dir);
        assert_eq!(history.is_ancestor(&a, &c), Some(true));
        assert_eq!(history.is_ancestor(&c, &b), Some(false));
        assert_eq!(history.is_ancestor(&a, &"0".repeat(40)), None);
        assert!(history.on_main(&b));
        assert!(!history.on_main(&s));
        assert_eq!(history.branch_head("t-s"), Some(s.clone()));
        assert_eq!(history.branch_head("t-gone"), None);
        assert!(judge(&c, &a, &history).is_err());
        assert!(judge(&b, &s, &history).is_err());
        assert_eq!(judge(&a, &c, &history), Ok(vec![]));
        assert_eq!(judge_pin(&c, "t-s", &s, &history), Ok(vec![]));
        assert!(judge_pin(&c, "t-s", &b, &history).is_err());
    }

    fn script() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(RESOLVE)
    }

    /// Ticket 0195: two changes made from one base each pin their branch,
    /// and the effective pin merges them the same way every time; a
    /// conflict between them fails and leaves the checkout clean.
    #[test]
    fn resolve_merges_the_pin_files_and_fails_on_a_conflict() {
        let scratch = Scratch::new("resolve");
        let dir = &scratch.0;
        git(dir, &["init", "-q"]);
        std::fs::write(dir.join("shared"), "one\n").unwrap();
        let base = commit_file(dir, "base", "base\n");
        let branch = |name: &str, path: &str, text: &str| {
            git(dir, &["switch", "-q", "-c", name, &base]);
            commit_file(dir, path, text)
        };
        let first = branch("t-first", "first", "first\n");
        let second = branch("t-second", "second", "second\n");
        let left = branch("t-left", "shared", "left\n");
        let right = branch("t-right", "shared", "right\n");
        let resolve = |pins: &[&String]| {
            let pins: Vec<String> = pins.iter().map(|p| (*p).clone()).collect();
            run_resolve(&script(), dir, &base, &pins)
        };

        assert_eq!(resolve(&[]), Ok(base.clone()));
        assert_eq!(
            resolve(&[&first]),
            Ok(first.clone()),
            "a descendant is taken"
        );
        let merged = resolve(&[&first, &second]).unwrap();
        assert_ne!(merged, first);
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), merged);
        assert!(dir.join("first").is_file() && dir.join("second").is_file());
        assert_eq!(
            resolve(&[&first, &second]),
            Ok(merged.clone()),
            "reproducible"
        );
        assert_eq!(
            resolve(&[&first, &first]),
            Ok(first.clone()),
            "a held pin is skipped"
        );

        assert!(resolve(&[&left, &right]).is_err());
        assert_eq!(
            git(dir, &["status", "--porcelain"]),
            "",
            "the merge is undone"
        );
        assert_eq!(git(dir, &["rev-parse", "HEAD"]), left);
    }

    /// `--add` writes the pushed branch head into its pin file, and refuses
    /// a checkout on no branch, with changes, or ahead of its push.
    #[test]
    fn add_writes_the_pushed_branch_head() {
        let scratch = Scratch::new("add");
        let root = &scratch.0;
        let remote = root.join("remote.git");
        std::fs::create_dir_all(&remote).unwrap();
        git(&remote, &["init", "-q", "--bare"]);
        git(root, &["clone", "-q", "remote.git", CHECKOUT]);
        let checkout = root.join(CHECKOUT);
        commit_file(&checkout, "base", "base\n");
        git(&checkout, &["push", "-q", "origin", "HEAD:refs/heads/main"]);
        git(&checkout, &["switch", "-q", "-c", "t0001-test"]);
        let head = commit_file(&checkout, "change", "change\n");

        let e = add(root).unwrap_err();
        assert!(e.contains("isn't pushed"), "{e}");
        git(
            &checkout,
            &["push", "-q", "origin", "HEAD:refs/heads/t0001-test"],
        );
        add(root).unwrap();
        let written = std::fs::read_to_string(root.join(PIN_DIR).join("t0001-test.rev")).unwrap();
        assert_eq!(written, format!("{head}\n"));
        assert_eq!(
            read_pins(root),
            Ok(vec![("t0001-test".to_owned(), head.clone())])
        );

        std::fs::write(checkout.join("change"), "edited\n").unwrap();
        assert!(add(root).unwrap_err().contains("aren't committed"));
        git(&checkout, &["checkout", "--", "change"]);
        git(&checkout, &["switch", "-q", "--detach"]);
        assert!(add(root).unwrap_err().contains("on no branch"));
    }

    #[test]
    fn read_pins_refuses_a_stray_file() {
        let scratch = Scratch::new("stray");
        let dir = scratch.0.join(PIN_DIR);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("notes.txt"), "x").unwrap();
        assert!(read_pins(&scratch.0).unwrap_err().contains("notes.txt"));
    }
}
