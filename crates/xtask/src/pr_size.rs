//! `cargo xtask pr-size [--base <ref>] [--branch <name>] [--hook]`: the
//! PR's changed lines as GitHub counts them, without the generated and
//! binary files, against the size a session can finish well (ADR-0101).
//! It warns; it never fails a build.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use crate::code_lines::git;

pub const USAGE: &str = "usage: cargo xtask pr-size [--base <ref>] [--branch <name>] [--hook]\n\n\
counts the lines added plus removed between the base (default origin/main;\n\
its merge base with HEAD) and the working tree, untracked files included,\n\
leaving out binary files and files marked linguist-generated in\n\
.gitattributes (snapshots, Cargo.lock, generated assets, font sources, the\n\
engine's skill pack). Past 2,000 lines it says to plan the split, past 3,000 to make it\n\
(ADR-0101). It never fails: the exit code is 0 unless git or the arguments\n\
are wrong. An engine-pin/ or sonar/ branch is reported with no warning.\n\
--branch names the branch (default: the current one; CI passes the PR's).\n\
--hook prints only a warning, as a Claude Code hook's JSON, or nothing. It reads\n\
the hook's input on stdin: it measures the repository of the session's cwd\n\
(a worktree, not the main checkout the hook runs in) and speaks only before\n\
git commit, git push or gh pr create.";

/// A ticket is planned at about this many changed lines or fewer.
pub const PLAN: usize = 2_000;
/// A PR passes this many changed lines only with a reason.
pub const LIMIT: usize = 3_000;

/// Leaves out every file `.gitattributes` marks `linguist-generated`.
const NOT_GENERATED: &str = ":(exclude,attr:linguist-generated)";

/// How a PR's size reads against [`PLAN`] and [`LIMIT`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Fine,
    PlanTheSplit,
    OverTheLimit,
    /// An `engine-pin/` or `sonar/` PR: its size is the engine's or the
    /// findings', not a ticket's.
    Exempt,
}

impl Verdict {
    pub fn of(lines: usize, branch: &str) -> Self {
        if branch.starts_with("engine-pin/") || branch.starts_with("sonar/") {
            Verdict::Exempt
        } else if lines > LIMIT {
            Verdict::OverTheLimit
        } else if lines > PLAN {
            Verdict::PlanTheSplit
        } else {
            Verdict::Fine
        }
    }

    /// The warning to give, if any.
    pub fn warning(self, lines: usize) -> Option<String> {
        let lines = thousands(lines);
        match self {
            Verdict::Fine | Verdict::Exempt => None,
            Verdict::PlanTheSplit => Some(format!(
                "this PR is at {lines} changed lines, past the 2,000 a ticket is planned at: \
                 note where it would split cleanly before it grows further (work-ticket §3, ADR-0101)"
            )),
            Verdict::OverTheLimit => Some(format!(
                "this PR is at {lines} changed lines, over the limit of about 3,000: \
                 can it split cleanly into a second PR? Move what can wait into a new ticket \
                 with write-ticket (work-ticket §3, ADR-0101)"
            )),
        }
    }
}

/// Each file's added plus removed lines from `git diff --numstat -z`
/// output; binary files (`-`) are left out. A rename is
/// `added\tremoved\t\0old\0new\0` and counts under its new path.
pub fn numstat(output: &str) -> Vec<(String, usize)> {
    let mut files = Vec::new();
    let mut fields = output.split('\0');
    while let Some(entry) = fields.next() {
        let mut parts = entry.splitn(3, '\t');
        let (Some(added), Some(removed), Some(path)) = (parts.next(), parts.next(), parts.next())
        else {
            continue;
        };
        let path = if path.is_empty() {
            fields.next();
            fields.next().unwrap_or_default()
        } else {
            path
        };
        if let (Ok(added), Ok(removed)) = (added.parse::<usize>(), removed.parse::<usize>()) {
            files.push((path.to_owned(), added + removed));
        }
    }
    files
}

/// The lines of an untracked file, or `None` for a binary one (a NUL byte,
/// as git judges it).
pub fn untracked_lines(bytes: &[u8]) -> Option<usize> {
    if bytes.contains(&0) {
        return None;
    }
    let pieces = bytes.split(|&b| b == b'\n').count();
    Some(pieces - usize::from(bytes.is_empty() || bytes.ends_with(b"\n")))
}

/// `n` with a comma every three digits.
fn thousands(n: usize) -> String {
    let digits = n.to_string();
    let mut out = String::new();
    for (i, digit) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(digit);
    }
    out
}

/// The report: the total, the five largest files and the verdict.
fn report(files: &[(String, usize)], branch: &str, from: &str) -> (usize, String) {
    let total: usize = files.iter().map(|(_, lines)| lines).sum();
    let verdict = Verdict::of(total, branch);
    let mut out = format!(
        "pr-size: {} changed lines (added + removed, generated and binary files left out), {from} to the working tree\n",
        thousands(total)
    );
    let mut largest: Vec<_> = files.iter().collect();
    largest.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    for (path, lines) in largest.into_iter().take(5) {
        let _ = writeln!(out, "{lines:>7}  {path}");
    }
    match (verdict, verdict.warning(total)) {
        (Verdict::Exempt, _) => {
            let _ = writeln!(out, "{branch}: reported only, no limit (ADR-0101)");
        }
        (_, Some(warning)) => {
            let _ = writeln!(out, "warning: {warning}");
        }
        _ => out.push_str("within the 2,000 a ticket is planned at\n"),
    }
    (total, out)
}

/// A Claude Code `PreToolUse` hook's answer that gives the session
/// `message` and lets the command run.
fn hook_json(message: &str) -> String {
    serde_json::json!({
        "hookSpecificOutput": {
            "hookEventName": "PreToolUse",
            "additionalContext": format!("pr-size: {message}"),
        }
    })
    .to_string()
}

/// What the command prints: the hook's answer if there is a warning with
/// `hook`, else the report, with a warning annotation on the pull request
/// past the limit when `github_actions` (the variable of that name) says
/// this is CI. The job stays green (ADR-0101).
pub fn output(
    files: &[(String, usize)],
    branch: &str,
    from: &str,
    hook: bool,
    github_actions: Option<&str>,
) -> String {
    let (total, report) = report(files, branch, from);
    let verdict = Verdict::of(total, branch);
    let warning = verdict.warning(total);
    if hook {
        return warning.map_or_else(String::new, |w| hook_json(&w) + "\n");
    }
    match warning {
        Some(warning) if verdict == Verdict::OverTheLimit && github_actions == Some("true") => {
            format!("{report}::warning title=PR size::{warning}\n")
        }
        _ => report,
    }
}

/// The changed files of `repo_root`'s working tree against `base`'s merge
/// base with `HEAD`, and the short hash it counted from.
pub fn changed_files(
    repo_root: &Path,
    base: &str,
) -> Result<(Vec<(String, usize)>, String), String> {
    let from = git(repo_root, &["merge-base", base, "HEAD"])?
        .trim()
        .to_owned();
    let diff = git(
        repo_root,
        &[
            "diff",
            "--numstat",
            "-z",
            "-M",
            "--no-ext-diff",
            &from,
            "--",
            ".",
            NOT_GENERATED,
        ],
    )?;
    let mut files = numstat(&diff);
    let untracked = git(
        repo_root,
        &[
            "ls-files",
            "--others",
            "--exclude-standard",
            "-z",
            "--",
            ".",
            NOT_GENERATED,
        ],
    )?;
    for path in untracked.split('\0').filter(|p| !p.is_empty()) {
        if let Some(lines) = std::fs::read(repo_root.join(path))
            .ok()
            .and_then(|bytes| untracked_lines(&bytes))
        {
            files.push((path.to_owned(), lines));
        }
    }
    Ok((files, from.chars().take(10).collect()))
}

/// What a Claude Code `PreToolUse` hook is given on stdin, as far as
/// `pr-size` needs it: the session's directory and the shell command.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct HookInput {
    pub cwd: Option<PathBuf>,
    pub command: String,
}

/// Reads the hook's input; a missing field (or no JSON at all, from a
/// person running `--hook` by hand) is left empty.
pub fn hook_input(json: &str) -> HookInput {
    let value: serde_json::Value = serde_json::from_str(json).unwrap_or_default();
    HookInput {
        cwd: value["cwd"]
            .as_str()
            .filter(|cwd| !cwd.is_empty())
            .map(PathBuf::from),
        command: value["tool_input"]["command"]
            .as_str()
            .unwrap_or_default()
            .to_owned(),
    }
}

/// Whether a shell command runs `git commit`, `git push` or `gh pr
/// create`. The hooks' `if` rules also fire on commands Claude Code can't
/// analyze (a `$VAR`, a `$(...)`), so the hook checks for itself.
pub fn runs_a_pr_step(command: &str) -> bool {
    let words: Vec<&str> = command
        .split(|c: char| c.is_whitespace() || ";&|(){}".contains(c))
        .filter(|word| !word.is_empty())
        .collect();
    words
        .windows(2)
        .any(|pair| matches!(pair, ["git", "commit" | "push"]))
        || words
            .windows(3)
            .any(|three| three == ["gh", "pr", "create"])
}

/// The top of the git working tree that holds `dir`.
pub fn repo_of(dir: &Path) -> Result<PathBuf, String> {
    Ok(PathBuf::from(
        git(dir, &["rev-parse", "--show-toplevel"])?.trim(),
    ))
}

/// The hook's answer for its input `stdin`: nothing unless the command is a
/// PR step, else the warning for the repository and branch of the session's
/// `cwd` (ticket 0161: the hook process itself runs in the main checkout).
/// `fallback` is measured when the input names no directory. Any git error
/// is silence: the hook never gets in the way.
pub fn hook_answer(stdin: &str, fallback: &Path, base: &str, branch: Option<&str>) -> String {
    let input = hook_input(stdin);
    if !input.command.is_empty() && !runs_a_pr_step(&input.command) {
        return String::new();
    }
    let measured = input
        .cwd
        .as_deref()
        .map_or_else(|| Ok(fallback.to_path_buf()), repo_of)
        .and_then(|root| {
            let (files, from) = changed_files(&root, base)?;
            let branch = branch.map_or_else(|| current_branch(&root), |b| Ok(b.to_owned()))?;
            Ok((files, from, branch))
        });
    measured.map_or_else(
        |_| String::new(),
        |(files, from, branch)| output(&files, &branch, &from, true, None),
    )
}

/// The current branch's name (`HEAD` when detached).
pub fn current_branch(repo_root: &Path) -> Result<String, String> {
    Ok(git(repo_root, &["rev-parse", "--abbrev-ref", "HEAD"])?
        .trim()
        .to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numstat_reads_plain_renamed_and_binary_files() {
        let output = "10\t2\tcrates/ui/src/a.rs\0\
                      0\t0\t\0docs/old.md\0docs/new.md\0\
                      -\t-\tassets/music/a.ogg\0\
                      3\t1\t\0x.rs\0y.rs\0";
        assert_eq!(
            numstat(output),
            [
                ("crates/ui/src/a.rs".to_owned(), 12),
                ("docs/new.md".to_owned(), 0),
                ("y.rs".to_owned(), 4),
            ]
        );
        assert_eq!(numstat(""), []);
    }

    #[test]
    fn untracked_lines_count_a_last_line_without_a_newline() {
        assert_eq!(untracked_lines(b"a\nb\n"), Some(2));
        assert_eq!(untracked_lines(b"a\nb"), Some(2));
        assert_eq!(untracked_lines(b""), Some(0));
        assert_eq!(untracked_lines(b"PNG\0\x01"), None);
    }

    #[test]
    fn verdicts_follow_the_plan_the_limit_and_the_branch() {
        assert_eq!(Verdict::of(2_000, "t0135-x"), Verdict::Fine);
        assert_eq!(Verdict::of(2_001, "t0135-x"), Verdict::PlanTheSplit);
        assert_eq!(Verdict::of(3_000, "claude/x"), Verdict::PlanTheSplit);
        assert_eq!(Verdict::of(3_001, "claude/x"), Verdict::OverTheLimit);
        assert_eq!(Verdict::of(9_000, "engine-pin/50ae33db"), Verdict::Exempt);
        assert_eq!(Verdict::of(9_000, "sonar/ui-smells"), Verdict::Exempt);
        assert_eq!(Verdict::Fine.warning(10), None);
        assert_eq!(Verdict::Exempt.warning(9_000), None);
        assert!(
            Verdict::PlanTheSplit
                .warning(2_500)
                .unwrap()
                .contains("2,500")
        );
        assert!(
            Verdict::OverTheLimit
                .warning(7_250)
                .unwrap()
                .contains("split cleanly")
        );
    }

    #[test]
    fn thousands_puts_a_comma_every_three_digits() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(999), "999");
        assert_eq!(thousands(1_000), "1,000");
        assert_eq!(thousands(1_234_567), "1,234,567");
    }

    #[test]
    fn the_report_lists_the_largest_files_and_the_verdict() {
        let files: Vec<_> = [("a", 1), ("b", 5), ("c", 3), ("d", 2), ("e", 4), ("f", 4)]
            .iter()
            .map(|&(p, n)| (p.to_owned(), n))
            .collect();
        let (total, out) = report(&files, "t0135-x", "abc");
        assert_eq!(total, 19);
        assert_eq!(
            out,
            "pr-size: 19 changed lines (added + removed, generated and binary files left out), abc to the working tree\n\
             \x20     5  b\n\
             \x20     4  e\n\
             \x20     4  f\n\
             \x20     3  c\n\
             \x20     2  d\n\
             within the 2,000 a ticket is planned at\n"
        );
        let big = [("x".to_owned(), 3_500)];
        assert!(
            report(&big, "t1", "abc")
                .1
                .ends_with("(work-ticket §3, ADR-0101)\n")
        );
        assert!(
            report(&big, "sonar/a", "abc")
                .1
                .ends_with("sonar/a: reported only, no limit (ADR-0101)\n")
        );
    }

    #[test]
    fn the_hook_answer_is_context_for_the_session() {
        let json: serde_json::Value = serde_json::from_str(&hook_json("a \"b\"\nc")).unwrap();
        assert_eq!(json["hookSpecificOutput"]["hookEventName"], "PreToolUse");
        assert_eq!(
            json["hookSpecificOutput"]["additionalContext"],
            "pr-size: a \"b\"\nc"
        );
    }

    #[test]
    fn the_output_is_the_report_the_hook_answer_or_the_annotation() {
        const NOTE: &str = "::warning title=PR size::";
        let size = |lines| vec![("x".to_owned(), lines)];
        let output = |lines, branch, hook, ci| output(&size(lines), branch, "abc", hook, ci);

        // The hook speaks only with a warning, and always without the report.
        assert_eq!(output(2_000, "t1", true, None), "");
        assert_eq!(output(9_000, "sonar/a", true, None), "");
        for lines in [2_500, 3_500] {
            let answer = output(lines, "t1", true, None);
            assert!(answer.starts_with("{\"hookSpecificOutput\""), "{answer}");
            assert!(answer.ends_with("}\n"), "{answer}");
            assert!(
                !answer.contains("pr-size: 2,500 changed lines (added"),
                "{answer}"
            );
        }

        // CI annotates past the limit only, and after the report.
        let annotated = output(3_500, "t1", false, Some("true"));
        assert!(
            annotated.starts_with("pr-size: 3,500 changed lines"),
            "{annotated}"
        );
        assert!(annotated.contains(NOTE), "{annotated}");
        assert!(!output(2_500, "t1", false, Some("true")).contains(NOTE));
        assert!(!output(3_500, "sonar/a", false, Some("true")).contains(NOTE));
        assert!(!output(3_500, "t1", false, Some("false")).contains(NOTE));
        assert!(!output(3_500, "t1", false, None).contains(NOTE));
    }

    fn is_snapshot(path: &str) -> bool {
        Path::new(path).extension().is_some_and(|e| e == "snap")
    }

    /// `.gitattributes` and the pathspec together: the tracked files they
    /// leave out are the generated ones, and only those.
    #[test]
    fn the_pathspec_leaves_out_exactly_the_generated_files() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let all = git(&root, &["ls-files"]).unwrap();
        let kept = git(&root, &["ls-files", "--", ".", NOT_GENERATED]).unwrap();
        let kept: Vec<&str> = kept.lines().collect();
        let left_out: Vec<&str> = all.lines().filter(|p| !kept.contains(p)).collect();
        for generated in [
            "Cargo.lock",
            "assets/fonts/atlas.ron",
            "assets-src/fonts/ter-u16n.bdf",
            "assets-test/tilesets/test.ron",
        ] {
            assert!(left_out.contains(&generated), "{generated} is counted");
        }
        assert!(left_out.iter().any(|p| is_snapshot(p)));
        assert!(
            left_out.iter().all(|p| is_snapshot(p)
                || *p == "Cargo.lock"
                || p.starts_with(".claude/skills/shuyi-")
                || p.starts_with("assets-test/tilesets/test")
                || *p == "assets/fonts/atlas.ron"
                || p.starts_with("assets-src/fonts/")),
            "{left_out:?}"
        );
        assert!(kept.contains(&"crates/xtask/src/code_lines.rs"));
        assert!(
            left_out
                .iter()
                .any(|p| p.starts_with(".claude/skills/shuyi-"))
        );
        assert!(
            kept.iter()
                .any(|p| p.starts_with(".claude/skills/work-ticket/"))
        );
    }

    #[test]
    fn the_hook_input_names_the_directory_and_the_command() {
        let json = r#"{"cwd":"D:\\w\\t1","tool_input":{"command":"git push -u origin HEAD"}}"#;
        assert_eq!(
            hook_input(json),
            HookInput {
                cwd: Some(PathBuf::from(r"D:\w\t1")),
                command: "git push -u origin HEAD".to_owned(),
            }
        );
        assert_eq!(hook_input(r#"{"cwd":""}"#), HookInput::default());
        assert_eq!(hook_input(""), HookInput::default());
        assert_eq!(hook_input("not json"), HookInput::default());
    }

    #[test]
    fn only_a_commit_a_push_or_a_new_pr_is_a_pr_step() {
        for command in [
            "git commit -m x",
            "cd /d/w && git commit -F msg.txt",
            "git add . ; git push",
            "echo \"$X\" | (git push)",
            "gh pr create --title t",
        ] {
            assert!(runs_a_pr_step(command), "{command}");
        }
        for command in [
            "",
            "git status",
            "cd x; echo \"$VAR\"; git log",
            "git commit-tree abc",
            "gh pr view 641",
            "cat .claude/settings.json",
        ] {
            assert!(!runs_a_pr_step(command), "{command}");
        }
    }

    /// Ticket 0161: the hook process runs in the main checkout, whose
    /// untracked files were counted (4,127 lines on a 64-line engine pin PR)
    /// and whose branch is never an exempt one. Measured from the session's
    /// `cwd`, a worktree counts its own lines and keeps its exemption.
    #[test]
    fn the_hook_measures_the_sessions_worktree() {
        let root = std::env::temp_dir().join(format!("trpg-pr-size-wt-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let main = root.join("main");
        std::fs::create_dir_all(main.join("src")).unwrap();
        let run = |dir: &Path, args: &[&str]| git(dir, args).unwrap();
        run(&main, &["init", "-q", "-b", "trunk"]);
        run(&main, &["config", "user.email", "test@example.com"]);
        run(&main, &["config", "user.name", "Test"]);
        std::fs::write(main.join("src/a.rs"), "a\nb\n").unwrap();
        run(&main, &["add", "."]);
        run(&main, &["commit", "-q", "-m", "first"]);
        // The main checkout's leftovers: a big untracked log.
        std::fs::write(main.join("test.log"), "line\n".repeat(LIMIT + 1)).unwrap();
        let input = |dir: &Path, command: &str| {
            serde_json::json!({ "cwd": dir, "tool_input": { "command": command } }).to_string()
        };

        // The miscount: with no directory named, the main checkout is measured.
        let before = hook_answer("", &main, "trunk", None);
        assert!(before.contains("3,001 changed lines"), "{before}");

        for (name, branch) in [("wt-pin", "engine-pin/85e8d5c5"), ("wt-ticket", "t1-x")] {
            let worktree = root.join(name);
            let path = worktree.to_str().unwrap();
            run(
                &main,
                &["worktree", "add", "-q", "-b", branch, path, "trunk"],
            );
            std::fs::write(worktree.join("src/a.rs"), "a\nc\nd\n").unwrap();
            let top = repo_of(&worktree.join("src")).unwrap();
            assert_eq!(
                changed_files(&top, "trunk").unwrap().0,
                [("src/a.rs".to_owned(), 3)]
            );
            assert_eq!(current_branch(&top).unwrap(), branch);
            let answer = hook_answer(
                &input(&worktree.join("src"), "git commit -m x"),
                &main,
                "trunk",
                None,
            );
            assert_eq!(answer, "", "{name}");
        }

        // A big worktree warns before a PR step, unless its branch is exempt.
        for name in ["wt-pin", "wt-ticket"] {
            std::fs::write(root.join(name).join("big.md"), "line\n".repeat(LIMIT)).unwrap();
        }
        let ask = |name: &str, command: &str, base: &str| {
            hook_answer(&input(&root.join(name), command), &main, base, None)
        };
        assert_eq!(ask("wt-pin", "git push", "trunk"), "");
        let warned = ask("wt-ticket", "git push", "trunk");
        assert!(warned.contains("3,003 changed lines"), "{warned}");
        assert_eq!(ask("wt-ticket", "git status", "trunk"), "");
        assert_eq!(ask("wt-ticket", "git push", "no-such-ref-0161"), "");
        // A directory outside any repository: silence.
        let outside = input(&std::env::temp_dir(), "git push");
        assert_eq!(hook_answer(&outside, &main, "trunk", None), "");

        for name in ["wt-pin", "wt-ticket"] {
            run(
                &main,
                &[
                    "worktree",
                    "remove",
                    "--force",
                    root.join(name).to_str().unwrap(),
                ],
            );
        }
        std::fs::remove_dir_all(&root).unwrap();
    }

    /// A throwaway repository: an edit, a rename, a generated file, a binary
    /// file and untracked files, against the commit before.
    #[test]
    fn changed_files_count_a_real_diff() {
        let root = std::env::temp_dir().join(format!("trpg-pr-size-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let write = |path: &str, text: &str| std::fs::write(root.join(path), text).unwrap();
        let run = |args: &[&str]| git(&root, args).unwrap();
        run(&["init", "-q", "-b", "trunk"]);
        run(&["config", "user.email", "test@example.com"]);
        run(&["config", "user.name", "Test"]);
        write(".gitattributes", "*.snap linguist-generated\n");
        write("edit.rs", "a\nb\n");
        write("moved.md", "one\ntwo\nthree\nfour\nfive\nsix\n");
        run(&["add", "."]);
        run(&["commit", "-q", "-m", "first"]);
        let first = run(&["rev-parse", "HEAD"]);

        write("edit.rs", "a\nc\nd\n");
        run(&["mv", "moved.md", "renamed.md"]);
        write("a.snap", "generated\n");
        write("new.snap", "generated\n");
        write("picture.bin", "\0\0");
        write("new.md", "x\ny");
        run(&["add", "a.snap", "edit.rs", "picture.bin"]);
        run(&["commit", "-q", "-m", "second"]);

        let (mut files, from) = changed_files(&root, first.trim()).unwrap();
        files.sort();
        assert_eq!(from, first.trim()[..10]);
        assert_eq!(
            files,
            [
                ("edit.rs".to_owned(), 3),
                ("new.md".to_owned(), 2),
                ("renamed.md".to_owned(), 0),
            ]
        );
        assert_eq!(current_branch(&root).unwrap(), "trunk");
        assert!(changed_files(&root, "no-such-ref-0135").is_err());
        std::fs::remove_dir_all(&root).unwrap();
    }
}
