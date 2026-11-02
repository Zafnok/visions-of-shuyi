use std::fs;
use std::sync::atomic::{AtomicU32, Ordering};

use super::*;

fn args(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| (*s).to_owned()).collect()
}

fn new(list: &[&str]) -> Result<Options, String> {
    let mut all = vec!["new", "--at", "10,10", "--level", "5"];
    all.extend(list);
    parse_args(&args(&all))
}

/// A folder holding what a run reads from the repository.
fn sandbox() -> PathBuf {
    static NEXT: AtomicU32 = AtomicU32::new(0);
    let n = NEXT.fetch_add(1, Ordering::Relaxed);
    let root = std::env::temp_dir().join(format!("xtask-skirmish-{}-{n}", std::process::id()));
    let repo = crate::repo_root();
    for file in [
        "crates/bots/personas.ron",
        "crates/bots/targets.ron",
        "assets-test/worldmaps/test_world.ron",
        "assets-test/skirmishes/test_world.ron",
        "assets/worldmaps/act1.ron",
    ] {
        let to = root.join(file);
        fs::create_dir_all(to.parent().expect("a folder")).expect("a folder");
        fs::copy(repo.join(file), to).expect("copied");
    }
    root
}

/// One casual try, enough to go through the whole command quickly.
const QUICK: [&str; 11] = [
    "--map",
    "test_world",
    "--force",
    "--entrant",
    "casual",
    "--tries",
    "1",
    "--budget",
    "1",
    "--turns",
    "3",
];

#[test]
fn options_parse_and_bad_ones_are_named() {
    let options = new(&["--units", "mage", "--chapters", "2,ch03", "--seed", "9"]).expect("good");
    assert_eq!(options.at, Some((10, 10)));
    assert_eq!(options.level, Some(5));
    assert_eq!(options.units, Some(Units::Only("mage".to_owned())));
    assert_eq!(options.chapters, ["2", "ch03"]);
    assert_eq!(options.seed, 9);
    assert_eq!(
        new(&["--units", "mix"]).expect("good").units,
        Some(Units::Mix)
    );
    assert_eq!(
        new(&["--tier", "easy"]).expect("good").tier.as_deref(),
        Some("Easy")
    );
    for bad in [
        vec!["--at", "ten"],
        vec!["--level", "many"],
        vec!["--kind", "other"],
        vec!["--tier", "hard"],
        vec!["--bogus", "1"],
        vec!["--kind", "random", "--chapters", "2"],
        vec!["--kind", "random", "--tier", "normal"],
        vec!["--units"],
    ] {
        assert!(new(&bad).is_err(), "{bad:?}");
    }
    assert!(parse_args(&args(&["new", "--level", "5"])).is_err());
    assert!(parse_args(&args(&["old"])).is_err());
    assert!(parse_args(&[]).is_err());
}

#[test]
fn help_and_bad_arguments_answer_without_generating() {
    let root = crate::repo_root();
    assert_eq!(command(&root, &args(&["new", "--help"])), 0);
    assert_eq!(command(&root, &args(&["new", "--at", "x"])), 2);
}

#[test]
fn chapters_are_numbers_or_ids() {
    let content = trpg_content::load_embedded().expect("content");
    assert_eq!(
        chapter_ids(&content, &args(&["2", "ch01"])),
        Ok(args(&["ch02", "ch01"]))
    );
    assert!(chapter_ids(&content, &args(&["77"])).is_err());
}

#[test]
fn a_new_file_goes_in_the_data_folder_of_its_world_map() {
    assert_eq!(
        beside("assets-test/worldmaps/a.ron", "maps/b.map"),
        "assets-test/maps/b.map"
    );
    assert_eq!(
        beside("assets/worldmaps/a.ron", "maps/b.map"),
        "assets/maps/b.map"
    );
}

#[test]
fn a_skirmish_end_to_end_writes_valid_files_and_the_same_ones_again() {
    let (one, two) = (sandbox(), sandbox());
    let mut list = args(&[
        "new",
        "--at",
        "10,10",
        "--level",
        "5",
        "--units",
        "swordsman",
    ]);
    list.extend(args(&QUICK));
    let options = parse_args(&list).expect("good");
    let ran = run(&one, &options).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(ran.written.len(), 3, "{:?}", ran.written);
    run(&two, &options).unwrap_or_else(|e| panic!("{e}"));
    let id = "test_world_skirmish_10_10";
    for file in [
        format!("assets-test/battles/{id}.ron"),
        format!("assets-test/maps/{id}.map"),
        "assets-test/worldmaps/test_world.ron".to_owned(),
    ] {
        let read = |root: &PathBuf| fs::read_to_string(root.join(&file)).expect(&file);
        assert_eq!(read(&one), read(&two), "{file}");
    }
    let battle = fs::read_to_string(one.join(format!("assets-test/battles/{id}.ron"))).expect("it");
    assert!(
        battle.contains("bots_army: (chapter: \"ch02\", level: 5)"),
        "{battle}"
    );
    assert!(
        battle.contains("template: \"skirmish_swordsman\""),
        "{battle}"
    );
    let world = fs::read_to_string(one.join("assets-test/worldmaps/test_world.ron")).expect("it");
    assert!(world.contains("(id: \"skirmish_10_10\""));
    assert!(
        run(&one, &options).is_err(),
        "a second run mustn't overwrite the first"
    );
    let _ = (fs::remove_dir_all(one), fs::remove_dir_all(two));
}

#[test]
fn without_a_map_it_goes_on_the_story_s_first_and_writes_beside_it() {
    let root = sandbox();
    let list = args(&[
        "new",
        "--at",
        "10,10",
        "--level",
        "5",
        "--force",
        "--entrant",
        "casual",
        "--tries",
        "1",
        "--budget",
        "1",
        "--turns",
        "3",
    ]);
    let ran = run(&root, &parse_args(&list).expect("good")).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        ran.written,
        [
            "assets/maps/act1_skirmish_10_10.map",
            "assets/battles/act1_skirmish_10_10.ron",
            "assets/worldmaps/act1.ron",
        ]
    );
    let world = fs::read_to_string(root.join("assets/worldmaps/act1.ron")).expect("it");
    assert!(world.contains("(id: \"skirmish_10_10\""));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_random_skirmish_joins_the_pool_and_makes_no_place() {
    let root = sandbox();
    let mut list = args(&["new", "--at", "10,10", "--level", "5", "--kind", "random"]);
    list.extend(args(&QUICK));
    run(&root, &parse_args(&list).expect("good")).unwrap_or_else(|e| panic!("{e}"));
    let pool = fs::read_to_string(root.join("assets-test/skirmishes/test_world.ron")).expect("it");
    assert!(pool.contains("\"test_world_skirmish_10_10\""));
    let world = fs::read_to_string(root.join("assets-test/worldmaps/test_world.ron")).expect("it");
    assert!(!world.contains("skirmish_10_10"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn a_request_that_cant_be_written_stops_before_the_search() {
    let root = sandbox();
    let mut list = args(&["new", "--at", "14,20", "--level", "5"]); // Millbrook stands there.
    list.extend(args(&QUICK));
    let error = run(&root, &parse_args(&list).expect("good")).expect_err("a taken cell");
    assert!(error.contains("already stands"), "{error}");
    let _ = fs::remove_dir_all(root);
}
