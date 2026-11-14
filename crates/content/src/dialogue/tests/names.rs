//! Tests of name tokens (`{n:<id>}`) and names written out (0709).
//!
//! They check the mechanism against a made-up names table ([`table`]),
//! not the game's (ADR-0118 §1 (b)); only the README's example is checked
//! against the real one.

use super::*;

/// A names table of made-up names under the game's ids, for the checks
/// of names written out.
fn table() -> Names {
    let names = [
        ("lead", "Ellery"),
        ("king", "Osric"),
        ("poacher", "Brannoc"),
        ("heretic", "Hale"),
        ("retainer", "Corin Ashby"),
        ("retainer.first", "Corin"),
        ("family.marr", "Ashby"),
        ("family.veyne", "Veyne"),
        ("red_captain.nickname", "Black Tam"),
        ("sergeant.last", "Pike"),
        ("vowmaster.last", "Heron"),
        ("vosse.last", "Garrow"),
        ("envoy.last", "Fenn"),
        ("god.mother.name", "Eda"),
        ("god.mother.title", "the Weaver"),
        ("place.harrowby", "Dunmere"),
        ("place.thornmarch", "the Fenwold"),
        ("faction.crown", "the Crown of Elmar"),
        ("term.vow", "a vow"),
        ("term.breath", "breath"),
    ];
    Names {
        names: names
            .iter()
            .map(|&(id, name)| (id.to_owned(), name.to_owned()))
            .collect(),
    }
}

/// Every error for one file `t.dlg`, checked against the embedded
/// (placeholder) characters and [`table`].
fn errors(src: &str) -> Vec<String> {
    from_sources(
        &[("t.dlg", src)],
        Some(&characters()),
        None,
        Some(&table()),
        None,
    )
    .err()
    .unwrap_or_default()
    .iter()
    .map(ToString::to_string)
    .collect()
}

#[test]
fn known_name_tokens_pass() {
    let src = scene(
        "> {n:king} rides for {n:place.thornmarch}. {N:place.thornmarch} is cold.
@caption {N:place.harrowby}, dusk
test_lord: {n:faction.crown}, {lead}.",
    );
    assert_eq!(errors(&src), Vec::<String>::new());
}

#[test]
fn unknown_name_ids() {
    let src = scene("> {n:kingg} and {n:} and {N:King}.\n@caption {n:place.nowhere}");
    let hint = "name ids are listed in assets/data/names.ron";
    assert_eq!(
        errors(&src),
        [
            format!("t.dlg:4: unknown name id \"kingg\" in {{n:kingg}}; {hint}"),
            format!("t.dlg:4: unknown name id \"\" in {{n:}}; {hint}"),
            format!("t.dlg:4: unknown name id \"King\" in {{N:King}}; {hint}"),
            format!("t.dlg:5: unknown name id \"place.nowhere\" in {{n:place.nowhere}}; {hint}"),
        ]
    );
}

#[test]
fn without_a_names_table_only_the_id_syntax_is_checked() {
    let src = scene("> {n:anything} and {n:Bad}.");
    let errs: Vec<String> = from_sources(&[("t.dlg", src.as_str())], None, None, None, None)
        .err()
        .unwrap_or_default()
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        errs,
        [
            "t.dlg:4: unknown name id \"Bad\" in {n:Bad}; name ids are listed in assets/data/names.ron"
        ]
    );
}

#[test]
fn the_lead_is_not_a_name_token() {
    assert_eq!(
        errors(&scene("> {n:lead} waits.")),
        [
            "t.dlg:4: {n:lead} is only the lead's default name; write {lead} for the name the \
             player chose"
        ]
    );
}

#[test]
fn unterminated_name_token() {
    assert_eq!(
        errors(&scene("> Ride for {n:king.")),
        ["t.dlg:4: \"{\" has no closing \"}\""]
    );
}

#[test]
fn names_written_out_are_errors() {
    let src = scene(
        "> Osric rides.
@caption Dunmere, dusk
test_lord: Back to Fenwold.
test_knight: For the Crown of Elmar!
@choice
* a: Brannoc can shoot.
  > Fine.
* b: No.
  > Fine.
@endchoice",
    );
    assert_eq!(
        errors(&src),
        [
            "t.dlg:4: \"Osric\" is written out; write {n:king} (\"Osric\") so a rename reaches \
             this line",
            "t.dlg:5: \"Dunmere\" is written out; write {n:place.harrowby} (\"Dunmere\") so a \
             rename reaches this line",
            "t.dlg:6: \"Fenwold\" is written out; write {n:place.thornmarch} (\"the \
             Fenwold\") so a rename reaches this line",
            "t.dlg:7: \"Crown of Elmar\" is written out; write {n:faction.crown} (\"the Crown \
             of Elmar\") so a rename reaches this line",
            "t.dlg:9: \"Brannoc\" is written out; write {n:poacher} (\"Brannoc\") so a rename \
             reaches this line",
        ]
    );
}

#[test]
fn short_forms_written_out_are_errors() {
    let src = scene(
        "test_knight: Corin, wait.
> Corin Ashby did not wait.
test_lord: An Ashby never does. Ask Black Tam.
> By Eda. By the Weaver.
test_lord: {N:retainer.first}, wait. {N:god.mother.title} keep you, {n:family.veyne}.",
    );
    assert_eq!(
        errors(&src),
        [
            "t.dlg:4: \"Corin\" is written out; write {n:retainer.first} (\"Corin\") so a \
             rename reaches this line",
            "t.dlg:5: \"Corin Ashby\" is written out; write {n:retainer} (\"Corin Ashby\") so \
             a rename reaches this line",
            "t.dlg:6: \"Black Tam\" is written out; write {n:red_captain.nickname} (\"Black \
             Tam\") so a rename reaches this line",
            "t.dlg:7: \"Weaver\" is written out; write {n:god.mother.title} (\"the Weaver\") \
             so a rename reaches this line",
        ]
    );
}

#[test]
fn surnames_written_out_are_errors() {
    let src = scene(
        "test_lord: Sergeant Pike, with me.
> Master Heron said nothing.
test_knight: Ask Garrow.
test_lord: {N:sergeant.last}, fetch {n:envoy.last}.",
    );
    assert_eq!(
        errors(&src),
        [
            "t.dlg:4: \"Pike\" is written out; write {n:sergeant.last} (\"Pike\") so a \
             rename reaches this line",
            "t.dlg:5: \"Heron\" is written out; write {n:vowmaster.last} (\"Heron\") so a \
             rename reaches this line",
            "t.dlg:6: \"Garrow\" is written out; write {n:vosse.last} (\"Garrow\") so a \
             rename reaches this line",
        ]
    );
}

#[test]
fn the_lead_default_name_written_out() {
    assert_eq!(
        errors(&scene("test_knight: Ellery, wait.")),
        [
            "t.dlg:4: \"Ellery\" is the lead's default name; write {lead} for the name the \
             player chose"
        ]
    );
}

#[test]
fn only_whole_capitalised_names_in_text_count() {
    // Comments, ordinary words, other case and parts of words aren't hits.
    let src = scene(
        "# Osric's scene: Brannoc and Hale.
> You'll be hale again. I swore a vow; breath fails.
> Osrics, Brannocks, dunmere.",
    );
    assert_eq!(errors(&src), Vec::<String>::new());
}

#[test]
fn name_tokens_count_as_the_longest_name() {
    let longest = table().longest(&shuyi_assets::font::CellWidths::NARROW);
    assert!(longest > "Osric".len());
    // Every token counts as the longest name, whatever its own name's length.
    let base = "a".repeat(MAX_TEXT_LEN - longest);
    assert_eq!(
        errors(&scene(&format!("> {base}{{n:king}}\n> {base}a{{n:king}}"))),
        [format!(
            "t.dlg:5: text is {} characters; the limit is {MAX_TEXT_LEN}",
            MAX_TEXT_LEN + 1
        )]
    );
}

#[test]
fn a_rename_reaches_every_use() {
    let src = scene("> {n:king} rode out. {n:king} came back.");
    let parsed = from_sources(&[("t.dlg", src.as_str())], None, None, Some(&table()), None);
    let text = parsed
        .ok()
        .and_then(|t| {
            t.get("s")
                .and_then(|s| s.steps.last()?.text().map(str::to_owned))
        })
        .unwrap_or_default();
    // The stored scene keeps the tokens; the table fills them in.
    assert_eq!(text, "{n:king} rode out. {n:king} came back.");
    let mut renamed = table();
    renamed.names.insert("king".into(), "Aldric".into());
    assert_eq!(
        table().substitute(&text),
        "Osric rode out. Osric came back."
    );
    assert_eq!(
        renamed.substitute(&text),
        "Aldric rode out. Aldric came back."
    );
}

/// The names example in `assets/dialogue/README.md` passes every check
/// against the real names table and shows the caption and line the README
/// describes (a test of the content, ADR-0118 §1 (a)). The README names
/// no display name (ADR-0118 §2), so only the tokens are compared.
#[test]
fn readme_names_example_is_valid() {
    if crate::bundle::skip_without_the_game() {
        return;
    }
    let example = README.split("```").nth(7).unwrap_or_default();
    let table = from_sources(&[("README.md", example)], None, None, Some(&names()), None);
    assert!(table.is_ok(), "{table:?}");
    let scene = table.unwrap_or_default().scenes.remove("ch01_road");
    let steps = scene.map(|s| s.steps).unwrap_or_default();
    let shown: Vec<String> = steps
        .iter()
        .filter_map(|s| match s {
            Step::Caption { text } => Some(text.as_str()),
            _ => s.text(),
        })
        .map(|t| names().substitute(t).into_owned())
        .collect();
    // What the README says the example shows: the place capitalised in
    // the caption, the king and the place in the line.
    let said = [
        "{N:place.thornmarch}, dusk",
        "{N:king} wants you out of {n:place.thornmarch}.",
    ]
    .map(|t| names().substitute(t).into_owned());
    assert_eq!(shown, said);
}

#[test]
fn unknown_tokens_count_as_written() {
    // "{x}" is 3 characters: 198 + 3 = 201.
    let base = "a".repeat(MAX_TEXT_LEN - 2);
    let errs = errors(&scene(&format!("> {base}{{x}}")));
    assert_eq!(errs.len(), 2, "{errs:?}");
    assert_eq!(
        errs[1],
        format!(
            "t.dlg:4: text is {} characters; the limit is {MAX_TEXT_LEN}",
            MAX_TEXT_LEN + 1
        )
    );
}
