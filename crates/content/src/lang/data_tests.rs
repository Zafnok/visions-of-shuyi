//! Tests of a pack's data and dialogue text (ticket 0235): what English
//! offers to translate, what a pack's files may hold, the lookups, and
//! what a pack lacks.

use trpg_core::lead::LeadGender;
use trpg_core::{BattleNote, ClassId, ItemId};

use super::*;
use crate::dialogue::{self, Scene, Step, caption_key};

/// A scene with every kind of text: a caption, narration, speech with a
/// name token, replies with reactions, an `@if` block and a lead token.
const DLG: &str = "\
@scene gate
@caption The gate, dusk
> The gate is shut.
@left hero neutral
hero: Open it, {n:king}.
@if hero
hero: Still here.
@else
> Gone.
@endif
@choice
* earnest: I will wait here.
  > Nobody answers.
* wry: Then I climb.
  > The wall is high.
@endchoice
> {lead} waits.
@end
";

fn scene() -> Scene {
    let table = dialogue::from_sources(&[("gate.dlg", DLG)], None, None, None, None).unwrap();
    table.scenes["gate"].clone()
}

fn names(entries: &[(&str, &str)]) -> Names {
    Names {
        names: entries
            .iter()
            .map(|&(id, name)| (id.to_owned(), name.to_owned()))
            .collect(),
    }
}

/// English: a little of each kind of data text, and the scene `gate`.
fn data() -> DataText {
    let mut data = DataText::default();
    for (key, text) in [
        ("items.potion.name", "Potion"),
        ("classes.guard.name", "Guard"),
        ("terrain.forest.name", "Forest"),
        ("names.king", "the King"),
        ("names.keep", "Heth Keep"),
        ("tips.move.title", "Your move"),
        ("tips.move.text", "Press {Confirm}.\nThen wait."),
        ("tips.wait.text", "Wait."),
        ("battles.gate.note_1", "Hold the gate."),
        ("battles.road.note_1", "Hold the road."),
        ("battles.wall.note_1", "Hold the road."),
    ] {
        data.data.insert(key.to_owned(), text.to_owned());
    }
    data.names = names(&[("king", "the King"), ("keep", "Heth Keep")]);
    let scenes = [("gate".to_owned(), scene())].into();
    data.add_dialogue(&dialogue::DialogueTable { scenes });
    data
}

/// The id of the line (or the key of the caption) of `data` that says
/// `text`.
fn id(text: &str) -> String {
    let data = data();
    let line = data.lines.iter().find(|(_, line)| line.text == text);
    line.map(|(id, _)| id.clone()).unwrap()
}

const INFO: &str = r#"(name: "Xx", made_by: Machine)"#;

/// The pack with `data_ron` and the dialogue files `dialogue`, checked
/// against [`data`], or its errors as text.
fn pack(data_ron: &str, dialogue: &[(&str, &str)]) -> Result<LangPack, Vec<String>> {
    let sources = PackSources {
        info: ("xx/lang.ron", INFO),
        ui: ("xx/ui.ron", "[]"),
        data: Some(("xx/data.ron", data_ron)),
        dialogue: dialogue.to_vec(),
    };
    pack_from_files(&sources, &BTreeMap::new(), Some(&data()))
        .map_err(|errors| errors.iter().map(ToString::to_string).collect())
}

/// The errors of a pack whose `data.ron` is `data_ron`.
fn data_errors(data_ron: &str) -> Vec<String> {
    pack(data_ron, &[]).err().unwrap_or_default()
}

/// The errors of a pack whose one dialogue file holds `entries`.
fn line_errors(entries: &str) -> Vec<String> {
    pack("[]", &[("xx/dialogue/gate.ron", entries)])
        .err()
        .unwrap_or_default()
}

fn code(code: &str) -> LangCode {
    LangCode::new(code).unwrap()
}

/// English ([`data`]) with the pack `xx` over it.
fn lang(pack: LangPack) -> Lang {
    Lang::with_data(BTreeMap::new(), data(), [(code("xx"), pack)].into())
}

/// One dialogue entry: `(key: …, source: …, text: …)`.
fn entry(text: &str, translation: &str) -> String {
    format!(
        "(key: {:?}, source: {text:?}, text: {translation:?})",
        id(text)
    )
}

#[test]
fn english_lines_replies_and_captions_have_their_kind_and_scene() {
    let data = data();
    let kind = |text: &str| data.lines[&id(text)].kind;
    assert_eq!(kind("The gate is shut."), LineKind::Speech);
    assert_eq!(kind("Open it, {n:king}."), LineKind::Speech);
    assert_eq!(kind("I will wait here."), LineKind::Reply);
    assert_eq!(kind("Nobody answers."), LineKind::Speech);
    assert_eq!(kind("The gate, dusk"), LineKind::Caption);
    // Both parts of an `@if` block.
    assert_eq!(kind("Still here."), LineKind::Speech);
    assert_eq!(kind("Gone."), LineKind::Speech);
    assert_eq!(id("The gate, dusk"), caption_key("gate", "The gate, dusk"));
    assert!(id("The gate, dusk").starts_with("caption.gate_"));
    assert!(data.lines.values().all(|line| line.scene == "gate"));
    // 9 lines and replies, 1 caption.
    assert_eq!(data.lines.len(), 10);
}

#[test]
fn the_embedded_content_offers_every_kind_of_data_text() {
    let content = crate::load_embedded().unwrap();
    let source = content.lang.source();
    for (key, english) in [
        ("items.iron_sword.name", "Iron Sword"),
        ("items.potion.name", "Potion"),
        ("classes.guard.name", "Guard"),
        ("spells.fire.name", "Fire"),
        ("skills.keen_edge.name", "Keen Edge"),
        ("arts.guard_break.name", "Guard Break"),
        ("terrain.forest.name", "Forest"),
        ("names.lead", "Ellery"),
        ("names.retainer.first", "Hollis"),
        ("tips.battle_start.title", "Your move"),
        ("chapters.test.title", "Test Chapter"),
        (
            "battles.test.note_2",
            "Seize the fort within 3 turns: send your lead straight there.",
        ),
    ] {
        assert_eq!(source.data.get(key).map(String::as_str), Some(english));
        assert_eq!(content.lang.english(key), Some(english), "{key}");
    }
    assert!(source.data["tips.battle_start.text"].starts_with("Steer the cursor"));
    assert_eq!(source.names, content.names);
    let line = &source.lines["test_5da5d147"];
    assert_eq!(line.text, "The rain had not stopped for three days.");
    assert_eq!((line.scene.as_str(), line.kind), ("test", LineKind::Speech));
    assert_eq!(source.lines["test_ee965a73"].kind, LineKind::Reply);
    assert_eq!(
        source.lines["caption.test_18a7341c"].kind,
        LineKind::Caption
    );
    assert_eq!(
        content.lang.english("test_36a692b7"),
        Some("Let's move."),
        "a line's English is found by its id"
    );
}

#[test]
fn a_data_entry_needs_a_key_english_has_once_and_its_sources_placeholders() {
    let ron = r#"[
        (key: "items.nope.name", source: "x", text: "y"),
        (key: "items.potion.name", source: "Potion", text: "Trank"),
        (key: "items.potion.name", source: "Potion", text: "Trunk"),
        (key: "tips.move.text", source: "Press {Confirm}.", text: "Drück."),
    ]"#;
    assert_eq!(
        data_errors(ron),
        [
            "xx/data.ron: \"items.nope.name\": no such key in the data",
            "xx/data.ron: \"items.potion.name\": the key is used twice",
            "xx/data.ron: \"tips.move.text\": the text and its source must have the same \
             placeholders",
        ]
    );
    // A file that doesn't parse is one error, at its place in the file.
    let syntax = data_errors("[");
    assert_eq!(syntax.len(), 1);
    assert!(syntax[0].starts_with("xx/data.ron:1:2: "), "{syntax:?}");
}

#[test]
fn a_name_in_a_pack_is_not_empty_and_holds_no_brace() {
    let ron = r#"[
        (key: "names.king", source: "the King", text: " "),
        (key: "names.keep", source: "Heth Keep", text: "Burg}"),
    ]"#;
    assert_eq!(
        data_errors(ron),
        [
            "xx/data.ron: \"names.king\": a name must not be empty",
            "xx/data.ron: \"names.keep\": a name can't hold a brace",
        ]
    );
    let ron = r#"[(key: "names.keep", source: "Heth Keep", text: "{Burg")]"#;
    assert_eq!(
        data_errors(ron),
        ["xx/data.ron: \"names.keep\": a name can't hold a brace"]
    );
    // Any other data text may be anything, even empty.
    let ron = r#"[(key: "items.potion.name", source: "Potion", text: "")]"#;
    assert_eq!(data_errors(ron), Vec::<String>::new());
}

#[test]
fn a_tip_in_a_pack_keeps_to_the_tip_limits_in_cells() {
    let title = |text: &str| {
        data_errors(&format!(
            r#"[(key: "tips.move.title", source: "Your move", text: "{text}")]"#
        ))
    };
    let too_long = ["xx/data.ron: \"tips.move.title\": the title must be 1 to 30 cells"];
    assert_eq!(title(&"é".repeat(30)), Vec::<String>::new());
    assert_eq!(title(&"é".repeat(31)), too_long);
    assert_eq!(title(""), too_long);
    let text = |text: &str| {
        data_errors(&format!(
            r#"[(key: "tips.wait.text", source: "Wait.", text: "{text}")]"#
        ))
    };
    let wide = "é".repeat(60);
    assert_eq!(
        text(&format!("{wide}\\n{wide}\\n{wide}")),
        Vec::<String>::new()
    );
    let lines = ["xx/data.ron: \"tips.wait.text\": the text must have 1 to 3 lines"];
    assert_eq!(text("1\\n2\\n3\\n4"), lines);
    assert_eq!(text(""), lines);
    assert_eq!(
        text(&format!("ok\\n{wide}é")),
        [format!(
            "xx/data.ron: \"tips.wait.text\": a text line is over 60 cells: \"{wide}é\""
        )]
    );
}

#[test]
fn without_english_data_a_packs_data_and_dialogue_are_left_out() {
    let sources = PackSources {
        info: ("xx/lang.ron", INFO),
        ui: ("xx/ui.ron", "[]"),
        data: Some(("xx/data.ron", "not even RON")),
        dialogue: vec![("xx/dialogue/gate.ron", "nor this")],
    };
    let pack = pack_from_files(&sources, &BTreeMap::new(), None).unwrap();
    assert_eq!(pack.entry("items.potion.name"), None);
    assert_eq!(pack.line(&id("Gone.")), None);
}

const DATA_RON: &str = r#"[
    (key: "items.potion.name", source: "Potion", text: "Trank"),
    (key: "classes.guard.name", source: "Guard", text: "Wache"),
    (key: "terrain.forest.name", source: "Wood", text: "Wald"),
    (key: "names.king", source: "the King", text: "der König"),
    (key: "tips.move.title", source: "Your move", text: "Dein Zug"),
    (key: "battles.gate.note_1", source: "Hold the gate.", text: "Haltet das Tor."),
    (key: "battles.wall.note_1", source: "Hold the road.", text: "Haltet den Weg."),
]"#;

#[test]
fn data_lookups_use_the_pack_while_its_source_is_todays_english() {
    let lang = lang(pack(DATA_RON, &[]).unwrap());
    let (xx, en, zz) = (code("xx"), LangCode::english(), code("zz"));
    let potion = ItemId::new("potion");
    assert_eq!(lang.item_name(&xx, &potion, "Potion"), "Trank");
    // English, and a code with no pack, are English.
    assert_eq!(lang.item_name(&en, &potion, "Potion"), "Potion");
    assert_eq!(lang.item_name(&zz, &potion, "Potion"), "Potion");
    // The data changed since the entry was made: it is stale.
    assert_eq!(lang.item_name(&xx, &potion, "Elixir"), "Elixir");
    // No entry.
    assert_eq!(lang.item_name(&xx, &ItemId::new("ether"), "Ether"), "Ether");
    let guard = ClassId("guard".to_owned());
    assert_eq!(lang.class_name(&xx, &guard, "Guard"), "Wache");
    assert_eq!(lang.class_name(&en, &guard, "Guard"), "Guard");
    assert_eq!(lang.name(&xx, "king", "the King"), "der König");
    assert_eq!(lang.name(&xx, "keep", "Heth Keep"), "Heth Keep");
    assert_eq!(lang.name(&xx, "king", "the Queen"), "the Queen");
    assert_eq!(
        lang.pack(&xx).unwrap().entry("names.king").unwrap().text,
        "der König"
    );
}

#[test]
fn terrain_and_battle_notes_are_found_by_their_english() {
    let lang = lang(pack(DATA_RON, &[]).unwrap());
    let (xx, en) = (code("xx"), LangCode::english());
    // The entry was made from "Wood": stale.
    assert_eq!(lang.terrain_name(&xx, "Forest"), "Forest");
    let fresh = DATA_RON.replace("\"Wood\"", "\"Forest\"");
    let lang_fresh = self::lang(pack(&fresh, &[]).unwrap());
    assert_eq!(lang_fresh.terrain_name(&xx, "Forest"), "Wald");
    assert_eq!(lang_fresh.terrain_name(&en, "Forest"), "Forest");
    // Not the name of a terrain of the data.
    assert_eq!(lang_fresh.terrain_name(&xx, "Swamp"), "Swamp");
    // "Potion" is English for a key, but not a terrain's.
    assert_eq!(lang_fresh.terrain_name(&xx, "Potion"), "Potion");
    let note = |text: &str| BattleNote {
        text: text.to_owned(),
        units: Vec::new(),
    };
    assert_eq!(
        lang.battle_note(&xx, &note("Hold the gate.")),
        "Haltet das Tor."
    );
    assert_eq!(
        lang.battle_note(&en, &note("Hold the gate.")),
        "Hold the gate."
    );
    // Two battles have this note; only the second one's is translated.
    assert_eq!(
        lang.battle_note(&xx, &note("Hold the road.")),
        "Haltet den Weg."
    );
    // A note no battle file has.
    assert_eq!(lang.battle_note(&xx, &note("Run.")), "Run.");
}

#[test]
fn the_names_table_of_a_language_has_the_packs_names() {
    let lang = lang(pack(DATA_RON, &[]).unwrap());
    let english = data().names;
    let theirs = lang.names(&code("xx"), &english);
    assert_eq!(theirs.get("king"), Some("der König"));
    assert_eq!(theirs.get("keep"), Some("Heth Keep"));
    assert_eq!(lang.names(&LangCode::english(), &english), english);
    assert_eq!(lang.names(&code("zz"), &english), english);
    // A name the data has renamed since keeps its new English.
    let renamed = names(&[("king", "the Queen")]);
    assert_eq!(lang.names(&code("xx"), &renamed), renamed);
}

#[test]
fn the_test_pack_translates_a_little_of_every_kind() {
    let content = crate::load_embedded().unwrap();
    let lang = &content.lang;
    let (test, en) = (code(TEST), LangCode::english());
    let guard = &content.classes.classes[&ClassId("guard".to_owned())];
    assert_eq!(lang.class_name(&test, &guard.id, &guard.name), "GUARD");
    assert_eq!(lang.class_name(&en, &guard.id, &guard.name), "Guard");
    let fire = content.spells.spells.values().find(|s| s.id.0 == "fire");
    assert_eq!(fire.map(|s| lang.spell_name(&test, s)), Some("FIRE"));
    assert_eq!(fire.map(|s| lang.spell_name(&en, s)), Some("Fire"));
    let frost = content.spells.spells.values().find(|s| s.id.0 == "frost");
    assert_eq!(frost.map(|s| lang.spell_name(&test, s)), Some("Frost"));
    let skill = |id: &str| content.skills.skills.values().find(|s| s.id.0 == id);
    let keen = skill("keen_edge").map(|s| lang.skill_name(&test, s));
    assert_eq!(keen, Some("KEEN EDGE"));
    assert_eq!(
        skill("flurry").map(|s| lang.skill_name(&test, s)),
        Some("Flurry")
    );
    let art = |id: &str| content.arts.arts.values().find(|a| a.id.0 == id);
    let guard_break = art("guard_break").map(|a| lang.art_name(&test, a));
    assert_eq!(guard_break, Some("GUARD BREAK"));
    let flowing = art("flowing_cut").map(|a| lang.art_name(&test, a));
    assert_eq!(flowing, Some("Flowing Cut"));
    // The axe's entry is stale.
    assert_eq!(
        lang.item_name(&test, &ItemId::new("iron_axe"), "Iron Axe"),
        "Iron Axe"
    );
    assert_eq!(
        lang.item_name(&test, &ItemId::new("potion"), "Potion"),
        "POTION"
    );
    assert_eq!(lang.terrain_name(&test, "Forest"), "FOREST");
    assert_eq!(lang.terrain_name(&test, "Plain"), "Plain");
    let tip = content
        .tips
        .tips
        .iter()
        .find(|t| t.id == "battle_start")
        .unwrap();
    assert_eq!(lang.tip_title(&test, tip), "YOUR MOVE");
    assert_eq!(lang.tip_title(&en, tip), "Your move");
    assert!(
        lang.tip_text(&test, tip)
            .starts_with("STEER THE CURSOR WITH {Cursor}.")
    );
    assert!(lang.tip_text(&en, tip).starts_with("Steer the cursor"));
    let other = content
        .tips
        .tips
        .iter()
        .find(|t| t.id == "unit_selected")
        .unwrap();
    assert_eq!(lang.tip_title(&test, other), "Moving a unit");
    assert_eq!(lang.tip_text(&test, other), other.text);
    let chapter = &content.chapters["test"];
    assert_eq!(lang.chapter_title(&test, chapter), "TEST CHAPTER");
    assert_eq!(lang.chapter_title(&en, chapter), "Test Chapter");
    assert_eq!(
        lang.chapter_title(&test, &content.chapters["quick"]),
        "Quick Battle"
    );
    let notes = &content.battles["test"].battle_notes;
    assert!(
        lang.battle_note(&test, &notes[0])
            .starts_with("Brigand: guards")
    );
    assert!(
        lang.battle_note(&test, &notes[1])
            .starts_with("SEIZE THE FORT")
    );
}

#[test]
fn a_dialogue_entry_has_text_or_both_gendered_texts() {
    let key = id("Gone.");
    let entry = |texts: &str| format!("[(key: {key:?}, source: \"Gone.\", {texts})]");
    let wrong = [format!(
        "xx/dialogue/gate.ron: {key:?}: an entry has `text`, or both `text_m` and `text_f`"
    )];
    assert_eq!(line_errors(&entry("text: \"Weg.\"")), Vec::<String>::new());
    let both = entry("text_m: \"Weg.\", text_f: \"Fort.\"");
    assert_eq!(line_errors(&both), Vec::<String>::new());
    for texts in [
        "text_m: \"Weg.\"",
        "text_f: \"Weg.\"",
        "text: \"Weg.\", text_m: \"Weg.\"",
        "text: \"Weg.\", text_f: \"Weg.\"",
        "text: \"Weg.\", text_m: \"Weg.\", text_f: \"Weg.\"",
    ] {
        assert_eq!(line_errors(&entry(texts)), wrong, "{texts}");
    }
    let none = format!("[(key: {key:?}, source: \"Gone.\")]");
    assert_eq!(line_errors(&none), wrong);
    // `text` is written bare, never as an option.
    assert_eq!(line_errors(&entry("text: Some(\"Weg.\")")).len(), 1);
}

#[test]
fn a_dialogue_key_is_a_line_id_or_a_caption_key_used_once() {
    let entry = |key: &str| format!("(key: {key:?}, source: \"x\", text: \"y\")");
    let all = |keys: &[&str]| {
        let entries: Vec<String> = keys.iter().map(|k| entry(k)).collect();
        line_errors(&format!("[{}]", entries.join(",")))
    };
    let not_an_id =
        |key: &str| format!("xx/dialogue/gate.ron: {key:?}: not a line id or a caption key");
    // Lines English doesn't have are orphans, not errors.
    assert_eq!(
        all(&[
            "gate_00000000",
            "gate_0123abcd_2",
            "caption.gate_00000000",
            "gone_ffffffff"
        ]),
        Vec::<String>::new()
    );
    assert_eq!(
        all(&[
            "gate",
            "caption.gate",
            "names.king",
            "gate_0000000",
            "gate_0000000g",
            ""
        ]),
        [
            "gate",
            "caption.gate",
            "names.king",
            "gate_0000000",
            "gate_0000000g",
            ""
        ]
        .map(not_an_id)
    );
    // Once in the pack, whichever file it is in.
    let first = format!("[{}]", entry("gate_00000000"));
    let errors = pack(
        "[]",
        &[("xx/dialogue/a.ron", &first), ("xx/dialogue/b.ron", &first)],
    )
    .unwrap_err();
    assert_eq!(
        errors,
        ["xx/dialogue/b.ron: \"gate_00000000\": the key is used twice"]
    );
    let syntax = line_errors("[");
    assert_eq!(syntax.len(), 1);
    assert!(
        syntax[0].starts_with("xx/dialogue/gate.ron:1:2: "),
        "{syntax:?}"
    );
}

#[test]
fn a_packs_dialogue_text_follows_the_token_rule_of_english() {
    let errors = |text: &str| {
        let entries = format!("[{}]", entry("Gone.", text));
        let prefix = format!("xx/dialogue/gate.ron: {:?}: ", id("Gone."));
        line_errors(&entries)
            .iter()
            .map(|e| e.strip_prefix(&prefix).unwrap_or(e).to_owned())
            .collect::<Vec<_>>()
    };
    assert_eq!(
        errors("{lead} {They} {N:king} {n:keep}"),
        Vec::<String>::new()
    );
    assert_eq!(errors(" "), ["the text must not be empty"]);
    assert_eq!(errors("{open"), ["\"{\" has no closing \"}\""]);
    let unknown = errors("{nope}");
    assert_eq!(unknown.len(), 1);
    assert!(
        unknown[0].starts_with("unknown token \"{nope}\""),
        "{unknown:?}"
    );
    assert_eq!(
        errors("{n:nobody}"),
        [
            "unknown name id \"nobody\" in {n:nobody}; name ids are listed in \
          assets/data/names.ron"
        ]
    );
    let lead = errors("{N:lead}");
    assert!(
        lead[0].starts_with("{N:lead} is only the lead's default name"),
        "{lead:?}"
    );
    // Every problem of every text of an entry, and an orphan's too.
    assert_eq!(errors("{a} {b}").len(), 2);
    let gendered = r#"[(key: "gate_00000000", source: "x", text_m: "{a}", text_f: "{b}")]"#;
    assert_eq!(line_errors(gendered).len(), 2);
}

#[test]
fn a_packs_dialogue_text_keeps_to_its_lines_limit_in_cells() {
    let errors = |english: &str, text: &str| {
        let entries = format!("[{}]", entry(english, text));
        let prefix = format!("xx/dialogue/gate.ron: {:?}: ", id(english));
        line_errors(&entries)
            .iter()
            .map(|e| e.strip_prefix(&prefix).unwrap_or(e).to_owned())
            .collect::<Vec<_>>()
    };
    let none = Vec::<String>::new();
    // A speech or narration line: 200. Accents are one cell each.
    assert_eq!(errors("Gone.", &"é".repeat(200)), none);
    assert_eq!(
        errors("Gone.", &"é".repeat(201)),
        ["text is 201 cells; the limit is 200"]
    );
    assert_eq!(
        errors("Still here.", &"x".repeat(201)),
        ["text is 201 cells; the limit is 200"]
    );
    // A reply: 60.
    assert_eq!(errors("Then I climb.", &"x".repeat(60)), none);
    assert_eq!(
        errors("Then I climb.", &"x".repeat(61)),
        ["text is 61 cells; the limit is 60"]
    );
    // A caption has no limit, and neither has an orphan (nobody knows
    // what it was).
    assert_eq!(errors("The gate, dusk", &"x".repeat(300)), none);
    let orphan = format!(
        "[(key: \"gate_00000000\", source: \"x\", text: \"{}\")]",
        "x".repeat(300)
    );
    assert_eq!(line_errors(&orphan), none);
    // A token counts as the longest it can become: the lead's name is 12.
    assert_eq!(
        errors("Then I climb.", &format!("{{lead}}{}", "x".repeat(48))),
        none
    );
    assert_eq!(
        errors("Then I climb.", &format!("{{lead}}{}", "x".repeat(49))),
        ["text is 61 cells; the limit is 60"]
    );
    // Either text of a gendered entry.
    let key = id("Then I climb.");
    let gendered = format!(
        "[(key: {key:?}, source: \"Then I climb.\", text_m: \"ok\", text_f: \"{}\")]",
        "x".repeat(61)
    );
    assert_eq!(line_errors(&gendered).len(), 1);
}

#[test]
fn a_name_token_counts_as_the_packs_longest_name() {
    // English's longest name is "Heth Keep" (9); the pack's is 20.
    let long = "k".repeat(20);
    let data_ron =
        |source: &str| format!("[(key: \"names.king\", source: {source:?}, text: {long:?})]");
    let reply = |filler: usize| {
        let text = format!("{{n:keep}}{}", "x".repeat(filler));
        format!("[{}]", entry("Then I climb.", &text))
    };
    let errors = |data_ron: &str, filler: usize| {
        pack(data_ron, &[("xx/dialogue/gate.ron", &reply(filler))])
            .err()
            .unwrap_or_default()
            .len()
    };
    let fresh = data_ron("the King");
    assert_eq!(errors(&fresh, 40), 0);
    assert_eq!(errors(&fresh, 41), 1);
    // A stale name shows in English, so it doesn't count.
    let stale = data_ron("the Queen");
    assert_eq!(errors(&stale, 51), 0);
    assert_eq!(errors(&stale, 52), 1);
}

/// A pack translating the caption, the first line, a reply, the line
/// inside the `@if` block and, by gender, the last line.
fn dialogue_pack() -> LangPack {
    let last = "{lead} waits.";
    let entries = [
        entry("The gate, dusk", "THE GATE, DUSK"),
        entry("The gate is shut.", "THE GATE IS SHUT."),
        entry("Then I climb.", "THEN I CLIMB."),
        entry("Still here.", "STILL HERE."),
        entry("The wall is high.", "THE WALL IS HIGH."),
        format!(
            "(key: {:?}, source: {last:?}, text_m: \"HE WAITS.\", text_f: \"SHE WAITS.\")",
            id(last)
        ),
    ];
    let file = format!("[{}]", entries.join(","));
    pack(DATA_RON, &[("xx/dialogue/gate.ron", &file)]).unwrap()
}

#[test]
fn a_line_is_looked_up_by_its_id_for_the_leads_gender() {
    let lang = lang(dialogue_pack());
    let (xx, en) = (code("xx"), LangCode::english());
    let line = |code: &LangCode, english: &'static str, gender| {
        lang.line(code, &id(english), english, gender).to_owned()
    };
    let (m, f) = (LeadGender::Male, LeadGender::Female);
    assert_eq!(line(&xx, "The gate is shut.", m), "THE GATE IS SHUT.");
    assert_eq!(line(&xx, "The gate is shut.", f), "THE GATE IS SHUT.");
    assert_eq!(line(&xx, "{lead} waits.", m), "HE WAITS.");
    assert_eq!(line(&xx, "{lead} waits.", f), "SHE WAITS.");
    assert_eq!(line(&xx, "The gate, dusk", m), "THE GATE, DUSK");
    // English, a code with no pack, and a line with no entry.
    assert_eq!(line(&en, "The gate is shut.", m), "The gate is shut.");
    assert_eq!(
        line(&code("zz"), "The gate is shut.", m),
        "The gate is shut."
    );
    assert_eq!(line(&xx, "Gone.", m), "Gone.");
    // An entry whose source isn't the English asked about is stale.
    let key = id("The gate is shut.");
    assert_eq!(
        lang.line(&xx, &key, "The gate is open.", m),
        "The gate is open."
    );
    let entry = lang.pack(&xx).unwrap().line(&id("{lead} waits.")).unwrap();
    assert_eq!(entry.source, "{lead} waits.");
    assert_eq!(entry.text.for_lead(f), "SHE WAITS.");
}

/// Every text of `steps`, in script order: captions, lines, replies.
fn texts(steps: &[Step], out: &mut Vec<String>) {
    for step in steps {
        match step {
            Step::Caption { text } | Step::Say { text, .. } | Step::Narrate { text, .. } => {
                out.push(text.clone());
            }
            Step::Choice { options } => {
                for option in options {
                    out.push(option.text.clone());
                    texts(&option.steps, out);
                }
            }
            Step::If {
                then, otherwise, ..
            } => {
                texts(then, out);
                texts(otherwise, out);
            }
            Step::Place { .. } | Step::Clear { .. } | Step::Music(_) => {}
        }
    }
}

#[test]
fn a_scene_in_a_language_has_the_packs_text_and_the_same_ids() {
    let lang = lang(dialogue_pack());
    let english = scene();
    let told = |gender| {
        let mut out = Vec::new();
        texts(&lang.scene(&code("xx"), &english, gender).steps, &mut out);
        out
    };
    assert_eq!(
        told(LeadGender::Female),
        [
            "THE GATE, DUSK",
            "THE GATE IS SHUT.",
            "Open it, {n:king}.",
            "STILL HERE.",
            "Gone.",
            "I will wait here.",
            "Nobody answers.",
            "THEN I CLIMB.",
            "THE WALL IS HIGH.",
            "SHE WAITS.",
        ]
    );
    assert_eq!(told(LeadGender::Male)[9], "HE WAITS.");
    // Only the words change: who stands where, and every line's id.
    let translated = lang.scene(&code("xx"), &english, LeadGender::Male);
    let ids =
        |scene: &Scene| -> Vec<String> { scene.lines().iter().map(|l| l.id.to_string()).collect() };
    assert_eq!(ids(&translated), ids(&english));
    assert_eq!(translated.id, "gate");
    assert_eq!(translated.steps.len(), english.steps.len());
    assert_eq!(translated.steps[2], english.steps[2]);
    // English, and a code with no pack, are the scene itself.
    let male = LeadGender::Male;
    assert_eq!(lang.scene(&LangCode::english(), &english, male), english);
    assert_eq!(lang.scene(&code("zz"), &english, male), english);
}

#[test]
fn status_lists_what_a_pack_lacks_of_each_kind() {
    let data_ron = r#"[
        (key: "items.potion.name", source: "Potion", text: "Trank"),
        (key: "classes.guard.name", source: "Warden", text: "Wache"),
    ]"#;
    let lines = format!(
        "[{}, (key: {:?}, source: \"The gate is open.\", text: \"x\")]",
        entry("Gone.", "WEG."),
        id("The gate is shut."),
    );
    let pack = pack(data_ron, &[("xx/dialogue/gate.ron", &lines)]).unwrap();
    let english = english_from_source("en/ui.ron", r#"{"a.one": "One"}"#).unwrap();
    let lang = Lang::with_data(english, data(), [(code("xx"), pack)].into());
    let status = lang.status(&code("xx")).unwrap();
    // Screen text, then data, then dialogue.
    assert_eq!(status.missing[0], "a.one");
    assert_eq!(status.missing[1], "battles.gate.note_1");
    assert!(status.missing.contains(&"names.king".to_owned()));
    assert!(!status.missing.contains(&"items.potion.name".to_owned()));
    assert!(status.missing.contains(&id("Nobody answers.")));
    assert!(status.missing.contains(&id("The gate, dusk")));
    assert!(!status.missing.contains(&id("Gone.")));
    // 1 screen text, 9 data texts of 11, 8 lines of 10.
    assert_eq!(status.missing.len(), 1 + 9 + 8);
    assert_eq!(
        status.stale,
        ["classes.guard.name".to_owned(), id("The gate is shut.")]
    );
    assert_eq!(status.orphans, []);
    assert_eq!(lang.english(&id("Gone.")), Some("Gone."));
    assert_eq!(lang.english("names.king"), Some("the King"));
    assert_eq!(lang.english("a.one"), Some("One"));
    assert_eq!(lang.english("gate_00000000"), None);
}

#[test]
fn an_orphan_is_paired_with_the_nearest_untranslated_line_of_its_scene() {
    let orphan =
        |key: &str, source: &str| format!("(key: {key:?}, source: {source:?}, text: \"ALT\")");
    let status = |entries: &[String]| {
        let file = format!("[{}]", entries.join(","));
        let pack = pack("[]", &[("xx/dialogue/gate.ron", &file)]).unwrap();
        lang(pack).status(&code("xx")).unwrap()
    };
    // "The gate was shut." was reworded to "The gate is shut.".
    let found = status(&[orphan("gate_00000001", "The gate was shut.")]);
    assert_eq!(
        found.orphans,
        [Orphan {
            key: "gate_00000001".to_owned(),
            source: "The gate was shut.".to_owned(),
            text: LineText::One("ALT".to_owned()),
            nearest: Some(id("The gate is shut.")),
        }]
    );
    // An orphan isn't missing or stale: it has no key of today's.
    assert!(!found.missing.contains(&"gate_00000001".to_owned()));
    assert_eq!(found.stale, Vec::<String>::new());
    // A line the pack already has isn't a candidate.
    let found = status(&[
        orphan("gate_00000001", "The gate was shut."),
        entry("The gate is shut.", "ZU."),
    ]);
    let nearest = found.orphans[0].nearest.clone();
    assert!(nearest.is_some());
    assert_ne!(nearest, Some(id("The gate is shut.")));
    // A caption is only paired with a caption, and a line never with one.
    let found = status(&[orphan("caption.gate_00000001", "The gate is shut")]);
    assert_eq!(found.orphans[0].nearest, Some(id("The gate, dusk")));
    let found = status(&[orphan("gate_00000001", "The gate, dusk.")]);
    assert_ne!(found.orphans[0].nearest, Some(id("The gate, dusk")));
    let found = status(&[
        orphan("caption.gate_00000001", "The gate, dawn"),
        entry("The gate, dusk", "ABENDS"),
    ]);
    assert_eq!(found.orphans[0].nearest, None);
    // A scene English no longer has: nothing to pair with.
    let found = status(&[orphan("road_00000001", "The gate was shut.")]);
    assert_eq!(found.orphans[0].nearest, None);
    // A repeat's id (`_2`) belongs to its scene too; orphans come in key
    // order.
    let found = status(&[
        orphan("gate_00000001_2", "Nobody answered."),
        orphan("gate_00000000", "Gone!"),
    ]);
    let pairs: Vec<_> = found
        .orphans
        .iter()
        .map(|o| (o.key.as_str(), o.nearest.clone()))
        .collect();
    assert_eq!(
        pairs,
        [
            ("gate_00000000", Some(id("Gone."))),
            ("gate_00000001_2", Some(id("Nobody answers."))),
        ]
    );
}

/// The languages made of `files` (path, text) with [`data`], or the
/// errors as text.
fn files(files: &[(&str, Option<&str>)], data: Option<DataText>) -> Result<Lang, Vec<String>> {
    let files: Vec<LangFile<'_>> = files
        .iter()
        .map(|&(path, source)| LangFile { path, source })
        .collect();
    from_files(&files, data, |path| format!("lang/{path}"))
        .map_err(|errors| errors.iter().map(ToString::to_string).collect())
}

#[test]
fn a_packs_data_and_dialogue_files_are_loaded_and_checked() {
    let lines = format!("[{}]", entry("Gone.", "WEG."));
    let good = [
        ("en/ui.ron", Some("{}")),
        ("xx/lang.ron", Some(INFO)),
        ("xx/ui.ron", Some("[]")),
        ("xx/data.ron", Some(DATA_RON)),
        ("xx/dialogue/gate.ron", Some(lines.as_str())),
        // A pack needs neither file.
        ("yy/lang.ron", Some(INFO)),
        ("yy/ui.ron", Some("[]")),
    ];
    let lang = files(&good, Some(data())).unwrap();
    let (xx, yy) = (code("xx"), code("yy"));
    assert_eq!(lang.name(&xx, "king", "the King"), "der König");
    assert_eq!(
        lang.line(&xx, &id("Gone."), "Gone.", LeadGender::Male),
        "WEG."
    );
    assert_eq!(lang.name(&yy, "king", "the King"), "the King");
    assert_eq!(lang.source(), &data());
    // Without the data's English, the files are there but not read.
    let lang = files(&good, None).unwrap();
    assert_eq!(lang.name(&xx, "king", "the King"), "the King");
    assert_eq!(lang.source(), &DataText::default());
    // Errors carry the file's name.
    let mut bad = good;
    bad[3].1 = Some(r#"[(key: "items.nope.name", source: "x", text: "y")]"#);
    bad[4].1 = Some(r#"[(key: "nope", source: "x", text: "y")]"#);
    assert_eq!(
        files(&bad, Some(data())).unwrap_err(),
        [
            "lang/xx/data.ron: \"items.nope.name\": no such key in the data",
            "lang/xx/dialogue/gate.ron: \"nope\": not a line id or a caption key",
        ]
    );
    // A file that isn't text.
    bad[3].1 = None;
    bad[4].1 = None;
    assert_eq!(
        files(&bad, Some(data())).unwrap_err(),
        [
            "lang/xx/data.ron: file not found in asset bundle",
            "lang/xx/dialogue/gate.ron: file not found in asset bundle",
        ]
    );
}

#[test]
fn the_embedded_test_pack_has_an_orphan_and_stale_data() {
    let content = crate::load_embedded().unwrap();
    let status = content.lang.status(&code(TEST)).unwrap();
    assert_eq!(status.stale, ["title.subtitle", "items.iron_axe.name"]);
    assert_eq!(status.orphans.len(), 1);
    let orphan = &status.orphans[0];
    assert_eq!(orphan.key, "test_00000000");
    assert_eq!(orphan.nearest.as_deref(), Some("test_92dc3bd5"));
    assert!(status.missing.contains(&"title.credits".to_owned()));
    assert!(
        status
            .missing
            .contains(&"items.steel_sword.name".to_owned())
    );
    assert!(status.missing.contains(&"test_36a692b7".to_owned()));
    assert!(!status.missing.contains(&"test_5da5d147".to_owned()));
    assert!(!status.missing.contains(&"items.potion.name".to_owned()));
}
