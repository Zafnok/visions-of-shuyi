//! The report as the `engine-pin` issue's title and body, and as the JSON
//! the workflow reads.

use std::collections::HashSet;
use std::fmt::Write as _;
use std::hash::BuildHasher;

use crate::ledger::{self, Section};
use crate::{ENGINE_REPO, Kind, Report, Row, Why};

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

/// One section's table, or "None.".
fn table(out: &mut String, rows: &[&Row], extra: Option<&str>) {
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
    for row in rows {
        let more = match &row.kind {
            Kind::Breaking(lines) | Kind::Unblocks(lines) => {
                format!(" {} |", cell(&lines.join("; ")))
            }
            Kind::NewForGames | Kind::Nothing => String::new(),
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
    let of = |pick: fn(&Kind) -> bool| {
        report
            .rows
            .iter()
            .filter(|r| pick(&r.kind))
            .collect::<Vec<_>>()
    };
    body.push_str(
        "\n### Breaking\n\nFixed in the pin PR, on the game side, with behaviour unchanged.\n\n",
    );
    table(
        &mut body,
        &of(|k| matches!(k, Kind::Breaking(_))),
        Some("Breaking"),
    );
    body.push_str("\n### Unblocks\n\nThese game tickets become ready; the pin PR does nothing else for them.\n\n");
    table(
        &mut body,
        &of(|k| matches!(k, Kind::Unblocks(_))),
        Some("Game tickets"),
    );
    body.push_str("\n### New for games\n\nEach gets a decision in the review: `adopt → NNNN` or `not for the game: …`.\n\n");
    table(&mut body, &of(|k| *k == Kind::NewForGames), None);
    let nothing = of(|k| *k == Kind::Nothing);
    let _ = writeln!(
        body,
        "\n### Nothing for the game\n\n<details><summary>{} PR{}</summary>\n",
        nothing.len(),
        plural(nothing.len())
    );
    table(&mut body, &nothing, None);
    body.push_str("\n</details>\n\n### Adoption tickets still open\n\n");
    body.push_str(&adoptions(ledger, open_ids, report.now));
    body.push_str(
        "\n### The work\n\nWork this issue with the `engine-pin` skill (`.claude/skills/engine-pin/SKILL.md`), in one PR, \
         on Opus 5.5 at medium effort:\n\n\
         - [ ] `cargo xtask engine-pin --report`, then pin the engine's `main` as it is then\n\
         - [ ] fix each *Breaking* row on the game side, behaviour unchanged\n\
         - [ ] decide each *New for games* row: `adopt → NNNN` or `not for the game: …`\n\
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

/// The ledger's `adopt → NNNN` tickets still in `tickets/open/`, with the
/// day of their pin move and their age at `now`.
fn adoptions<S: BuildHasher>(
    ledger: &[Section],
    open_ids: &HashSet<String, S>,
    now: i64,
) -> String {
    let lines: Vec<String> = ledger
        .iter()
        .flat_map(|s| {
            s.rows
                .iter()
                .flat_map(|r| ledger::adopted(&r.decision))
                .map(|id| (id, s.date.clone()))
        })
        .filter(|(id, _)| open_ids.contains(id))
        .map(|(id, date)| {
            let days = crate::epoch_seconds(&format!("{date}T00:00:00Z"))
                .map_or(String::new(), |t| format!(", {} days", (now - t) / 86_400));
            format!("- {id}, adopted by the pin move of {date}{days}\n")
        })
        .collect();
    if lines.is_empty() {
        "None.\n".to_owned()
    } else {
        lines.concat()
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
