use super::*;

const FILE: &str = "\
(
    id: \"t\",
    enemies: [
        // The road.
        (template: \"brigand\", pos: (1, 2), id: \"road\"),
        (template: \"archer\", level: Some(2), pos: (3, 4)), // \"a (comment)\"
        // The fort.
        (
            template: \"brigand\",
            pos: (5, 6),
        ),
    ],
    seed: 1,
)
";

fn text() -> BattleText {
    BattleText::parse("t.ron", "t", FILE).unwrap_or_else(|e| panic!("{e}"))
}

#[test]
fn as_authored_it_is_the_file_itself() {
    assert_eq!(text().text(), FILE);
    for path in crate::bundle::files_in(super::super::BATTLES_DIR) {
        let source = crate::bundle::file(path).unwrap_or_default();
        let parsed = BattleText::parse(path, "", source).unwrap_or_else(|e| panic!("{e}"));
        assert_eq!(parsed.text(), source, "{path}");
        assert!(!parsed.entries().is_empty(), "{path}");
    }
}

#[test]
fn reads_each_entry() {
    let t = text();
    let entries = t.entries();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[1].template.as_deref(), Some("archer"));
    assert_eq!(entries[1].level, Some(2));
    assert_eq!(entries[2].pos, (5, 6));
    // An `id` names it.
    assert!(!entries[0].unnamed);
    assert!(entries[1].unnamed && entries[2].unnamed);
}

#[test]
fn a_changed_field_is_rewritten_in_place_and_a_new_one_added() {
    let mut t = text();
    t.foes[1].level = Some(4);
    t.foes[0].level = Some(3);
    t.foes[2].pos = (7, 8);
    let out = t.text();
    assert!(
        out.contains("(template: \"archer\", level: Some(4), pos: (3, 4)), // \"a (comment)\""),
        "{out}"
    );
    assert!(
        out.contains("(template: \"brigand\", pos: (1, 2), id: \"road\", level: Some(3)),"),
        "{out}"
    );
    assert!(
        out.contains("            pos: (7, 8),\n        ),"),
        "{out}"
    );
}

#[test]
fn a_loadout_and_a_template_are_written_as_the_format_has_them() {
    let mut t = text();
    t.foes[1].template = Some("brigand".to_owned());
    t.foes[1].loadout = Some(LoadoutDef {
        weapons: vec![ItemId::new("steel_axe")],
        armour: Some(ItemId::new("chain_mail")),
        accessory: None,
    });
    assert!(t.text().contains(
        "(template: \"brigand\", level: Some(2), pos: (3, 4), loadout: \
         Some((weapons: [\"steel_axe\"], armour: Some(\"chain_mail\")))),"
    ));
}

#[test]
fn a_copy_follows_its_original_and_a_removed_entry_takes_its_line() {
    let mut t = text();
    let mut copy = t.foes[1].clone();
    copy.copy = true;
    copy.pos = (3, 5);
    t.foes.insert(2, copy);
    t.foes.remove(0);
    let out = t.text();
    // The first entry's comment is the list's head: it stays.
    assert!(!out.contains("\"road\""), "{out}");
    assert!(
        out.contains(
            "[\n        // The road.\n        (template: \"archer\", level: Some(2), pos: (3, 4)),\n        \
             (template: \"archer\", level: Some(2), pos: (3, 5)), // \"a (comment)\"\n        \
             // The fort."
        ),
        "{out}"
    );
    let reparsed = BattleText::parse("t.ron", "t", &out).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(reparsed.foes.len(), 3);
}

#[test]
fn with_every_entry_removed_the_list_is_empty() {
    let mut t = text();
    t.foes.clear();
    let out = t.text();
    let reparsed = BattleText::parse("t.ron", "t", &out).unwrap_or_else(|e| panic!("{e}\n{out}"));
    assert!(reparsed.foes.is_empty(), "{out}");
    assert!(out.contains("seed: 1"));
}

#[test]
fn set_field_finds_the_field_by_name_not_inside_strings() {
    let entry = "(template: \"pos: (9, 9)\", pos: (1, 2))";
    assert_eq!(
        set_field(entry, "pos", "(3, 4)"),
        "(template: \"pos: (9, 9)\", pos: (3, 4))"
    );
    assert_eq!(set_field("()", "level", "Some(1)"), "(level: Some(1))");
    assert_eq!(
        set_field("(\n  pos: (1, 2),\n)", "level", "Some(1)"),
        "(\n  pos: (1, 2), level: Some(1),\n)"
    );
}
