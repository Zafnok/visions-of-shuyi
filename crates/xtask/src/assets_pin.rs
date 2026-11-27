//! `assets-pin --check` (ticket 0194): the pin of the private repository,
//! `assets-private.rev`, only ever moves forward. Two sessions that both
//! move it can merge out of order; the second must not put `main` back on
//! a commit from before the first's content (the `content-change` skill).
//!
//! Forward means the base's pin is the new pin or one of its ancestors in
//! the private repository's history, read from the checkout in
//! `assets-private/` (CI checks out its whole history for this).

use std::path::Path;
use std::process::Command;

use crate::engine::parse_rev;

pub const USAGE: &str = "usage: cargo xtask assets-pin --check [--base <ref>]\n\n\
fail when assets-private.rev moves back, or sideways, from its value at <ref> (default\n\
origin/main): the new pin must be the old one or a commit after it in the private\n\
repository's history, read from assets-private/ (ticket 0194)";

/// The file that names the pin, and the checkout it names a commit of.
const PIN_FILE: &str = "assets-private.rev";
const CHECKOUT: &str = "assets-private";
/// The private repository's `main`, as the checkout knows it.
const MAIN: &str = "origin/main";

/// What the private repository's history says of two commits.
pub trait History {
    /// Whether `old` is `new` or one of its ancestors; `None` when either
    /// commit isn't in the history.
    fn is_ancestor(&self, old: &str, new: &str) -> Option<bool>;
    /// Whether `commit` is on the private repository's `main`.
    fn on_main(&self, commit: &str) -> bool;
}

/// The verdict on moving the pin from `old` to `new`: the warnings to
/// print, or why the move is refused.
pub fn judge(old: &str, new: &str, history: &impl History) -> Result<Vec<String>, String> {
    if old == new {
        return Ok(Vec::new());
    }
    let mut warnings = Vec::new();
    if !history.on_main(new) {
        warnings.push(format!(
            "the pin {new} is not on the private repository's main: fine while its \
             private pull request is open, but move it to main with `cargo shuyi \
             private-assets --pin` before this merges (the content-change skill)"
        ));
    }
    if !history.on_main(old) {
        // The base broke the rule above; its commit may be gone or squashed
        // into another, so ancestry can't tell forward from back.
        warnings.push(format!(
            "the base's pin {old} is not on the private repository's main, so the \
             direction of this move can't be checked: taken as a repair"
        ));
        return Ok(warnings);
    }
    match history.is_ancestor(old, new) {
        Some(true) => Ok(warnings),
        Some(false) => Err(format!(
            "{PIN_FILE} moves back: {new} does not come after the base's pin {old} in \
             the private repository's history, so main would lose the content between \
             them (ticket 0194).\n\
             Merge the game's main into this branch, then pin the private main, which \
             holds both changes:\n  \
             git -C {CHECKOUT} fetch origin\n  \
             git -C {CHECKOUT} switch --detach origin/main\n  \
             cargo shuyi private-assets --pin"
        )),
        None => Err(format!(
            "{PIN_FILE} names {new}, which is not in the private repository's history: \
             push that commit to a branch there first (the content-change skill)"
        )),
    }
}

/// The history of a git checkout.
pub struct Git<'a>(pub &'a Path);

impl Git<'_> {
    /// Whether `git <args>` succeeds in the checkout (`None` when git
    /// itself couldn't run), never asking for a password.
    fn succeeds(&self, args: &[&str]) -> Option<bool> {
        Command::new("git")
            .args(args)
            .current_dir(self.0)
            .env("GIT_TERMINAL_PROMPT", "0")
            .env("GIT_SSH_COMMAND", "ssh -o BatchMode=yes")
            .output()
            .ok()
            .map(|out| out.status.success())
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
}

/// Checks the move of the pin in `root`'s working tree from its value at
/// `base`: what to print, or why it fails.
pub fn check(root: &Path, base: &str) -> Result<String, String> {
    let new = std::fs::read_to_string(root.join(PIN_FILE))
        .map_err(|e| format!("read {PIN_FILE}: {e}"))
        .and_then(|text| parse_rev(&text))?;
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
    if old == new {
        return Ok(format!("{PIN_FILE} is unchanged from {base}"));
    }
    let checkout = root.join(CHECKOUT);
    if !checkout.join(".git").exists() {
        return Err(format!(
            "{PIN_FILE} moves, and checking its direction needs the private repository: \
             run `cargo shuyi private-assets` first"
        ));
    }
    let git = Git(&checkout);
    if !git.has(&old) || !git.has(&new) || !git.on_main(&new) {
        // A local checkout may be behind; CI's has the whole history and
        // no key left to fetch with, so this fails there and changes nothing.
        let _ = git.succeeds(&["fetch", "-q", "origin"]);
    }
    let warnings = judge(&old, &new, &git)?;
    let said = std::iter::once(format!("{PIN_FILE} moves forward: {old} -> {new}"))
        .chain(warnings.into_iter().map(|w| format!("warning: {w}")))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(said)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A line of commits `a` → `b` → `c` on main, and `s`, a branch
    /// commit after `a`.
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
        assert!(e.contains("private-assets --pin"), "{e}");
    }

    #[test]
    fn a_move_sideways_fails() {
        assert!(judge("b", "s", &Line).is_err());
    }

    #[test]
    fn a_forward_move_to_a_branch_commit_warns() {
        let warnings = judge("a", "s", &Line).unwrap();
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("not on the private repository's main"));
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

    /// The real git calls, on a scratch repository shaped like [`Line`].
    #[test]
    fn git_reads_ancestry_and_main() {
        let dir = std::env::temp_dir().join(format!("xtask-assets-pin-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let git = |args: &[&str]| {
            let out = Command::new("git")
                .args([
                    "-c",
                    "user.name=t",
                    "-c",
                    "user.email=t@t",
                    "-c",
                    "commit.gpgsign=false",
                ])
                .args(args)
                .current_dir(&dir)
                .output()
                .unwrap();
            assert!(out.status.success(), "{out:?}");
            String::from_utf8(out.stdout).unwrap().trim().to_owned()
        };
        git(&["init", "-q"]);
        let commit = |message: &str| {
            git(&["commit", "-q", "--allow-empty", "-m", message]);
            git(&["rev-parse", "HEAD"])
        };
        let a = commit("a");
        let b = commit("b");
        let c = commit("c");
        git(&["update-ref", "refs/remotes/origin/main", &c]);
        git(&["switch", "-q", "--detach", &a]);
        let s = commit("s");
        let history = Git(&dir);
        assert_eq!(history.is_ancestor(&a, &c), Some(true));
        assert_eq!(history.is_ancestor(&c, &b), Some(false));
        assert_eq!(history.is_ancestor(&a, &"0".repeat(40)), None);
        assert!(history.on_main(&b));
        assert!(!history.on_main(&s));
        assert!(judge(&c, &a, &history).is_err());
        assert!(judge(&b, &s, &history).is_err());
        assert_eq!(judge(&a, &c, &history), Ok(vec![]));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
