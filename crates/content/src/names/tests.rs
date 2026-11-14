//! Tests of the game's names table (the table's own rules are tested in
//! the engine, `shuyi_assets::names`).

use trpg_core::lead;

use super::*;

#[test]
fn embedded_table_loads() {
    let names = load();
    assert!(names.is_ok(), "{names:?}");
    if crate::bundle::skip_without_the_game() {
        return;
    }
    // The game's names: a person's, and a faction's with its article.
    let names = names.unwrap_or_default();
    assert!(names.get("king").is_some_and(|n| !n.is_empty()));
    let crown = names.get("faction.crown");
    assert!(crown.is_some_and(|n| n.starts_with("the ")), "{crown:?}");
}

/// The ids in the first column of every table of the names registry, the
/// private repository's `story/names.md` (ticket 0159); `None` without it.
fn registry_ids() -> Option<std::collections::BTreeSet<String>> {
    let path = crate::bundle::story_doc("names.md")?;
    let text = std::fs::read_to_string(&path).unwrap_or_default();
    Some(
        text.lines()
            .filter_map(|l| l.strip_prefix("| `"))
            .filter_map(|l| l.split_once('`'))
            .map(|(id, _)| id.to_owned())
            .collect(),
    )
}

#[test]
fn table_holds_every_registered_id() {
    // The registry names the game's own names (a test of the content).
    if crate::bundle::skip_without_the_game() {
        return;
    }
    let Some(ids) = registry_ids() else {
        return;
    };
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
        "ids not in the names registry (story/names.md)"
    );
}

#[test]
fn lead_default_matches_core() {
    let names = load().unwrap_or_default();
    assert_eq!(names.get(lead::LEAD_ID), Some(lead::DEFAULT_NAME));
}

#[test]
fn every_two_word_name_has_short_forms() {
    if crate::bundle::skip_without_the_game() {
        return;
    }
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
            // A god: "<name>, <title>".
            gods += 1;
            assert_eq!(names.get(&format!("{id}.name")), Some(god), "{id}");
            assert_eq!(names.get(&format!("{id}.title")), Some(title), "{id}");
        } else if !id.contains('.') && name.contains(' ') {
            // A person: "<first> <last>".
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
    assert_eq!((people, gods, lasts), (19, 5, 16));
    // The retainer's surname is the family's; the lead's family is Veyne
    // (public, `trpg_core::lead`); the red captain's nickname ends in
    // the first name.
    let last = |id: &str| {
        names
            .get(id)
            .and_then(|n| n.rsplit_once(' '))
            .map(|(_, l)| l)
    };
    assert_eq!(names.get("family.marr"), last("retainer"));
    assert_eq!(names.get("family.veyne"), Some(lead::FAMILY_NAME));
    assert_eq!(last("red_captain.nickname"), names.get("red_captain.first"));
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
    // The six test characters' and the test speaker's.
    assert_eq!(shipped.names.len() + 7, names.names.len());
}
