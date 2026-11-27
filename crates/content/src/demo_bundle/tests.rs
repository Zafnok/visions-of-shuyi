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
    let out = plan.left_out(&files(ASSETS), None);
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
    let no_file = plan("chapters/\nsidequests/\n").left_out(&files(ASSETS), None);
    assert_eq!(
        no_file,
        Err("line 2: `sidequests/` is no file under assets/ or assets-private/content/".into())
    );
    let keeps_nothing = plan("chapters/\n+ maps/ch01.map\n").left_out(&files(ASSETS), None);
    assert_eq!(
        keeps_nothing,
        Err("line 2: `maps/ch01.map` keeps no file a line above leaves out".into())
    );
    let cut_gone = plan("chapters/\nkeep chapters/ch02.ron: x\n").left_out(&files(ASSETS), None);
    assert_eq!(
        cut_gone,
        Err("`chapters/ch02.ron` is cut, but the demo doesn't hold it".into())
    );
    let cut_missing = plan("chapters/\nkeep data/nope.ron: x\n").left_out(&files(ASSETS), None);
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

/// Ticket 0858: `art` lines name files of the bought art, checked against
/// them in a build that has them and not at all in one without.
#[test]
fn art_lines_are_checked_only_where_the_art_is() {
    let plan =
        Plan::parse("chapters/\nart portraits/later/\nart swap tilesets/t.ron: \"later\": x, =>\n")
            .unwrap_or_else(|e| panic!("{e}"));
    let art = files(&["portraits/later/neutral.png", "tilesets/t.ron"]);
    let out = plan.left_out(&files(ASSETS), Some(&art));
    assert_eq!(out, Ok(files(&["chapters/ch01.ron", "chapters/ch02.ron"])));
    assert!(plan.leaves_out("portraits/later/neutral.png"));
    assert!(!plan.leaves_out("tilesets/t.ron"));
    let cut = plan.rewrite("tilesets/t.ron", "{\"later\": x, \"lead\": y}");
    assert_eq!(cut, Ok(Some("{ \"lead\": y}".to_owned())));
    // Without the art, nothing to check them against.
    assert!(plan.left_out(&files(ASSETS), None).is_ok());
    // With it, a path or a swap that names none of it stops the build.
    let no_portrait = plan.left_out(&files(ASSETS), Some(&files(&["tilesets/t.ron"])));
    assert_eq!(
        no_portrait,
        Err("line 2: `portraits/later/` is no file under assets-private/game/".into())
    );
    let no_tileset = files(&["portraits/later/neutral.png"]);
    assert_eq!(
        plan.left_out(&files(ASSETS), Some(&no_tileset)),
        Err("`tilesets/t.ron` is cut, but the demo's art doesn't hold it".into())
    );
    // An art line isn't a `+` or a `keep`.
    assert!(Plan::parse("art + portraits/later/").is_err());
    assert!(Plan::parse("art keep tilesets/t.ron: lead").is_err());
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
    // A quote right after another opens its own: what is in it stays.
    let adjacent = "(a: \"x\"\"// kept\", b: 'q''/', c: \"\\\\\"//gone\n)";
    assert_eq!(
        strip_comments(adjacent).as_deref(),
        Ok("(a: \"x\"\"// kept\", b: 'q''/', c: \"\\\\\"\n)")
    );
}

#[test]
fn keep_entries_keeps_a_maps_keys_and_their_dotted_forms() {
    let names = "{\n    \"lead\": \"Ellery\",\n    \"rival\": \"Ann Bell\",\n    \
                 \"rival.first\": \"Ann\",\n    \"rivalry\": \"x\",\n    \"king\": \"Cole\",\n}\n";
    let cut =
        keep_entries(names, None, &keys(&["lead", "rival"])).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(
        cut,
        "{\n\"lead\": \"Ellery\",\n\"rival\": \"Ann Bell\",\n\"rival.first\": \"Ann\",\n}\n"
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
    if crate::bundle::skip_without_the_game() {
        return;
    }
    let list = include_str!("../../demo/bundle.txt");
    let plan = Plan::parse(list).unwrap_or_else(|e| panic!("{e}"));
    let all: Vec<String> = crate::bundle::files_under("")
        .into_iter()
        .filter(|path| crate::bundle::story_only(|| crate::bundle::bytes(path).is_some()))
        .map(str::to_owned)
        .collect();
    let out = plan.left_out(&all, None).unwrap_or_else(|e| panic!("{e}"));
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
        plan.rewrite(path, &text)
            .unwrap_or_else(|e| panic!("{e}"))
            .unwrap_or_else(|| text.into_owned())
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
    for later in [
        "\"defector.first\"",
        "\"king\"",
        "place.saltmere",
        "vowmaster.first",
    ] {
        assert!(!names.contains(later), "{later}");
    }
    assert!(read("chapters/ch01.ron").contains("next: None"));
}

/// The writer's preview's list (ticket 0764): its build, the
/// `story-preview` edition, leaves out every script, in `assets/`, the
/// content and the test files alike, and nothing else.
#[test]
fn the_writers_preview_leaves_out_every_script_and_nothing_else() {
    let list = include_str!("../../story-preview/bundle.txt");
    let plan = Plan::parse(list).unwrap_or_else(|e| panic!("{e}"));
    let all: Vec<String> = crate::bundle::files_under("")
        .into_iter()
        .map(str::to_owned)
        .collect();
    let scripts: Vec<&String> = all.iter().filter(|p| p.starts_with("dialogue/")).collect();
    assert!(!scripts.is_empty(), "the test files have scripts");
    let out = plan.left_out(&all, None).unwrap_or_else(|e| panic!("{e}"));
    let out: Vec<&String> = out.iter().collect();
    assert_eq!(out, scripts);
    assert!(scripts.iter().all(|p| plan.leaves_out(p)));
    assert!(!plan.leaves_out("data/names.ron"));
    assert!(!plan.leaves_out("portraits/lead_m.ron"));
}

/// The files of a game with a language pack `zz`, for the cut of packs
/// (ticket 0934): Chapter 1 kept, Chapter 2 left out.
const PACKED: &[&str] = &[
    "battles/ch01.ron",
    "battles/ch02.ron",
    "chapters/ch01.ron",
    "chapters/ch02.ron",
    "data/characters.ron",
    "data/names.ron",
    "dialogue/ch01.dlg",
    "dialogue/ch02.dlg",
    "dialogue/notes.txt",
    "lang/zz/data.ron",
    "lang/zz/dialogue/ch01.ron",
    "lang/zz/dialogue/ch02.ron",
    "lang/zz/dialogue/talks.ron",
    "lang/zz/lang.ron",
    "lang/zz/ui.ron",
    "worldmaps/act2.ron",
];

/// The plan of [`PACKED`]'s demo, told its files.
fn packed_plan() -> Plan {
    let mut plan = Plan::parse(
        "chapters/\nbattles/\ndialogue/\nworldmaps/\n+ chapters/ch01.ron\n+ battles/ch01.ron\n\
         + dialogue/ch01.dlg\n+ dialogue/notes.txt\nkeep data/names.ron: lead red_captain place.harrowby\n\
         keep data/characters.ron characters: lead\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let scripts = |path: &str| match path {
        "dialogue/ch01.dlg" => Some("# Chapter 1\n@scene ch01_intro\n> Rain.\n@scene ch01_tbc\n"),
        "dialogue/ch02.dlg" => Some("@scene ch02_road\n"),
        "dialogue/notes.txt" => Some("@scene ch01_notes\n"),
        _ => None,
    };
    plan.hold(&files(PACKED), |path| scripts(path).map(str::to_owned));
    plan
}

/// A pack's entries of `keys`, each its key in capitals.
fn pack_text(keys: &[&str]) -> String {
    let entries: Vec<String> = keys
        .iter()
        .map(|k| {
            let text = k.to_uppercase();
            format!("    (key: \"{k}\", source: \"{k}\", text: \"{text}\"),\n")
        })
        .collect();
    format!("[\n{}]\n", entries.concat())
}

#[test]
fn a_packs_data_is_cut_to_the_names_chapters_and_battles_the_demo_holds() {
    let plan = packed_plan();
    let text = pack_text(&[
        "names.lead",
        "names.red_captain.nickname",
        "names.king",
        "names.place.harrowby",
        "names.place.saltmere",
        "characters.lead.title",
        "characters.king.title",
        "chapters.ch01.title",
        "chapters.ch02.title",
        "battles.ch01.note_1",
        "battles.ch02.place",
        "items.iron_sword.name",
        "worldmaps.act1.harrowby.name",
        "worldmaps.act2.harrowby.name",
    ]);
    let cut = plan
        .rewrite("lang/zz/data.ron", &text)
        .unwrap_or_else(|e| panic!("{e}"))
        .unwrap_or_else(|| panic!("the pack's data.ron is cut"));
    for kept in [
        "names.lead\"",
        "names.red_captain.nickname\"",
        "names.place.harrowby\"",
        "characters.lead.title",
        "chapters.ch01.title",
        "battles.ch01.note_1",
        "items.iron_sword.name",
        // No file of the game, so none the demo lacks.
        "worldmaps.act1.harrowby.name",
    ] {
        assert!(cut.contains(kept), "{kept}: {cut}");
    }
    for gone in [
        "names.king",
        "place.saltmere",
        "characters.king",
        "ch02",
        "act2",
    ] {
        assert!(!cut.contains(gone), "{gone}: {cut}");
    }
}

#[test]
fn a_packs_dialogue_follows_the_scripts_the_demo_holds() {
    let plan = packed_plan();
    let out = plan.left_out(&files(PACKED), None);
    assert_eq!(
        out.as_deref(),
        Ok(&files(&[
            "battles/ch02.ron",
            "chapters/ch02.ron",
            "dialogue/ch02.dlg",
            "lang/zz/dialogue/ch02.ron",
            "worldmaps/act2.ron",
        ])[..])
    );
    for kept in [
        "lang/zz/dialogue/ch01.ron",
        "lang/zz/dialogue/talks.ron",
        "lang/zz/data.ron",
        "lang/zz/ui.ron",
    ] {
        assert!(!plan.leaves_out(kept), "{kept}");
    }
    let ch01 = pack_text(&["ch01_intro_0123abcd", "names.king", "ch02_road_0123abcd"]);
    let whole = plan.rewrite("lang/zz/dialogue/ch01.ron", &ch01);
    assert_eq!(whole, Ok(None), "a held script's file is whole");
    assert_eq!(plan.rewrite("lang/zz/ui.ron", &ch01), Ok(None));
    assert_eq!(plan.rewrite("lang/zz/lang.ron", "(name: \"ZZ\")"), Ok(None));
    let talks = pack_text(&[
        "ch01_intro_0123abcd",
        "ch01_intro_0123abcd_2",
        "caption.ch01_tbc_89abcdef",
        "ch02_road_0123abcd",
        "caption.ch02_road_89abcdef",
        "ch01_intro_extra_0123abcd",
        "ch01_intro_0123abcx",
        "ch01_notes_0123abcd",
    ]);
    let cut = plan
        .rewrite("lang/zz/dialogue/talks.ron", &talks)
        .unwrap_or_else(|e| panic!("{e}"))
        .unwrap_or_else(|| panic!("a file of no script is cut"));
    let kept: Vec<&str> = cut
        .lines()
        .filter_map(|line| line.split('"').nth(1))
        .collect();
    assert_eq!(
        kept,
        [
            "ch01_intro_0123abcd",
            "ch01_intro_0123abcd_2",
            "caption.ch01_tbc_89abcdef"
        ]
    );
}

#[test]
fn a_pack_is_cut_only_once_the_plan_knows_the_games_files() {
    let plan = Plan::parse("dialogue/\n").unwrap_or_else(|e| panic!("{e}"));
    assert!(!plan.leaves_out("lang/zz/dialogue/ch02.ron"));
    let text = pack_text(&["names.king"]);
    assert_eq!(
        plan.rewrite("lang/zz/data.ron", &text),
        Err("lang/zz/data.ron: the demo's files are unknown: `Plan::hold` first".into())
    );
    assert_eq!(plan.rewrite("lang/zz/ui.ron", &text), Ok(None));
    assert_eq!(plan.rewrite("lang/data.ron", &text), Ok(None), "no pack");
    assert_eq!(plan.rewrite("lang//data.ron", &text), Ok(None), "no code");
}

#[test]
fn a_line_key_is_its_scenes_by_its_hash() {
    assert!(of_scene("a", "a_0123abcd"));
    assert!(of_scene("a", "caption.a_0123ABCD"));
    assert!(of_scene("a", "a_0123abcd_12"));
    for not in [
        "a_0123abc",
        "a_0123abcd_",
        "a_0123abcd_x",
        "a_0123abcdx",
        "b_0123abcd",
        "a_b_0123abcd",
        "a0123abcd",
    ] {
        assert!(!of_scene("a", not), "{not}");
    }
    let scenes: Vec<&str> =
        scenes_of("@scene one\n  @scene two # note\n@scenes x\n> @scene three").collect();
    assert_eq!(scenes, ["one", "two"]);
}

/// The demo's own list and files (ticket 0934): a pack whose `data.ron`
/// names every name of the full game, cut, loads in the demo's bundle,
/// and holds no name the demo doesn't.
#[test]
fn a_pack_naming_every_name_loads_in_the_demo() {
    use crate::bundle::{
        Bundle, Layer, bytes, demo_layers, file, files_under, reading, story_only,
    };
    if crate::bundle::skip_without_the_game() {
        return;
    }
    let names = story_only(|| file("data/names.ron")).unwrap_or_default();
    let names = strip_comments(&names).unwrap_or_else(|e| panic!("{e}"));
    let mut ids = Vec::new();
    keep_by(&names, None, "key", |id| {
        ids.push(format!("names.{id}"));
        true
    })
    .unwrap_or_else(|e| panic!("{e}"));
    assert!(ids.len() > 100, "{} names", ids.len());
    let keys: Vec<&str> = ids.iter().map(String::as_str).collect();
    let list = include_str!("../../demo/bundle.txt");
    let mut plan = Plan::parse(list).unwrap_or_else(|e| panic!("{e}"));
    let all: Vec<String> = files_under("")
        .into_iter()
        .filter(|path| story_only(|| bytes(path).is_some()))
        .map(str::to_owned)
        .collect();
    plan.hold(&all, |path| {
        story_only(|| file(path)).map(std::borrow::Cow::into_owned)
    });
    let every = pack_text(&keys);
    let cut = plan
        .rewrite("lang/zz/data.ron", &every)
        .unwrap_or_else(|e| panic!("{e}"))
        .unwrap_or_default();
    assert!(cut.contains("\"names.red_captain.nickname\""), "{cut}");
    assert!(!cut.contains("\"names.king\""), "{cut}");
    let load = |data: String| {
        let pack = [
            ("lang/zz/data.ron", data),
            (
                "lang/zz/lang.ron",
                "(name: \"ZZ\", made_by: Machine)".to_owned(),
            ),
            ("lang/zz/ui.ron", "[]".to_owned()),
        ];
        let pack = pack.map(|(path, text)| (path.to_owned(), text.into_bytes()));
        let pack = Layer::owned("pack/".to_owned(), pack.into()).unwrap_or_else(|e| panic!("{e}"));
        let mut layers = vec![pack];
        layers.extend(demo_layers());
        let demo: &'static Bundle = Box::leak(Box::new(Bundle::new(layers)));
        reading(demo, || {
            let content = crate::load_embedded().map_err(|e| e.to_string())?;
            Ok::<_, String>(
                content
                    .lang
                    .codes()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>(),
            )
        })
    };
    let codes = load(cut).unwrap_or_else(|e| panic!("{e}"));
    assert!(codes.iter().any(|c| c == "zz"), "{codes:?}");
    let whole = load(every).map(|_| ()).unwrap_err();
    assert!(
        whole.contains("no such key in the data"),
        "uncut, the pack fails: {whole}"
    );
}
