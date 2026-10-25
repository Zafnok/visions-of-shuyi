use std::sync::LazyLock;

use super::*;

static CONTENT: LazyLock<Content> =
    LazyLock::new(|| trpg_content::load_embedded().unwrap_or_else(|e| panic!("content: {e}")));

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn personas() -> String {
    std::fs::read_to_string(root().join(PERSONAS_FILE)).unwrap()
}

fn targets() -> String {
    std::fs::read_to_string(root().join(TARGETS_FILE)).unwrap()
}

/// A fresh, empty folder for test `name`.
fn temp_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("xtask-playtest-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

#[test]
fn parse_args_defaults() {
    let options = parse_args(&args(&["quick"])).unwrap();
    assert_eq!(options, Options::new("quick"));
    assert_eq!(options.battle, "quick");
    assert_eq!((options.runs, options.seed, options.turn_cap), (100, 1, 60));
    assert_eq!(options.mode, GameMode::Classic);
    assert_eq!(options.bot, Bot::Baseline);
    assert_eq!(
        (options.json, options.history, options.threads),
        (None, None, None)
    );
}

#[test]
fn parse_args_reads_every_option() {
    let options = parse_args(&args(&[
        "--runs",
        "20",
        "ch01",
        "--seed",
        "7",
        "--mode",
        "casual",
        "--turn-cap",
        "30",
        "--bot",
        "all",
        "--json",
        "out.json",
        "--history",
        "runs",
        "--threads",
        "2",
    ]))
    .unwrap();
    let expected = Options {
        battle: "ch01".to_owned(),
        runs: 20,
        seed: 7,
        mode: GameMode::Casual,
        turn_cap: 30,
        bot: Bot::All,
        json: Some(PathBuf::from("out.json")),
        history: Some(PathBuf::from("runs")),
        threads: Some(2),
    };
    assert_eq!(options, expected);
    let classic = parse_args(&args(&["ch01", "--mode", "classic", "--seed", "0"])).unwrap();
    assert_eq!((classic.mode, classic.seed), (GameMode::Classic, 0));
    for bot in [
        Bot::Baseline,
        Bot::Casual,
        Bot::Normal,
        Bot::Hardcore,
        Bot::All,
    ] {
        let parsed = parse_args(&args(&["ch01", "--bot", bot.name()])).unwrap();
        assert_eq!(parsed.bot, bot);
    }
}

#[test]
fn parse_args_refuses_bad_arguments() {
    let error = |items: &[&str]| parse_args(&args(items)).unwrap_err();
    assert_eq!(error(&[]), "no battle given");
    assert_eq!(error(&["--runs", "5"]), "no battle given");
    assert_eq!(error(&["a", "b"]), "unexpected argument: b");
    assert_eq!(error(&["a", "--bogus", "1"]), "unknown option: --bogus");
    assert_eq!(error(&["a", "--runs"]), "--runs requires a value");
    assert_eq!(error(&["a", "--runs", "0"]), "--runs: bad value \"0\"");
    assert_eq!(error(&["a", "--runs", "x"]), "--runs: bad value \"x\"");
    assert_eq!(error(&["a", "--seed", "-1"]), "--seed: bad value \"-1\"");
    assert_eq!(
        error(&["a", "--mode", "hard"]),
        "--mode: bad value \"hard\""
    );
    assert_eq!(
        error(&["a", "--turn-cap", "0"]),
        "--turn-cap: bad value \"0\""
    );
    assert_eq!(
        error(&["a", "--bot", "veteran"]),
        "--bot: bad value \"veteran\""
    );
    assert_eq!(
        error(&["a", "--threads", "0"]),
        "--threads: bad value \"0\""
    );
}

#[test]
fn all_is_the_three_personas() {
    assert_eq!(Bot::All.entrants(), ["casual", "normal", "hardcore"]);
    assert_eq!(Bot::Normal.entrants(), ["normal"]);
    assert_eq!(Bot::Baseline.entrants(), ["baseline"]);
}

// ---- The game's report lines --------------------------------------------------

/// A try of seed `seed`.
fn tried(seed: u64, measures: &[(&str, i64)]) -> Tried {
    Tried {
        seed,
        outcome: Some("Victory".to_owned()),
        turns: 9,
        measures: measures.iter().map(|&(n, v)| (n.to_owned(), v)).collect(),
    }
}

/// Five tries of a Classic persona.
fn five_tries() -> Vec<Tried> {
    vec![
        tried(1, &[("won", 1), ("deaths", 0), ("items", 2), ("turns", 14)]),
        tried(
            2,
            &[
                ("won", 1),
                ("deaths", 1),
                ("fell Mira", 1),
                ("fell Mira turn", 6),
                ("items", 0),
                ("turns", 11),
            ],
        ),
        tried(
            3,
            &[
                ("won", 0),
                ("deaths", 3),
                ("fell Mira", 1),
                ("fell Mira turn", 6),
                ("fell Kael", 1),
                ("fell Kael turn", 8),
                ("fell Shuyi", 1),
                ("fell Shuyi turn", 9),
                ("items", 2),
                ("turns", 9),
            ],
        ),
        tried(
            4,
            &[
                ("won", 0),
                ("deaths", 1),
                ("fell Kael", 1),
                ("items", 0),
                ("turns", 60),
            ],
        ),
        tried(5, &[("won", 1), ("deaths", 5), ("items", 4), ("turns", 22)]),
    ]
}

#[test]
fn lines_count_the_tries() {
    let lines = Lines::of("Classic", &five_tries());
    assert_eq!(
        lines,
        Lines {
            count: 5,
            won: 3,
            falls_name: "deaths",
            falls: [1, 2, 0, 2],
            fell_most: Some("Kael 2×".to_owned()),
            items_median: 2,
            items_max: 4,
            turns_median: 14,
        }
    );
    assert_eq!(lines.falls_per_try(), "0: 1  1: 2  2: 0  3+: 2");
    // In Casual the falls are retreats.
    let casual = vec![tried(1, &[("retreats", 2), ("deaths", 0)])];
    let lines = Lines::of("Casual", &casual);
    assert_eq!((lines.falls_name, lines.falls), ("retreats", [0, 0, 1, 0]));
    // Nothing at all.
    let none = Lines::of("Classic", &[]);
    assert_eq!((none.won, none.fell_most, none.turns_median), (0, None, 0));
}

#[test]
fn the_unit_that_fell_most_ties_go_by_name() {
    let tries = vec![
        tried(1, &[("fell Bors", 1), ("fell Anna", 1)]),
        tried(2, &[("fell Bors", 1), ("fell Anna", 1)]),
    ];
    assert_eq!(
        Lines::of("Classic", &tries).fell_most.as_deref(),
        Some("Anna 2×")
    );
}

#[test]
fn median_is_the_lower_middle() {
    assert_eq!(median(vec![]), 0);
    assert_eq!(median(vec![5]), 5);
    assert_eq!(median(vec![9, 1]), 1);
    assert_eq!(median(vec![3, 9, 1]), 3);
    assert_eq!(median(vec![4, 3, 9, 1]), 3);
}

/// A run of Normal's `five_tries` on a Normal-tier `ch01`, judged by the
/// real targets.
fn record() -> Record {
    let checks = Targets::from_ron(&targets())
        .unwrap()
        .checks(trpg_core::Difficulty::Normal, &["normal", "baseline"]);
    let report = judge(
        vec![
            ("normal".to_owned(), five_tries()),
            ("baseline".to_owned(), five_tries()[..2].to_vec()),
        ],
        &checks,
    )
    .unwrap();
    Record {
        date: "2026-10-01 14:05 UTC".to_owned(),
        commit: "abc1234".to_owned(),
        battle: "ch01".to_owned(),
        tier: "Normal".to_owned(),
        turn_cap: 60,
        played: vec![
            Played {
                name: "normal".to_owned(),
                mode: "Classic".to_owned(),
                seconds_per_try: 1.234,
            },
            Played {
                name: "baseline".to_owned(),
                mode: "Classic".to_owned(),
                seconds_per_try: 0.01,
            },
        ],
        report,
    }
}

#[test]
fn render_matches_the_layout() {
    let record = record();
    let text = render(&record, None);
    let table = record.report.to_string();
    let expected = format!(
        "ch01 · Normal tier · normal, baseline · turn cap 60\n\
         {table}\n\
         normal   · Classic · won 3/5 · deaths per try  0: 1  1: 2  2: 0  3+: 2 · most: Kael 2×\n\
         \x20                    items per try  median 2  max 4 · turns median 14 · 1.23 s per try\n\
         baseline · Classic · won 2/2 · deaths per try  0: 1  1: 1  2: 0  3+: 0 · most: Mira 1×\n\
         \x20                    items per try  median 0  max 2 · turns median 11 · 0.01 s per try\n"
    );
    assert_eq!(text, expected);
    // The engine's table has the band check.
    assert!(table.contains("normal · 1/5 succeed"), "{table}");
    assert!(table.contains("band 50–75%   ✗ too hard"), "{table}");
}

#[test]
fn render_shows_the_previous_runs_numbers() {
    let mut previous = record();
    previous.commit = "fd6920d".to_owned();
    previous.report.entrants[1].tries.truncate(1);
    let mut now = record();
    now.report.entrants[0].tries[3]
        .measures
        .insert("deaths".to_owned(), 0);
    now.report = judge(
        now.report
            .entrants
            .iter()
            .map(|e| (e.name.clone(), e.tries.clone()))
            .collect(),
        &now.report
            .checks
            .iter()
            .map(|j| j.check.clone())
            .collect::<Vec<_>>(),
    )
    .unwrap();
    let text = render(&now, Some(&previous));
    assert!(
        text.contains("1.23 s per try · was 1/5 succeed\n"),
        "{text}"
    );
    assert!(text.contains("0.01 s per try · was won 1/1\n"), "{text}");
    assert!(
        text.ends_with("previous  2026-10-01 14:05 UTC · commit fd6920d\n"),
        "{text}"
    );
    // An entrant the previous run didn't have: no `was`.
    let mut other = record();
    other.report.entrants.retain(|e| e.name == "normal");
    other.report.checks.clear();
    let text = render(&record(), Some(&other));
    assert!(text.contains("0.01 s per try\n"), "{text}");
}

#[test]
fn was_reads_the_previous_rate_or_wins() {
    let record = record();
    assert_eq!(was(&record.report, None, "normal"), "");
    assert_eq!(
        was(&record.report, Some(&record), "normal"),
        " · was 1/5 succeed"
    );
    assert_eq!(
        was(&record.report, Some(&record), "baseline"),
        " · was won 2/2"
    );
    assert_eq!(was(&record.report, Some(&record), "casual"), "");
}

#[test]
fn utc_stamp_gives_the_date_and_time() {
    let day = 86_400;
    assert_eq!(utc_stamp(0), "1970-01-01 00:00 UTC");
    assert_eq!(utc_stamp(59), "1970-01-01 00:00 UTC");
    assert_eq!(utc_stamp(60), "1970-01-01 00:01 UTC");
    assert_eq!(utc_stamp(day - 1), "1970-01-01 23:59 UTC");
    assert_eq!(utc_stamp(day), "1970-01-02 00:00 UTC");
    assert_eq!(utc_stamp(30 * day), "1970-01-31 00:00 UTC");
    assert_eq!(utc_stamp(31 * day), "1970-02-01 00:00 UTC");
    assert_eq!(utc_stamp(364 * day), "1970-12-31 00:00 UTC");
    assert_eq!(utc_stamp(365 * day), "1971-01-01 00:00 UTC");
    // 1972 is a leap year.
    assert_eq!(utc_stamp(789 * day), "1972-02-29 00:00 UTC");
    assert_eq!(utc_stamp(790 * day), "1972-03-01 00:00 UTC");
    assert_eq!(utc_stamp(1095 * day), "1972-12-31 00:00 UTC");
    assert_eq!(utc_stamp(1096 * day), "1973-01-01 00:00 UTC");
    // 2000 is one (divisible by 400); 2100 is not (by 100).
    assert_eq!(utc_stamp(951_782_400), "2000-02-29 00:00 UTC");
    assert_eq!(utc_stamp(4_107_456_000), "2100-02-28 00:00 UTC");
    assert_eq!(utc_stamp(4_107_542_400), "2100-03-01 00:00 UTC");
    assert_eq!(utc_stamp(1_790_863_500), "2026-10-01 14:05 UTC");
}

#[test]
fn now_is_after_2023() {
    assert!(now() > 1_700_000_000);
}

#[test]
fn git_commit_is_the_short_hash_or_unknown() {
    let root = root();
    let commit = git_commit(&root);
    // A copy of the sources without their history (cargo-mutants makes
    // one) has no commit.
    if root.join(".git").exists() {
        assert!(commit.len() >= 7, "{commit}");
        assert!(commit.chars().all(|c| c.is_ascii_hexdigit()), "{commit}");
    } else {
        assert_eq!(commit, "unknown");
    }
    let nowhere = temp_dir("no-repo");
    assert_eq!(git_commit(&nowhere), "unknown");
    std::fs::remove_dir_all(&nowhere).unwrap();
}

#[test]
fn history_keeps_every_run_and_gives_the_last() {
    let dir = temp_dir("history");
    let first = record();
    let file = history_file(&dir.join("kept"), &Options::new("ch01"));
    assert_eq!(file, dir.join("kept").join("ch01-baseline-classic.jsonl"));
    assert_eq!(last_record(&file), None);

    append(&file, &first).unwrap();
    assert_eq!(last_record(&file), Some(first.clone()));
    let mut second = record();
    second.commit = "second".to_owned();
    append(&file, &second).unwrap();
    assert_eq!(last_record(&file), Some(second));
    assert_eq!(std::fs::read_to_string(&file).unwrap().lines().count(), 2);

    // A line that isn't a run (an older format, a cut-off write) is passed
    // over.
    let mut text = std::fs::read_to_string(&file).unwrap();
    text.push_str("{\"date\": \n");
    std::fs::write(&file, text).unwrap();
    assert_eq!(
        last_record(&file).map(|r| r.commit),
        Some("second".to_owned())
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn history_has_a_file_per_battle_and_bot() {
    let file = |bot, mode| {
        let options = Options {
            mode,
            bot,
            ..Options::new("quick")
        };
        history_file(Path::new("h"), &options)
    };
    let h = Path::new("h");
    assert_eq!(
        file(Bot::Baseline, GameMode::Casual),
        h.join("quick-baseline-casual.jsonl")
    );
    assert_eq!(file(Bot::All, GameMode::Casual), h.join("quick-all.jsonl"));
    assert_eq!(
        file(Bot::Normal, GameMode::Classic),
        h.join("quick-normal.jsonl")
    );
}

// ---- Playing ------------------------------------------------------------------

/// `runs` tries of `bot` on the Quick Battle, two turns each.
fn quick(bot: Bot, runs: u32) -> Options {
    Options {
        runs,
        bot,
        turn_cap: 2,
        ..Options::new("quick")
    }
}

fn play_quick(options: &Options) -> Record {
    play(&CONTENT, &personas(), &targets(), options).unwrap()
}

#[test]
fn play_reports_every_try_by_seed() {
    let options = Options {
        seed: 5,
        mode: GameMode::Casual,
        turn_cap: 40,
        ..quick(Bot::Baseline, 6)
    };
    let record = play_quick(&options);
    assert_eq!(record.battle, "quick");
    assert_eq!(record.tier, "Normal");
    assert_eq!(record.turn_cap, 40);
    assert_eq!(record.played.len(), 1);
    assert_eq!(
        (
            record.played[0].name.as_str(),
            record.played[0].mode.as_str()
        ),
        ("baseline", "Casual")
    );
    assert!(record.played[0].seconds_per_try > 0.0);
    let seeds: Vec<u64> = record.report.entrants[0]
        .tries
        .iter()
        .map(|t| t.seed)
        .collect();
    assert_eq!(seeds, [5, 6, 7, 8, 9, 10]);
    // The baseline has no target.
    assert_eq!(
        record.report.checks,
        [] as [shuyi_core::bots::playtest::Judged; 0]
    );
    assert!(!record.report.blocked);
}

#[test]
fn all_plays_the_three_personas_in_their_modes_with_their_checks() {
    let record = play_quick(&quick(Bot::All, 1));
    let played: Vec<(&str, &str)> = record
        .played
        .iter()
        .map(|p| (p.name.as_str(), p.mode.as_str()))
        .collect();
    assert_eq!(
        played,
        [
            ("casual", "Casual"),
            ("normal", "Classic"),
            ("hardcore", "Classic")
        ]
    );
    let checked: Vec<&Check> = record.report.checks.iter().map(|j| &j.check).collect();
    assert_eq!(checked.len(), 4);
    assert!(
        matches!(checked[0], Check::Rate { entrant, floor: Some(20), .. } if entrant == "casual")
    );
    assert!(
        matches!(checked[3], Check::Median { entrant, than, .. } if entrant == "hardcore" && than == "normal")
    );
    // A band line per persona, from the engine's table.
    let text = render(&record, None);
    for persona in ["casual   · ", "normal   · ", "hardcore · "] {
        let band = text
            .lines()
            .find(|l| l.starts_with(persona) && l.contains("/1 succeed"));
        assert!(
            band.is_some_and(|l| l.contains("band ")),
            "{persona}\n{text}"
        );
    }
}

/// Every persona's report from the same arguments is the same, on any
/// thread count (only the speed differs).
#[test]
fn play_gives_the_same_report_on_any_thread_count() {
    let on = |threads| Options {
        threads: Some(threads),
        ..quick(Bot::All, 2)
    };
    let one = play_quick(&on(1));
    assert_eq!(one.report.entrants.len(), 3);
    assert!(one.report.entrants.iter().all(|e| e.tries.len() == 2));
    for threads in [1, 4] {
        assert_eq!(
            play_quick(&on(threads)).report,
            one.report,
            "{threads} threads"
        );
    }
}

#[test]
fn a_try_depends_only_on_its_seed() {
    let batch = play_quick(&quick(Bot::Baseline, 6));
    let alone = play_quick(&Options {
        seed: 5,
        ..quick(Bot::Baseline, 1)
    });
    assert_eq!(
        alone.report.entrants[0].tries,
        [batch.report.entrants[0].tries[4].clone()]
    );
}

#[test]
fn play_refuses_an_unknown_battle_or_persona() {
    let error = play(&CONTENT, &personas(), &targets(), &Options::new("nope")).unwrap_err();
    assert!(
        error.starts_with("no battle \"nope\" in assets/battles (there: "),
        "{error}"
    );
    assert!(error.contains("quick"), "{error}");
    let error = play(&CONTENT, "{}", &targets(), &quick(Bot::Normal, 1)).unwrap_err();
    assert_eq!(error, "no persona \"normal\" in crates/bots/personas.ron");
    assert!(play(&CONTENT, "nonsense", &targets(), &quick(Bot::Normal, 1)).is_err());
    assert!(play(&CONTENT, &personas(), "nonsense", &quick(Bot::Normal, 1)).is_err());
}

#[test]
fn a_rate_under_its_floor_blocks_the_run() {
    // Nobody wins the Quick Battle in two turns: under any floor.
    let floors = "floors: {\"casual\": 20}";
    assert!(targets().contains(floors));
    let targets = targets().replace(floors, "floors: {\"casual\": 20, \"normal\": 1}");
    let record = play(&CONTENT, &personas(), &targets, &quick(Bot::Normal, 1)).unwrap();
    assert!(record.report.blocked);
    let unblocked = play_quick(&quick(Bot::Normal, 1));
    assert!(!unblocked.report.blocked);
}

#[test]
fn run_keeps_the_run_and_compares_the_next_with_it() {
    let dir = temp_dir("run");
    let json = dir.join("out").with_extension("json");
    let options = Options {
        json: Some(json.clone()),
        history: Some(dir.join("history")),
        ..quick(Bot::Baseline, 4)
    };
    let first = run(&root(), &options).unwrap();
    let written = std::fs::read_to_string(&json).unwrap();
    assert!(written.ends_with("}\n"));
    let record: Record = serde_json::from_str(&written).unwrap();
    assert_eq!(first.text, render(&record, None));
    assert!(!first.blocked);
    assert_eq!(record.report, play_quick(&options).report);
    assert_eq!(record.commit, git_commit(&root()));
    assert!(record.date.starts_with("20") && record.date.ends_with(" UTC"));
    assert_eq!(record.date.len(), "2026-10-01 14:05 UTC".len());
    let kept = dir.join("history").join("quick-baseline-classic.jsonl");
    assert_eq!(last_record(&kept), Some(record.clone()));

    let second = run(&root(), &options).unwrap();
    let again: Record = serde_json::from_str(&std::fs::read_to_string(&json).unwrap()).unwrap();
    assert_eq!(second.text, render(&again, Some(&record)));
    assert_eq!(again.report, record.report);
    assert!(second.text.contains(" · was won "), "{}", second.text);
    assert!(second.text.contains("\nprevious  20"), "{}", second.text);
    assert_eq!(std::fs::read_to_string(&kept).unwrap().lines().count(), 2);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn run_keeps_history_under_target_by_default() {
    // A stand-in repo root, so the real history isn't touched.
    let root = temp_dir("default-history");
    std::fs::create_dir_all(root.join("crates/bots")).unwrap();
    std::fs::write(root.join(PERSONAS_FILE), personas()).unwrap();
    std::fs::write(root.join(TARGETS_FILE), targets()).unwrap();
    run(&root, &quick(Bot::Baseline, 2)).unwrap();
    let kept = root.join("target/playtest-history/quick-baseline-classic.jsonl");
    let record = last_record(&kept).unwrap();
    assert_eq!(record.report.entrants[0].tries.len(), 2);
    assert_eq!(record.commit, "unknown");
    std::fs::remove_dir_all(&root).unwrap();
}

#[test]
fn run_fails_on_an_unknown_battle_or_a_missing_file() {
    let root = temp_dir("unknown-battle");
    let error = run(&root, &Options::new("quick")).unwrap_err();
    assert!(error.starts_with("crates/bots/personas.ron: "), "{error}");
    assert!(run(&self::root(), &Options::new("nope")).is_err());
    assert!(!root.join("target").exists());
    std::fs::remove_dir_all(&root).unwrap();
}
