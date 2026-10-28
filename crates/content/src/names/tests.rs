//! Tests of the game's names table (the table's own rules are tested in
//! the engine, `shuyi_assets::names`).

use trpg_core::lead;

use super::*;

#[test]
fn embedded_table_loads() {
    let names = load();
    assert!(names.is_ok(), "{names:?}");
    let names = names.unwrap_or_default();
    assert_eq!(names.get("king"), Some("Emeric"));
    assert_eq!(names.get("faction.crown"), Some("the Crown of Ardeval"));
}

/// The ids in the first column of every table of `docs/story/names.md`.
fn registry_ids() -> std::collections::BTreeSet<String> {
    include_str!("../../../../docs/story/names.md")
        .lines()
        .filter_map(|l| l.strip_prefix("| `"))
        .filter_map(|l| l.split_once('`'))
        .map(|(id, _)| id.to_owned())
        .collect()
}

#[test]
fn table_holds_every_registered_id() {
    let ids = registry_ids();
    assert!(ids.len() > 60, "parsed only {} ids", ids.len());
    let names = load().unwrap_or_default();
    let missing: Vec<&String> = ids.iter().filter(|id| names.get(id).is_none()).collect();
    assert_eq!(missing, Vec::<&String>::new(), "ids missing from names.ron");
    // Every extra entry is a placeholder test character.
    let extra: Vec<&String> = names
        .names
        .keys()
        .filter(|id| !ids.contains(*id))
        .filter(|id| !id.starts_with("test_"))
        .collect();
    assert_eq!(
        extra,
        Vec::<&String>::new(),
        "ids not in docs/story/names.md"
    );
}

#[test]
fn lead_default_matches_core() {
    let names = load().unwrap_or_default();
    assert_eq!(names.get(lead::LEAD_ID), Some(lead::DEFAULT_NAME));
}

#[test]
fn every_two_word_name_has_short_forms() {
    let names = load().unwrap_or_default();
    let story = || {
        names
            .names
            .iter()
            .filter(|(id, _)| !id.starts_with("test_"))
    };
    let mut people = 0;
    let mut gods = 0;
    let mut lasts = 0;
    for (id, name) in story() {
        if let Some((god, title)) = name.split_once(", ") {
            // A god: "Ama, the Mother".
            gods += 1;
            assert_eq!(names.get(&format!("{id}.name")), Some(god), "{id}");
            assert_eq!(names.get(&format!("{id}.title")), Some(title), "{id}");
        } else if !id.contains('.') && name.contains(' ') {
            // A person: "Hollis Marr".
            people += 1;
            let (first, last) = name.split_once(' ').unwrap_or_default();
            assert_eq!(names.get(&format!("{id}.first")), Some(first), "{id}");
            // The surname is the character's own, or a family's.
            let shared = story().any(|(i, n)| i.starts_with("family.") && n == last);
            let own = names.get(&format!("{id}.last"));
            assert_eq!(own, (!shared).then_some(last), "{id}");
            lasts += usize::from(own.is_some());
        }
    }
    assert_eq!((people, gods, lasts), (16, 5, 13));
    assert_eq!(names.get("family.marr"), Some("Marr"));
    assert_eq!(names.get("family.veyne"), Some("Veyne"));
    assert_eq!(names.get("red_captain.nickname"), Some("Red Harl"));
}

/// The embedded table has the test characters' names only with the test
/// files (ADR-0057).
#[test]
fn the_embedded_table_has_the_test_names_only_with_the_test_files() {
    let names = load().unwrap_or_default();
    assert_eq!(names.get("test_lord"), Some("Test Lord"));
    let shipped = bundle::story_only(load).unwrap_or_default();
    assert_eq!(shipped.get("test_lord"), None);
    assert_eq!(shipped.get("lead"), names.get("lead"));
    assert_eq!(shipped.names.len() + 6, names.names.len());
}
