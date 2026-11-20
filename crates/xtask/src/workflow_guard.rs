//! The check that no workflow job with the private files keeps a build of
//! them where a public repository hands it out: in the Actions cache or an
//! artifact (ADR-0117 §4, ticket 0171). A build made where the content is
//! checked out embeds it, and a fork's pull request can restore `main`'s
//! caches and download any artifact.
//!
//! It reads the workflows as text, as the test beside it reads
//! `changes.yml`: jobs are the two-space-indented keys under `jobs:`, steps
//! the `- ` items of a job.

use std::collections::BTreeMap;

/// The one build ADR-0117 §4 lets leave a job as an artifact, and two
/// uploads under `dist/` that hold no build: (workflow file, job, path).
pub const ALLOWED_UPLOADS: &[(&str, &str, &str)] = &[
    // ADR-0117 §4: the playtest's trpg-tools, kept one day.
    ("playtest.yml", "build", "target/release/trpg-tools"),
    // The bench's numbers and table, no program (ticket 1244).
    ("ci.yml", "bench", "dist/bench"),
    // The release's reports and SHA-256; its own step checks no other file
    // and no line of the scripts leaves (ticket 0158).
    ("release.yml", "package", "dist/release/kept/*"),
];

/// A step: its name (or what it uses) and its lines.
struct Step<'a> {
    label: String,
    lines: Vec<&'a str>,
}

impl Step<'_> {
    /// What the step `uses:`, without the version.
    fn uses(&self) -> Option<&str> {
        self.value("uses").map(|v| v.split('@').next().unwrap_or(v))
    }

    /// The value of `key:` on any of the step's lines, comments and quotes
    /// taken off.
    fn value(&self, key: &str) -> Option<&str> {
        self.lines.iter().find_map(|line| {
            let line = uncomment(line).trim().trim_start_matches("- ");
            let (k, v) = line.split_once(':')?;
            (k.trim() == key).then(|| v.trim().trim_matches('"').trim_matches('\''))
        })
    }

    /// The values of `key:`, one per line for a block (`key: |`).
    fn values(&self, key: &str) -> Vec<String> {
        let mut out = Vec::new();
        let mut block_indent = None;
        for line in &self.lines {
            let text = uncomment(line);
            let indent = text.len() - text.trim_start().len();
            if let Some(at) = block_indent {
                if indent > at && !text.trim().is_empty() {
                    out.push(text.trim().to_string());
                    continue;
                }
                block_indent = None;
            }
            let trimmed = text.trim().trim_start_matches("- ");
            if let Some((k, v)) = trimmed.split_once(':')
                && k.trim() == key
            {
                let v = v.trim().trim_matches('"').trim_matches('\'');
                if v == "|" || v == ">" || v.is_empty() {
                    block_indent = Some(indent);
                } else {
                    out.push(v.to_string());
                }
            }
        }
        out
    }
}

/// `line` without a trailing YAML comment (` #` outside quotes is enough
/// here) or a whole-line one.
fn uncomment(line: &str) -> &str {
    if line.trim_start().starts_with('#') {
        return "";
    }
    match line.find(" #") {
        Some(at) => &line[..at],
        None => line,
    }
}

/// The jobs of a workflow, by name, each its lines.
fn jobs(text: &str) -> BTreeMap<String, Vec<&str>> {
    let mut jobs: BTreeMap<String, Vec<&str>> = BTreeMap::new();
    let mut in_jobs = false;
    let mut current: Option<String> = None;
    for line in text.lines() {
        if !line.starts_with(' ') && !line.trim().is_empty() && !line.starts_with('#') {
            in_jobs = line.trim_end() == "jobs:";
            current = None;
            continue;
        }
        if !in_jobs {
            continue;
        }
        let is_key = line.starts_with("  ")
            && !line.starts_with("   ")
            && !line.trim_start().starts_with('#')
            && line.trim_end().ends_with(':');
        if is_key {
            let name = line.trim().trim_end_matches(':').to_string();
            jobs.insert(name.clone(), Vec::new());
            current = Some(name);
        } else if let Some(name) = &current {
            jobs.get_mut(name).expect("inserted above").push(line);
        }
    }
    jobs
}

/// A job's steps: the `- ` items at the indent of its first one under
/// `steps:`.
fn steps<'a>(job: &[&'a str]) -> Vec<Step<'a>> {
    let mut steps: Vec<Step<'a>> = Vec::new();
    let Some(start) = job.iter().position(|l| l.trim_end() == "    steps:") else {
        return steps;
    };
    let mut item_indent = None;
    for line in &job[start + 1..] {
        let trimmed = line.trim_start();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if indent <= 4 {
            break;
        }
        if trimmed.starts_with("- ") && item_indent.is_none_or(|at| at == indent) {
            item_indent = Some(indent);
            steps.push(Step {
                label: String::new(),
                lines: vec![line],
            });
        } else if let Some(step) = steps.last_mut() {
            step.lines.push(line);
        }
    }
    for (i, step) in steps.iter_mut().enumerate() {
        step.label = step
            .value("name")
            .or_else(|| step.value("uses"))
            .map_or_else(|| format!("step {}", i + 1), str::to_string);
    }
    steps
}

/// Whether `path` is under `dir` (`dir/…`, or `dir` itself).
fn is_under(path: &str, dir: &str) -> bool {
    let path = path.trim_start_matches("./");
    path == dir || path.starts_with(&format!("{dir}/"))
}

/// Whether any part of `path` is a `target` folder.
fn names_target(path: &str) -> bool {
    path.split(['/', '\\']).any(|part| part == "target")
}

/// What is wrong in the workflow `file` (its file name) with text `text`:
/// one line per step, naming the workflow, the job and the step.
pub fn check(file: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (job, lines) in jobs(text) {
        let steps = steps(&lines);
        let private = steps
            .iter()
            .any(|s| s.uses() == Some("./.github/actions/private-assets"));
        if !private {
            continue;
        }
        for step in &steps {
            let mut problem = |what: String| {
                problems.push(format!(
                    "{file}: job `{job}`: step `{}`: {what}",
                    step.label
                ));
            };
            match step.uses() {
                Some("Swatinem/rust-cache") => {
                    let targets_off = step.value("cache-targets") == Some("false");
                    let defaults_stated = step.value("cache-workspace-crates") == Some("false")
                        && step.value("cache-all-crates") == Some("false");
                    if !targets_off && !defaults_stated {
                        problem(
                            "the job checks out the private files: rust-cache needs \
                             `cache-targets: false`, or `cache-workspace-crates: \"false\"` and \
                             `cache-all-crates: \"false\"` (ADR-0117 §4, ticket 0171)"
                                .to_string(),
                        );
                    }
                    for key in ["cache-workspace-crates", "cache-all-crates"] {
                        if step.value(key) == Some("true") {
                            problem(format!(
                                "`{key}: \"true\"` caches the workspace's builds, which embed \
                                 the private files (ADR-0117 §4)"
                            ));
                        }
                    }
                }
                Some("actions/cache" | "actions/cache/save") => {
                    if step.values("path").iter().any(|p| names_target(p)) {
                        problem(
                            "caches `target`, whose builds embed the private files \
                             (ADR-0117 §4)"
                                .to_string(),
                        );
                    }
                }
                Some("actions/upload-artifact") => {
                    for path in step.values("path") {
                        let allowed = ALLOWED_UPLOADS
                            .iter()
                            .any(|&(f, j, p)| f == file && j == job && p == path);
                        if !allowed && (is_under(&path, "target") || is_under(&path, "dist")) {
                            problem(format!(
                                "uploads `{path}`, a build that embeds the private files; \
                                 a public repository's artifacts are public (ADR-0117 §4)"
                            ));
                        }
                    }
                }
                _ => {}
            }
        }
    }
    problems
}

/// The workflows whose itch.io pushes run only by hand (ticket 0778): the
/// writer trial's preview page changes only when Nick runs Story kit.
pub const PUSH_BY_HAND_ONLY: &[&str] = &["story-kit.yml"];

/// The steps of the workflow `file` with text `text` that install butler or
/// run `butler push` without an `if:` requiring `workflow_dispatch`: one line
/// per step, naming the workflow, the job and the step.
pub fn pushes_not_by_hand(file: &str, text: &str) -> Vec<String> {
    let mut problems = Vec::new();
    for (job, lines) in jobs(text) {
        for step in steps(&lines) {
            let installs = step.uses() == Some("./.github/actions/butler");
            let pushes = step.values("run").iter().any(|line| {
                let line = line.to_lowercase();
                line.contains("butler") && line.contains(" push ")
            });
            // `value` takes the closing quote off a condition ending in one.
            let by_hand = step
                .value("if")
                .is_some_and(|c| c.contains("github.event_name == 'workflow_dispatch"));
            if (installs || pushes) && !by_hand {
                problems.push(format!(
                    "{file}: job `{job}`: step `{}`: pushes to itch.io without                      `github.event_name == 'workflow_dispatch'` in its `if:` (ticket 0778)",
                    step.label
                ));
            }
        }
    }
    problems
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "name: Made up\n\non:\n  push:\n\njobs:\n";

    fn job(name: &str, steps: &str) -> String {
        format!(
            "  {name}:\n    runs-on: ubuntu-latest\n    steps:\n      \
             - uses: actions/checkout@abc # v1\n      \
             - uses: ./.github/actions/private-assets\n        with:\n          \
             content: \"true\"\n{steps}"
        )
    }

    #[test]
    fn a_guarded_job_and_one_without_private_files_pass() {
        let text = format!(
            "{HEAD}{}{}{}",
            job(
                "a",
                "      - uses: Swatinem/rust-cache@abc # v2\n        with:\n          \
                 # a comment\n          cache-targets: false\n"
            ),
            job(
                "b",
                "      - uses: Swatinem/rust-cache@abc # v2\n        with:\n          \
                 key: b\n          cache-workspace-crates: \"false\"\n          \
                 cache-all-crates: \"false\"\n      \
                 - uses: actions/upload-artifact@abc\n        with:\n          \
                 path: reports/\n"
            ),
            "  public:\n    runs-on: ubuntu-latest\n    steps:\n      \
             - uses: Swatinem/rust-cache@abc # v2\n      \
             - uses: actions/upload-artifact@abc\n        with:\n          \
             path: target/release/x\n"
        );
        assert_eq!(check("made-up.yml", &text), Vec::<String>::new());
    }

    #[test]
    fn an_unguarded_rust_cache_fails_naming_the_job_and_step() {
        let text = format!(
            "{HEAD}{}",
            job(
                "build",
                "      - name: Cache\n        uses: Swatinem/rust-cache@abc # v2\n        \
                 with:\n          cache-targets: false\n          \
                 cache-all-crates: \"true\"\n      \
                 - uses: Swatinem/rust-cache@abc # v2\n"
            )
        );
        let problems = check("made-up.yml", &text);
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(
            problems[0].starts_with("made-up.yml: job `build`: step `Cache`: `cache-all-crates")
        );
        assert!(problems[1].starts_with(
            "made-up.yml: job `build`: step `Swatinem/rust-cache@abc`: the job checks out"
        ));
    }

    #[test]
    fn uploading_a_build_or_caching_target_fails() {
        let text = format!(
            "{HEAD}{}",
            job(
                "build",
                "      - uses: actions/upload-artifact@abc\n        with:\n          \
                 name: x\n          path: target/release/x\n      \
                 - uses: actions/upload-artifact@abc\n        with:\n          \
                 path: |\n            reports/\n            dist/export/web\n      \
                 - uses: actions/cache@abc\n        with:\n          \
                 path: |\n            ~/.cargo/registry\n            target\n          \
                 key: k\n"
            )
        );
        let problems = check("made-up.yml", &text);
        assert_eq!(problems.len(), 3, "{problems:#?}");
        assert!(problems[0].contains("uploads `target/release/x`"));
        assert!(problems[1].contains("uploads `dist/export/web`"));
        assert!(problems[2].contains("caches `target`"));
        // The one ADR-0117 §4 allows is allowed only where it is.
        let allowed = text.replace("x\n", "trpg-tools\n");
        let in_playtest = check("playtest.yml", &allowed);
        assert_eq!(in_playtest.len(), 2, "{in_playtest:#?}");
        assert_eq!(check("other.yml", &allowed).len(), 3);
    }

    #[test]
    fn an_itch_push_outside_a_run_by_hand_fails() {
        let push = |cond: &str| {
            format!(
                "{HEAD}  web:
    runs-on: ubuntu-latest
    steps:
                       - id: butler
        if: {cond}
        uses: ./.github/actions/butler
                       - name: Push
        if: {cond}
        run: |
                           set -e
          \"$BUTLER\" push dist/web \"$PROJECT:web\"
                       - name: Status
        run: echo pushed
"
            )
        };
        let by_hand =
            "steps.itch.outputs.ready == 'true' && github.event_name == 'workflow_dispatch'";
        assert_eq!(
            pushes_not_by_hand("x.yml", &push(by_hand)),
            Vec::<String>::new()
        );
        let problems = pushes_not_by_hand("x.yml", &push("steps.itch.outputs.ready == 'true'"));
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(problems[0].starts_with("x.yml: job `web`: step `./.github/actions/butler`"));
        assert!(problems[1].starts_with("x.yml: job `web`: step `Push`: pushes to itch.io"));
    }
}
