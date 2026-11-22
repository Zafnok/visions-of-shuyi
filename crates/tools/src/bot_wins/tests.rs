use super::*;

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

#[test]
fn a_battle_or_all_with_the_lines_and_seed() {
    let one = parse(&args(&[
        "ch03", "--lines", "4", "--seed", "9", "--out", "x",
    ]))
    .unwrap();
    assert_eq!(
        one,
        Options {
            from: Some("ch03".to_owned()),
            lines: 4,
            seed: 9,
            out: Some("x".to_owned()),
        }
    );
    let all = parse(&args(&["--all"])).unwrap();
    assert_eq!(
        (all.from, all.lines, all.seed, all.out),
        (None, LINES, 1, None)
    );
}

#[test]
fn a_usage_error_says_what_is_wrong() {
    let error = |items: &[&str]| parse(&args(items)).unwrap_err();
    assert!(error(&[]).contains("or --all"));
    assert!(error(&["ch01", "--all"]).contains("not both"));
    assert!(error(&["ch01", "ch02"]).contains("one battle at a time"));
    assert!(error(&["--lines"]).contains("needs a value"));
    assert!(error(&["--all", "--lines", "x"]).contains("not a number"));
    assert!(error(&["--all", "--lines", "0"]).contains("records nothing"));
    assert!(error(&["--all", "--bogus"]).contains("unknown option --bogus"));
    assert_eq!(command(Path::new("."), &args(&["--bogus"])), 2);
    assert_eq!(command(Path::new("."), &args(&["--help"])), 0);
}

/// A battle no later battle's army plays has nothing to record, and the
/// error lists those that do.
#[test]
fn a_battle_off_the_way_is_refused() {
    let content = trpg_content::load_embedded().unwrap();
    let options = parse(&args(&["quick"])).unwrap();
    let error = record_lines(&content, &Wins::default(), &options).unwrap_err();
    assert!(
        error.contains("no later battle's army plays \"quick\""),
        "{error}"
    );
}

/// Line i's search is seeded the first seed plus i; a battle's retries are
/// 1,000 apart, past every other line's seed.
#[test]
fn each_line_and_each_retry_has_a_seed_of_its_own() {
    assert_eq!(line_seed(9, 0), 9);
    assert_eq!(line_seed(9, 3), 12);
    assert_eq!(search_seed(5, 0), 5);
    assert_eq!(search_seed(5, 2), 2005);
}

/// A line that can't start a battle says which, and records nothing.
#[test]
fn a_line_names_the_battle_it_cannot_start() {
    let content = trpg_content::load_embedded().unwrap();
    let error = record_line(&content, &Wins::default(), &["quick".to_owned()], 0, 1).unwrap_err();
    assert!(
        error.contains("no story chapter fights \"quick\""),
        "{error}"
    );
}

/// Recording starts at the battle named, or at the first; one off the way
/// is refused, listing those on it.
#[test]
fn recording_starts_at_the_battle_named() {
    let on_the_way = ["ch01".to_owned(), "ch02".to_owned(), "ch03".to_owned()];
    assert_eq!(first_to_record(&on_the_way, None), Ok(0));
    assert_eq!(first_to_record(&on_the_way, Some("ch02")), Ok(1));
    assert_eq!(first_to_record(&on_the_way, Some("ch03")), Ok(2));
    let error = first_to_record(&on_the_way, Some("quick")).unwrap_err();
    assert!(
        error.ends_with("(those that do: ch01, ch02, ch03)"),
        "{error}"
    );
}
