//! Tests of this game's rules on reply choices (`@choice`), lead tokens and
//! lead lines (0708). The format's own cases (parsing, errors, printing,
//! ids, `Scene::resolved`) are the engine's tests
//! (`shuyi_assets::dialogue`, ticket 1231).

use super::*;

#[test]
fn reactions_take_directives_and_continuations() {
    let src = scene(
        "@choice
* a: One.
  @right clear
  @right test_archer happy
  test_archer: Long
    and longer.
* b: Two.
  @right test_archer sad
  > Quiet.
@endchoice",
    );
    assert_eq!(errors(&src), Vec::<String>::new());
    let steps = &scenes(&src)[0].steps;
    let Some(Step::Choice { options }) = steps.get(2) else {
        panic!("no choice: {steps:?}");
    };
    assert_eq!(
        options[0].steps,
        [
            Step::Clear { side: Side::Right },
            place(Side::Right, "test_archer", "happy"),
            say("test_archer", None, "Long and longer."),
        ]
    );
}

#[test]
fn every_option_leaves_the_same_screen() {
    let src = scene(
        "@choice
* a: A.
  @right clear
  @right test_archer happy
  > r
* b: B.
  > r
* c: C.
  @left clear
  @right test_archer sad
@endchoice
test_archer: Who's there?",
    );
    assert_eq!(
        errors(&src),
        [
            "t.dlg:9: after this reaction test_lord is on the left and test_knight on the right; after the first option's test_lord is on the left and test_archer on the right; every option must leave the same characters on screen",
            "t.dlg:11: after this reaction nobody is on the left and test_archer on the right; after the first option's test_lord is on the left and test_archer on the right; every option must leave the same characters on screen",
        ]
    );
    // Expressions may differ: the scene takes back the old ones.
    let src = scene(
        "@choice\n* a: A.\n  test_knight[happy]: Ha.\n* b: B.\n  test_knight[sad]: Oh.\n@endchoice",
    );
    assert_eq!(errors(&src), Vec::<String>::new());
    let caption = scene("@choice\n* a: A.\n  @caption Later\n* b: B.\n@endchoice\n> x");
    assert_eq!(
        errors(&caption),
        [
            "t.dlg:7: this reaction leaves a different caption from the first option's; every option must leave the same caption"
        ]
    );
}

#[test]
fn unknown_tokens() {
    let src = scene(
        "test_lord: {lead}, {they} {Them} {themself}.
> {nope} and {he} and {Lead}.
@caption {x} Heth
@choice
* a: {bogus}
* b: {
@endchoice",
    );
    let hint = "use {lead}, {they}, {them}, {their}, {theirs}, {themself} (capitalised: {They}...) or a name, {n:<id>}";
    assert_eq!(
        errors(&src),
        [
            format!("t.dlg:5: unknown token \"{{nope}}\"; {hint}"),
            format!("t.dlg:5: unknown token \"{{he}}\"; {hint}"),
            format!("t.dlg:5: unknown token \"{{Lead}}\"; {hint}"),
            format!("t.dlg:6: unknown token \"{{x}}\"; {hint}"),
            format!("t.dlg:8: unknown token \"{{bogus}}\"; {hint}"),
            "t.dlg:9: \"{\" has no closing \"}\"".to_owned(),
        ]
    );
}

#[test]
fn text_length_counts_the_longest_substitution() {
    // 188 + {lead} (12) = 200: at the limit; {they} (3) more is 203.
    let base = "a".repeat(188);
    assert_eq!(
        errors(&scene(&format!(
            "> {base}{{lead}}\n> {base}{{lead}}{{they}}"
        ))),
        ["t.dlg:5: text is 203 cells; the limit is 200"]
    );
}

#[test]
fn the_lead_speaks_in_short_lines() {
    let body = |text: &str| format!("@right clear\n@right lead neutral\nlead: {text}");
    let at_limit = "a".repeat(MAX_LEAD_LINE_LEN);
    assert_eq!(errors(&scene(&body(&at_limit))), Vec::<String>::new());
    assert_eq!(errors(&scene(&body("Let's move."))), Vec::<String>::new());
    let message = "the lead's line is 41 cells; the limit is 40: the lead speaks in short, \
                   neutral lines and otherwise through reply choices \
                   (assets-private/story/design/setting-and-tone.md, \"Rules for writing the lead\")";
    assert_eq!(
        errors(&scene(&body(&format!("{at_limit}a")))),
        [format!("t.dlg:6: {message}")]
    );
    // {lead} counts as 12: 29 + 12 = 41.
    let with_name = format!("{}{{lead}}", "a".repeat(29));
    assert_eq!(
        errors(&scene(&body(&with_name))),
        [format!("t.dlg:6: {message}")]
    );
    // Other speakers' lines aren't limited.
    assert_eq!(
        errors(&scene(&format!("test_lord: {at_limit}a"))),
        Vec::<String>::new()
    );
}

#[test]
fn the_leads_expressions_come_from_both_portraits() {
    let images = crate::image::load().unwrap_or_default();
    let mut portraits = crate::portrait::load_all(&images).unwrap_or_default();
    assert!(portraits.contains_key("lead_m") && portraits.contains_key("lead_f"));
    let src = scene("@right clear\n@right lead neutral\nlead[sad]: Hm.");
    assert_eq!(errors_with(&src, &portraits), Vec::<String>::new());
    // One portrait lacks "sad": the lead can't use it.
    if let Some(p) = portraits.get_mut("lead_f") {
        p.expressions.retain(|e| e.name != "sad");
    }
    assert_eq!(
        errors_with(&src, &portraits),
        [
            "t.dlg:6: \"lead_f\"'s portrait has no expression \"sad\"; use one of neutral, happy, angry, surprised"
        ]
    );
    // With neither portrait, the standard five.
    portraits.retain(|id, _| !id.starts_with("lead"));
    assert_eq!(errors_with(&src, &portraits), Vec::<String>::new());
}

#[test]
fn a_scene_whose_only_text_is_in_a_reaction_has_text() {
    let src = "@scene s\n@choice\n* a: A.\n  > r\n* b: B.\n@endchoice\n@end\n";
    assert_eq!(errors(src), Vec::<String>::new());
    let src = "@scene s\n@choice\n* a: A.\n* b: B.\n@endchoice\n@end\n";
    assert_eq!(
        errors(src),
        ["t.dlg:1: scene \"s\" has no speech or narration"]
    );
}

/// The choice example in `assets/dialogue/README.md` parses and passes
/// every check except the character ids (its cast isn't in
/// `characters.ron`).
#[test]
fn readme_choice_example_is_valid() {
    let example = README.split("```").nth(5).unwrap_or_default();
    let table = from_sources(&[("README.md", example)], None, None, None, None);
    assert!(table.is_ok(), "{table:?}");
    let scene = table.unwrap_or_default().scenes.remove("ch01_gate");
    let choice = scene.and_then(|s| {
        s.steps
            .into_iter()
            .find(|s| matches!(s, Step::Choice { .. }))
    });
    assert!(matches!(choice, Some(Step::Choice { options }) if options.len() == 3));
}
