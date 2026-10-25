use std::collections::HashSet;

use super::*;
use crate::ledger;

const HOUR: i64 = 3600;
const MERGED: &str = "2026-10-23T20:23:14Z";

fn file(path: &str, diff: &str) -> PrFile {
    PrFile {
        path: path.to_owned(),
        patch: Some(diff.to_owned()),
    }
}

fn pr(number: u32, title: &str, files: Vec<PrFile>) -> EnginePr {
    EnginePr {
        number,
        title: title.to_owned(),
        merged_at: MERGED.to_owned(),
        merge_sha: format!("{number:040}"),
        tickets: ticket_ids(title),
        files,
        ..EnginePr::default()
    }
}

fn labelled(mut pr: EnginePr, label: &str) -> EnginePr {
    pr.labels.push(label.to_owned());
    pr
}

fn dirs() -> Vec<String> {
    fallback_api_dirs()
}

fn merged() -> i64 {
    epoch_seconds(MERGED).unwrap()
}

fn gap(prs: Vec<EnginePr>) -> Gap {
    Gap {
        pin: "3f2256d46ac066ee4656489d6d63d007990eb8be".to_owned(),
        main: "6e7f71fd10e6ef49bc0c6f0f2250a9527c29a1e8".to_owned(),
        prs,
        notes: Vec::new(),
    }
}

fn open_1180() -> Vec<OpenTicket> {
    vec![OpenTicket {
        id: "1180".to_owned(),
        blocked_by: vec!["0506".to_owned(), "engine#2131".to_owned()],
    }]
}

/// An engine PR that finishes ticket 2131 (moves it to `tickets/done/`).
fn finishes_2131(extra: Vec<PrFile>) -> EnginePr {
    let mut files = vec![file("tickets/done/2131-threat-areas.md", "+done")];
    files.extend(extra);
    pr(102, "[2131] Threat areas in the tactics kit", files)
}

fn api_change() -> PrFile {
    file(
        "crates/core/src/threat.rs",
        "@@ -1 +1,2 @@\n+pub fn threat() {}\n",
    )
}

fn internal_change() -> PrFile {
    file(
        "crates/core/src/lib.rs",
        "@@ -1 +1 @@\n-fn old() {}\n+fn new() {}\n",
    )
}

fn kinds(report: &Report) -> Vec<(u32, Kind)> {
    report
        .rows
        .iter()
        .map(|r| (r.pr.number, r.kind.clone()))
        .collect()
}

#[test]
fn a_ticket_id_comes_from_each_title_form() {
    assert_eq!(ticket_ids("[2013] Public types that can grow"), ["2013"]);
    assert_eq!(
        ticket_ids("Ticket 3056: One-bit sprites drawn fast"),
        ["3056"]
    );
    assert_eq!(
        ticket_ids("Tickets 3063, 2553, 3064 and 2555: perf traces"),
        ["3063", "2553", "3064", "2555"]
    );
    assert_eq!(
        ticket_ids("Tickets 2741-2768: dungeon crawlers"),
        ["2741", "2768"]
    );
    assert!(ticket_ids("Bump rust-toolchain in / (#83)").is_empty());
    assert!(ticket_ids("[fix] typo").is_empty());
    assert!(ticket_ids("A title with 2026 in it").is_empty());
}

#[test]
fn api_crates_come_from_the_engine_s_api_rs() {
    let api_rs = "//! The api check.\n\n/// The crates.\npub const API_CRATES: &[&str] = &[\n    \"shuyi-core\",\n    \"shuyi-ui\",\n    \"shuyi\",\n];\n\npub const OTHER: &[&str] = &[\"x\"];\n";
    assert_eq!(
        api_dirs(api_rs).unwrap(),
        ["crates/core/", "crates/ui/", "crates/shuyi/"]
    );
    assert_eq!(api_dirs("pub const API_CRATES: &[&str] = &[];"), None);
    assert_eq!(api_dirs("fn main() {}"), None);
    let fallback = fallback_api_dirs();
    assert_eq!(fallback.len(), 6);
    assert_eq!(fallback[0], "crates/core/");
    assert_eq!(fallback[5], "crates/cli/");
}

#[test]
fn a_public_item_in_an_api_crate_s_src_is_an_api_change() {
    let public = |f: PrFile| changes_public_api(&pr(1, "[2000] x", vec![f]), &dirs());
    assert!(public(api_change()));
    assert!(public(file(
        "crates/ui/src/a.rs",
        "-    pub name: String,\n"
    )));
    assert!(public(file(
        "crates/app/src/a.rs",
        "+\tpub const fn new() -> Self {\n"
    )));
    assert!(public(PrFile {
        path: "crates/core/src/huge.rs".to_owned(),
        patch: None,
    }));
    assert!(!public(file(
        "crates/core/src/a.rs",
        "+pub(crate) fn hidden() {}\n"
    )));
    assert!(!public(file(
        "crates/core/src/a.rs",
        "+    pub(in crate::x) fn y() {}\n"
    )));
    assert!(!public(file(
        "crates/core/src/a.rs",
        " pub fn context_only() {}\n"
    )));
    assert!(!public(file(
        "crates/core/src/a.rs",
        "+/// pub fn in a comment\n"
    )));
    assert!(!public(file("crates/core/src/a.rs", "+public_thing();\n")));
    assert!(!public(file(
        "crates/core/tests/it/a.rs",
        "+pub fn helper() {}\n"
    )));
    assert!(!public(file(
        "crates/xtask/src/api.rs",
        "+pub fn run() {}\n"
    )));
    assert!(!public(file(
        "tickets/open/2000-x.md",
        "+pub fn in a ticket\n"
    )));
}

#[test]
fn the_api_label_wins_over_the_patch() {
    let with_pub = pr(1, "[2000] x", vec![api_change()]);
    assert!(!changes_public_api(
        &labelled(with_pub.clone(), "api: none"),
        &dirs()
    ));
    let no_pub = pr(2, "[2001] y", vec![internal_change()]);
    assert!(!changes_public_api(&no_pub, &dirs()));
    assert!(changes_public_api(
        &labelled(no_pub.clone(), "api: added"),
        &dirs()
    ));
    assert!(changes_public_api(
        &labelled(no_pub.clone(), "api: breaking"),
        &dirs()
    ));
    // Another label of the same shape is no verdict.
    assert!(changes_public_api(
        &labelled(with_pub, "api: maybe"),
        &dirs()
    ));
    let report = classify(
        gap(vec![labelled(no_pub, "api: breaking")]),
        &dirs(),
        &[],
        &[],
        merged(),
    );
    assert_eq!(
        kinds(&report),
        [(
            2,
            Kind::Breaking(vec!["the `api: breaking` label".to_owned()])
        )]
    );
    assert_eq!(report.due, Some(Why::PublicApi(2)));
}

#[test]
fn breaking_lines_come_from_the_body_and_the_ticket_notes() {
    let mut breaking = pr(88, "[2013] Public types", vec![internal_change()]);
    breaking.body = "## API changes\n\nBreaking: shuyi-app: Window: build it with Window::new\n- Breaking: shuyi-ui: BoxPlace::new\nThe contract doesn't change.\n".to_owned();
    assert_eq!(
        breaking_lines(&breaking),
        [
            "shuyi-app: Window: build it with Window::new",
            "shuyi-ui: BoxPlace::new"
        ]
    );
    let mut noted = pr(90, "[2024] More constructors", vec![]);
    noted.ticket_notes = Some("\n- Done.\n  Breaking: shuyi-core: Grid::new\n".to_owned());
    assert_eq!(breaking_lines(&noted), ["shuyi-core: Grid::new"]);
    let mut none = pr(91, "[2025] No breaks", vec![]);
    none.body = "No breaking change.\nNo break here.\nA `Breaking:` line is required.\n".to_owned();
    none.ticket_notes = Some("Breaking: none\nBreaking: no break\nBreaking:\n".to_owned());
    assert!(breaking_lines(&none).is_empty());
    // A `Breaking:` line counts as an API change, whatever the label says.
    assert!(changes_public_api(&labelled(noted, "api: none"), &dirs()));
}

#[test]
fn breaking_lines_are_listed_once_across_the_body_and_the_notes() {
    let mut copied = pr(88, "[2013] Public types", vec![internal_change()]);
    copied.body =
        "Breaking: shuyi-app: Window::new\nBreaking: shuyi-ui: BoxPlace::new\n".to_owned();
    copied.ticket_notes = Some(
        "- Breaking: shuyi-ui: BoxPlace::new\nBreaking: shuyi-app: Window::new\nBreaking: shuyi-cli: LogEntry\n"
            .to_owned(),
    );
    assert_eq!(
        breaking_lines(&copied),
        [
            "shuyi-app: Window::new",
            "shuyi-ui: BoxPlace::new",
            "shuyi-cli: LogEntry"
        ]
    );
}

#[test]
fn a_breaking_pr_that_also_unblocks_is_in_breaking() {
    let mut pr = finishes_2131(vec![api_change()]);
    pr.body = "Breaking: shuyi-core: threat moved\n".to_owned();
    let report = classify(gap(vec![pr]), &dirs(), &open_1180(), &[], merged());
    assert_eq!(
        kinds(&report),
        [(
            102,
            Kind::Breaking(vec!["shuyi-core: threat moved".to_owned()])
        )]
    );
}

#[test]
fn each_pr_goes_in_the_first_section_that_fits() {
    let prs = vec![
        finishes_2131(vec![api_change()]),
        pr(78, "[2511] A prelude", vec![api_change()]),
        pr(
            75,
            "Tickets 2741-2768: dungeon crawlers",
            vec![file("tickets/open/2741-x.md", "+pub fn")],
        ),
        pr(79, "[2012] The pinned Rust", vec![internal_change()]),
    ];
    let report = classify(gap(prs), &dirs(), &open_1180(), &[], merged());
    assert_eq!(
        kinds(&report),
        [
            (102, Kind::Unblocks(vec!["1180".to_owned()])),
            (78, Kind::NewForGames),
            (75, Kind::Nothing),
            (79, Kind::Nothing),
        ]
    );
}

#[test]
fn only_a_ticket_moved_to_done_unblocks() {
    // Filing the ticket (`Ticket 2131: …` adding it to `tickets/open/`)
    // unblocks nothing.
    let filed = pr(
        94,
        "Ticket 2131: Threat areas",
        vec![file("tickets/open/2131-threat-areas.md", "+")],
    );
    let report = classify(gap(vec![filed]), &dirs(), &open_1180(), &[], merged());
    assert_eq!(kinds(&report), [(94, Kind::Nothing)]);
    assert_eq!(report.due, None);
    // A friction marker waits as a ticket does.
    let report = classify(
        gap(vec![finishes_2131(vec![])]),
        &dirs(),
        &[],
        &["2131".to_owned()],
        merged(),
    );
    let marker = "the `engine-friction: engine#2131` marker".to_owned();
    assert_eq!(
        kinds(&report),
        [(102, Kind::Unblocks(vec![marker.clone()]))]
    );
    assert_eq!(report.due, Some(Why::Unblocks(marker)));
}

#[test]
fn a_move_is_due_for_an_api_change_an_unblock_or_code_that_waited() {
    let due = |prs: Vec<EnginePr>, open: &[OpenTicket], now: i64| {
        classify(gap(prs), &dirs(), open, &[], now).due
    };
    let api = pr(78, "[2511] A prelude", vec![api_change()]);
    let internal = pr(79, "[2012] The pinned Rust", vec![internal_change()]);
    let tickets_only = pr(
        75,
        "Tickets 2741-2768: x",
        vec![file("tickets/open/2741-x.md", "+")],
    );
    let soon = merged() + 47 * HOUR;
    let later = merged() + 49 * HOUR;
    assert_eq!(
        due(vec![internal.clone(), api], &[], soon),
        Some(Why::PublicApi(78))
    );
    assert_eq!(
        due(
            vec![internal.clone(), finishes_2131(vec![])],
            &open_1180(),
            soon
        ),
        Some(Why::Unblocks("1180".to_owned()))
    );
    assert_eq!(due(vec![internal.clone()], &[], soon), None);
    // More than 48 hours: exactly 48 isn't yet.
    assert_eq!(due(vec![internal.clone()], &[], merged() + 48 * HOUR), None);
    assert_eq!(
        due(vec![internal], &[], later),
        Some(Why::Waited("2026-10-23".to_owned()))
    );
    assert_eq!(
        due(vec![tickets_only.clone()], &[], merged() + 100 * HOUR),
        None
    );
    let mut docs = tickets_only;
    docs.files = vec![
        file("docs/adr/0039-x.md", "+pub fn"),
        file(".github/workflows/ci.yml", "+"),
    ];
    assert_eq!(due(vec![docs], &[], later), None);
}

#[test]
fn the_api_reason_comes_before_an_unblock_and_a_wait() {
    let prs = vec![
        pr(79, "[2012] The pinned Rust", vec![internal_change()]),
        finishes_2131(vec![]),
        pr(78, "[2511] A prelude", vec![api_change()]),
    ];
    let report = classify(gap(prs), &dirs(), &open_1180(), &[], merged() + 49 * HOUR);
    assert_eq!(report.due, Some(Why::PublicApi(78)));
}

#[test]
fn github_times_count_seconds_since_1970() {
    assert_eq!(epoch_seconds("1970-01-01T00:00:00Z"), Some(0));
    assert_eq!(epoch_seconds("2000-03-01T00:00:00Z"), Some(951_868_800));
    assert_eq!(epoch_seconds("2024-02-29T12:00:00Z"), Some(1_709_208_000));
    assert_eq!(epoch_seconds("2026-10-23T20:23:14Z"), Some(1_792_786_994));
    assert_eq!(epoch_seconds("yesterday"), None);
    assert_eq!(epoch_seconds(""), None);
}

#[test]
fn an_open_ticket_gives_its_id_and_blockers() {
    let text = "---\nid: \"1180\"\nblocked_by: [\"0506\", \"engine#2131\"]\nstatus: todo\n---\n\nblocked_by: [\"9999\"]\n";
    assert_eq!(
        open_ticket("1180-x.md", text),
        Some(OpenTicket {
            id: "1180".to_owned(),
            blocked_by: vec!["0506".to_owned(), "engine#2131".to_owned()],
        })
    );
    let empty = open_ticket("1190-x.md", "---\nblocked_by: []\n---\n").unwrap();
    assert!(empty.blocked_by.is_empty());
    assert_eq!(open_ticket("README.md", text), None);
    assert_eq!(open_ticket("1180-x.md", "no frontmatter"), None);
}

#[test]
fn friction_markers_and_completion_notes_are_found() {
    let source = "fn a() {\n    // engine-friction: engine#2540 (the crank)\n}\n// engine-friction: engine#25\n// engine-friction: engine#2633\n";
    assert_eq!(friction_markers(source), ["2540", "2633"]);
    let ticket = "# 2013\n\n## Context\n\nBreaking: not this\n\n## Completion notes\n\nBreaking: this\n\n## Later\n\nnot this either\n";
    assert_eq!(completion_notes(ticket), Some("\n\nBreaking: this\n"));
    assert_eq!(completion_notes("# no notes\n"), None);
}

fn run() -> Run {
    Run {
        trigger: "dispatch".to_owned(),
        dispatch_seen: true,
        dispatched: "6e7f71fd10e6ef49bc0c6f0f2250a9527c29a1e8".to_owned(),
        missed_dispatch: false,
    }
}

#[test]
fn an_empty_gap_renders_no_issue() {
    let report = classify(gap(vec![]), &dirs(), &[], &[], merged());
    assert_eq!(render(&report, &[], &HashSet::new(), &run()), None);
    let json: serde_json::Value = serde_json::from_str(&json(&report, None)).unwrap();
    assert_eq!(json["behind"], 0);
    assert_eq!(json["title"], "");
    assert!(json["due"].is_null());
    assert!(markdown(&report, None).contains("nothing to move"));
}

#[test]
fn the_issue_lists_every_pr_in_its_section() {
    let mut breaking = pr(
        88,
        "[2013] Public types for @music",
        vec![internal_change()],
    );
    breaking.body = "Breaking: shuyi-ui: BoxPlace | MapScene".to_owned();
    let prs = vec![
        breaking,
        finishes_2131(vec![]),
        pr(78, "[2511] A prelude", vec![api_change()]),
        pr(75, "Tickets 2741-2768: x", vec![]),
    ];
    let report = classify(gap(prs), &dirs(), &open_1180(), &[], merged());
    let (title, body) = render(&report, &[], &HashSet::new(), &run()).unwrap();
    assert_eq!(
        title,
        "Engine pin: 4 engine PRs behind (public API changed in #88)"
    );
    assert!(body.starts_with(
        "<!-- engine-pin:v1 main=6e7f71fd10e6ef49bc0c6f0f2250a9527c29a1e8 pin=3f2256d46ac066ee4656489d6d63d007990eb8be trigger=dispatch dispatch_seen=true dispatched=6e7f71fd10e6ef49bc0c6f0f2250a9527c29a1e8 -->\n"
    ));
    let section = |name: &str| {
        let start = body.find(&format!("### {name}\n")).unwrap();
        let rest = &body[start + 4..];
        rest[..rest.find("\n### ").unwrap_or(rest.len())].to_owned()
    };
    assert!(section("Breaking").contains("| [#88](https://github.com/Zafnok/shuyi-engine/pull/88) | 2013 | [2013] Public types for `@music` | shuyi-ui: BoxPlace \\| MapScene |"));
    assert!(section("Unblocks").contains("[#102]"));
    assert!(section("Unblocks").contains("| 1180 |"));
    assert!(section("New for games").contains("[#78]"));
    assert!(section("Nothing for the game").contains("<details><summary>1 PR</summary>"));
    assert!(section("Nothing for the game").contains("[#75]"));
    assert!(section("Adoption tickets still open").contains("None."));
    assert!(section("The work").contains("`engine-pin` skill"));
    assert!(!body.contains("dispatch didn't reach"));
}

#[test]
fn the_title_says_why_a_move_is_due() {
    let title = |prs, open: &[OpenTicket], now| {
        let report = classify(gap(prs), &dirs(), open, &[], now);
        render(&report, &[], &HashSet::new(), &run()).unwrap().0
    };
    let internal = pr(79, "[2012] x", vec![internal_change()]);
    assert_eq!(
        title(vec![internal.clone()], &[], merged()),
        "Engine pin: 1 engine PR behind (no move due yet)"
    );
    assert_eq!(
        title(vec![internal.clone()], &[], merged() + 49 * HOUR),
        "Engine pin: 1 engine PR behind (oldest merged 2026-10-23)"
    );
    assert_eq!(
        title(
            vec![internal, finishes_2131(vec![])],
            &open_1180(),
            merged()
        ),
        "Engine pin: 2 engine PRs behind (unblocks 1180)"
    );
}

#[test]
fn the_issue_lists_open_adoptions_and_a_missed_dispatch() {
    let ledger = ledger::parse(
        "## 2026-10-20: baseline → 11111111\n\n## 2026-10-21: 11111111 → 3f2256d4\n| #78 | 2511 | A prelude | adopt → 1191 |\n| #93 | 2358 | Modules | adopt → 1192 |\n| #95 | 2372 | Menus | not for the game: no modules |\n",
    )
    .unwrap();
    let open: HashSet<String> = ["1191".to_owned()].into();
    let mut report = classify(
        gap(vec![pr(79, "[2012] x", vec![])]),
        &dirs(),
        &[],
        &[],
        merged(),
    );
    report
        .notes
        .push("Commit abcd1234 (a push) has no PR, so it isn't sorted.".to_owned());
    let missed = Run {
        trigger: "schedule".to_owned(),
        dispatch_seen: true,
        dispatched: String::new(),
        missed_dispatch: true,
    };
    let (_, body) = render(&report, &ledger, &open, &missed).unwrap();
    assert!(
        body.contains("- 1191, adopted by the pin move of 2026-10-21, 2 days\n"),
        "{body}"
    );
    assert!(!body.contains("1192"));
    assert!(body.contains("> The engine's dispatch didn't reach the game: check the token (engine 2031's *Nick input*)."));
    assert!(body.contains("> Commit abcd1234 (a push) has no PR"));
    assert!(body.contains("trigger=schedule dispatch_seen=true dispatched=none -->"));
}

#[test]
fn the_json_carries_what_the_workflow_reads() {
    let report = classify(
        gap(vec![pr(78, "[2511] x", vec![api_change()])]),
        &dirs(),
        &[],
        &[],
        merged(),
    );
    let rendered = render(&report, &[], &HashSet::new(), &run());
    let json: serde_json::Value = serde_json::from_str(&json(&report, rendered.as_ref())).unwrap();
    assert_eq!(json["behind"], 1);
    assert_eq!(json["due"], "public API changed in #78");
    assert_eq!(json["main"], report.main);
    assert_eq!(json["pin"], report.pin);
    assert_eq!(
        json["title"],
        "Engine pin: 1 engine PR behind (public API changed in #78)"
    );
    assert!(
        json["body"]
            .as_str()
            .unwrap()
            .starts_with("<!-- engine-pin:v1 ")
    );
    assert!(markdown(&report, rendered.as_ref()).starts_with("# Engine pin: 1 engine PR behind"));
}
