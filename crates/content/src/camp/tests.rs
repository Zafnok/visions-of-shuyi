//! Tests of the camp file's checks.

use super::*;

/// The embedded game's tables, for the references.
fn content() -> crate::Content {
    crate::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

fn refs(c: &crate::Content) -> CampRefs<'_> {
    CampRefs {
        characters: Some(&c.characters),
        dialogue: Some(&c.dialogue),
        battles: Some(&c.battles),
    }
}

fn errors(source: &str, refs: CampRefs) -> Vec<String> {
    match from_sources(&[("camp.ron", source)], refs) {
        Ok(_) => vec![],
        Err(e) => e.iter().map(ToString::to_string).collect(),
    }
}

#[test]
fn an_event_reads_with_its_conditions_and_without_them() {
    let c = content();
    let src = r#"(events: [
        (id: "a", title: "A", scene: "test_camp_campfire",
         flags: ["f"], battles: ["test"], units: ["test_lord"]),
        (id: "b", title: "B", scene: "test_camp_campfire"),
    ])"#;
    let events = from_sources(&[("camp.ron", src)], refs(&c)).unwrap();
    assert_eq!(events.len(), 2);
    assert_eq!(events[0].flags, ["f"]);
    assert_eq!(events[0].battles, ["test"]);
    assert_eq!(events[0].units, [CharacterId("test_lord".into())]);
    assert!(events[1].flags.is_empty() && events[1].units.is_empty());
}

#[test]
fn every_problem_is_reported() {
    let c = content();
    let src = r#"(events: [
        (id: "a", title: " ", scene: "nope", battles: ["no_battle"], units: ["nobody"]),
        (id: "a", title: "A", scene: "test_camp_campfire"),
    ])"#;
    let found = errors(src, refs(&c));
    let has = |text: &str| found.iter().any(|e| e.contains(text));
    assert!(has("camp event \"a\": the title is empty"), "{found:?}");
    assert!(has("unknown dialogue scene \"nope\""), "{found:?}");
    assert!(has("unknown battle \"no_battle\""), "{found:?}");
    assert!(has("unknown character \"nobody\""), "{found:?}");
    assert!(has("the id is listed twice"), "{found:?}");
    assert!(found[0].starts_with("camp.ron:2"), "{found:?}");
}

/// A reference whose files failed to load isn't checked.
#[test]
fn references_that_didnt_load_are_not_checked() {
    let src = r#"(events: [(id: "a", title: "A", scene: "nope", battles: ["x"], units: ["y"])])"#;
    assert_eq!(errors(src, CampRefs::default()), [] as [&str; 0]);
}

/// The id is unique across the story's file and the test files'.
#[test]
fn an_id_in_both_files_is_listed_twice() {
    let a = r#"(events: [(id: "a", title: "A", scene: "s")])"#;
    let files = [("one.ron", a), ("two.ron", a)];
    let e = from_sources(&files, CampRefs::default()).unwrap_err();
    assert_eq!(e.len(), 1);
    assert!(e[0].to_string().starts_with("two.ron"), "{}", e[0]);
}

#[test]
fn the_game_has_the_test_event_in_a_build_with_the_test_files() {
    let c = content();
    let ids: Vec<&str> = c.camp.iter().map(|e| e.id.as_str()).collect();
    if bundle::test_part(CAMP_PATH).is_some() {
        assert_eq!(ids, ["test_campfire"]);
    } else {
        assert_eq!(ids, [] as [&str; 0]);
    }
}
