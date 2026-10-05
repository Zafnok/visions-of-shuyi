//! Tests of `@if` blocks and of who is there when a scene plays (0715,
//! ADR-0055).

use std::collections::BTreeSet;

use trpg_core::{Phase, Trigger, TriggerWhen, Who};

use super::*;

fn when(character: &str, then: Vec<Step>, otherwise: Vec<Step>) -> Step {
    Step::If {
        character: id(character),
        then,
        otherwise,
    }
}

fn part(line: u32, steps: &[u32]) -> PartLines {
    PartLines {
        line,
        lines: Lines {
            steps: steps.to_vec(),
            blocks: vec![],
        },
    }
}

fn ids(characters: &[&str]) -> BTreeSet<CharacterId> {
    characters.iter().map(|c| id(c)).collect()
}

/// A block for the knight with an `@else`, and one for the archer without.
const BLOCKS: &str = "\
@if test_knight
test_knight: Here.
@else
> Nobody answers.
@endif
@if test_archer
> She waves.
@endif";

// --- Parsing ------------------------------------------------------------------

#[test]
fn parses_if_blocks() {
    let src = scene(BLOCKS);
    let (parsed, errs) = parse_dlg("t.dlg", &src);
    assert_eq!(errs, []);
    assert_eq!(
        bare(&parsed[0].scene.steps)[2..],
        [
            when(
                "test_knight",
                vec![say("test_knight", None, "Here.")],
                vec![narrate("Nobody answers.")]
            ),
            when("test_archer", vec![narrate("She waves.")], vec![]),
        ]
    );
    assert_eq!(parsed[0].lines.steps, [2, 3, 4, 9]);
    // A block without an `@else` has its second part on the `@if`'s line.
    assert_eq!(
        parsed[0].lines.blocks,
        [
            vec![part(4, &[5]), part(6, &[7])],
            vec![part(9, &[10]), part(9, &[])]
        ]
    );
    assert_eq!(errors(&src), Vec::<String>::new());
}

/// Blocks nest, hold choices, and stand in a reply's reaction (indented
/// like the rest of it), where a line still continues the one above.
const NESTED: &str = "\
@if test_knight
@if test_archer
test_knight: Both of us.
@else
test_knight: Only me.
@endif
@choice
* earnest: Good.
  @if test_archer
  > She nods.
    Twice.
  @else
  > Nobody nods.
  @endif
* blunt: Move.
  test_knight: Moving.
@endchoice
@endif";

#[test]
fn blocks_nest_and_stand_in_reactions() {
    let src = scene(NESTED);
    let (parsed, errs) = parse_dlg("t.dlg", &src);
    assert_eq!(errs, []);
    let option = |tone: &str, text: &str, steps| ChoiceOption {
        tone: tone.into(),
        text: text.into(),
        line: LineId::default(),
        steps,
    };
    let nods = when(
        "test_archer",
        vec![narrate("She nods. Twice.")],
        vec![narrate("Nobody nods.")],
    );
    let both = when(
        "test_archer",
        vec![say("test_knight", None, "Both of us.")],
        vec![say("test_knight", None, "Only me.")],
    );
    let choice = Step::Choice {
        options: vec![
            option("earnest", "Good.", vec![nods]),
            option("blunt", "Move.", vec![say("test_knight", None, "Moving.")]),
        ],
    };
    assert_eq!(
        bare(&parsed[0].scene.steps)[2..],
        [when("test_knight", vec![both, choice], vec![])]
    );
    let lines = &parsed[0].lines;
    assert_eq!(lines.steps, [2, 3, 4]);
    let [with, without] = &lines.blocks[0][..] else {
        panic!("{lines:?}");
    };
    assert_eq!((with.line, without.line), (4, 4));
    assert_eq!(with.lines.steps, [5, 10]);
    assert_eq!(with.lines.blocks[0], [part(5, &[6]), part(7, &[8])]);
    let reply = &with.lines.blocks[1];
    assert_eq!((reply[0].line, &reply[0].lines.steps), (11, &vec![12]));
    assert_eq!(
        reply[0].lines.blocks,
        [vec![part(12, &[13]), part(15, &[16])]]
    );
    assert_eq!(reply[1], part(18, &[19]));
    assert_eq!(errors(&src), Vec::<String>::new());
}

#[test]
fn if_block_syntax_errors() {
    let bad_id = "\"Knight\" is not a valid id; use lowercase letters, digits and _";
    let cases: [(&str, &[&str]); 12] = [
        ("@else\n> x", &["t.dlg:4: @else without an open @if"]),
        ("> x\n@endif", &["t.dlg:5: @endif without an open @if"]),
        (
            "@if test_knight\n> a\n@else\n> b\n@else\n> c\n@endif",
            &["t.dlg:8: @if already has an @else, on line 6"],
        ),
        ("@if test_knight\n> a", &["t.dlg:4: @if has no @endif"]),
        (
            "@if Knight\n> a\n@endif",
            &[
                &format!("t.dlg:4: {bad_id}"),
                "t.dlg:4: unknown character \"Knight\"",
            ],
        ),
        (
            "@if\n> a\n@endif",
            &[
                "t.dlg:4: @if needs one character id",
                "t.dlg:4: unknown character \"\"",
            ],
        ),
        (
            "@if test_knight test_archer\n> a\n@endif",
            &[
                "t.dlg:4: @if needs one character id",
                "t.dlg:4: unknown character \"\"",
            ],
        ),
        (
            "@if test_knight\n> a\n@else now\n@endif soon",
            &[
                "t.dlg:6: @else takes nothing after it",
                "t.dlg:7: @endif takes nothing after it",
            ],
        ),
        // A reaction's block ends in its reaction.
        (
            "@choice\n* a: One.\n  @if test_knight\n  > x\n* b: Two.\n  > y\n@endchoice",
            &["t.dlg:6: @if has no @endif"],
        ),
        (
            "@choice\n* a: One.\n  > x\n* b: Two.\n  @if test_knight\n  > y\n@endchoice",
            &["t.dlg:8: @if has no @endif"],
        ),
        // A block's lines aren't indented: an indented line continues.
        (
            "> x\n@if test_knight\n  > a\n@endif",
            &["t.dlg:6: indented line doesn't continue a speech or narration line"],
        ),
        (
            "> x\n@choice\n  @if test_knight\n* a: One.\n  @endif\n* b: Two.\n@endchoice",
            &[
                "t.dlg:6: a reaction line needs an option above it: \"* tone: text\"",
                "t.dlg:8: @endif without an open @if",
            ],
        ),
    ];
    for (body, expected) in cases {
        assert_eq!(errors(&scene(body)), expected, "{body}");
    }
}

#[test]
fn blocks_left_open_and_out_of_place() {
    // Both left open at `@end`, the inner one first in the file's order.
    assert_eq!(
        errors("@scene a\n> x\n@if test_lord\n@if test_knight\n@end\n"),
        ["t.dlg:3: @if has no @endif", "t.dlg:4: @if has no @endif"]
    );
    // Left open at the next scene and at the end of the file.
    assert_eq!(
        errors("@scene a\n> x\n@if test_lord\n@scene b\n> y\n@if test_lord\n> z\n"),
        [
            "t.dlg:1: scene \"a\" has no @end",
            "t.dlg:3: @if has no @endif",
            "t.dlg:4: scene \"b\" has no @end",
            "t.dlg:6: @if has no @endif",
        ]
    );
    assert_eq!(
        errors("@if test_lord\n@scene a\n> x\n@end\n"),
        ["t.dlg:1: line is outside a scene; start one with @scene <id>"]
    );
    // A choice in a block in a reaction is a choice in a choice.
    let nested =
        "> x\n@choice\n* a: One.\n  @if test_knight\n  @choice\n  @endif\n* b: Two.\n@endchoice";
    assert_eq!(
        errors(&scene(nested)),
        ["t.dlg:8: @choice can't be nested inside another @choice"]
    );
    // Inside a choice, a block's lines are reaction lines.
    let column_0 = "@if test_knight\n> x\n@choice\n* a: One.\n@else\n* b: Two.\n@endchoice\n@endif";
    let inside = "inside @choice, a line is an option (\"* tone: text\"), an indented reaction \
                  line or @endchoice";
    assert_eq!(errors(&scene(column_0)), [format!("t.dlg:8: {inside}")]);
    // `@endchoice` in a reaction ends the choice, and a block left open.
    let indented = "> x\n@choice\n* a: One.\n* b: Two.\n  @if test_knight\n  @endchoice";
    assert_eq!(errors(&scene(indented)), ["t.dlg:8: @if has no @endif"]);
    let (parsed, _) = parse_dlg("t.dlg", &scene(indented));
    assert!(matches!(
        parsed[0].scene.steps.last(),
        Some(Step::Choice { options }) if options.len() == 2
    ));
}

// --- The scene checks ---------------------------------------------------------

#[test]
fn a_blocks_character_must_be_known() {
    assert_eq!(
        errors(&scene("@if ghost\n> x\n@endif")),
        ["t.dlg:4: unknown character \"ghost\""]
    );
    // Someone who only speaks may be asked about too.
    assert_eq!(
        errors(&scene("@if messenger\n> x\n@endif")),
        Vec::<String>::new()
    );
}

#[test]
fn both_parts_leave_the_same_screen_and_caption() {
    let screens = |with: &str, without: &str| {
        format!(
            "with \"test_knight\" here test_lord is on the left and {with} on the right; \
             without, test_lord is on the left and {without} on the right; an @if block must \
             leave the same characters on screen either way"
        )
    };
    // Without an `@else`: reported on the `@if`.
    assert_eq!(
        errors(&scene("@if test_knight\n@right clear\n> a\n@endif")),
        [format!("t.dlg:4: {}", screens("nobody", "test_knight"))]
    );
    // With one: on the `@else`.
    assert_eq!(
        errors(&scene(
            "@if test_knight\n> a\n@else\n@right test_archer neutral\n> b\n@endif"
        )),
        [format!(
            "t.dlg:6: {}",
            screens("test_knight", "test_archer")
        )]
    );
    let caption = "this @if block leaves a different caption with \"test_knight\" here and \
                   without; it must leave the same caption either way";
    assert_eq!(
        errors(&scene(
            "@if test_knight\n@caption Dusk\n> a\n@else\n> b\n@endif"
        )),
        [format!("t.dlg:7: {caption}")]
    );
    assert_eq!(
        errors(&scene(
            "@caption Dawn\n@if test_knight\n@caption Dusk\n> a\n@endif"
        )),
        [format!("t.dlg:5: {caption}")]
    );
    // The same either way: fine, and the scene goes on from there.
    let swap = "@if test_knight\n@right test_archer neutral\n@caption Dusk\n> a\n@else\n\
                @right test_archer sad\n@caption Dusk\n@endif\ntest_archer: Hi.\ntest_knight: Hi.";
    assert_eq!(
        errors(&scene(swap)),
        [
            "t.dlg:13: \"test_knight\" speaks but is not on screen; place them with @left or \
             @right first"
        ]
    );
}

/// The steps inside a block get every check a step outside one gets, in
/// both parts.
#[test]
fn steps_in_a_block_are_checked() {
    let body = "@if test_knight\ntest_archer: Hi.\n@else\n> {oops}\n@left nobody neutral\n\
                @left test_lord neutral\n@endif";
    assert_eq!(
        errors(&scene(body)),
        [
            "t.dlg:5: \"test_archer\" speaks but is not on screen; place them with @left or \
             @right first",
            "t.dlg:7: unknown token \"{oops}\"; use {lead}, {they}, {them}, {their}, {theirs}, \
             {themself} (capitalised: {They}...) or a name, {n:<id>}",
            "t.dlg:8: unknown character \"nobody\"",
        ]
    );
}

#[test]
fn a_reactions_limit_counts_a_blocks_longer_part() {
    let choice = |reaction: &str| {
        scene(&format!(
            "> x\n@choice\n* a: One.\n{reaction}\n* b: Two.\n@endchoice"
        ))
    };
    let limit = "reaction has 5 speech or narration lines; the limit is 4, so the scene \
                 rejoins quickly";
    // Two outside the block, three in its longer part, whichever it is.
    let long = "  > 1\n  > 2\n  > 3";
    let with = format!("  > a\n  > b\n  @if test_knight\n{long}\n  @else\n  > 1\n  @endif");
    let without = format!("  > a\n  > b\n  @if test_knight\n  > 1\n  @else\n{long}\n  @endif");
    assert_eq!(errors(&choice(&with)), [format!("t.dlg:6: {limit}")]);
    assert_eq!(errors(&choice(&without)), [format!("t.dlg:6: {limit}")]);
    // Four at most either way: fine, though it holds five lines.
    let fits = "  > a\n  > b\n  @if test_knight\n  > 1\n  > 2\n  @else\n  > 3\n  @endif";
    assert_eq!(errors(&choice(fits)), Vec::<String>::new());
}

#[test]
fn a_scene_whose_text_is_all_in_blocks_has_text() {
    let only_else = "@scene a\n@if test_knight\n@else\n> x\n@endif\n@end\n";
    assert_eq!(errors(only_else), Vec::<String>::new());
    let only_then = "@scene a\n@if test_knight\n> x\n@endif\n@end\n";
    assert_eq!(errors(only_then), Vec::<String>::new());
    assert_eq!(
        errors("@scene a\n@if test_knight\n@caption Dusk\n@else\n@caption Dusk\n@endif\n@end\n"),
        ["t.dlg:1: scene \"a\" has no speech or narration"]
    );
    let step = |src: &str| parse_dlg("t.dlg", src).0[0].scene.steps[0].clone();
    assert!(step(only_else).has_text());
    assert!(step(only_then).has_text());
    assert_eq!(step(only_then).text(), None);
}

#[test]
fn lines_in_both_parts_get_ids_in_script_order() {
    let (parsed, _) = parse_dlg("t.dlg", &scene(NESTED));
    let lines = parsed[0].scene.lines();
    let texts: Vec<&str> = lines.iter().map(|l| l.text).collect();
    assert_eq!(
        texts,
        [
            "Both of us.",
            "Only me.",
            "Good.",
            "She nods. Twice.",
            "Nobody nods.",
            "Move.",
            "Moving."
        ]
    );
    let ids: BTreeSet<&str> = lines.iter().map(|l| l.id.as_str()).collect();
    assert_eq!(ids.len(), 7);
    assert!(ids.iter().all(|id| id.starts_with("s_") && id.len() == 10));
    // The same line in both parts is a repeat.
    let (parsed, _) = parse_dlg(
        "t.dlg",
        "@scene s\n@if test_knight\n> x\n@else\n> x\n@endif\n@end\n",
    );
    let lines = parsed[0].scene.lines();
    assert_eq!(format!("{}_2", lines[0].id), lines[1].id.as_str());
}

#[test]
fn blocks_print_as_written() {
    let src = scene(&format!("{BLOCKS}\n{NESTED}"));
    let (parsed, _) = parse_dlg("t.dlg", &src);
    // Printing puts one space after `@left`/`@right` and joins a
    // continuation line to its line.
    let expected = src.replace("  > She nods.\n    Twice.", "  > She nods. Twice.");
    assert_eq!(print_scene(&parsed[0].scene), expected);
    // An `@else` with nothing after it isn't printed.
    let (parsed, _) = parse_dlg(
        "t.dlg",
        "@scene s\n@if test_knight\n> x\n@else\n@endif\n@end\n",
    );
    assert_eq!(
        print_scene(&parsed[0].scene),
        "@scene s\n@if test_knight\n> x\n@endif\n@end\n"
    );
}

// --- Who is there -------------------------------------------------------------

#[test]
fn present_is_everyone_or_only_some() {
    assert!(Present::Everyone.has(&id("anyone")));
    let some = Present::only([id("test_knight")]);
    assert!(some.has(&id("test_knight")));
    assert!(!some.has(&id("test_archer")));
    assert_eq!(some, Present::Only(ids(&["test_knight"])));
}

#[test]
fn a_resolved_scene_keeps_the_parts_that_hold() {
    let (parsed, _) = parse_dlg("t.dlg", &scene(&format!("{BLOCKS}\n{NESTED}")));
    let scene = &parsed[0].scene;
    let texts = |present: &Present| -> Vec<String> {
        let resolved = scene.resolved(present);
        assert_eq!(resolved.id, scene.id);
        // No block is left, in the scene or in a reaction.
        let no_blocks = |steps: &[Step]| !steps.iter().any(|s| matches!(s, Step::If { .. }));
        assert!(no_blocks(&resolved.steps));
        for step in &resolved.steps {
            if let Step::Choice { options } = step {
                assert!(options.iter().all(|o| no_blocks(&o.steps)));
            }
        }
        // Each line that stays keeps its id.
        let all = scene.lines();
        for line in resolved.lines() {
            assert!(all.contains(&line), "{line:?}");
        }
        resolved.lines().iter().map(|l| l.text.to_owned()).collect()
    };
    assert_eq!(
        texts(&Present::Everyone),
        [
            "Here.",
            "She waves.",
            "Both of us.",
            "Good.",
            "She nods. Twice.",
            "Move.",
            "Moving."
        ]
    );
    assert_eq!(
        texts(&Present::only([id("test_knight")])),
        [
            "Here.",
            "Only me.",
            "Good.",
            "Nobody nods.",
            "Move.",
            "Moving."
        ]
    );
    assert_eq!(
        texts(&Present::only([id("test_archer")])),
        ["Nobody answers.", "She waves."]
    );
    assert_eq!(texts(&Present::only([])), ["Nobody answers."]);
    // The steps outside blocks stay, in order.
    let resolved = scene.resolved(&Present::only([]));
    assert_eq!(
        bare(&resolved.steps),
        [
            place(Side::Left, "test_lord", "neutral"),
            place(Side::Right, "test_knight", "neutral"),
            narrate("Nobody answers."),
        ]
    );
}

#[test]
fn a_scene_with_nothing_left_to_say_has_no_text() {
    let (parsed, _) = parse_dlg(
        "t.dlg",
        "@scene s\n@left test_lord neutral\n@if test_knight\n> x\n@endif\n@end\n",
    );
    let scene = &parsed[0].scene;
    assert!(scene.has_text());
    assert!(scene.resolved(&Present::Everyone).has_text());
    assert!(!scene.resolved(&Present::only([])).has_text());
}

// --- The presence check ---------------------------------------------------------

/// A cast in which `absent` may be gone and scene `s` can't play without
/// `certain`.
fn cast(absent: &[&str], certain: &[&str]) -> Cast {
    Cast {
        may_be_absent: ids(absent),
        certain: [("s".to_owned(), ids(certain))].into(),
    }
}

/// The presence errors of `src` (which must parse) for `cast`.
fn presence(src: &str, cast: &Cast) -> Vec<String> {
    let (parsed, errors) = parse_dlg("t.dlg", src);
    assert_eq!(errors, []);
    let errors = parsed.iter().flat_map(|p| check_presence(p, cast));
    errors.map(|e| e.to_string()).collect()
}

fn may_be_gone(line: u32, id: &str) -> String {
    format!(
        "t.dlg:{line}: \"{id}\" may have fallen or left the army by now; put their lines inside \
         \"@if {id}\" ... \"@endif\" (assets/dialogue/README.md, \"Who is still there\")"
    )
}

#[test]
fn someone_who_may_be_gone_appears_only_in_their_block() {
    let knight = cast(&["test_knight"], &[]);
    // Placed (line 3) and speaking (line 4) outside a block.
    assert_eq!(
        presence(&scene("test_knight: Hi."), &knight),
        [may_be_gone(3, "test_knight"), may_be_gone(4, "test_knight")]
    );
    // Nobody else is asked for one.
    assert_eq!(
        presence(&scene("test_lord: Hi."), &knight)[1..],
        [] as [&str; 0]
    );
    assert_eq!(
        presence(&scene("test_knight: Hi."), &cast(&["test_archer"], &[])),
        Vec::<String>::new()
    );
    let src = "@scene s\n@left test_lord neutral\n@if test_knight\n@right test_knight neutral\n\
               test_knight: Hi.\n@right clear\n@endif\n> x\n@end\n";
    assert_eq!(presence(src, &knight), Vec::<String>::new());
    // The block is over: they may be gone again.
    let after = src.replace("> x", "test_knight: Still here.");
    assert_eq!(presence(&after, &knight), [may_be_gone(8, "test_knight")]);
    // Another character's block doesn't do.
    let other = src.replace("@if test_knight", "@if test_archer");
    assert_eq!(
        presence(&other, &knight),
        [may_be_gone(4, "test_knight"), may_be_gone(5, "test_knight")]
    );
    // A block inside theirs still has them, and so does a reaction in it.
    let inside = "@scene s\n@if test_knight\n@if test_archer\ntest_knight: Hi.\n@endif\n\
                  @choice\n* a: One.\n  test_knight: One.\n* b: Two.\n@endchoice\n@endif\n@end\n";
    assert_eq!(presence(inside, &knight), Vec::<String>::new());
    // A reaction is checked like the rest: its own block, or an error.
    let reaction = "@scene s\n> x\n@choice\n* a: One.\n  @if test_knight\n  test_knight: One.\n  \
                    @endif\n* b: Two.\n  test_knight: Two.\n@endchoice\n@end\n";
    assert_eq!(presence(reaction, &knight), [may_be_gone(9, "test_knight")]);
}

#[test]
fn nobody_appears_in_the_else_of_their_own_block() {
    let src = "@scene s\n@if test_knight\n> x\n@else\n@right test_knight sad\n\
               test_knight: Boo.\n@if test_archer\ntest_knight: Boo again.\n@endif\n@endif\n\
               @right test_knight sad\n@end\n";
    let gone = |line: u32| {
        format!(
            "t.dlg:{line}: \"test_knight\" is gone here: this is the @else of \"@if test_knight\""
        )
    };
    // Whether or not the army can lose them.
    assert_eq!(presence(src, &cast(&[], &[])), [gone(5), gone(6), gone(8)]);
    assert_eq!(
        presence(src, &cast(&["test_knight"], &[])),
        [gone(5), gone(6), gone(8), may_be_gone(11, "test_knight")]
    );
}

#[test]
fn a_scene_that_cant_play_without_someone_needs_no_block_for_them() {
    let src = scene("test_knight: Hi.");
    let certain = cast(&["test_knight"], &["test_knight"]);
    assert_eq!(presence(&src, &certain), Vec::<String>::new());
    // Only in that scene.
    let other = src.replace("@scene s", "@scene other");
    assert_eq!(
        presence(&other, &certain),
        [may_be_gone(3, "test_knight"), may_be_gone(4, "test_knight")]
    );
}

#[test]
fn a_block_that_can_only_go_one_way_is_an_error() {
    let always = |line: u32, id: &str, why: &str| {
        format!(
            "t.dlg:{line}: \"{id}\" is always here at this line ({why}); this block's @else \
             would never play"
        )
    };
    let inside = "this is inside an \"@if\" for them, or the scene can't play without them";
    let src = "@scene s\n@if test_knight\n@if test_knight\n> x\n@endif\n@else\n@if test_knight\n\
               > y\n@endif\n@endif\n@if test_knight\n> z\n@endif\n@if lead\n> w\n@endif\n@end\n";
    assert_eq!(
        presence(src, &cast(&["test_knight"], &[])),
        [
            always(3, "test_knight", inside),
            "t.dlg:7: \"test_knight\" is known to be gone here (this is inside the @else of an \
             \"@if test_knight\"); this block's first part would never play"
                .to_owned(),
            always(14, "lead", "the lead is in every scene's army"),
        ]
    );
    // In a scene that can't play without them, every block for them.
    let src = "@scene s\n@if test_knight\n> x\n@endif\n@end\n";
    assert_eq!(
        presence(src, &cast(&[], &["test_knight"])),
        [always(2, "test_knight", inside)]
    );
    assert_eq!(presence(src, &cast(&[], &[])), Vec::<String>::new());
}

/// The two examples under "Who is still there" in
/// `assets/dialogue/README.md` pass every check, with everyone they put
/// in a block able to be gone.
#[test]
fn readme_if_examples_are_valid() {
    let readme = bundle::file("dialogue/README.md").unwrap_or_default();
    let everyone = cast(&["sergeant", "keeper", "heretic"], &[]);
    for (nth, blocks) in [(9, 1), (11, 3)] {
        let example = readme.split("```").nth(nth).unwrap_or_default();
        let scripts = scripts_from_sources(
            &[("README.md", example)],
            Some(&characters()),
            None,
            Some(&names()),
            None,
        );
        let scripts = scripts.unwrap_or_else(|e| panic!("{e:?}"));
        let scene = &scripts.parsed[0];
        let count = print_scene(&scene.scene).matches("@if ").count();
        assert_eq!(count, blocks, "{example}");
        assert_eq!(check_presence(scene, &everyone), [], "{example}");
    }
}

/// `ch01.dlg` never shows a companion who may have fallen (0716). The
/// check only bites for characters in the New Game roster, and the five
/// companions join it in 0803; until then this runs it with the cast the
/// script's header describes. 0803 deletes this test once the battle and
/// chapter files say the same.
#[test]
fn chapter_1_never_shows_a_companion_who_may_have_fallen() {
    const COMPANIONS: [&str; 5] = ["retainer", "sergeant", "poacher", "keeper", "heretic"];
    let source = bundle::file("dialogue/ch01.dlg").unwrap_or_default();
    let scripts = scripts_from_sources(
        &[("ch01.dlg", source)],
        Some(&characters()),
        None,
        Some(&names()),
        None,
    );
    let scripts = scripts.unwrap_or_else(|e| panic!("{e:?}"));
    // Who each scene can't play without, as its trigger will name them.
    let certain = |scene: &str| match scene {
        // Before anyone can have fallen.
        "ch01_intro" | "ch01_prebattle" | "ch01_first_turn" => ids(&COMPANIONS),
        "ch01_boss_engage_sergeant" => ids(&["sergeant"]),
        // A death quote or a retreat line is said by the one who falls.
        _ => {
            let fallen = ["ch01_death_", "ch01_retreat_"]
                .iter()
                .find_map(|prefix| scene.strip_prefix(prefix));
            fallen.map(|who| ids(&[who])).unwrap_or_default()
        }
    };
    let scenes = scripts.parsed.iter().map(|p| p.scene.id.clone());
    let cast = Cast {
        may_be_absent: ids(&COMPANIONS),
        certain: scenes.map(|s| (s.clone(), certain(&s))).collect(),
    };
    assert_eq!(cast.certain["ch01_victory"], ids(&[]));
    assert_eq!(cast.certain["ch01_death_keeper"], ids(&["keeper"]));
    let errors: Vec<String> = scripts
        .parsed
        .iter()
        .flat_map(|p| check_presence(p, &cast))
        .map(|e| e.to_string())
        .collect();
    assert_eq!(errors, Vec::<String>::new());
}

// --- The cast -------------------------------------------------------------------

fn embedded() -> crate::Content {
    crate::load_embedded().unwrap_or_else(|e| panic!("{e}"))
}

fn cast_of(content: &crate::Content) -> Cast {
    Cast::new(
        &content.new_game,
        &content.battles,
        &content.chapters,
        &content.supports,
    )
}

/// The placeholder game: the army can lose the knight (in the New Game
/// roster) and the rogue (joins if defeated), not the lead.
#[test]
fn the_cast_comes_from_the_roster_the_battles_the_chapters_and_the_supports() {
    let content = embedded();
    let cast = cast_of(&content);
    assert_eq!(cast.may_be_absent, ids(&["test_knight", "test_rogue"]));
    let certain = |scene: &str| cast.certain.get(scene).cloned();
    // A trigger's scene can't play without those the trigger is about.
    assert_eq!(certain("test_turn_3"), Some(ids(&[])));
    assert_eq!(certain("test_fort"), Some(ids(&[])));
    assert_eq!(certain("test_engage"), Some(ids(&["test_rogue"])));
    assert_eq!(certain("test_rogue_half"), Some(ids(&["test_rogue"])));
    assert_eq!(certain("test_rogue_falls"), Some(ids(&["test_rogue"])));
    assert_eq!(certain("test_knight_dies"), Some(ids(&["test_knight"])));
    assert_eq!(
        certain("test_talk"),
        Some(ids(&["test_lord", "test_rogue"]))
    );
    // A support conversation, without its pair.
    assert_eq!(
        certain("test_support_lord_knight_c"),
        Some(ids(&["test_knight", "test_lord"]))
    );
    assert_eq!(
        certain("test_support_lord_mage_a"),
        Some(ids(&["test_lord", "test_mage"]))
    );
    // The first chapter's intro, before anyone can have fallen: the
    // starting roster. Its victory scenes can play without anyone; a scene
    // nothing plays isn't listed.
    assert_eq!(certain("test_intro"), Some(ids(&["lead", "test_knight"])));
    assert_eq!(certain("test_victory"), Some(ids(&[])));
    assert_eq!(certain("test"), None);
}

/// Before anyone can have fallen, a scene can count on the starting
/// roster: the first chapter's intro scenes, and the scenes its battle
/// plays as turn 1's player phase starts, for those the battle places.
#[test]
fn the_starting_roster_is_certain_until_the_first_battle_is_under_way() {
    let mut content = embedded();
    assert_eq!(content.new_game.first_chapter, "test");
    let turn_start = |turn, phase, scene: &str| Trigger {
        when: TriggerWhen::TurnStart { turn, phase },
        scene: scene.into(),
        once: true,
    };
    let first = content.battles.get_mut("test").unwrap();
    first.triggers = vec![
        turn_start(1, Phase::Player, "first_opening"),
        turn_start(1, Phase::Enemy, "first_enemy"),
        turn_start(2, Phase::Player, "first_turn_2"),
    ];
    let quick = content.battles.get_mut("quick").unwrap();
    quick
        .triggers
        .push(turn_start(1, Phase::Player, "quick_opening"));
    // The test archer starts in the army, but the first battle has no
    // place for them: not on its map.
    content.new_game.roster.push(id("test_archer"));
    // A second chapter, with the same battle.
    let mut second = content.chapters["test"].clone();
    second.intro_scenes = vec!["second_intro".into()];
    content.chapters.insert("second".into(), second);
    let cast = cast_of(&content);
    let certain = |scene: &str| cast.certain.get(scene).cloned();
    let roster = ids(&["lead", "test_archer", "test_knight"]);
    assert_eq!(certain("test_intro"), Some(roster));
    assert_eq!(certain("second_intro"), Some(ids(&[])));
    assert_eq!(
        certain("first_opening"),
        Some(ids(&["lead", "test_knight"]))
    );
    assert_eq!(certain("first_enemy"), Some(ids(&[])));
    assert_eq!(certain("first_turn_2"), Some(ids(&[])));
    // Another battle's opening can't: its chapter may come later. Nor
    // can it count on those it places who aren't in the starting roster.
    assert_eq!(certain("quick_opening"), Some(ids(&[])));
    content.new_game.first_chapter = "quick".into();
    let cast = cast_of(&content);
    assert_eq!(
        cast.certain.get("quick_opening"),
        Some(&ids(&["test_archer", "test_knight"]))
    );
    assert_eq!(cast.certain.get("first_opening"), Some(&ids(&[])));
    assert_eq!(cast.certain.get("test_intro"), Some(&ids(&[])));
}

#[test]
fn every_kind_of_trigger_names_who_its_scene_cant_play_without() {
    let mut content = embedded();
    let area = trpg_core::TileRect {
        x: 0,
        y: 0,
        w: 1,
        h: 1,
    };
    let whens = [
        TriggerWhen::UnitEntersArea {
            who: Who::Character(id("test_archer")),
            area,
        },
        TriggerWhen::CombatStart {
            unit: id("test_rogue"),
            against: Some(id("test_mage")),
        },
        TriggerWhen::UnitFell {
            unit: id("test_archer"),
            mode: None,
            recruit: false,
        },
    ];
    let battle = content.battles.get_mut("quick").unwrap();
    for (i, when) in whens.into_iter().enumerate() {
        battle.triggers.push(Trigger {
            when,
            scene: format!("new_{i}"),
            once: true,
        });
    }
    let cast = cast_of(&content);
    let certain = |scene: &str| cast.certain.get(scene).cloned();
    assert_eq!(certain("new_0"), Some(ids(&["test_archer"])));
    assert_eq!(certain("new_1"), Some(ids(&["test_mage", "test_rogue"])));
    assert_eq!(certain("new_2"), Some(ids(&["test_archer"])));
    // A fall that doesn't recruit adds nobody the army can lose.
    assert_eq!(cast.may_be_absent, ids(&["test_knight", "test_rogue"]));
}

#[test]
fn a_scene_played_from_several_places_keeps_who_all_of_them_have() {
    let mut cast = Cast::default();
    cast.played("s", ids(&["a", "b"]));
    assert_eq!(cast.certain["s"], ids(&["a", "b"]));
    cast.played("s", ids(&["b", "c"]));
    assert_eq!(cast.certain["s"], ids(&["b"]));
    cast.played("t", ids(&["c"]));
    assert_eq!(cast.certain["t"], ids(&["c"]));
    cast.played("s", ids(&[]));
    assert_eq!(cast.certain["s"], ids(&[]));
    // A trigger's scene that a chapter plays too: nobody is certain.
    let mut content = embedded();
    let chapter = content.chapters.get_mut("test").unwrap();
    chapter.intro_scenes.push("test_engage".into());
    chapter.victory_scenes.push("test_talk".into());
    let cast = cast_of(&content);
    assert_eq!(cast.certain["test_engage"], ids(&[]));
    assert_eq!(cast.certain["test_talk"], ids(&[]));
}

// --- Properties -----------------------------------------------------------------

const CAST: [&str; 3] = ["a", "b", "c"];

fn arb_member() -> impl Strategy<Value = &'static str> {
    proptest::sample::select(&CAST[..])
}

/// A line of `who`: placed, or speaking.
fn arb_line(who: impl Strategy<Value = &'static str>) -> impl Strategy<Value = Step> {
    (who, any::<bool>()).prop_map(|(who, places)| {
        if places {
            place(Side::Left, who, "neutral")
        } else {
            say(who, None, "Hi.")
        }
    })
}

/// A block for one of the cast, `depth` more blocks deep: mostly that
/// character's lines in its first part and narration in its second, so
/// that many scenes pass the check, and now and then anyone's line in
/// either, so that many don't.
fn arb_block(depth: u32) -> BoxedStrategy<Step> {
    arb_member()
        .prop_flat_map(move |who| {
            let inner = |weight: u32| {
                if depth == 0 {
                    Just(narrate("...")).boxed()
                } else {
                    prop_oneof![weight => arb_block(depth - 1), 1 => Just(narrate("..."))].boxed()
                }
            };
            let then = prop_oneof![
                5 => arb_line(Just(who)),
                1 => arb_line(arb_member()),
                2 => Just(narrate("...")),
                2 => inner(4),
            ];
            let otherwise = prop_oneof![
                6 => Just(narrate("...")),
                1 => arb_line(arb_member()),
                2 => inner(4),
            ];
            let steps = |s| proptest::collection::vec(s, 0..4);
            (steps(then.boxed()), steps(otherwise.boxed()))
                .prop_map(move |(then, otherwise)| when(who, then, otherwise))
        })
        .boxed()
}

/// A scene of blocks, narration, a choice now and then, and now and then
/// a line outside any block.
fn arb_presence_scene() -> impl Strategy<Value = Scene> {
    let reaction =
        || proptest::collection::vec(prop_oneof![arb_block(1), Just(narrate("..."))], 0..3);
    let choice = (reaction(), reaction()).prop_map(|(a, b)| {
        let option = |steps| ChoiceOption {
            tone: "t".into(),
            text: "Reply.".into(),
            line: LineId::default(),
            steps,
        };
        Step::Choice {
            options: vec![option(a), option(b)],
        }
    });
    let step = prop_oneof![
        6 => arb_block(2),
        3 => Just(narrate("...")),
        2 => choice,
        1 => arb_line(arb_member()),
    ];
    proptest::collection::vec(step, 0..6).prop_map(|steps| Scene {
        id: "s".into(),
        steps,
    })
}

/// The characters `steps` place or give a line to, reactions included.
fn shown(steps: &[Step], out: &mut BTreeSet<CharacterId>) {
    for step in steps {
        match step {
            Step::Place { character, .. } => {
                out.insert(character.clone());
            }
            Step::Say { speaker, .. } => {
                out.insert(speaker.clone());
            }
            Step::Choice { options } => {
                for o in options {
                    shown(&o.steps, out);
                }
            }
            Step::If { .. } => panic!("a resolved scene has no blocks"),
            _ => {}
        }
    }
}

proptest! {
    /// A scene that passes the presence check never shows a character who
    /// isn't there, whoever is there, whichever replies are picked.
    #[test]
    fn a_checked_scene_never_shows_someone_who_is_gone(
        scene in arb_presence_scene(),
        there in proptest::collection::btree_set(arb_member(), 0..=3),
    ) {
        let (parsed, errors) = parse_dlg("t.dlg", &print_scene(&scene));
        prop_assert_eq!(errors, []);
        let everyone = cast(&CAST, &[]);
        if check_presence(&parsed[0], &everyone).is_empty() {
            let present = Present::only(there.iter().map(|c| id(c)));
            let mut on_screen = BTreeSet::new();
            shown(&parsed[0].scene.resolved(&present).steps, &mut on_screen);
            let gone: Vec<_> = on_screen.iter().filter(|c| !present.has(c)).collect();
            prop_assert!(gone.is_empty(), "{gone:?} shown with only {there:?} there");
        }
    }

    /// Resolving a scene for everyone, or for nobody, takes the first or
    /// the second part of every block: the lines left are a subset, in
    /// the same order.
    #[test]
    fn resolving_keeps_lines_in_order(scene in arb_presence_scene(), all in any::<bool>()) {
        let mut scene = scene;
        scene.assign_line_ids();
        let present = if all { Present::Everyone } else { Present::only([]) };
        let resolved = scene.resolved(&present);
        let all_lines = scene.lines();
        let mut rest = all_lines.iter();
        for line in resolved.lines() {
            prop_assert!(rest.any(|l| *l == line), "{line:?} out of order");
        }
    }
}

/// The generator above makes scenes that pass and scenes that don't, so
/// the property is tested on both.
#[test]
fn the_presence_property_sees_passing_and_failing_scenes() {
    use proptest::strategy::ValueTree;
    use proptest::test_runner::TestRunner;
    let mut runner = TestRunner::deterministic();
    let everyone = cast(&CAST, &[]);
    let (mut ok, mut with_lines) = (0, 0);
    for _ in 0..200 {
        let scene = arb_presence_scene()
            .new_tree(&mut runner)
            .unwrap()
            .current();
        let (parsed, _) = parse_dlg("t.dlg", &print_scene(&scene));
        if check_presence(&parsed[0], &everyone).is_empty() {
            ok += 1;
            let mut on_screen = BTreeSet::new();
            shown(
                &parsed[0].scene.resolved(&Present::Everyone).steps,
                &mut on_screen,
            );
            with_lines += usize::from(!on_screen.is_empty());
        }
    }
    assert!((20..180).contains(&ok), "{ok} of 200 pass");
    assert!(with_lines >= 10, "{with_lines} passing scenes show someone");
}
