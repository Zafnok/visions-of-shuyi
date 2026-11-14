//! The watcher's input: the engine's PRs through `gh`, and the game's open
//! tickets, friction markers and ledger from the repository.
//!
//! Mutation testing leaves this file out (`.cargo/mutants.toml`): it only
//! asks `gh` and reads files, and the parsing it hands the answers to is
//! tested in the rest of the crate.

use std::collections::HashSet;
use std::fs;
use std::path::Path;
use std::process::Command;

use serde_json::Value;

use crate::ledger::{self, Section};
use crate::{ENGINE_REPO, EnginePr, Gap, Marker, OpenTicket, PrFile, PrMarker, Report};

/// The most commits GitHub's compare lists.
const MAX_COMMITS: u64 = 250;
/// The PRs fetched at once.
const AT_ONCE: usize = 8;

/// The report and what it is rendered with.
pub struct Watch {
    /// The sorted gap.
    pub report: Report,
    /// The ledger's sections.
    pub ledger: Vec<Section>,
    /// The tickets in `tickets/open/`.
    pub open_ids: HashSet<String>,
}

/// Runs `gh <args>`: what it printed, or why it failed.
pub fn gh(args: &[&str]) -> Result<String, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|e| format!("spawn gh: {e}"))?;
    if !output.status.success() {
        let said = String::from_utf8_lossy(&output.stderr);
        return Err(format!("gh {}: {}", args.join(" "), said.trim()));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn json(text: &str) -> Result<Value, String> {
    serde_json::from_str(text).map_err(|e| format!("gh's answer isn't JSON: {e}"))
}

/// A file of the engine at `commit`, as text.
fn engine_file(path: &str, commit: &str) -> Result<String, String> {
    gh(&[
        "api",
        &format!("repos/{ENGINE_REPO}/contents/{path}?ref={commit}"),
        "-H",
        "Accept: application/vnd.github.raw",
    ])
}

/// The PR number at the end of a squash commit's title: `… (#88)`.
fn pr_number(title: &str) -> Option<u32> {
    let inner = title.trim_end().strip_suffix(')')?.rsplit_once("(#")?.1;
    inner.parse().ok()
}

/// One engine PR with its files and its done tickets' notes.
fn engine_pr(number: u32, merge_sha: &str) -> Result<EnginePr, String> {
    let n = number.to_string();
    let view = json(&gh(&[
        "pr",
        "view",
        &n,
        "--repo",
        ENGINE_REPO,
        "--json",
        "title,body,mergedAt,labels",
    ])?)?;
    let text = |key: &str| view[key].as_str().unwrap_or_default().to_owned();
    let labels = view["labels"]
        .as_array()
        .into_iter()
        .flatten()
        .filter_map(|l| l["name"].as_str().map(str::to_owned))
        .collect();
    let files_text = gh(&[
        "api",
        &format!("repos/{ENGINE_REPO}/pulls/{n}/files"),
        "--paginate",
        "--jq",
        ".[] | {filename, patch}",
    ])?;
    let mut files = Vec::new();
    for line in files_text.lines().filter(|l| !l.trim().is_empty()) {
        let file = json(line)?;
        files.push(PrFile {
            path: file["filename"].as_str().unwrap_or_default().to_owned(),
            patch: file["patch"].as_str().map(str::to_owned),
        });
    }
    let title = text("title");
    let tickets = crate::ticket_ids(&title);
    let mut notes = Vec::new();
    for id in &tickets {
        let prefix = format!("tickets/done/{id}-");
        if let Some(file) = files.iter().find(|f| f.path.starts_with(&prefix)) {
            let ticket = engine_file(&file.path, merge_sha)?;
            notes.extend(crate::completion_notes(&ticket).map(str::to_owned));
        }
    }
    Ok(EnginePr {
        number,
        title,
        merged_at: text("mergedAt"),
        merge_sha: merge_sha.to_owned(),
        tickets,
        body: text("body"),
        labels,
        files,
        ticket_notes: (!notes.is_empty()).then(|| notes.join("\n")),
    })
}

/// The engine's PRs between `from` and `to` by their squash commits'
/// titles, oldest first, with those commits; and a note for each commit
/// without a PR.
type Numbered = (Vec<(u32, String)>, Vec<String>);

fn numbered_commits(from: &str, to: &str) -> Result<Numbered, String> {
    let compare = json(&gh(&[
        "api",
        &format!("repos/{ENGINE_REPO}/compare/{from}...{to}"),
        "--jq",
        "{status, total_commits, commits: [.commits[] | {sha, title: (.commit.message | split(\"\\n\")[0])}]}",
    ])?)?;
    match compare["status"].as_str() {
        Some("ahead" | "identical") => {}
        other => {
            return Err(format!(
                "{from} isn't on the engine's main before {to} (compare says {other:?})"
            ));
        }
    }
    let total = compare["total_commits"].as_u64().unwrap_or_default();
    if total > MAX_COMMITS {
        return Err(format!(
            "{from} is {total} engine commits behind {to}, more than GitHub's compare lists ({MAX_COMMITS}): move the pin in steps"
        ));
    }
    let mut numbered = Vec::new();
    let mut notes = Vec::new();
    for commit in compare["commits"].as_array().into_iter().flatten() {
        let sha = commit["sha"].as_str().unwrap_or_default().to_owned();
        let title = commit["title"].as_str().unwrap_or_default();
        match pr_number(title) {
            Some(n) => numbered.push((n, sha)),
            None => notes.push(format!(
                "Commit {} ({title}) has no PR, so it isn't sorted.",
                sha.get(..8).unwrap_or(&sha)
            )),
        }
    }
    Ok((numbered, notes))
}

/// The engine commits between `pin` and its `main`, with their PRs.
pub fn gap(pin: &str) -> Result<Gap, String> {
    let main = gh(&[
        "api",
        &format!("repos/{ENGINE_REPO}/commits/main"),
        "--jq",
        ".sha",
    ])?
    .trim()
    .to_owned();
    let (numbered, notes) = numbered_commits(pin, &main)?;
    let mut prs = Vec::new();
    for chunk in numbered.chunks(AT_ONCE) {
        let fetched: Vec<Result<EnginePr, String>> = std::thread::scope(|scope| {
            let handles: Vec<_> = chunk
                .iter()
                .map(|(n, sha)| scope.spawn(move || engine_pr(*n, sha)))
                .collect();
            handles
                .into_iter()
                .map(|h| {
                    h.join()
                        .unwrap_or_else(|_| Err("a fetch panicked".to_owned()))
                })
                .collect()
        });
        for pr in fetched {
            prs.push(pr?);
        }
    }
    Ok(Gap {
        pin: pin.to_owned(),
        main,
        prs,
        notes,
    })
}

/// Which of the engine PRs `numbers` the report would list under both
/// *Breaking* and *New for games*, by the public crates at `rev`. Asks `gh`
/// nothing when there are none to ask about.
pub fn breaking_and_new(numbers: &[u32], rev: &str) -> Result<Vec<u32>, String> {
    if numbers.is_empty() {
        return Ok(Vec::new());
    }
    let (dirs, _) = api_dirs(rev);
    let mut both = Vec::new();
    for &number in numbers {
        let sha = gh(&[
            "pr",
            "view",
            &number.to_string(),
            "--repo",
            ENGINE_REPO,
            "--json",
            "mergeCommit",
            "--jq",
            ".mergeCommit.oid",
        ])?;
        if crate::breaks_and_adds(&engine_pr(number, sha.trim())?, &dirs) {
            both.push(number);
        }
    }
    Ok(both)
}

/// The engine PRs a ledger section's move passed that have a `Breaking:` or
/// `Behaviour:` line and no row (ticket 1254). Asks `gh` only about the PRs
/// without a row, and nothing for a section from `baseline`.
pub fn unrowed_marked(section: &Section) -> Result<Vec<u32>, String> {
    if !ledger::has_start(section) {
        return Ok(Vec::new());
    }
    let rowed = ledger::row_prs(section);
    let (numbered, _) = numbered_commits(&section.from, &section.to)?;
    let mut marked = Vec::new();
    for (number, sha) in numbered.iter().filter(|(n, _)| !rowed.contains(n)) {
        if !crate::breaks(&engine_pr(*number, sha)?).is_empty() {
            marked.push(*number);
        }
    }
    Ok(marked)
}

/// The open game PRs that add a friction marker to `crates/`, from `gh pr
/// list` and each one's diff.
pub fn open_pr_markers() -> Result<Vec<PrMarker>, String> {
    let list = json(&gh(&[
        "pr",
        "list",
        "--state",
        "open",
        "--limit",
        "100",
        "--json",
        "number,title,files",
    ])?)?;
    let mut out = Vec::new();
    for pr in list.as_array().into_iter().flatten() {
        let touches_code = pr["files"].as_array().into_iter().flatten().any(|f| {
            f["path"]
                .as_str()
                .is_some_and(|p| p.starts_with("crates/") && crate::is_rust(p))
        });
        let Some(number) = pr["number"].as_u64().and_then(|n| u32::try_from(n).ok()) else {
            continue;
        };
        if !touches_code {
            continue;
        }
        let engine_ids = crate::added_markers(&gh(&["pr", "diff", &number.to_string()])?);
        if !engine_ids.is_empty() {
            out.push(PrMarker {
                number,
                title: pr["title"].as_str().unwrap_or_default().to_owned(),
                engine_ids,
            });
        }
    }
    Ok(out)
}

/// The folders of the engine's public crates at `main`, and a note when
/// the fallback list had to stand in.
pub fn api_dirs(main: &str) -> (Vec<String>, Option<String>) {
    let read = engine_file("crates/xtask/src/api.rs", main).ok();
    match read.as_deref().and_then(crate::api_dirs) {
        Some(dirs) => (dirs, None),
        None => (
            crate::fallback_api_dirs(),
            Some(
                "`API_CRATES` couldn't be read from the engine's `crates/xtask/src/api.rs`: \
                 the public crates are engine ADR-0038's six."
                    .to_owned(),
            ),
        ),
    }
}

/// Each `.md` file in a folder: its name and text.
fn markdown_files(dir: &Path) -> Result<Vec<(String, String)>, String> {
    let entries = fs::read_dir(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut out = Vec::new();
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        let is_md = Path::new(&name)
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("md"));
        if is_md && name != "README.md" {
            let text = fs::read_to_string(entry.path()).map_err(|e| format!("{name}: {e}"))?;
            out.push((name, text));
        }
    }
    Ok(out)
}

/// The open tickets with their `blocked_by`.
pub fn open_tickets(root: &Path) -> Result<Vec<OpenTicket>, String> {
    let files = markdown_files(&root.join("tickets/open"))?;
    Ok(files
        .iter()
        .filter_map(|(name, text)| crate::open_ticket(name, text))
        .collect())
}

/// The ids of the ticket files in `tickets/open/` and `tickets/done/`.
pub fn ticket_ids(root: &Path) -> Result<HashSet<String>, String> {
    let mut ids = HashSet::new();
    for folder in ["tickets/open", "tickets/done"] {
        let dir = root.join(folder);
        let entries = fs::read_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        for entry in entries.flatten() {
            let name = entry.file_name().to_string_lossy().into_owned();
            if let Some(id) = name
                .get(..4)
                .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
            {
                ids.insert(id.to_owned());
            }
        }
    }
    Ok(ids)
}

/// Every `// engine-friction: engine#NNNN` under `root`'s `crates/`, with
/// its file and line, leaving out this crate, whose tests write markers as
/// fixtures.
pub fn friction_markers(root: &Path) -> Vec<Marker> {
    let mut out = Vec::new();
    markers_under(root, &root.join("crates"), &mut out);
    out.sort_by(|a, b| (&a.path, a.line).cmp(&(&b.path, b.line)));
    out
}

fn markers_under(root: &Path, dir: &Path, out: &mut Vec<Marker>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name();
        if path.is_dir() && name != "target" && name != "engine-watch" {
            markers_under(root, &path, out);
        } else if path.extension().is_some_and(|e| e == "rs") {
            let text = fs::read_to_string(&path).unwrap_or_default();
            let shown = path.strip_prefix(root).unwrap_or(&path);
            let shown = shown.to_string_lossy().replace('\\', "/");
            for (line, engine_id) in crate::marker_lines(&text) {
                out.push(Marker {
                    path: shown.clone(),
                    line,
                    engine_id,
                });
            }
        }
    }
}

/// The ledger's sections.
pub fn ledger(root: &Path) -> Result<Vec<Section>, String> {
    let text = fs::read_to_string(root.join(ledger::PATH))
        .map_err(|e| format!("{}: {e}", ledger::PATH))?;
    ledger::parse(&text)
}

/// The pin, from `shuyi-engine.rev`.
pub fn pin(root: &Path) -> Result<String, String> {
    let text = fs::read_to_string(root.join("shuyi-engine.rev"))
        .map_err(|e| format!("shuyi-engine.rev: {e}"))?;
    Ok(text.trim().to_owned())
}

/// Everything the report needs, fetched and sorted. `now` is in seconds
/// since 1970.
pub fn watch(root: &Path, now: i64) -> Result<Watch, String> {
    let mut gap = gap(&pin(root)?)?;
    let (dirs, note) = api_dirs(&gap.main);
    gap.notes.extend(note);
    let open = open_tickets(root)?;
    let open_ids = open.iter().map(|t| t.id.clone()).collect();
    let markers = friction_markers(root);
    let mut report = crate::classify(gap, &dirs, &open, &markers, now);
    match open_pr_markers() {
        Ok(found) => report.pr_markers = found,
        Err(e) => report.notes.push(format!(
            "The open PRs couldn't be read for friction markers: {e}"
        )),
    }
    Ok(Watch {
        report,
        ledger: ledger(root)?,
        open_ids,
    })
}

/// The time now, in seconds since 1970.
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn root() -> &'static Path {
        Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."))
    }

    #[test]
    fn a_squash_commit_names_its_pr_at_the_end() {
        assert_eq!(pr_number("[2013] Public types (#88)"), Some(88));
        assert_eq!(pr_number("Fix (#12) and more (#105) "), Some(105));
        assert_eq!(pr_number("A direct push"), None);
        assert_eq!(pr_number("Odd (#x)"), None);
    }

    #[test]
    fn the_real_repository_reads() {
        let open = open_tickets(root()).unwrap();
        assert!(
            open.iter()
                .any(|t| t.blocked_by.iter().any(|b| b.starts_with("engine#")))
        );
        let ids = ticket_ids(root()).unwrap();
        assert!(ids.contains("1190") && ids.contains("0101"));
        assert!(
            friction_markers(root())
                .iter()
                .all(|m| m.engine_id.len() == 4 && m.path.starts_with("crates/") && m.line > 0)
        );
        assert_ne!(ledger(root()).unwrap(), [] as [crate::ledger::Section; 0]);
        assert_eq!(pin(root()).unwrap().len(), 40);
    }
}
