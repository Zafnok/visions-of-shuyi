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

#[test]
fn a_copy_gets_its_own_line_not_the_comment_above_its_original() {
    let mut t = text();
    let mut copy = t.foes[2].clone();
    copy.copy = true;
    t.foes.push(copy);
    let out = t.text();
    assert_eq!(out.matches("// The fort.").count(), 1, "{out}");
    assert!(
        out.contains("        ),\n        (\n            template"),
        "{out}"
    );
}

#[test]
fn the_first_entry_moved_later_gets_a_line_of_its_own() {
    let mut t = text();
    t.foes.swap(0, 1);
    let out = t.text();
    let reparsed = BattleText::parse("t.ron", "t", &out).unwrap_or_else(|e| panic!("{e}\n{out}"));
    assert_eq!(reparsed.entries()[1].template.as_deref(), Some("brigand"));
    assert_eq!(reparsed.entries()[1].pos, (1, 2));
}

#[test]
fn a_copy_on_a_one_line_list_takes_the_line_s_indent() {
    let source =
        "(\n    enemies: [(template: \"a\", pos: (1, 1)), (template: \"b\", pos: (2, 2))],\n)\n";
    let mut t = BattleText::parse("t.ron", "t", source).unwrap_or_else(|e| panic!("{e}"));
    let mut copy = t.foes[1].clone();
    copy.copy = true;
    copy.pos = (3, 3);
    t.foes.push(copy);
    assert_eq!(
        t.text(),
        "(\n    enemies: [(template: \"a\", pos: (1, 1)), (template: \"b\", pos: (2, 2)),\n    \
         (template: \"b\", pos: (3, 3))],\n)\n"
    );
}

#[test]
fn strings_and_comments_are_blanked_with_their_lengths_kept() {
    let blank = |s: &str| String::from_utf8(code(s)).unwrap_or_default();
    assert_eq!(blank("a\"x(\"b"), "a\"  \"b");
    assert_eq!(blank("a\"x\\\"y\"b"), "a\"    \"b");
    assert_eq!(blank("a\"open"), "a\"    ");
    assert_eq!(blank("a // (c)\nb"), "a       \nb");
    assert_eq!(blank("ab/*(*/c"), "ab     c");
    assert_eq!(blank("ab/* open"), "ab       ");
}

#[test]
fn field_at_finds_a_whole_name_at_its_depth() {
    let at = |s: &str, key, depth| field_at(s.as_bytes(), key, depth);
    assert_eq!(at("pos: 1", "pos", 0), Some(4));
    assert_eq!(at("(xpos: 1, pos : 2)", "pos", 1), Some(15));
    assert_eq!(at("(my_pos: 1, pos: 2)", "pos", 1), Some(16));
    assert_eq!(at("(a: (pos: 1))", "pos", 1), None);
    assert_eq!(at("(posx: 1)", "pos", 1), None);
}

#[test]
fn set_field_replaces_the_whole_value_whatever_the_spacing() {
    assert_eq!(
        set_field("(pos:  (1, 2), ai: Guard)", "pos", "(3, 4)"),
        "(pos:  (3, 4), ai: Guard)"
    );
    assert_eq!(
        set_field("(level: Some(2) )", "level", "Some(3)"),
        "(level: Some(3) )"
    );
}

#[test]
fn add_to_list_keeps_the_text_and_adds_a_line_at_the_end() {
    let source = "(
    a: [
        1, // one
        2
    ],
    b: [],
)
";
    let added = add_to_list(source, "a", "3", "        ").expect("the list");
    assert_eq!(
        added,
        "(
    a: [
        1, // one
        2,
        3,
    ],
    b: [],
)
"
    );
    let empty = add_to_list(source, "b", "x", "  ").expect("the empty list");
    assert!(
        empty.contains(
            "b: [
  x,
    ],"
        ),
        "{empty}"
    );
    assert_eq!(add_to_list(source, "c", "x", ""), None);
}
