//! `cargo xtask code-lines [--base <ref>]`: production Rust lines added and
//! removed per crate since a base, the measure of the engine-friction rule
//! (ADR-0091). It informs; it never fails a build.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

pub const USAGE: &str = "usage: cargo xtask code-lines [--base <ref>]\n\n\
counts production Rust lines added and removed per crate between the base\n\
(default origin/main; its merge base with HEAD) and the working tree,\n\
untracked files included. Left out: files under tests/, tests.rs and\n\
*_tests.rs, #[cfg(test)] items, blank lines and comment-only lines.\n\
A changed line counts once as removed and once as added, so \"added\" is\n\
the measure the engine-friction rule compares with its estimate (ADR-0091).\n\
--base 4b825dc642cb6eb9a060e54bf8d69288fbee4904 (git's empty tree) counts\n\
the whole working tree.";

/// The lines one crate gained and lost.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateLines {
    /// The crate's folder, e.g. `crates/xtask` or `engine/crates/ui`.
    pub krate: String,
    pub added: usize,
    pub removed: usize,
}

/// Which side of the diff a file's text is wanted for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    /// The base, for removed lines.
    Old,
    /// The working tree, for added lines.
    New,
}

/// Counts `diff` (`git diff --unified=0` output) per crate, sorted by crate.
/// `text` gives a file's whole text on one side, to find its `#[cfg(test)]`
/// items; it is asked once per file and side, only when that side has lines.
pub fn count(diff: &str, text: impl Fn(Side, &str) -> Option<String>) -> Vec<CrateLines> {
    let mut totals: BTreeMap<String, (usize, usize)> = BTreeMap::new();
    let mut old_path = None::<String>;
    let mut new_path = None::<String>;
    let mut ranges: [Option<Vec<(usize, usize)>>; 2] = [None, None];
    // The next line number on each side, and how many lines the hunk has
    // left there: a `---`/`+++` line is a header only between hunks.
    let (mut old_line, mut old_left, mut new_line, mut new_left) = (0, 0, 0, 0);
    for line in diff.lines() {
        if let Some(code) = line.strip_prefix('-').filter(|_| old_left > 0) {
            if let Some(path) = &old_path {
                let tests = ranges[0].get_or_insert_with(|| file_ranges(&text, Side::Old, path));
                if counts(path, code, old_line, tests) {
                    totals.entry(crate_of(path)).or_default().1 += 1;
                }
            }
            old_line += 1;
            old_left -= 1;
        } else if let Some(code) = line.strip_prefix('+').filter(|_| new_left > 0) {
            if let Some(path) = &new_path {
                let tests = ranges[1].get_or_insert_with(|| file_ranges(&text, Side::New, path));
                if counts(path, code, new_line, tests) {
                    totals.entry(crate_of(path)).or_default().0 += 1;
                }
            }
            new_line += 1;
            new_left -= 1;
        } else if let Some(path) = line.strip_prefix("--- ") {
            old_path = diff_path(path, "a/");
            ranges = [None, None];
        } else if let Some(path) = line.strip_prefix("+++ ") {
            new_path = diff_path(path, "b/");
        } else if let Some(hunk) = line.strip_prefix("@@ ") {
            (old_line, old_left, new_line, new_left) = hunk_ranges(hunk);
        }
    }
    totals
        .into_iter()
        .map(|(krate, (added, removed))| CrateLines {
            krate,
            added,
            removed,
        })
        .collect()
}

/// The path of a `---`/`+++` header, or `None` for `/dev/null`.
fn diff_path(header: &str, prefix: &str) -> Option<String> {
    header.strip_prefix(prefix).map(str::to_owned)
}

/// The old side's first line and line count, then the new side's, of a
/// hunk header's `-a[,b] +c[,d] @@` (after the leading `@@ `); a missing
/// count is 1.
fn hunk_ranges(hunk: &str) -> (usize, usize, usize, usize) {
    let side = |prefix: char| {
        let range = hunk
            .split_whitespace()
            .find_map(|part| part.strip_prefix(prefix))
            .unwrap_or("0,0");
        let (start, count) = range.split_once(',').unwrap_or((range, "1"));
        (start.parse().unwrap_or(0), count.parse().unwrap_or(0))
    };
    let (old_start, old_count) = side('-');
    let (new_start, new_count) = side('+');
    (old_start, old_count, new_start, new_count)
}

fn file_ranges(
    text: &impl Fn(Side, &str) -> Option<String>,
    side: Side,
    path: &str,
) -> Vec<(usize, usize)> {
    text(side, path).map_or_else(Vec::new, |source| test_ranges(&source))
}

/// Whether line `number` of `path`, reading `code`, is a production line.
fn counts(path: &str, code: &str, number: usize, tests: &[(usize, usize)]) -> bool {
    let code = code.trim();
    is_production_file(path)
        && !code.is_empty()
        && !code.starts_with("//")
        && !tests
            .iter()
            .any(|&(from, to)| (from..=to).contains(&number))
}

/// A Rust file outside `tests/` that isn't a `tests.rs` or `*_tests.rs`
/// module (those are declared under `#[cfg(test)]` by their parent).
fn is_production_file(path: &str) -> bool {
    let Some(stem) = path.strip_suffix(".rs") else {
        return false;
    };
    let file = stem.rsplit('/').next().unwrap_or(stem);
    !path.split('/').any(|part| part == "tests") && file != "tests" && !file.ends_with("_tests")
}

/// The crate folder `path` belongs to: `…/crates/<name>`, else the folder
/// above `src`, `examples` or `benches` (or a `build.rs`), else the file's
/// folder (`.` at the top).
fn crate_of(path: &str) -> String {
    let parts: Vec<&str> = path.split('/').collect();
    let last = parts.len() - 1;
    let end = parts
        .iter()
        .position(|&p| p == "crates")
        .filter(|&i| i + 1 < last)
        .map(|i| i + 2)
        .or_else(|| {
            (0..=last).rev().find(|&i| {
                matches!(parts[i], "src" | "examples" | "benches")
                    || (i == last && parts[i] == "build.rs")
            })
        })
        .unwrap_or(last);
    if end == 0 {
        ".".to_owned()
    } else {
        parts[..end].join("/")
    }
}

/// The 1-based line ranges, inclusive, of the `#[cfg(test)]` items in
/// `source`: from the attribute to the item's `;` or closing brace. Brace
/// counting skips strings, char literals and line comments; order of
/// magnitude is the goal, not a parser.
pub fn test_ranges(source: &str) -> Vec<(usize, usize)> {
    let lines = scan(source);
    let mut ranges = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        let (in_string, text, _) = &lines[i];
        if !in_string && text.trim_start().starts_with("#[cfg(test)]") {
            let end = item_end(&lines, i);
            ranges.push((i + 1, end + 1));
            i = end;
        }
        i += 1;
    }
    ranges
}

/// The index of the line where the item whose attribute is on line `start`
/// ends (the last line if it never closes).
fn item_end(lines: &[(bool, String, Vec<char>)], start: usize) -> usize {
    let mut depth = 0usize;
    for (i, (_, _, marks)) in lines.iter().enumerate().skip(start) {
        for &mark in marks {
            match mark {
                ';' if depth == 0 => return i,
                '{' => depth += 1,
                '}' => {
                    depth = depth.saturating_sub(1);
                    if depth == 0 {
                        return i;
                    }
                }
                _ => {}
            }
        }
    }
    lines.len() - 1
}

/// Each line of `source` as (starts inside a string, its text, its `{`,
/// `}` and `;` outside strings, char literals and comments).
fn scan(source: &str) -> Vec<(bool, String, Vec<char>)> {
    let mut in_string = false;
    source
        .lines()
        .map(|line| {
            let starts_in_string = in_string;
            let chars: Vec<char> = line.chars().collect();
            let mut marks = Vec::new();
            let mut i = 0;
            while i < chars.len() {
                let c = chars[i];
                if in_string {
                    match c {
                        '\\' => i += 1,
                        '"' => in_string = false,
                        _ => {}
                    }
                } else {
                    match c {
                        '"' => in_string = true,
                        '/' if chars.get(i + 1) == Some(&'/') => break,
                        '\'' => i += char_literal_len(&chars[i..]),
                        '{' | '}' | ';' => marks.push(c),
                        _ => {}
                    }
                }
                i += 1;
            }
            (starts_in_string, line.to_owned(), marks)
        })
        .collect()
}

/// How many characters after the opening `'` a char literal runs to its
/// closing `'` (`'x'`, `'\n'`, `'\''`, `'\u{7b}'`), or 0 for a lifetime.
fn char_literal_len(chars: &[char]) -> usize {
    match chars.get(1) {
        Some('\\') => chars
            .iter()
            .skip(3)
            .position(|&c| c == '\'')
            .map_or(0, |p| p + 3),
        Some(_) if chars.get(2) == Some(&'\'') => 2,
        _ => 0,
    }
}

/// The report: one row per crate, then the game's total and, when the
/// engine changed too (`engine/`, before ticket 1131), the engine's.
pub fn table(rows: &[CrateLines]) -> String {
    let width = rows.iter().fold(14, |w, r| w.max(r.krate.len()));
    let line = |name: &str, added: usize, removed: usize| {
        format!("{name:<width$}  {added:>7}  {removed:>7}\n")
    };
    let mut out = format!("{:<width$}  {:>7}  {:>7}\n", "crate", "added", "removed");
    for row in rows {
        out += &line(&row.krate, row.added, row.removed);
    }
    let (engine, game): (Vec<_>, Vec<_>) =
        rows.iter().partition(|r| r.krate.starts_with("engine/"));
    let sum = |rows: &[&CrateLines]| {
        rows.iter()
            .fold((0, 0), |(a, r), row| (a + row.added, r + row.removed))
    };
    let (added, removed) = sum(&game);
    out += &line("total (game)", added, removed);
    if !engine.is_empty() {
        let (added, removed) = sum(&engine);
        out += &line("total (engine)", added, removed);
    }
    out
}

/// A diff of `path`, untracked, as all added lines.
fn untracked_diff(path: &str, source: &str) -> String {
    let lines = source.lines().count();
    let mut diff = format!("--- /dev/null\n+++ b/{path}\n@@ -0,0 +1,{lines} @@\n");
    for line in source.lines() {
        diff.push('+');
        diff.push_str(line);
        diff.push('\n');
    }
    diff
}

/// Runs `git` in `repo_root` with `args`; its stdout, or its stderr as the
/// error.
fn git(repo_root: &Path, args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo_root)
        .args(["-c", "core.quotePath=false"])
        .args(args)
        .output()
        .map_err(|e| format!("cannot run git: {e}"))?;
    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_owned())
    }
}

/// The report for `repo_root`'s working tree against `base`'s merge base
/// with `HEAD` (or `base` itself when it has none, e.g. the empty tree).
pub fn run(repo_root: &Path, base: &str) -> Result<String, String> {
    let from = git(repo_root, &["merge-base", base, "HEAD"])
        .map_or_else(|_| base.to_owned(), |hash| hash.trim().to_owned());
    let mut diff = git(
        repo_root,
        &[
            "diff",
            "--unified=0",
            "--no-color",
            "--no-ext-diff",
            "-M",
            &from,
            "--",
            "*.rs",
        ],
    )?;
    let untracked = git(
        repo_root,
        &["ls-files", "--others", "--exclude-standard", "--", "*.rs"],
    )?;
    for path in untracked.lines() {
        if let Ok(source) = std::fs::read_to_string(repo_root.join(path)) {
            diff += &untracked_diff(path, &source);
        }
    }
    let rows = count(&diff, |side, path| match side {
        Side::Old => git(repo_root, &["show", &format!("{from}:{path}")]).ok(),
        Side::New => std::fs::read_to_string(repo_root.join(path)).ok(),
    });
    let short: String = from.chars().take(10).collect();
    Ok(format!(
        "code-lines: production Rust lines, {base} (from {short}) to the working tree\n{}",
        table(&rows)
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// No file text: no `#[cfg(test)]` ranges.
    fn none(_: Side, _: &str) -> Option<String> {
        None
    }

    fn row(krate: &str, added: usize, removed: usize) -> CrateLines {
        CrateLines {
            krate: krate.to_owned(),
            added,
            removed,
        }
    }

    const DIFF: &str = "diff --git a/crates/ui/src/a.rs b/crates/ui/src/a.rs\n\
        --- a/crates/ui/src/a.rs\n\
        +++ b/crates/ui/src/a.rs\n\
        @@ -3,2 +3,3 @@ fn f() {\n\
        -    let x = 1;\n\
        -\n\
        +    let x = 2;\n\
        +    // two\n\
        +    let y = x;\n\
        diff --git a/crates/core/src/b.rs b/crates/core/src/b.rs\n\
        --- /dev/null\n\
        +++ b/crates/core/src/b.rs\n\
        @@ -0,0 +1,2 @@\n\
        +/// Docs.\n\
        +pub fn b() {}\n";

    #[test]
    fn lines_are_grouped_per_crate_and_sorted() {
        assert_eq!(
            count(DIFF, none),
            [row("crates/core", 1, 0), row("crates/ui", 2, 1)]
        );
    }

    #[test]
    fn blank_and_comment_lines_are_skipped() {
        let diff = "--- /dev/null\n+++ b/crates/ui/src/a.rs\n@@ -0,0 +1,5 @@\n\
                    +\n+   \n+// c\n+    //! d\n+let a = 1; // e\n";
        assert_eq!(count(diff, none), [row("crates/ui", 1, 0)]);
    }

    #[test]
    fn a_removed_line_counts_as_removed() {
        let diff = "--- a/crates/ui/src/a.rs\n+++ /dev/null\n@@ -1,2 +0,0 @@\n\
                    -fn a() {}\n-fn b() {}\n";
        assert_eq!(count(diff, none), [row("crates/ui", 0, 2)]);
    }

    #[test]
    fn test_files_and_other_files_are_skipped() {
        let diff = "--- /dev/null\n+++ b/crates/ui/tests/it/main.rs\n@@ -0,0 +1 @@\n+fn t() {}\n\
                    --- /dev/null\n+++ b/crates/ui/src/a/tests.rs\n@@ -0,0 +1 @@\n+fn t() {}\n\
                    --- /dev/null\n+++ b/crates/ui/src/data_tests.rs\n@@ -0,0 +1 @@\n+fn t() {}\n\
                    --- /dev/null\n+++ b/crates/ui/src/a/tests/b.rs\n@@ -0,0 +1 @@\n+fn t() {}\n\
                    --- /dev/null\n+++ b/assets/data/a.ron\n@@ -0,0 +1 @@\n+(a: 1)\n\
                    --- /dev/null\n+++ b/docs/a.md\n@@ -0,0 +1 @@\n+text\n\
                    --- /dev/null\n+++ b/crates/ui/src/sprite_test.rs\n@@ -0,0 +1 @@\n+fn s() {}\n";
        assert_eq!(count(diff, none), [row("crates/ui", 1, 0)]);
    }

    #[test]
    fn a_test_module_is_skipped_on_both_sides() {
        let old = "fn a() {}\nfn z() {}\n#[cfg(test)]\nmod tests {\n    fn t() {}\n}\n";
        let new =
            "fn a() {}\nfn b() {}\n#[cfg(test)]\nmod tests {\n    fn t() {}\n    fn u() {}\n}\n";
        // Each side's first hunk runs from a production line into the
        // test module.
        let diff = "--- a/crates/ui/src/a.rs\n+++ b/crates/ui/src/a.rs\n\
                    @@ -2,2 +2,2 @@\n-fn z() {}\n-#[cfg(test)]\n+fn b() {}\n+#[cfg(test)]\n\
                    @@ -5,0 +6 @@\n+    fn u() {}\n";
        let text = |side, path: &str| {
            assert_eq!(path, "crates/ui/src/a.rs");
            Some(if side == Side::Old { old } else { new }.to_owned())
        };
        assert_eq!(count(diff, text), [row("crates/ui", 1, 1)]);
    }

    #[test]
    fn a_changed_line_that_looks_like_a_header_is_a_line() {
        let diff = "--- a/crates/ui/src/a.rs\n+++ b/crates/ui/src/a.rs\n\
                    @@ -1 +1 @@\n--- a\n+++ b\n";
        assert_eq!(count(diff, none), [row("crates/ui", 1, 1)]);
    }

    #[test]
    fn a_renamed_file_counts_removed_lines_in_the_old_crate() {
        let diff = "--- a/crates/ui/src/a.rs\n+++ b/engine/crates/ui/src/a.rs\n\
                    @@ -1 +1 @@\n-fn a() {}\n+pub fn a() {}\n";
        assert_eq!(
            count(diff, none),
            [row("crates/ui", 0, 1), row("engine/crates/ui", 1, 0)]
        );
    }

    #[test]
    fn hunk_ranges_read_both_sides_with_or_without_counts() {
        assert_eq!(hunk_ranges("-3,2 +4,5 @@ fn f() {"), (3, 2, 4, 5));
        assert_eq!(hunk_ranges("-7 +0,0 @@"), (7, 1, 0, 0));
        assert_eq!(hunk_ranges("garbage"), (0, 0, 0, 0));
    }

    #[test]
    fn test_ranges_cover_modules_items_and_declarations() {
        let source = "fn a() {}\n\
                      #[cfg(test)]\n\
                      mod tests;\n\
                      #[cfg(test)]\n\
                      #[allow(clippy::x)]\n\
                      fn helper() -> u8 {\n\
                          let s = \"}\"; let c = '}'; let d = '\\u{7b}'; // }\n\
                          if true { 1 } else { 2 }\n\
                      }\n\
                      fn b<'a>(x: &'a str) {}\n\
                      #[cfg(test)]\n\
                      mod more {\n\
                          const S: &str = \"line one\\\n\
                          #[cfg(test)] { still a string\";\n\
                      }\n\
                      fn c() {}\n";
        assert_eq!(test_ranges(source), [(2, 3), (4, 9), (11, 15)]);
    }

    #[test]
    fn an_unclosed_test_item_runs_to_the_end() {
        assert_eq!(test_ranges("fn a() {}\n#[cfg(test)]\nmod t {\n"), [(2, 3)]);
    }

    #[test]
    fn char_literal_lengths() {
        let len = |s: &str| char_literal_len(&s.chars().collect::<Vec<_>>());
        assert_eq!(len("'x'"), 2);
        assert_eq!(len("'\\n'"), 3);
        assert_eq!(len("'\\''"), 3);
        assert_eq!(len("'\\u{7b}'"), 7);
        assert_eq!(len("'a str"), 0);
        assert_eq!(len("'\\"), 0);
        assert_eq!(len("'"), 0);
    }

    #[test]
    fn crate_of_finds_the_crate_folder() {
        assert_eq!(crate_of("crates/xtask/src/main.rs"), "crates/xtask");
        assert_eq!(
            crate_of("engine/crates/cli/examples/toy.rs"),
            "engine/crates/cli"
        );
        assert_eq!(
            crate_of("engine/examples/hello-grid/src/lib.rs"),
            "engine/examples/hello-grid"
        );
        assert_eq!(
            crate_of("engine/examples/hello-grid/build.rs"),
            "engine/examples/hello-grid"
        );
        assert_eq!(crate_of("tools/x/y.rs"), "tools/x");
        assert_eq!(
            crate_of("engine/examples/hello-grid/examples/tools.rs"),
            "engine/examples/hello-grid"
        );
        assert_eq!(crate_of("build.rs"), ".");
        assert_eq!(crate_of("src/lib.rs"), ".");
    }

    #[test]
    fn the_table_has_a_row_per_crate_and_totals() {
        let rows = [
            row("crates/core", 1, 0),
            row("crates/ui", 2, 1),
            row("engine/crates/ui", 4, 3),
        ];
        assert_eq!(
            table(&rows),
            "crate               added  removed\n\
             crates/core             1        0\n\
             crates/ui               2        1\n\
             engine/crates/ui        4        3\n\
             total (game)            3        1\n\
             total (engine)          4        3\n"
        );
        assert_eq!(
            table(&[]),
            "crate             added  removed\ntotal (game)          0        0\n"
        );
    }

    #[test]
    fn an_untracked_file_counts_as_added() {
        let diff = untracked_diff(
            "crates/ui/src/n.rs",
            "fn n() {}\n\n#[cfg(test)]\nmod t {}\n",
        );
        let text = |_, _: &str| Some("fn n() {}\n\n#[cfg(test)]\nmod t {}\n".to_owned());
        assert_eq!(count(&diff, text), [row("crates/ui", 1, 0)]);
    }

    /// The acceptance check of ticket 1151: this file's own count leaves
    /// its test module out.
    #[test]
    fn this_files_count_leaves_out_its_test_module() {
        let source = include_str!("code_lines.rs");
        let production = source
            .lines()
            .take_while(|l| !l.starts_with("#[cfg(test)]"))
            .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//"))
            .count();
        let diff = untracked_diff("crates/xtask/src/code_lines.rs", source);
        let rows = count(&diff, |_, _| Some(source.to_owned()));
        assert_eq!(rows, [row("crates/xtask", production, 0)]);
        assert!(production < source.lines().count() / 2);
    }

    #[test]
    fn run_reports_against_head_and_fails_on_a_bad_base() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let report = run(&root, "HEAD").unwrap();
        assert!(report.starts_with("code-lines: production Rust lines, HEAD (from "));
        assert!(report.contains("total (game)"));
        assert!(run(&root, "no-such-ref-1151").is_err());
    }

    #[test]
    fn git_reports_its_error() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(git(root, &["rev-parse", "HEAD"]).unwrap().trim().len() >= 40);
        assert!(git(root, &["no-such-command-1151"]).is_err());
    }
}
