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
