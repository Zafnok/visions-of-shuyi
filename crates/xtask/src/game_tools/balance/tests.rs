use std::fs;
use std::path::PathBuf;

use shuyi_cli::contract::Answer;
use shuyi_cli::contract::autobalance::{AutobalanceOutput, autobalance};
use shuyi_cli::frame_png::{Script, Shot};
use shuyi_cli::{GameArg, GameLang, GameOption, GameTools};

use super::*;
use crate::game_tools::Trpg;
use crate::repo_root;

/// The game's tools, but reading the personas and writing battle files
/// under `.0`.
struct At(PathBuf);

impl GameTools for At {
    fn frame_options(&self) -> &[GameOption] {
        Trpg.frame_options()
    }

    fn frame(
        &self,
        options: &[GameArg],
        script: &Script,
        scale: Option<u32>,
    ) -> Result<Shot, String> {
        Trpg.frame(options, script, scale)
    }

    fn lang(&self) -> Result<GameLang, String> {
        Trpg.lang()
    }

    fn playtest_stages(&self) -> Vec<String> {
        Trpg.playtest_stages()
    }

    fn autobalance_plan(
        &self,
        stage: Option<&str>,
        input: &AutobalanceInput,
    ) -> Result<AutobalancePlan, String> {
        plan(&self.0, stage, input)
    }
}

/// A folder under the system's temp folder with the personas and the
/// targets, and the test battles' folder, empty.
fn root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("xtask-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    for file in [PERSONAS_FILE, TARGETS_FILE] {
        let to = root.join(file);
        fs::create_dir_all(to.parent().unwrap_or(&root)).unwrap();
        fs::copy(repo_root().join(file), to).unwrap();
    }
    fs::create_dir_all(root.join("assets-test/battles")).unwrap();
    root
}

/// `test_too_hard` balanced with a small search, from seed 1.
fn balance(root: PathBuf, apply: bool) -> Answer<AutobalanceOutput> {
    let mut input = AutobalanceInput::default();
    input.stage = Some("test_too_hard".to_owned());
    input.tries = Some(20);
    input.quick = Some(10);
    input.budget = Some(40);
    input.seed = Some(1);
    input.turns = Some(20);
    input.apply = apply;
    autobalance(&At(root), &input)
}

#[test]
fn the_too_hard_battle_is_brought_into_its_bands_and_written() {
    let root = root("autobalance");
    let answer = balance(root.clone(), true);
    let found = answer
        .result
        .unwrap_or_else(|| panic!("{:?}", answer.error));
    assert!(answer.ok, "{found:#?}");
    assert!(found.in_bands);
    assert_eq!(found.changes.len(), 1, "{:?}", found.changes);
    let shown = "assets-test/battles/test_too_hard.ron";
    assert_eq!(found.written, Some(vec![shown.to_owned()]));

    // The file written is the battle changed, its comments kept.
    let written = fs::read_to_string(root.join(shown)).unwrap();
    let authored = bundle::file("battles/test_too_hard.ron").unwrap_or_default();
    assert_ne!(written, authored);
    assert!(written.starts_with("// TEST FILE: a skirmish made too hard"));
    assert!(written.contains("        // The raider rides in from the north.\n"));
    let content = Arc::new(trpg_content::load_embedded().unwrap());
    let text = BattleText::parse(shown, "test_too_hard", &written).unwrap();
    assert!(TunedBattle::new(text, content).load().is_ok());

    // Same seed, same changes.
    let again = balance(root.clone(), false).result.unwrap();
    assert_eq!(again.changes, found.changes);
    assert_eq!(again.after, found.after);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn the_plan_names_the_battle_and_refuses_what_it_cant_balance() {
    let root = root("autobalance-plan");
    let input = AutobalanceInput::default();
    let plan = plan(&root, Some("test_too_hard"), &input).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(plan.title, "test_too_hard · Normal tier · FixedSkirmish");
    let names: Vec<&str> = plan.entrants.iter().map(|(n, _)| n.as_str()).collect();
    assert_eq!(names, ENTRANTS);
    // Each persona's band, then Hardcore faster than Normal.
    assert_eq!((plan.checks)(&names).len(), 4);

    let err = |stage| plan_error(&root, stage, &input);
    assert_eq!(err(None), "name the battle to balance with --stage");
    assert_eq!(err(Some("nope")), "no battle \"nope\"");
    let mut with_option = AutobalanceInput::default();
    with_option.start = vec![GameArg {
        flag: "--map".to_owned(),
        value: None,
    }];
    assert_eq!(
        plan_error(&root, Some("quick"), &with_option),
        "autobalance takes no game option, not --map"
    );
    assert!(Trpg.playtest_stages().contains(&"ch01".to_owned()));
    let _ = fs::remove_dir_all(&root);
}

fn plan_error(root: &std::path::Path, stage: Option<&str>, input: &AutobalanceInput) -> String {
    match plan(root, stage, input) {
        Ok(_) => panic!("planned"),
        Err(e) => e,
    }
}
