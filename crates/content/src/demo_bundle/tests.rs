use super::*;

fn files(paths: &[&str]) -> Vec<String> {
    paths.iter().map(|p| (*p).to_owned()).collect()
}

fn keys(list: &[&str]) -> BTreeSet<String> {
    list.iter().map(|k| (*k).to_owned()).collect()
}

const ASSETS: &[&str] = &[
    "chapters/ch01.ron",
    "chapters/ch02.ron",
    "data/names.ron",
    "maps/ch01.map",
    "maps/ch02.map",
    "maps/veyne_study.map",
    "worldmaps/act1.ron",
];

#[test]
fn the_list_leaves_out_folders_and_files_and_keeps_back_what_it_names() {
    let plan = Plan::parse(
        "# Chapter 1 only\n\nchapters/\nmaps/\n+ chapters/ch01.ron\n+maps/ch01.map\n\
         + maps/veyne_study.map\nworldmaps/act1.ron\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let out = plan.left_out(&files(ASSETS));
    assert_eq!(
        out.as_deref(),
        Ok(&files(&["chapters/ch02.ron", "maps/ch02.map", "worldmaps/act1.ron"])[..])
    );
    assert!(plan.leaves_out("maps/ch05.map"), "a new file is left out");
    assert!(!plan.leaves_out("data/names.ron"));
}

#[test]
fn a_line_that_matches_nothing_stops_the_build() {
    let plan = |text: &str| Plan::parse(text).unwrap_or_else(|e| panic!("{e}"));
    let no_file = plan("chapters/\nsidequests/\n").left_out(&files(ASSETS));
    assert_eq!(
        no_file,
        Err("line 2: `sidequests/` is no file under assets/ or assets-private/content/".into())
    );
    let keeps_nothing = plan("chapters/\n+ maps/ch01.map\n").left_out(&files(ASSETS));
    assert_eq!(
        keeps_nothing,
        Err("line 2: `maps/ch01.map` keeps no file a line above leaves out".into())
    );
    let cut_gone = plan("chapters/\nkeep chapters/ch02.ron: x\n").left_out(&files(ASSETS));
    assert_eq!(
        cut_gone,
        Err("`chapters/ch02.ron` is cut, but the demo doesn't hold it".into())
    );
    let cut_missing = plan("chapters/\nkeep data/nope.ron: x\n").left_out(&files(ASSETS));
    assert_eq!(
        cut_missing,
        Err("`data/nope.ron` is cut, but the demo doesn't hold it".into())
    );
    let bad = Plan::parse("leave out chapters/");
    assert_eq!(
        bad,
        Err("line 1: not a path, `+`, `keep` or `swap`: `leave out chapters/`".into())
    );
    assert!(Plan::parse("keep data/a.ron one two three: x").is_err());
    assert!(Plan::parse("swap data/a.ron: a -> b").is_err());
}

#[test]
fn comments_go_and_lines_keep_their_numbers() {
    let text = "// The cast (Chapter 4's boss)\n(\n    url: \"https://a.b/c\", // a link\n    \
                glyph: '/', quote: '\\'', /* a /* nested */ note\n over two lines */ x: 1,\n)\n";
    let stripped = strip_comments(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        stripped,
        "\n(\n    url: \"https://a.b/c\", \n    glyph: '/', quote: '\\'', \n x: 1,\n)\n"
    );
    assert!(!stripped.contains("Chapter 4"));
    assert!(strip_comments("(a: \"open)").is_err());
    assert!(strip_comments("/* never").is_err());
    // A mark's bytes are its own: `/*/` opens once, `*/*` closes once.
    for (text, left) in [("/*/ a */x", "x"), ("/* /* */* */x", "x"), ("/**/x", "x")] {
        assert_eq!(strip_comments(text).as_deref(), Ok(left), "{text}");
    }
    assert!(
        strip_comments("/* /* */").is_err(),
        "the outer one never closes"
    );
}

#[test]
fn keep_entries_keeps_a_maps_keys_and_their_dotted_forms() {
    let names = "{\n    \"lead\": \"Ellery\",\n    \"rival\": \"Dace Marr\",\n    \
                 \"rival.first\": \"Dace\",\n    \"rivalry\": \"x\",\n    \"king\": \"Emeric\",\n}\n";
    let cut =
        keep_entries(names, None, &keys(&["lead", "rival"])).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        cut,
        "{\n\"lead\": \"Ellery\",\n\"rival\": \"Dace Marr\",\n\"rival.first\": \"Dace\",\n}\n"
    );
    let gone = keep_entries(names, None, &keys(&["queen"]));
    assert_eq!(gone, Err("keeps `queen`, which is no entry".into()));
}

#[test]
fn keep_entries_keeps_a_fields_structs_by_id_and_strings() {
    let characters = "(\n    characters: [\n        (\n            id: \"lead\", class: \"exile\",\n            \
                      loadout: (weapons: [\"iron_sword\"]),\n        ),\n        \
                      (id: \"defector\", class: \"swordsman\", loadout: (id: \"lead\")),\n    ],\n    \
                      generics: [(id: \"red_brigand\")],\n    speakers: [\"guide\", \"king\"],\n)\n";
    let kept = keep_entries(characters, Some("characters"), &keys(&["lead"]))
        .and_then(|t| keep_entries(&t, Some("speakers"), &keys(&["guide"])))
        .unwrap_or_else(|e| panic!("{e}"));
    assert!(kept.contains("id: \"lead\", class: \"exile\""), "{kept}");
    assert!(kept.contains("iron_sword"), "{kept}");
    assert!(
        !kept.contains("defector"),
        "a nested `id` is not the entry's: {kept}"
    );
    assert!(!kept.contains("king"), "{kept}");
    assert!(
        kept.contains("red_brigand"),
        "another field is as it was: {kept}"
    );
    let no_field = keep_entries(characters, Some("monsters"), &keys(&["lead"]));
    assert_eq!(no_field, Err("no monsters".into()));
    let no_key = keep_entries("[(class: \"x\")]", None, &keys(&["x"]));
    assert_eq!(no_key, Err("line 1: an entry with no key".into()));
    let unbalanced = keep_entries("[\"x\"]\n)", None, &keys(&["x"]));
    assert_eq!(unbalanced, Err("line 2: a bracket closes nothing".into()));
}

/// A field or an `id` is found by its whole name, followed by `:`, at its
/// own depth: not inside a longer name, not as a value, not deeper in.
#[test]
fn keep_entries_finds_a_field_by_its_whole_name() {
    let text = "(\n    old_speakers: [\"a\"],\n    speakers_left: [\"a\"],\n    \
                inner: (speakers: [\"a\"]),\n    speakers: [\"a\", \"b\"],\n    \
                cast: [(kind: id, id: \"x\"), (id: \"y\")],\n)\n";
    let kept =
        keep_entries(text, Some("speakers"), &keys(&["b"])).unwrap_or_else(|e| panic!("{e}"));
    assert!(kept.contains("speakers: [\n\"b\",\n]"), "{kept}");
    assert_eq!(
        kept.matches("[\"a\"]").count(),
        3,
        "the decoys are as they were: {kept}"
    );
    let cast = keep_entries(text, Some("cast"), &keys(&["x"])).unwrap_or_else(|e| panic!("{e}"));
    assert!(cast.contains("(kind: id, id: \"x\")"), "{cast}");
    assert!(!cast.contains("\"y\""), "{cast}");
}

#[test]
fn rewrite_strips_cuts_and_swaps_only_what_it_should() {
    let plan = Plan::parse(
        "keep data/names.ron: lead\nswap chapters/ch01.ron: next: Some(\"ch02\") => next: None\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let chapter = "// Chapter 1\n(id: \"ch01\", next: Some(\"ch02\"))\n";
    assert_eq!(
        plan.rewrite("chapters/ch01.ron", chapter),
        Ok(Some("\n(id: \"ch01\", next: None)\n".to_owned()))
    );
    assert_eq!(
        plan.rewrite("data/names.ron", "{\"lead\": \"E\", \"king\": \"K\"}"),
        Ok(Some("{\n\"lead\": \"E\",\n}".to_owned()))
    );
    assert_eq!(plan.rewrite("data/plain.ron", "(a: 1)\n"), Ok(None));
    assert_eq!(
        plan.rewrite("data/plain.RON", "(a: 1) // a note\n"),
        Ok(Some("(a: 1) \n".to_owned())),
        "a RON file nobody cuts loses its comments"
    );
    assert_eq!(plan.rewrite("dialogue/ch01.dlg", "# notes\n"), Ok(None));
    let dlg = Plan::parse("swap dialogue/ch01.dlg: ch02 => ch01").unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        dlg.rewrite("dialogue/ch01.dlg", "# ch02\n"),
        Ok(Some("# ch01\n".to_owned())),
        "a swap applies to a file that isn't RON"
    );
    let twice = plan.rewrite(
        "chapters/ch01.ron",
        "(next: Some(\"ch02\"), b: next: Some(\"ch02\"))",
    );
    assert_eq!(
        twice,
        Err("chapters/ch01.ron: `next: Some(\"ch02\")` is there 2 times, not once".into())
    );
}

/// The list as the repository has it: the demo holds Chapter 1's own
/// files, none of a later chapter's, and its shared files are cut to
/// what Chapter 1 names.
#[test]
fn the_demos_list_keeps_chapter_1_and_cuts_the_shared_files() {
    let list = include_str!("../../demo/bundle.txt");
    let plan = Plan::parse(list).unwrap_or_else(|e| panic!("{e}"));
    let all: Vec<String> = crate::bundle::files_under("")
        .into_iter()
        .filter(|path| crate::bundle::story_only(|| crate::bundle::bytes(path).is_some()))
        .map(str::to_owned)
        .collect();
    let out = plan.left_out(&all).unwrap_or_else(|e| panic!("{e}"));
    for kept in [
        "chapters/ch01.ron",
        "dialogue/ch01.dlg",
        "maps/veyne_study.map",
    ] {
        assert!(!out.iter().any(|p| p == kept), "{kept}");
    }
    for gone in [
        "chapters/ch02.ron",
        "dialogue/ch08.dlg",
        "maps/ch02.map",
        "worldmaps/act1.ron",
    ] {
        assert!(out.iter().any(|p| p == gone), "{gone}");
    }
    let read = |path: &str| {
        let text = crate::bundle::story_only(|| crate::bundle::file(path)).unwrap_or_default();
        plan.rewrite(path, text)
            .unwrap_or_else(|e| panic!("{e}"))
            .unwrap_or_else(|| text.to_owned())
    };
    let characters = read("data/characters.ron");
    assert!(characters.contains("id: \"rival\""), "ch01_tbc's rival");
    assert!(characters.contains("\"messenger\""));
    for later in [
        "defector",
        "hound_lieutenant",
        "privateer",
        "\"king\"",
        "Chapter 8",
    ] {
        assert!(!characters.contains(later), "{later}");
    }
    let names = read("data/names.ron");
    assert!(names.contains("\"red_captain.nickname\""));
    for later in ["Joss", "\"king\"", "place.saltmere", "vowmaster.first"] {
        assert!(!names.contains(later), "{later}");
    }
    assert!(read("chapters/ch01.ron").contains("next: None"));
}
