//! Two rules of the workflows that keep CI's minutes on runs a change can
//! fail (ticket 0174), read from the workflows as text, as
//! `workflow_guard` reads them:
//!
//! - every `Swatinem/rust-cache` step saves only on `main`, so a pull
//!   request's caches never evict `main`'s (10.52 GB against the 10 GB cap);
//! - the workflows `changes.yml` counts as docs ([`LIGHT`]) are called by
//!   no pull-request workflow, so a change to one can't break a pull
//!   request's checks.

use std::collections::BTreeSet;

/// The `save-if:` every rust-cache step carries.
pub const SAVE_IF: &str = "save-if: ${{ github.ref == 'refs/heads/main' }}";

/// The workflows and actions no pull-request workflow calls, and that are
/// no pull request's check: `changes.yml`'s `workflow, tested by xtask`
/// line lists the same paths. A path ending in `/*` is every file of that
/// folder. Not the butler action: `demo.yml` runs it on pull requests.
pub const LIGHT: &[&str] = &[
    ".github/workflows/release.yml",
    ".github/workflows/story-kit.yml",
    ".github/workflows/story-mirror.yml",
    ".github/workflows/playtest-itch.yml",
    ".github/workflows/engine-watch.yml",
    ".github/workflows/sonar-triage.yml",
    ".github/workflows/scorecard.yml",
    ".github/dependabot.yml",
];

/// `line` without a trailing comment (` #`) or a whole-line one.
fn uncomment(line: &str) -> &str {
    if line.trim_start().starts_with('#') {
        return "";
    }
    line.find(" #").map_or(line, |at| &line[..at])
}

fn indent(line: &str) -> usize {
    line.len() - line.trim_start().len()
}

/// The rust-cache steps of the workflow `file` with text `text` that don't
/// carry [`SAVE_IF`]: one line each, naming the file and the line.
pub fn caches_saving_off_main(file: &str, text: &str) -> Vec<String> {
    let lines: Vec<&str> = text.lines().collect();
    let mut problems = Vec::new();
    for (at, line) in lines.iter().enumerate() {
        let code = uncomment(line).trim_start();
        let dashed = code.starts_with("- ");
        if !code
            .trim_start_matches("- ")
            .starts_with("uses: Swatinem/rust-cache@")
        {
            continue;
        }
        // The step's keys sit at the `uses:` key's indent; the step ends at
        // a line indented less, or at the next `- ` item at the dash's.
        let keys = indent(line) + if dashed { 2 } else { 0 };
        let item = keys - 2;
        let step = lines[at + 1..].iter().take_while(|l| {
            let text = uncomment(l);
            text.trim().is_empty()
                || (indent(text) >= keys
                    && !(indent(text) == item && text.trim_start().starts_with("- ")))
        });
        let saves_on_main = step.into_iter().any(|l| uncomment(l).trim() == SAVE_IF);
        if !saves_on_main {
            problems.push(format!(
                "{file}:{}: a rust-cache step without `{SAVE_IF}` saves a cache from every \
                 pull request (ticket 0174)",
                at + 1
            ));
        }
    }
    problems
}

/// The lines of the top-level block `key:` (`on:`), without the key.
fn block<'a>(text: &'a str, key: &str) -> Vec<&'a str> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.lines() {
        let top = !line.starts_with(' ') && !uncomment(line).trim().is_empty();
        if top {
            inside = uncomment(line).trim_end() == format!("{key}:");
            continue;
        }
        if inside {
            out.push(line);
        }
    }
    out
}

/// The `on:` lines of the `pull_request` (or `pull_request_target`)
/// trigger, `None` when the workflow has none. A one-line `on: [...]` with
/// it gives an empty block.
fn pull_request_trigger(text: &str) -> Option<Vec<&str>> {
    let inline = text.lines().find(|l| l.starts_with("on:")).map(uncomment)?;
    let inline = inline.trim_start_matches("on:").trim();
    if !inline.is_empty() {
        return inline.contains("pull_request").then(Vec::new);
    }
    let on = block(text, "on");
    let start = on.iter().position(|l| {
        let t = uncomment(l);
        indent(t) == 2
            && matches!(
                t.trim().trim_end_matches(':'),
                "pull_request" | "pull_request_target"
            )
    })?;
    Some(
        on[start + 1..]
            .iter()
            .take_while(|l| uncomment(l).trim().is_empty() || indent(uncomment(l)) > 2)
            .copied()
            .collect(),
    )
}

/// The `paths:` of a trigger's lines, as a list or one line `[a, b]`.
fn trigger_paths(trigger: &[&str]) -> Option<Vec<String>> {
    let at = trigger
        .iter()
        .position(|l| uncomment(l).trim_start().starts_with("paths:"))?;
    let unquote = |s: &str| s.trim().trim_matches('"').trim_matches('\'').to_string();
    let rest = uncomment(trigger[at]).trim_start()["paths:".len()..].trim();
    if let Some(list) = rest.strip_prefix('[') {
        return Some(list.trim_end_matches(']').split(',').map(unquote).collect());
    }
    Some(
        trigger[at + 1..]
            .iter()
            .map(|l| uncomment(l).trim())
            .take_while(|l| l.is_empty() || l.starts_with("- "))
            .filter(|l| !l.is_empty())
            .map(|l| unquote(&l[2..]))
            .collect(),
    )
}

/// The local files a workflow or action `uses:`: `./.github/…` with no
/// version, as paths from the root (an action's folder ends in `/`).
fn local_uses(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(|l| {
            let code = uncomment(l).trim().trim_start_matches("- ");
            let target = code.strip_prefix("uses:")?.trim().trim_matches('"');
            let path = target.strip_prefix("./")?;
            let workflow = std::path::Path::new(path)
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("yml") || e.eq_ignore_ascii_case("yaml"));
            Some(if workflow {
                path.to_string()
            } else {
                format!("{}/", path.trim_end_matches('/'))
            })
        })
        .collect()
}

/// Whether `path` (a workflow's, or an action folder's ending in `/`) is on
/// the list `light`.
fn is_light(light: &[&str], path: &str) -> bool {
    light.iter().any(|&l| match l.strip_suffix('*') {
        Some(folder) => path.starts_with(folder),
        None => l == path,
    })
}

/// What breaks the [`LIGHT`] rule among `files`, each (path from the root,
/// text): a pull-request workflow that reaches a light workflow or action
/// through `uses:` (directly or through a workflow or action it calls),
/// and a light workflow whose own `pull_request` trigger has no `paths`, or
/// names a path that is neither its own file nor light.
pub fn light_problems(light: &[&str], files: &[(String, String)]) -> Vec<String> {
    let mut problems = Vec::new();
    let text_of = |path: &str| {
        files.iter().find_map(|(p, t)| {
            let action = format!("{path}action.yml");
            (p == path || p == &action).then_some(t.as_str())
        })
    };
    for (path, text) in files {
        let Some(trigger) = pull_request_trigger(text) else {
            continue;
        };
        if is_light(light, path) {
            match trigger_paths(&trigger) {
                None => problems.push(format!(
                    "{path}: a light workflow runs on every pull request: give its \
                     `pull_request` trigger `paths` naming only its own files"
                )),
                Some(paths) => {
                    for p in paths.iter().filter(|p| *p != path && !is_light(light, p)) {
                        problems.push(format!(
                            "{path}: its `pull_request` trigger's `paths` names `{p}`, \
                             not one of its own files"
                        ));
                    }
                }
            }
            continue;
        }
        // Every local file it reaches, through calls of calls.
        let mut seen = BTreeSet::new();
        let mut todo: Vec<String> = local_uses(text).into_iter().collect();
        while let Some(used) = todo.pop() {
            if !seen.insert(used.clone()) {
                continue;
            }
            if is_light(light, &used) {
                problems.push(format!(
                    "{path}: a pull-request workflow uses `{used}`, which changes.yml counts \
                     as docs (ticket 0174): take it off the list, or the call out"
                ));
            }
            if let Some(text) = text_of(&used) {
                todo.extend(local_uses(text));
            }
        }
    }
    problems
}

/// The paths of `changes.yml`'s `workflow, tested by xtask` case line.
pub fn changes_light_line(changes: &str) -> Vec<String> {
    changes
        .lines()
        .find(|l| l.contains("echo \"workflow, tested by xtask: "))
        .and_then(|l| l.trim().split_once(')'))
        .map_or_else(Vec::new, |(paths, _)| {
            paths.split('|').map(str::to_string).collect()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    const CACHED: &str = "name: Made up

on:
  push:

jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@abc # v1
      - uses: Swatinem/rust-cache@abc # v2
        with:
          cache-targets: false
          save-if: ${{ github.ref == 'refs/heads/main' }}
      - name: Cache again
        uses: Swatinem/rust-cache@abc # v2
        with:
          # a comment
          save-if: ${{ github.ref == 'refs/heads/main' }}
      - run: cargo test
";

    #[test]
    fn a_rust_cache_saving_only_on_main_passes_and_one_without_fails() {
        assert_eq!(
            caches_saving_off_main("x.yml", CACHED),
            Vec::<String>::new()
        );
        // Without it, or with it on the next step only.
        let bare = CACHED.replacen(
            "          save-if: ${{ github.ref == 'refs/heads/main' }}\n",
            "",
            1,
        );
        let problems = caches_saving_off_main("x.yml", &bare);
        assert_eq!(problems.len(), 1, "{problems:#?}");
        assert!(problems[0].starts_with("x.yml:11: a rust-cache step"));
        let commented = CACHED.replace(
            "          save-if: ${{ github.ref == 'refs/heads/main' }}",
            "          # save-if: ${{ github.ref == 'refs/heads/main' }}",
        );
        assert_eq!(caches_saving_off_main("x.yml", &commented).len(), 2);
        let next_step = "      - uses: Swatinem/rust-cache@abc\n      - uses: x/y@abc\n        with:\n          save-if: ${{ github.ref == 'refs/heads/main' }}\n";
        assert_eq!(caches_saving_off_main("x.yml", next_step).len(), 1);
    }

    fn files(list: &[(&str, &str)]) -> Vec<(String, String)> {
        list.iter()
            .map(|(p, t)| ((*p).to_string(), (*t).to_string()))
            .collect()
    }

    const LIGHT_LIST: &[&str] = &[".github/workflows/release.yml", ".github/actions/butler/*"];

    const RELEASE: &str = "on:\n  push:\n    tags: ['v*']\n  pull_request:\n    paths: [.github/workflows/release.yml]\njobs:\n  a:\n    steps:\n      - uses: ./.github/actions/butler\n";
    const CI: &str = "on:\n  pull_request:\n    types: [opened]\n  push:\njobs:\n  changes:\n    uses: ./.github/workflows/changes.yml\n  a:\n    steps:\n      - uses: ./.github/actions/private-assets\n";
    const CHANGES: &str =
        "on:\n  workflow_call:\njobs:\n  d:\n    steps:\n      - uses: actions/checkout@abc\n";
    const PRIVATE: &str = "runs:\n  using: composite\n  steps:\n    - uses: actions/checkout@abc\n";

    #[test]
    fn a_light_workflow_no_pull_request_workflow_calls_passes() {
        let all = files(&[
            (".github/workflows/release.yml", RELEASE),
            (".github/workflows/ci.yml", CI),
            (".github/workflows/changes.yml", CHANGES),
            (".github/actions/private-assets/action.yml", PRIVATE),
            (".github/actions/butler/action.yml", PRIVATE),
        ]);
        assert_eq!(light_problems(LIGHT_LIST, &all), Vec::<String>::new());
    }

    #[test]
    fn a_light_workflow_a_pull_request_workflow_reaches_fails() {
        // Called by a workflow ci.yml calls, and an action used directly.
        let changes = CHANGES.replace(
            "actions/checkout@abc",
            "./.github/actions/butler # through a call",
        );
        let ci = CI.replace(
            "./.github/actions/private-assets",
            "./.github/workflows/release.yml",
        );
        let all = files(&[
            (".github/workflows/ci.yml", &ci),
            (".github/workflows/changes.yml", &changes),
        ]);
        let problems = light_problems(LIGHT_LIST, &all);
        assert_eq!(problems.len(), 2, "{problems:#?}");
        assert!(
            problems
                .iter()
                .all(|p| p.starts_with(".github/workflows/ci.yml: a pull-request"))
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("`.github/actions/butler/`"))
        );
        assert!(
            problems
                .iter()
                .any(|p| p.contains("`.github/workflows/release.yml`"))
        );
    }

    #[test]
    fn a_light_workflow_on_every_pull_request_fails() {
        let everywhere = RELEASE.replace("    paths: [.github/workflows/release.yml]\n", "");
        let wider = RELEASE.replace(
            "[.github/workflows/release.yml]",
            "[.github/workflows/release.yml, \"crates/**\"]",
        );
        let listed = RELEASE.replace(
            "    paths: [.github/workflows/release.yml]\n",
            "    paths:\n      - .github/workflows/release.yml\n      - .github/actions/butler/action.yml\n",
        );
        let check =
            |t: &str| light_problems(LIGHT_LIST, &files(&[(".github/workflows/release.yml", t)]));
        assert!(check(&everywhere)[0].contains("runs on every pull request"));
        assert!(check(&wider)[0].contains("names `crates/**`"));
        assert_eq!(check(&listed), Vec::<String>::new());
    }

    #[test]
    fn the_pull_request_trigger_is_found_in_each_spelling() {
        assert!(pull_request_trigger("on: [push, pull_request]\n").is_some());
        assert!(pull_request_trigger("on: push\n").is_none());
        assert!(pull_request_trigger("on:\n  pull_request_target:\n").is_some());
        assert!(pull_request_trigger("on:\n  push:\n    branches: [pull_request]\n").is_none());
        assert_eq!(
            changes_light_line("  a|b/*) echo \"workflow, tested by xtask: $path\"; x ;;\n"),
            ["a", "b/*"]
        );
    }
}
