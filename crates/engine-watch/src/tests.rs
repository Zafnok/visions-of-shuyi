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

fn marker(path: &str, line: usize, engine_id: &str) -> Marker {
    Marker {
        path: path.to_owned(),
        line,
        engine_id: engine_id.to_owned(),
    }
}

fn kinds(report: &Report) -> Vec<(u32, Vec<Kind>)> {
    report
        .rows
        .iter()
        .map(|r| (r.pr.number, r.kinds.clone()))
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
    assert_eq!(
        ticket_ids("Bump rust-toolchain in / (#83)"),
        [] as [std::string::String; 0]
    );
    assert_eq!(ticket_ids("[fix] typo"), [] as [std::string::String; 0]);
    assert_eq!(
        ticket_ids("A title with 2026 in it"),
        [] as [std::string::String; 0]
    );
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
            vec![Kind::Breaking(vec!["the `api: breaking` label".to_owned()])]
        )]
    );
    assert_eq!(report.due, Some(Why::PublicApi(2)));
}

#[test]
fn breaking_lines_come_from_the_body_and_the_ticket_notes() {
    let mut breaking = pr(88, "[2013] Public types", vec![internal_change()]);
    breaking.body = "## API changes\n\nBreaking: shuyi-app: Window: build it with Window::new\n- Breaking: shuyi-ui: BoxPlace::new\n\nThe contract doesn't change.\n".to_owned();
    assert_eq!(
        breaking_lines(&breaking),
        [
            "shuyi-app: Window: build it with Window::new",
            "shuyi-ui: BoxPlace::new"
        ]
    );
    // A wrapped line goes on to the end of its paragraph, as in Markdown,
    // whether it is a list item or not.
    let mut wrapped = pr(410, "[2828] Exercises", vec![]);
    wrapped.ticket_notes = Some(
        "Behaviour: shuyi-app: on the web, a page\nwrites `shuyi-page-check`.\n\nSearched the game.\n- Breaking: shuyi-ui: a\n  b\n- Done.\nBreaking: shuyi-cli: c\nBehaviour: shuyi-cli: d\n```migration\nsince: 0.1.0\n```\nBreaking: e\n## Later\n"
            .to_owned(),
    );
    assert_eq!(
        breaking_lines(&wrapped),
        ["shuyi-ui: a b", "shuyi-cli: c", "e"]
    );
    assert_eq!(
        behaviour_lines(&wrapped),
        [
            "shuyi-app: on the web, a page writes `shuyi-page-check`.",
            "shuyi-cli: d"
        ]
    );
    let mut noted = pr(90, "[2024] More constructors", vec![]);
    noted.ticket_notes = Some("\n- Done.\n  Breaking: shuyi-core: Grid::new\n".to_owned());
    assert_eq!(breaking_lines(&noted), ["shuyi-core: Grid::new"]);
    let mut none = pr(91, "[2025] No breaks", vec![]);
    none.body = "No breaking change.\nNo break here.\nA `Breaking:` line is required.\n".to_owned();
    none.ticket_notes = Some("Breaking: none\nBreaking: no break\nBreaking:\n".to_owned());
    assert_eq!(breaking_lines(&none), [] as [std::string::String; 0]);
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
fn a_breaking_pr_that_also_adds_and_unblocks_is_in_breaking_and_new_for_games() {
    let mut pr = finishes_2131(vec![api_change()]);
    pr.body = "Breaking: shuyi-core: threat moved\n".to_owned();
    let report = classify(gap(vec![pr]), &dirs(), &open_1180(), &[], merged());
    assert_eq!(
        kinds(&report),
        [(
            102,
            vec![
                Kind::Breaking(vec!["shuyi-core: threat moved".to_owned()]),
                Kind::NewForGames,
            ]
        )]
    );
}

#[test]
fn a_breaking_pr_is_also_new_only_when_it_adds_public_items() {
    let breaking = |files, label: Option<&str>| {
        let mut pr = pr(88, "[2013] Public types", files);
        pr.body = "Breaking: shuyi-ui: BoxPlace\n".to_owned();
        pr.labels.extend(label.map(str::to_owned));
        breaks_and_adds(&pr, &dirs())
    };
    assert!(breaking(vec![api_change()], None));
    assert!(breaking(vec![internal_change()], Some("api: added")));
    assert!(breaking(
        vec![PrFile {
            path: "crates/core/src/huge.rs".to_owned(),
            patch: None,
        }],
        None
    ));
    // A public item removed, or added outside an API crate's `src/`, adds nothing.
    assert!(!breaking(
        vec![file("crates/ui/src/a.rs", "-pub fn gone() {}\n")],
        None
    ));
    assert!(!breaking(
        vec![file("crates/xtask/src/a.rs", "+pub fn run() {}\n")],
        None
    ));
    assert!(!breaking(vec![internal_change()], Some("api: breaking")));
    // Adding without breaking is only *New for games*.
    assert!(!breaks_and_adds(
        &pr(78, "[2511] A prelude", vec![api_change()]),
        &dirs()
    ));
    let label_only = labelled(pr(89, "[2014] x", vec![api_change()]), "api: breaking");
    assert!(breaks_and_adds(&label_only, &dirs()));
}

#[test]
fn a_pr_that_doesn_t_break_goes_in_the_first_other_section_that_fits() {
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
            (102, vec![Kind::Unblocks(vec!["1180".to_owned()])]),
            (78, vec![Kind::NewForGames]),
            (75, vec![Kind::Nothing]),
            (79, vec![Kind::Nothing]),
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
    assert_eq!(kinds(&report), [(94, vec![Kind::Nothing])]);
    assert_eq!(report.due, None);
    // A friction marker waits as a ticket does.
    let report = classify(
        gap(vec![finishes_2131(vec![])]),
        &dirs(),
        &[],
        &[
            marker("crates/ui/src/a.rs", 3, "2131"),
            marker("crates/ui/src/b.rs", 9, "2131"),
        ],
        merged(),
    );
    let marker = "the `engine-friction: engine#2131` marker".to_owned();
    assert_eq!(
        kinds(&report),
        [(102, vec![Kind::Unblocks(vec![marker.clone()])])]
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
    assert_eq!(empty.blocked_by, [] as [std::string::String; 0]);
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

/// Ticket 0193: a "moves" marker parses as a workaround's does, and covers
/// its item.
#[test]
fn a_moves_marker_parses_as_today_s() {
    let line = "// engine-friction: engine#2132, moves (game 1180 deletes it)";
    assert_eq!(friction_markers(line), ["2132"]);
    assert_eq!(marker_lines(line), [(1, "2132".to_owned())]);
    let diff = format!("+++ b/crates/bots/src/persona.rs\n+{line}\n");
    assert_eq!(added_markers(&diff), ["2132"]);
}

fn span(first: usize, last: usize, engine: &str, game: Option<&str>) -> MarkerSpan {
    MarkerSpan {
        first,
        last,
        engine: engine.to_owned(),
        game: game.map(str::to_owned),
    }
}

#[test]
fn a_marker_spans_its_item_and_nothing_past_it() {
    // Line 1 is code: a marker on the first line covers the whole file.
    let source = "\
use a;
// engine-friction: engine#2132, moves (game 1180 deletes it)
/// The order.
#[must_use]
pub fn order(a: u8) -> u8 {
    let b = a; // not the end
    // a comment;
    b
}
fn after() {}

    // engine-friction: engine#2131, moves (game 1180 deletes it)
    pub struct Targets {
        a: u8,
    }

// engine-friction: engine#2129
impl Tally {
    fn score(&self) {}
}
// engine-friction: engine#2148, moves (game 1207 deletes it)
const ONE: u8 = 1;
let x = y(); // engine-friction: engine#2540, moves (game 1182 deletes it)
// engine-friction: engine#2633, moves (game 12 deletes it)
fn open(
";
    assert_eq!(
        marker_spans(source),
        [
            span(2, 9, "2132", Some("1180")),
            span(12, 15, "2131", Some("1180")),
            span(17, 20, "2129", None),
            span(21, 22, "2148", Some("1207")),
            span(23, 23, "2540", Some("1182")),
            span(24, 25, "2633", None),
        ]
    );
    assert_eq!(
        marker_spans("fn a() {}\n// engine-friction: engine#25\n"),
        []
    );
    // On the first line: the whole file.
    let file =
        "// engine-friction: engine#2578, moves (game 1224 deletes it)\nuse a;\n\nfn b() {}\n";
    assert_eq!(marker_spans(file), [span(1, 4, "2578", Some("1224"))]);
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
        labelled(pr(75, "Tickets 2741-2768: x", vec![]), "api: none"),
        pr(76, "Bump a tool", vec![]),
    ];
    let report = classify(gap(prs), &dirs(), &open_1180(), &[], merged());
    let (title, body) = render(&report, &[], &HashSet::new(), &run()).unwrap();
    assert_eq!(
        title,
        "Engine pin: 5 engine PRs behind (public API changed in #88)"
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
    assert!(!body.contains("<details>"));
    assert!(section("Nothing for the game").contains("| Engine PR | Ticket | Title | Labels |"));
    assert!(section("Nothing for the game").contains("x | api: none |"));
    assert!(section("Nothing for the game").contains("Bump a tool | no labels |"));
    assert!(section("Adoption tickets still open").contains("None."));
    assert!(section("The work").contains("`engine-pin` skill"));
    assert!(!body.contains("dispatch didn't reach"));
}

#[test]
fn a_pr_that_breaks_and_adds_is_listed_in_both_sections() {
    let mut both = pr(152, "[2111] Stamped records", vec![api_change()]);
    both.body = "Breaking: shuyi-core: Record gains a stamp\n".to_owned();
    let report = classify(gap(vec![both]), &dirs(), &[], &[], merged());
    let (_, body) = render(&report, &[], &HashSet::new(), &run()).unwrap();
    let (breaking, new) = body.split_at(body.find("### New for games").unwrap());
    assert!(breaking.contains("[#152]"), "{breaking}");
    assert!(breaking.contains("Record gains a stamp"), "{breaking}");
    let new = &new[..new.find("### Nothing").unwrap()];
    assert!(new.contains("[#152]"), "{new}");
    assert!(!new.contains("Record gains a stamp"), "{new}");
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

#[test]
fn behaviour_lines_come_from_the_body_and_the_notes_and_go_under_breaking() {
    let mut changed = pr(256, "[2350] Looser .dlg text", vec![internal_change()]);
    changed.body =
        "Behaviour: shuyi-assets: .dlg rejects only control characters\nBehaviour: none\n"
            .to_owned();
    changed.ticket_notes = Some(
        "- Behaviour: shuyi-assets: .dlg rejects only control characters\n- Behavior: shuyi-cli: lang-status labels\n\nNo `Behaviour:` line here.\n"
            .to_owned(),
    );
    assert_eq!(
        behaviour_lines(&changed),
        [
            "shuyi-assets: .dlg rejects only control characters",
            "shuyi-cli: lang-status labels"
        ]
    );
    assert_eq!(breaking_lines(&changed), [] as [String; 0]);
    let report = classify(gap(vec![changed]), &dirs(), &[], &[], merged());
    assert_eq!(
        kinds(&report),
        [(
            256,
            vec![Kind::Breaking(vec![
                "Behaviour: shuyi-assets: .dlg rejects only control characters".to_owned(),
                "Behaviour: shuyi-cli: lang-status labels".to_owned(),
            ])]
        )]
    );
    // A behaviour change alone doesn't change the public API.
    assert_eq!(report.due, None);
    // Breaking lines come first; the label stands in for none.
    let mut both = labelled(pr(88, "[2013] x", vec![]), "api: breaking");
    both.body = "Behaviour: shuyi-ui: a default\n".to_owned();
    assert_eq!(
        breaks(&both),
        [
            "the `api: breaking` label",
            "Behaviour: shuyi-ui: a default"
        ]
    );
}

#[test]
fn markers_are_found_by_line_and_in_a_diff_s_added_code() {
    let source = "fn a() {}\n// engine-friction: engine#2255 (the voice)\nfn b() {\n    // engine-friction: engine#2540\n}\n";
    assert_eq!(
        marker_lines(source),
        [(2, "2255".to_owned()), (4, "2540".to_owned())]
    );
    let diff = "diff --git a/crates/ui/src/a.rs b/crates/ui/src/a.rs\n--- a/crates/ui/src/a.rs\n+++ b/crates/ui/src/a.rs\n@@ -1 +1,2 @@\n // engine-friction: engine#1111\n+// engine-friction: engine#2255\n-// engine-friction: engine#3333\n+++ b/crates/engine-watch/src/tests.rs\n+// engine-friction: engine#4444\n+++ b/docs/x.md\n+// engine-friction: engine#5555\n+++ b/crates/ui/README.md\n+// engine-friction: engine#7777\n+++ b/crates/app/src/b.rs\n+    // engine-friction: engine#2255 again\n+    // engine-friction: engine#6666\n";
    assert_eq!(added_markers(diff), ["2255", "6666"]);
}

/// A fixture `crates/` tree under the system's temporary folder.
fn fixture_tree(files: &[(&str, &str)]) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!("trpg-engine-watch-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    for (path, text) in files {
        let file = root.join(path);
        std::fs::create_dir_all(file.parent().unwrap()).unwrap();
        std::fs::write(file, text).unwrap();
    }
    root
}

/// The body's text from the heading `from` to the heading `to`.
fn between<'a>(body: &'a str, from: &str, to: &str) -> &'a str {
    &body[body.find(from).unwrap()..body.find(to).unwrap()]
}

#[test]
fn the_report_lists_behaviour_lines_and_the_workarounds_the_pin_ends() {
    let root = fixture_tree(&[
        (
            "crates/ui/src/dialogue.rs",
            "fn a() {}\n\n// engine-friction: engine#2255\nfn wait() {}\n",
        ),
        (
            "crates/ui/src/other.rs",
            "// engine-friction: engine#2540\n",
        ),
        (
            "crates/engine-watch/src/x.rs",
            "// engine-friction: engine#2255\n",
        ),
        ("crates/target/x.rs", "// engine-friction: engine#2255\n"),
        ("docs/x.rs", "// engine-friction: engine#2255\n"),
    ]);
    let markers = fetch::friction_markers(&root);
    let _ = std::fs::remove_dir_all(&root);
    assert_eq!(
        markers,
        [
            marker("crates/ui/src/dialogue.rs", 3, "2255"),
            marker("crates/ui/src/other.rs", 1, "2540"),
        ]
    );
    let mut voice = pr(
        301,
        "[2255] Screens know when a voice clip has ended",
        vec![file("tickets/done/2255-voice-end.md", "+done")],
    );
    voice.ticket_notes =
        Some("Breaking: none\nBehaviour: shuyi-ui: a clip's end comes as an event\n".to_owned());
    let mut report = classify(gap(vec![voice]), &dirs(), &[], &markers, merged());
    assert_eq!(report.passed, ["2255"]);
    assert_eq!(
        report.markers,
        [marker("crates/ui/src/dialogue.rs", 3, "2255")]
    );
    report.pr_markers = vec![
        PrMarker {
            number: 563,
            title: "A per-character wait for @voice".to_owned(),
            engine_ids: vec!["2255".to_owned()],
        },
        PrMarker {
            number: 564,
            title: "Old".to_owned(),
            engine_ids: vec!["2013".to_owned()],
        },
        PrMarker {
            number: 565,
            title: "Not pinned".to_owned(),
            engine_ids: vec!["2999".to_owned()],
        },
    ];
    let ledger = ledger::parse(
        "## 2026-10-21: 11111111 → 3f2256d4\n| #88 | 2013 | Constructors | fixed: x |\n",
    )
    .unwrap();
    let (_, body) = render(&report, &ledger, &HashSet::new(), &run()).unwrap();
    let breaking = between(&body, "### Breaking", "### Unblocks");
    assert!(breaking.contains("| Breaking or behaviour |"), "{breaking}");
    assert!(
        breaking.contains("| [#301](https://github.com/Zafnok/shuyi-engine/pull/301) | 2255 | [2255] Screens know when a voice clip has ended | Behaviour: shuyi-ui: a clip's end comes as an event |"),
        "{breaking}"
    );
    let ends = between(&body, "### Workarounds the pin ends", "### The work");
    assert!(
        ends.contains("- `crates/ui/src/dialogue.rs:3`: engine#2255\n"),
        "{ends}"
    );
    assert!(!ends.contains("other.rs"), "{ends}");
    assert!(
        ends.contains(
            "- open PR #563 (A per-character wait for `@voice`) adds a marker for engine#2255\n"
        ),
        "{ends}"
    );
    assert!(
        ends.contains(
            "- open PR #564 (Old) adds a marker for engine#2013, in the pin since 2026-10-21\n"
        ),
        "{ends}"
    );
    assert!(!ends.contains("#565"), "{ends}");
    // Nothing passed, nothing added: none.
    let report = classify(
        gap(vec![pr(79, "[2012] x", vec![])]),
        &dirs(),
        &[],
        &markers,
        merged(),
    );
    let (_, body) = render(&report, &[], &HashSet::new(), &run()).unwrap();
    let ends = between(&body, "### Workarounds the pin ends", "### The work");
    assert!(ends.ends_with("None.\n\n"), "{ends}");
}

#[test]
fn the_adoption_list_names_each_ticket_once_with_its_age_and_marks_the_idle() {
    let ledger = ledger::parse(
        "## 2026-10-12: 11111111 → 22222222\n| #70 | 2100 | A | adopt → 1170 |\n| #71 | 2101 | B | unblocks 1180 (its pin step struck) |\n\n## 2026-10-21: 22222222 → 3f2256d4\n| #78 | 2511 | A prelude | adopt → 1191; adopt → 1170 |\n| #79 | 2512 | C | fixed: x; unblocks 1180 and 1191 |\n| #80 | 2513 | D | adopt → 1192 |\n",
    )
    .unwrap();
    let open: HashSet<String> = ["1170", "1180", "1191"].map(str::to_owned).into();
    let report = classify(
        gap(vec![pr(79, "[2012] x", vec![])]),
        &dirs(),
        &[],
        &[],
        merged(),
    );
    let (_, body) = render(&report, &ledger, &open, &run()).unwrap();
    let list = between(&body, "### Adoption tickets still open", "### Workarounds");
    assert!(
        list.ends_with(
            "- 1170, adopted by the pin move of 2026-10-12, 11 days: **idle**\n\
             - 1180, unblocked by the pin move of 2026-10-12, 11 days: **idle**\n\
             - 1191, adopted by the pin move of 2026-10-21, 2 days\n\n"
        ),
        "{list}"
    );
    assert_eq!(list.matches("1170").count(), 1, "{list}");
    assert!(!list.contains("1192"), "{list}");
    // Five days is not yet idle, six is; one day is a day.
    let at = |days: i64| {
        let mut report = report.clone();
        report.now = epoch_seconds("2026-10-21T00:00:00Z").unwrap() + days * 86_400;
        let only: HashSet<String> = ["1191".to_owned()].into();
        render(&report, &ledger, &only, &run()).unwrap().1
    };
    assert!(at(5).contains("- 1191, adopted by the pin move of 2026-10-21, 5 days\n"));
    assert!(at(6).contains("- 1191, adopted by the pin move of 2026-10-21, 6 days: **idle**\n"));
    assert!(at(1).contains("- 1191, adopted by the pin move of 2026-10-21, 1 day\n"));
}
