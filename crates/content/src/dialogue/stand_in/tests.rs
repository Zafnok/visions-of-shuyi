//! Tests of the stand-ins for the player's name (ticket 0754).

use trpg_core::LeadGender;
use trpg_core::LeadProfile;

use super::*;
use crate::dialogue::{Scripts, scripts_from_sources};

fn scripts(body: &str) -> Scripts {
    let source = format!("@scene s\n@left test_lord neutral\n{body}\n@end\n");
    scripts_from_sources(&[("t.dlg", source.as_str())], None, None, None, None).unwrap()
}

fn texts(scripts: &Scripts) -> Vec<(String, String)> {
    let lines = scripts.table.scenes["s"].lines();
    let rows = lines.iter().map(|l| (l.id.to_string(), l.text.to_owned()));
    rows.collect()
}

#[test]
fn the_box_shows_the_name_and_the_stand_in_is_kept_apart() {
    let with = scripts("test_lord: Welcome back, {lead|the exile}.\n> Rain.");
    let plain = scripts("test_lord: Welcome back, {lead}.\n> Rain.");
    // The line is the plain one: same text, same id.
    assert_eq!(texts(&with), texts(&plain));
    let (id, shown) = &texts(&with)[0];
    let lead = LeadProfile::new("Mira", LeadGender::Female);
    assert_eq!(lead.substitute(shown), "Welcome back, Mira.");
    // Only the line with a stand-in is kept, as written.
    assert_eq!(with.stand_ins.len(), 1);
    assert_eq!(
        with.stand_ins[&LineId::new(id.as_str())],
        "Welcome back, {lead|the exile}."
    );
    assert!(plain.stand_ins.is_empty());
}

#[test]
fn changing_a_stand_in_keeps_the_line_id() {
    let one = scripts("> {lead|The exile} sits.");
    let other = scripts("> {lead|The heir} sits.");
    assert_eq!(texts(&one), texts(&other));
    assert_ne!(one.stand_ins, other.stand_ins);
}

#[test]
fn stand_ins_in_reactions_and_blocks_are_kept_by_their_lines_ids() {
    let s = scripts(
        "test_lord: Here, {lead|you}.\n@choice\n* a: Yes.\n  > {lead|The exile} nods.\n\
         * b: No, {lead|me}.\n@endchoice\n@if test_lord\n> {lead} and {lead|the exile}.\n\
         @else\n> Rain.\n@endif",
    );
    let written: Vec<&str> = s.stand_ins.values().map(String::as_str).collect();
    assert_eq!(written.len(), 4);
    for (id, shown) in texts(&s) {
        let id = LineId::new(id);
        let Some(as_written) = s.stand_ins.get(&id) else {
            assert!(!shown.contains("lead"), "{shown}");
            continue;
        };
        assert_eq!(strip(as_written).as_deref(), Some(shown.as_str()));
    }
    assert!(written.contains(&"{lead} and {lead|the exile}."));
}

#[test]
fn written_puts_the_stand_ins_back_under_the_same_ids() {
    let s = scripts("test_lord: Welcome back, {lead|the exile}.\n> Rain.");
    let voiced = written(&s.table, &s.stand_ins);
    let rows: Vec<(String, String)> = voiced.scenes["s"]
        .lines()
        .iter()
        .map(|l| (l.id.to_string(), l.text.to_owned()))
        .collect();
    let ids: Vec<String> = texts(&s).into_iter().map(|(id, _)| id).collect();
    assert_eq!(
        rows.iter().map(|(id, _)| id.clone()).collect::<Vec<_>>(),
        ids
    );
    assert_eq!(rows[0].1, "Welcome back, {lead|the exile}.");
    assert_eq!(rows[1].1, "Rain.");
}

#[test]
fn a_bar_with_no_words_or_on_another_token_is_an_unknown_token() {
    for body in ["> {lead|} sits.", "> {lead| } sits.", "> {they|she} sits."] {
        let source = format!("@scene s\n{body}\n@end\n");
        let errors = scripts_from_sources(&[("t.dlg", source.as_str())], None, None, None, None)
            .unwrap_err();
        assert!(errors[0].message.contains("unknown token"), "{errors:?}");
    }
}

fn missing_lines(body: &str) -> Vec<Option<u32>> {
    let s = scripts(body);
    missing(&s.parsed[0], &s.stand_ins)
        .iter()
        .map(|e| e.line)
        .collect()
}

#[test]
fn a_voiced_line_with_a_bare_name_is_an_error() {
    // Line 3 is the first after `@scene` and `@left`.
    assert_eq!(missing_lines("test_lord: Back, {lead}."), [Some(3)]);
    assert_eq!(missing_lines("> {lead} sits."), [Some(3)]);
    // Every name needs one.
    assert_eq!(missing_lines("> {lead|The exile} and {lead}."), [Some(3)]);
    // In a reaction and in an `@if` block too.
    assert_eq!(
        missing_lines(
            "@choice\n* a: Yes.\n  > {lead} nods.\n* b: No.\n@endchoice\n\
             @if test_lord\n> Rain.\n@else\n> {lead} sits.\n@endif"
        ),
        [Some(5), Some(11)]
    );
}

#[test]
fn replies_and_lines_with_stand_ins_need_nothing() {
    assert_eq!(missing_lines("> {lead|The exile} sits, {they} said."), []);
    assert_eq!(
        missing_lines("> Rain.\n@choice\n* a: I'm {lead}.\n* b: No.\n@endchoice"),
        []
    );
}
