//! The watcher on Shuyi Engine's `main` (ADR-0096, ticket 1190).
//!
//! The game builds on one engine commit, the pin (`shuyi-engine.rev`). This
//! crate lists every engine PR between the pin and the engine's `main`,
//! sorts each into *Breaking*, *Unblocks*, *New for games* or *Nothing for
//! the game* (a PR that breaks the API and adds to it is in the first and
//! the third), and says whether a pin move is due: a PR changed the public
//! API, unblocks a game ticket, or changed engine code more than 48 hours
//! ago. `.github/workflows/engine-watch.yml` files the answer as the one
//! open issue labelled `engine-pin`, and the `engine-pin` skill works it.
//!
//! Everything here is pure; [`fetch`] asks `gh` and reads the repository.
//! [`ledger`] reads `docs/engine/pins.md`, which `cargo xtask engine-pin
//! --check` checks.

pub mod fetch;
pub mod ledger;
mod render;

pub use render::{Run, json, markdown, render};

/// The engine's repository on GitHub.
pub const ENGINE_REPO: &str = "Zafnok/shuyi-engine";
/// The label of the pin-move issue.
pub const LABEL: &str = "engine-pin";
/// How long engine code waits for a move when nothing else makes one due.
const WAIT_SECS: i64 = 48 * 3600;
/// The engine's public crates by engine ADR-0038, for when `API_CRATES`
/// can't be read from its `crates/xtask/src/api.rs`.
pub const FALLBACK_API_CRATES: [&str; 6] = [
    "shuyi-core",
    "shuyi-assets",
    "shuyi-ui",
    "shuyi-platform",
    "shuyi-app",
    "shuyi-cli",
];

/// A file an engine PR changed.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PrFile {
    /// Its path in the engine's repository.
    pub path: String,
    /// Its diff, or `None` when GitHub left it out (a very large one).
    pub patch: Option<String>,
}

/// A merged engine PR between the pin and `main`.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct EnginePr {
    /// Its number in the engine's repository.
    pub number: u32,
    /// Its title.
    pub title: String,
    /// When it merged, as GitHub writes it (`2026-10-23T20:23:14Z`).
    pub merged_at: String,
    /// The commit it left on the engine's `main`.
    pub merge_sha: String,
    /// The engine tickets its title names ([`ticket_ids`]).
    pub tickets: Vec<String>,
    /// Its description.
    pub body: String,
    /// Its labels' names.
    pub labels: Vec<String>,
    /// The files it changed.
    pub files: Vec<PrFile>,
    /// The Completion notes of the tickets it moved to `tickets/done/`.
    pub ticket_notes: Option<String>,
}

/// An open game ticket's id and its `blocked_by`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct OpenTicket {
    /// The ticket's number.
    pub id: String,
    /// Its `blocked_by` ids, `engine#NNNN` for the engine's.
    pub blocked_by: Vec<String>,
}

/// A section of the report a PR goes in.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Kind {
    /// It breaks the public API: its `Breaking:` lines.
    Breaking(Vec<String>),
    /// It finishes engine tickets game work waits for: what waits.
    Unblocks(Vec<String>),
    /// It changes the public API, and needs an adoption decision.
    NewForGames,
    /// Internal code, tickets, docs, CI and the rest.
    Nothing,
}

/// Why a pin move is due.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// This PR changed the public API.
    PublicApi(u32),
    /// A PR unblocks this game ticket (or friction marker).
    Unblocks(String),
    /// Engine code merged on this day has waited more than 48 hours.
    Waited(String),
}

/// One engine PR in the report.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    /// The PR.
    pub pr: EnginePr,
    /// Its sections: one, or *Breaking* and *New for games*.
    pub kinds: Vec<Kind>,
}

/// The engine commits between the pin and `main`, as fetched.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Gap {
    /// The pinned commit.
    pub pin: String,
    /// The engine's `main`.
    pub main: String,
    /// The PRs between them, oldest first.
    pub prs: Vec<EnginePr>,
    /// What the report should say besides: commits without a PR, a
    /// fallback taken.
    pub notes: Vec<String>,
}

/// The sorted gap and whether a move is due.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Report {
    /// The pinned commit.
    pub pin: String,
    /// The engine's `main`.
    pub main: String,
    /// Every PR, oldest first.
    pub rows: Vec<Row>,
    /// Why a move is due, or `None`.
    pub due: Option<Why>,
    /// The gap's notes.
    pub notes: Vec<String>,
    /// The time it was made, in seconds since 1970.
    pub now: i64,
}

/// The digit runs of exactly four digits in `text`.
fn four_digit_ids(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_ascii_digit())
        .filter(|run| run.len() == 4)
        .map(str::to_owned)
        .collect()
}

/// The engine tickets a PR title names: `[NNNN] …`, `Ticket NNNN: …` or
/// `Tickets NNNN, NNNN and NNNN: …`.
pub fn ticket_ids(title: &str) -> Vec<String> {
    if let Some(inner) = title.strip_prefix('[').and_then(|t| t.split_once(']')) {
        return four_digit_ids(inner.0);
    }
    let named = title
        .strip_prefix("Tickets ")
        .or_else(|| title.strip_prefix("Ticket "));
    named
        .and_then(|rest| rest.split_once(':'))
        .map_or_else(Vec::new, |(ids, _)| four_digit_ids(ids))
}

/// The engine folder of an engine crate (`shuyi-core` → `crates/core/`).
pub fn crate_dir(name: &str) -> String {
    format!("crates/{}/", name.strip_prefix("shuyi-").unwrap_or(name))
}

/// The folders of the crates in `pub const API_CRATES` of the engine's
/// `crates/xtask/src/api.rs`, or `None` when it can't be read.
pub fn api_dirs(api_rs: &str) -> Option<Vec<String>> {
    let after = api_rs.split_once("pub const API_CRATES")?.1;
    // After the `=`: the type, `&[&str]`, has brackets too.
    let list = after
        .split_once('=')?
        .1
        .split_once('[')?
        .1
        .split_once(']')?
        .0;
    let names: Vec<String> = list.split('"').skip(1).step_by(2).map(crate_dir).collect();
    (!names.is_empty()).then_some(names)
}

/// The folders of [`FALLBACK_API_CRATES`].
pub fn fallback_api_dirs() -> Vec<String> {
    FALLBACK_API_CRATES.iter().map(|n| crate_dir(n)).collect()
}

/// The engine's verdict on a PR (engine#2030): `none`, `added` or
/// `breaking`, from its `api: …` label.
fn api_label(pr: &EnginePr) -> Option<&str> {
    pr.labels
        .iter()
        .filter_map(|l| l.strip_prefix("api: "))
        .find(|v| ["none", "added", "breaking"].contains(v))
}

/// Whether a patch adds or removes a line that declares a public item: a
/// line starting `pub ` (`pub fn`, `pub struct`, a `pub` field…), but not
/// `pub(crate)`, `pub(super)` or `pub(in …)`.
fn declares_public(patch: &str) -> bool {
    patch.lines().any(|line| {
        let code = line
            .strip_prefix('+')
            .or_else(|| line.strip_prefix('-'))
            .map(str::trim_start);
        code.and_then(|c| c.strip_prefix("pub"))
            .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    })
}

/// Whether a patch adds a line that declares a public item.
fn adds_public_line(patch: &str) -> bool {
    patch.lines().any(|line| {
        line.strip_prefix('+')
            .map(str::trim_start)
            .and_then(|c| c.strip_prefix("pub"))
            .is_some_and(|rest| rest.starts_with(char::is_whitespace))
    })
}

/// Whether a file is in the `src/` of a crate in `api_dirs`.
fn in_api_src(file: &PrFile, api_dirs: &[String]) -> bool {
    api_dirs
        .iter()
        .any(|dir| file.path.starts_with(&format!("{dir}src/")))
}

/// Whether a PR adds to the public API: the `api: added` label, or a public
/// item added in a `src/` file of a crate in `api_dirs` (a file without a
/// patch counts).
fn adds_public(pr: &EnginePr, api_dirs: &[String]) -> bool {
    api_label(pr) == Some("added")
        || pr.files.iter().any(|file| {
            in_api_src(file, api_dirs) && file.patch.as_deref().is_none_or(adds_public_line)
        })
}

/// What a PR breaks: its `Breaking:` lines, or its `api: breaking` label.
fn breaks(pr: &EnginePr) -> Vec<String> {
    let mut lines = breaking_lines(pr);
    if lines.is_empty() && api_label(pr) == Some("breaking") {
        lines.push("the `api: breaking` label".to_owned());
    }
    lines
}

/// Whether the report lists a PR under both *Breaking* and *New for games*.
pub fn breaks_and_adds(pr: &EnginePr, api_dirs: &[String]) -> bool {
    !breaks(pr).is_empty() && adds_public(pr, api_dirs)
}

/// Whether a PR changes the engine's public API: its `api:` label when it
/// has one, otherwise a public item declared or removed in a `src/` file of
/// a crate in `api_dirs` (a file without a patch counts). A `Breaking:`
/// line counts either way.
pub fn changes_public_api(pr: &EnginePr, api_dirs: &[String]) -> bool {
    if !breaking_lines(pr).is_empty() {
        return true;
    }
    if let Some(verdict) = api_label(pr) {
        return verdict != "none";
    }
    pr.files
        .iter()
        .any(|file| in_api_src(file, api_dirs) && file.patch.as_deref().is_none_or(declares_public))
}

/// The `Breaking:` lines of a PR's body and of its tickets' notes, without
/// the `Breaking:`. `Breaking: none` and `Breaking: no break` aren't breaks.
pub fn breaking_lines(pr: &EnginePr) -> Vec<String> {
    let texts = [Some(pr.body.as_str()), pr.ticket_notes.as_deref()];
    texts
        .into_iter()
        .flatten()
        .flat_map(str::lines)
        .filter_map(|line| {
            let line = line.trim_start();
            let line = line
                .strip_prefix("- ")
                .or_else(|| line.strip_prefix("* "))
                .unwrap_or(line);
            let what = line.strip_prefix("Breaking:")?.trim();
            let lower = what.to_ascii_lowercase();
            let none =
                what.is_empty() || lower.starts_with("none") || lower.starts_with("no break");
            (!none).then(|| what.to_owned())
        })
        // The engine copies its ticket's notes into the PR body, so the
        // same line is usually in both.
        .fold(Vec::new(), |mut lines, what| {
            if !lines.contains(&what) {
                lines.push(what);
            }
            lines
        })
}

/// The text under a ticket's `## Completion notes` heading.
pub fn completion_notes(ticket: &str) -> Option<&str> {
    let (_, notes) = ticket.split_once("## Completion notes")?;
    Some(notes.split_once("\n## ").map_or(notes, |(notes, _)| notes))
}

/// An open ticket's id (from its file name) and `blocked_by` (from its
/// frontmatter), or `None` for a file that isn't a ticket.
pub fn open_ticket(file_name: &str, text: &str) -> Option<OpenTicket> {
    let id = file_name
        .get(..4)
        .filter(|id| id.chars().all(|c| c.is_ascii_digit()))?;
    let front = text.strip_prefix("---")?.split_once("\n---")?.0;
    let list = front
        .lines()
        .find_map(|line| line.strip_prefix("blocked_by:"))
        .and_then(|rest| rest.split_once('[')?.1.split_once(']'))
        .map_or("", |(list, _)| list);
    let blocked_by = list
        .split(',')
        .map(|id| id.trim().trim_matches(['"', '\'']).to_owned())
        .filter(|id| !id.is_empty())
        .collect();
    Some(OpenTicket {
        id: id.to_owned(),
        blocked_by,
    })
}

/// The engine ticket ids of the `// engine-friction: engine#NNNN` markers in
/// a source file.
pub fn friction_markers(source: &str) -> Vec<String> {
    source
        .split("// engine-friction: engine#")
        .skip(1)
        .filter_map(|rest| {
            rest.get(..4)
                .filter(|id| id.chars().all(|c| c.is_ascii_digit()))
        })
        .map(str::to_owned)
        .collect()
}

/// What a PR unblocks: the open game tickets blocked by an engine ticket it
/// moved to `tickets/done/`, and the friction markers that name one. A PR
/// that only files a ticket unblocks nothing.
fn unblocks(pr: &EnginePr, open: &[OpenTicket], markers: &[String]) -> Vec<String> {
    let done = pr.tickets.iter().filter(|id| {
        let prefix = format!("tickets/done/{id}-");
        pr.files.iter().any(|f| f.path.starts_with(&prefix))
    });
    let mut out = Vec::new();
    for id in done {
        let engine_id = format!("engine#{id}");
        for ticket in open.iter().filter(|t| t.blocked_by.contains(&engine_id)) {
            if !out.contains(&ticket.id) {
                out.push(ticket.id.clone());
            }
        }
        if markers.contains(id) {
            out.push(format!("the `engine-friction: {engine_id}` marker"));
        }
    }
    out
}

/// Seconds since 1970 of a GitHub time (`2026-10-23T20:23:14Z`).
pub fn epoch_seconds(time: &str) -> Option<i64> {
    let num = |range: std::ops::Range<usize>| time.get(range)?.parse::<i64>().ok();
    let (y, m, d) = (num(0..4)?, num(5..7)?, num(8..10)?);
    let (hh, mm, ss) = (num(11..13)?, num(14..16)?, num(17..19)?);
    // Days from the civil date (Howard Hinnant's algorithm).
    let y = if m <= 2 { y - 1 } else { y };
    let era = y.div_euclid(400);
    let yoe = y - era * 400;
    let doy = (153 * ((m + 9) % 12) + 2) / 5 + d - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    let days = era * 146_097 + doe - 719_468;
    Some(days * 86_400 + hh * 3600 + mm * 60 + ss)
}

/// Sorts each PR of the gap and decides whether a move is due (ADR-0096 §2).
pub fn classify(
    gap: Gap,
    api_dirs: &[String],
    open: &[OpenTicket],
    markers: &[String],
    now: i64,
) -> Report {
    let mut due_api = None;
    let mut due_unblock = None;
    let mut due_wait = None;
    let mut rows = Vec::new();
    for pr in gap.prs {
        let public = changes_public_api(&pr, api_dirs);
        let breaking = breaks(&pr);
        let waits = unblocks(&pr, open, markers);
        let code = pr.files.iter().any(|f| f.path.starts_with("crates/"));
        let waited = epoch_seconds(&pr.merged_at).is_some_and(|t| now - t > WAIT_SECS);
        if public && due_api.is_none() {
            due_api = Some(Why::PublicApi(pr.number));
        }
        if let (None, Some(first)) = (&due_unblock, waits.first()) {
            due_unblock = Some(Why::Unblocks(first.clone()));
        }
        if code && waited && due_wait.is_none() {
            let day = pr.merged_at.get(..10).unwrap_or(&pr.merged_at).to_owned();
            due_wait = Some(Why::Waited(day));
        }
        let kinds = if !breaking.is_empty() {
            let mut kinds = vec![Kind::Breaking(breaking)];
            if adds_public(&pr, api_dirs) {
                kinds.push(Kind::NewForGames);
            }
            kinds
        } else if !waits.is_empty() {
            vec![Kind::Unblocks(waits)]
        } else if public {
            vec![Kind::NewForGames]
        } else {
            vec![Kind::Nothing]
        };
        rows.push(Row { pr, kinds });
    }
    Report {
        pin: gap.pin,
        main: gap.main,
        rows,
        due: due_api.or(due_unblock).or(due_wait),
        notes: gap.notes,
        now,
    }
}

#[cfg(test)]
mod tests;
