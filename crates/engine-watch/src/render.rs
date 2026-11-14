//! The report as the `engine-pin` issue's title and body, and as the JSON
//! the workflow reads.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::hash::BuildHasher;

use crate::ledger::{self, Section};
use crate::{ENGINE_REPO, Kind, Report, Why};

/// What the workflow's run adds to the issue's marker.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Run {
    /// What started the run: `dispatch`, `schedule` or `manual`.
    pub trigger: String,
    /// Whether a dispatch from the engine has ever reached the watcher.
    pub dispatch_seen: bool,
    /// The engine's `main` the last dispatch reported, or empty.
    pub dispatched: String,
    /// Whether this backstop run found engine commits no dispatch reported.
    pub missed_dispatch: bool,
}

/// The first 8 hex digits of a commit.
fn short(sha: &str) -> &str {
    sha.get(..8).unwrap_or(sha)
}

/// `text` fit for a table cell: `@word` in backticks (so GitHub mentions
/// nobody, `CLAUDE.md`), `|` escaped, on one line.
fn cell(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut prev: Option<char> = None;
    while let Some(c) = chars.next() {
        let starts_word = prev.is_none_or(|p| !p.is_alphanumeric() && p != '`');
        if c == '@' && starts_word && chars.peek().is_some_and(|n| n.is_alphanumeric()) {
            out.push_str("`@");
            while let Some(&n) = chars
                .peek()
                .filter(|n| n.is_alphanumeric() || **n == '-' || **n == '_')
            {
                out.push(n);
                chars.next();
            }
            out.push('`');
            prev = Some('`');
            continue;
        }
        match c {
            '|' => out.push_str("\\|"),
            '\n' | '\r' => out.push(' '),
            _ => out.push(c),
        }
        prev = Some(c);
    }
    out
}

fn pr_link(number: u32) -> String {
    format!("[#{number}](https://github.com/{ENGINE_REPO}/pull/{number})")
}

fn plural(n: usize) -> &'static str {
    if n == 1 { "" } else { "s" }
}

/// The title's reason in brackets.
fn title_why(due: Option<&Why>) -> String {
    match due {
        Some(Why::PublicApi(n)) => format!("public API changed in #{n}"),
        Some(Why::Unblocks(what)) => format!("unblocks {what}"),
        Some(Why::Waited(day)) => format!("oldest merged {day}"),
        None => "no move due yet".to_owned(),
    }
}

/// The table of the section `pick` chooses, or "None.".
fn table(out: &mut String, report: &Report, pick: fn(&Kind) -> bool, extra: Option<&str>) {
    let rows: Vec<_> = report
        .rows
        .iter()
        .filter_map(|r| r.kinds.iter().find(|k| pick(k)).map(|k| (r, k)))
        .collect();
    if rows.is_empty() {
        out.push_str("None.\n");
        return;
    }
    let _ = writeln!(
        out,
        "| Engine PR | Ticket | Title |{}\n| --- | --- | --- |{}",
        extra.map_or(String::new(), |e| format!(" {e} |")),
        if extra.is_some() { " --- |" } else { "" },
    );
    for (row, kind) in rows {
        let more = match kind {
            Kind::Breaking(lines) | Kind::Unblocks(lines) => {
                format!(" {} |", cell(&lines.join("; ")))
            }
            Kind::Nothing if row.pr.labels.is_empty() => " no labels |".to_owned(),
            Kind::Nothing => format!(" {} |", cell(&row.pr.labels.join(", "))),
            Kind::NewForGames => String::new(),
        };
        let _ = writeln!(
            out,
            "| {} | {} | {} |{more}",
            pr_link(row.pr.number),
            row.pr.tickets.join(", "),
            cell(&row.pr.title),
        );
    }
}

/// The issue's title and body, or `None` when the pin is at `main`.
/// `open_ids` are the tickets in `tickets/open/`.
pub fn render<S: BuildHasher>(
    report: &Report,
    ledger: &[Section],
    open_ids: &HashSet<String, S>,
    run: &Run,
) -> Option<(String, String)> {
    if report.rows.is_empty() {
        return None;
    }
    let n = report.rows.len();
    let title = format!(
        "Engine pin: {n} engine PR{} behind ({})",
        plural(n),
        title_why(report.due.as_ref())
    );
    let mut body = head(report, run);
    body.push_str(
        "\n### Breaking\n\nFixed in the pin PR, on the game side, with behaviour unchanged. A `Behaviour:` line is a change the game sees without a break: the pin PR keeps the game's rule or takes the engine's, and says which. Each PR gets a ledger row. A PR also under *New for games* needs that decision too.\n\n",
    );
    table(
        &mut body,
        report,
        |k| matches!(k, Kind::Breaking(_)),
        Some("Breaking or behaviour"),
    );
    body.push_str("\n### Unblocks\n\nThese game tickets become ready; the pin PR does nothing else for them.\n\n");
    table(
        &mut body,
        report,
        |k| matches!(k, Kind::Unblocks(_)),
        Some("Game tickets"),
    );
    body.push_str("\n### New for games\n\nWhat does each let the game do, check or delete? Each gets a decision in the review: `adopt → NNNN` or `not for the game: …`.\n\n");
    table(&mut body, report, |k| *k == Kind::NewForGames, None);
    body.push_str("\n### Nothing for the game\n\nNo public API added. Read each title: a PR that changes behavior or adds a suite or check in a crate the game links gets a ledger row and a decision, whatever its label says.\n\n");
    table(&mut body, report, |k| *k == Kind::Nothing, Some("Labels"));
    body.push_str("\n### Adoption tickets still open\n\nEach once, from the pin move that brought its engine piece. One that has had it more than 5 days is marked idle.\n\n");
    body.push_str(&adoptions(ledger, open_ids, report.now));
    body.push_str("\n### Workarounds the pin ends\n\nEach `engine-friction: engine#NNNN` marker whose engine ticket this move passes. The pin PR removes it, or names the game ticket that does.\n\n");
    body.push_str(&workarounds(report, ledger));
    body.push_str(
        "\n### The work\n\nWork this issue with the `engine-pin` skill (`.claude/skills/engine-pin/SKILL.md`), in one PR, \
         using `opus-5.5 / gpt-6.1-sol` at medium effort (choose the member for your harness; see tickets/README.md):\n\n\
         - [ ] `cargo xtask engine-pin --report`, then pin the engine's `main` as it is then\n\
         - [ ] fix each *Breaking* row on the game side, behaviour unchanged\n\
         - [ ] decide each *New for games* row: `adopt → NNNN` or `not for the game: …`\n\
         - [ ] read each *Nothing for the game* title for a gain its label hides\n\
         - [ ] remove each marker under *Workarounds the pin ends*, or name the ticket that does\n\
         - [ ] write the pin move's section in `docs/engine/pins.md`\n\
         - [ ] open the PR `Engine pin: <old 8> → <new 8> (N engine PRs)` that closes this issue\n",
    );
    Some((title, body))
}

/// The marker, the gap, why a move is due and the notes.
fn head(report: &Report, run: &Run) -> String {
    let n = report.rows.len();
    let mut body = format!(
        "<!-- engine-pin:v1 main={} pin={} trigger={} dispatch_seen={} dispatched={} -->\n",
        report.main,
        report.pin,
        run.trigger,
        run.dispatch_seen,
        if run.dispatched.is_empty() {
            "none"
        } else {
            &run.dispatched
        },
    );
    let _ = writeln!(
        body,
        "The game's engine pin is `{}`; the engine's `main` is [`{}`](https://github.com/{ENGINE_REPO}/commit/{}), {n} engine PR{} ahead.",
        short(&report.pin),
        short(&report.main),
        report.main,
        plural(n),
    );
    let why = match &report.due {
        Some(Why::PublicApi(n)) => format!("**A move is due:** {} changed the public API.", pr_link(*n)),
        Some(Why::Unblocks(what)) => format!("**A move is due:** it unblocks {what}."),
        Some(Why::Waited(day)) => format!("**A move is due:** engine code merged on {day} has waited more than 48 hours."),
        None => "No move is due yet: nothing public changed, nothing is unblocked, and no engine code has waited 48 hours.".to_owned(),
    };
    let _ = writeln!(body, "\n{why}");
    if run.missed_dispatch {
        body.push_str(
            "\n> The engine's dispatch didn't reach the game: check the token (engine 2031's *Nick input*).\n",
        );
    }
    for note in &report.notes {
        let _ = writeln!(body, "\n> {}", cell(note));
    }
    body
}

/// Days an adopter ticket may wait with its engine piece in the pin before
/// the issue marks it idle (ticket 1254).
const IDLE_DAYS: i64 = 5;

/// The ledger's `adopt → NNNN` and `unblocks NNNN` tickets still in
/// `tickets/open/`, each once, from the first pin move that names it, with
/// its day, its age at `now`, and *idle* past [`IDLE_DAYS`].
fn adoptions<S: BuildHasher>(
    ledger: &[Section],
    open_ids: &HashSet<String, S>,
    now: i64,
) -> String {
    let mut seen: Vec<(String, &str, &str)> = Vec::new();
    for section in ledger {
        for row in &section.rows {
            let adopted = ledger::adopted(&row.decision)
                .into_iter()
                .map(|id| (id, "adopted"));
            let unblocked = ledger::unblocked(&row.decision)
                .into_iter()
                .map(|id| (id, "unblocked"));
            for (id, how) in adopted.chain(unblocked) {
                if open_ids.contains(&id) && !seen.iter().any(|(s, _, _)| *s == id) {
                    seen.push((id, how, &section.date));
                }
            }
        }
    }
    let lines: Vec<String> = seen
        .into_iter()
        .map(|(id, how, date)| {
            let days =
                crate::epoch_seconds(&format!("{date}T00:00:00Z")).map(|t| (now - t) / 86_400);
            let age = match days {
                Some(d) if d > IDLE_DAYS => format!(", {d} days: **idle**"),
                Some(d) => format!(", {d} day{}", plural(usize::try_from(d).unwrap_or(0))),
                None => String::new(),
            };
            format!("- {id}, {how} by the pin move of {date}{age}\n")
        })
        .collect();
    if lines.is_empty() {
        "None.\n".to_owned()
    } else {
        lines.concat()
    }
}

/// The friction markers in `crates/` for the engine tickets the move passes,
/// and the open game PRs that add one for those or for a ticket an earlier
/// move pinned (by the ledger), or "None.".
fn workarounds(report: &Report, ledger: &[Section]) -> String {
    let mut out = String::new();
    for m in &report.markers {
        let _ = writeln!(out, "- `{}:{}`: engine#{}", m.path, m.line, m.engine_id);
    }
    for pr in &report.pr_markers {
        let ids: Vec<String> = pr
            .engine_ids
            .iter()
            .filter_map(|id| {
                if report.passed.contains(id) {
                    Some(format!("engine#{id}"))
                } else {
                    ledger::pinned_on(ledger, id)
                        .map(|day| format!("engine#{id}, in the pin since {day}"))
                }
            })
            .collect();
        if !ids.is_empty() {
            let _ = writeln!(
                out,
                "- open PR #{} ({}) adds a marker for {}",
                pr.number,
                cell(&pr.title),
                ids.join(", ")
            );
        }
    }
    if out.is_empty() {
        "None.\n".to_owned()
    } else {
        out
    }
}

/// The report as Markdown, for a person: the issue's title and body, or
/// that there is nothing to move.
pub fn markdown(report: &Report, rendered: Option<&(String, String)>) -> String {
    match rendered {
        Some((title, body)) => format!("# {title}\n\n{body}"),
        None => format!(
            "The pin is at the engine's main ({}): nothing to move.\n",
            report.main
        ),
    }
}

/// The report for the workflow: the title and body (empty when the pin is
/// at `main`), why a move is due (`null` when none is), `main`, the pin and
/// how many PRs are between them.
pub fn json(report: &Report, rendered: Option<&(String, String)>) -> String {
    let (title, body) = rendered.map_or(("", ""), |(t, b)| (t.as_str(), b.as_str()));
    let value = serde_json::json!({
        "title": title,
        "body": body,
        "due": report.due.as_ref().map(|d| title_why(Some(d))),
        "main": report.main,
        "pin": report.pin,
        "behind": report.rows.len(),
    });
    value.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_at_word_comes_out_in_backticks() {
        assert_eq!(cell("the @music directive"), "the `@music` directive");
        assert_eq!(cell("@speaker-name, then"), "`@speaker-name`, then");
        assert_eq!(cell("already `@music` here"), "already `@music` here");
        assert_eq!(cell("nick@example.com"), "nick@example.com");
        assert_eq!(cell("a | b\nc"), "a \\| b c");
        assert_eq!(cell("@ alone"), "@ alone");
    }

    #[test]
    fn short_takes_eight_hex_digits() {
        assert_eq!(short("0123456789abcdef"), "01234567");
        assert_eq!(short("abc"), "abc");
    }
}
