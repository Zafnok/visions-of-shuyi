use super::*;

fn errors(source: &str) -> Vec<String> {
    from_source("t.ron", source, Refs::default())
        .err()
        .unwrap_or_default()
        .into_iter()
        .map(|e| e.message)
        .collect()
}

/// A lesson with `items` (RON) and otherwise good fields.
fn lesson(id: &str, items: &str) -> String {
    format!(
        r#"(id: "{id}", level: Full, trigger: FirstBattleStart, title: "T",
        text: "Press {{Confirm}}.", items: [{items}])"#
    )
}

const UP: &str = r#"(label: "Up {CursorUp}", done_when: Pressed("CursorUp"))"#;

#[test]
fn the_shipped_files_load_with_the_test_lesson() {
    let table = load(Refs::default()).unwrap();
    let test = table.get("test_cursor").unwrap();
    assert_eq!(test.level, LessonLevel::Full);
    assert_eq!(test.trigger, TipTrigger::FirstBattleStart);
    assert_eq!(test.battles, ["test", "quick"]);
    let done: Vec<DoneWhen> = test.items.iter().map(|i| i.done_when).collect();
    assert_eq!(
        done,
        [
            Action::CursorUp,
            Action::CursorDown,
            Action::CursorLeft,
            Action::CursorRight
        ]
        .map(DoneWhen::Pressed)
    );
    // A build that ships has none of the test files: no test lesson.
    let story = bundle::story_only(|| load(Refs::default())).unwrap();
    assert_eq!(story.get("test_cursor"), None);
    assert!(story.lessons.iter().all(|l| l.battles.is_empty()));
}

#[test]
fn a_good_lesson_loads_with_every_condition() {
    let items = [
        UP,
        r#"(label: "a", done_when: UnitSelected)"#,
        r#"(label: "b", done_when: UnitMoved)"#,
        r#"(label: "c", done_when: ForecastOpened)"#,
        r#"(label: "d", done_when: Attacked)"#,
        r#"(label: "e", done_when: RewindOpened)"#,
    ]
    .join(",");
    let more = r#"(id: "b", level: Short, trigger: FirstEnemyPhase, battles: ["x"],
        title: "T", text: "x\ny", face: "sad",
        items: [(label: "{Cursor}", done_when: TurnEnded)])"#;
    let source = format!("(lessons: [{}, {more}])", lesson("a", &items));
    let table = from_source("t.ron", &source, Refs::default()).unwrap();
    let a = table.get("a").unwrap();
    assert_eq!(a.items.len(), 6);
    assert_eq!(a.items[0].label, "Up {CursorUp}");
    assert!(a.has(DoneWhen::Attacked) && !a.has(DoneWhen::TurnEnded));
    assert_eq!(a.face(), DEFAULT_FACE);
    let b = table.get("b").unwrap();
    assert_eq!((b.level, b.face()), (LessonLevel::Short, "sad"));
    assert!(b.has(DoneWhen::TurnEnded));
    let first: Vec<&str> = table
        .for_trigger(TipTrigger::FirstBattleStart)
        .map(|l| l.id.as_str())
        .collect();
    assert_eq!(first, ["a"]);
    assert_eq!(table.for_trigger(TipTrigger::FirstLowHp).count(), 0);
}

#[test]
fn the_lessons_conditions_load_and_say_which_need_a_unit() {
    let names = [
        ("CursorMoved", DoneWhen::CursorMoved, false),
        ("ActionMenuOpened", DoneWhen::ActionMenuOpened, true),
        ("ArtsOpened", DoneWhen::ArtsOpened, true),
        ("TileAimed", DoneWhen::TileAimed, true),
        ("ThreatPointed", DoneWhen::ThreatPointed, true),
        ("SafePointed", DoneWhen::SafePointed, true),
        ("RewindClosed", DoneWhen::RewindClosed, false),
        ("OptionsOpened", DoneWhen::OptionsOpened, false),
    ];
    let items: Vec<String> = names
        .iter()
        .map(|(name, _, _)| format!(r#"(label: "x", done_when: {name})"#))
        .collect();
    let source = format!(
        "(lessons: [{}, {}])",
        lesson("a", &items[..6].join(",")),
        lesson("b", &items[6..].join(","))
    );
    let table = from_source("t.ron", &source, Refs::default()).unwrap();
    let done: Vec<DoneWhen> = table
        .lessons
        .iter()
        .flat_map(|l| l.items.iter().map(|i| i.done_when))
        .collect();
    assert_eq!(done, names.map(|(_, d, _)| d));
    for (_, d, unit) in names {
        assert_eq!(d.needs_unit(), unit, "{d:?}");
    }
    for d in [
        DoneWhen::UnitSelected,
        DoneWhen::UnitMoved,
        DoneWhen::Attacked,
    ] {
        assert!(d.needs_unit(), "{d:?}");
    }
    let keys = [
        DoneWhen::Pressed(Action::Info),
        DoneWhen::RewindOpened,
        DoneWhen::TurnEnded,
    ];
    assert!(keys.iter().all(|d| !d.needs_unit()));
}

/// The story's lessons (0457): the basic controls at Full, what is
/// different here at Short, each with the trigger it starts at.
#[test]
fn the_story_has_the_ten_lessons_of_the_design() {
    use LessonLevel::{Full, Short};
    use TipTrigger as T;
    let story = bundle::story_only(|| load(Refs::default())).unwrap();
    let got: Vec<(&str, LessonLevel, TipTrigger)> = story
        .lessons
        .iter()
        .map(|l| (l.id.as_str(), l.level, l.trigger))
        .collect();
    assert_eq!(
        got,
        [
            ("cursor", Full, T::FirstBattleStart),
            ("keys", Short, T::FirstBattleStart),
            ("next_unit", Full, T::FirstBattleStart),
            ("moving", Full, T::FirstBattleStart),
            ("attacking", Full, T::FirstEnemyInRange),
            ("ending_turn", Full, T::NoUnitLeft),
            ("rewind", Short, T::SecondTurn),
            ("threat_arrows", Short, T::ThreatArrowsShown),
            ("combat_arts", Short, T::ArtsAvailable),
            ("terrain_magic", Short, T::TileSpellAvailable),
        ]
    );
    // Only ending the turn's lesson lets the turn end.
    let ends: Vec<&str> = story
        .lessons
        .iter()
        .filter(|l| l.has(DoneWhen::TurnEnded))
        .map(|l| l.id.as_str())
        .collect();
    assert_eq!(ends, ["ending_turn"]);
    // Narrower than the format allows: the box with the guide's face stays
    // over the map, off the side panel.
    for l in &story.lessons {
        for line in l.text.lines() {
            assert!(text_width(line) <= 45, "{}: {line:?}", l.id);
        }
        for item in &l.items {
            assert!(text_width(&item.label) <= 41, "{}: {:?}", l.id, item.label);
        }
    }
}

#[test]
fn a_lesson_starts_in_its_battles_or_any() {
    let source = format!("(lessons: [{}])", lesson("a", UP));
    let mut l = from_source("t.ron", &source, Refs::default())
        .unwrap()
        .lessons[0]
        .clone();
    assert!(l.starts_in(Some("ch01")) && l.starts_in(None));
    l.battles = vec!["test".into()];
    assert!(l.starts_in(Some("test")));
    assert!(!l.starts_in(Some("ch01")) && !l.starts_in(None));
}

#[test]
fn levels_show_at_their_tutorial_levels() {
    use TutorialLevel as T;
    let at = |l: LessonLevel| [T::Full, T::Short, T::None].map(|t| l.shows_at(t));
    assert_eq!(at(LessonLevel::Full), [true, false, false]);
    assert_eq!(at(LessonLevel::Short), [true, true, false]);
}

#[test]
fn bad_lessons_are_reported() {
    let long = "x".repeat(MAX_LINE_CHARS + 1);
    let label = "x".repeat(MAX_ITEM_CHARS + 1);
    let seven = [UP; 7].join(",");
    let source = format!(
        r#"(lessons: [
        (id: "A", level: Full, trigger: FirstForecast, title: "", text: "{{Nope}}",
            items: [(label: "{{Gone}}", done_when: Pressed("Jump"))]),
        (id: "b", level: Full, trigger: FirstForecast, title: "T", text: "1\n2\n3",
            items: []),
        (id: "b", level: Full, trigger: FirstForecast, title: "T", text: "{long}",
            items: [{UP}, {UP}, (label: "{label}", done_when: Attacked),
                (label: "", done_when: TurnEnded), (label: "a\nb", done_when: UnitMoved)]),
        (id: "c", level: Full, trigger: FirstForecast, title: "T", text: "",
            items: [{seven}]),
        ])"#
    );
    let all = errors(&source);
    let has = |part: &str| all.iter().any(|m| m.contains(part));
    assert!(has("lesson \"A\": the id must be lowercase"), "{all:?}");
    assert!(has("the title must be 1 to 30"), "{all:?}");
    assert!(has("unknown placeholder {Nope}"), "{all:?}");
    assert!(has("unknown placeholder {Gone}"), "{all:?}");
    assert!(has("unknown action in Pressed(\"Jump\")"), "{all:?}");
    assert!(
        has("lesson \"b\": the text must have 1 to 2 lines"),
        "{all:?}"
    );
    assert!(
        has("lesson \"b\": a lesson must have 1 to 6 items"),
        "{all:?}"
    );
    assert!(has("lesson \"b\": the id is used twice"), "{all:?}");
    assert!(has("a text line is over 60 cells"), "{all:?}");
    assert!(has("two items are done by Pressed(CursorUp)"), "{all:?}");
    assert!(has("an item must be one line of 1 to 56 cells"), "{all:?}");
    assert!(
        has("lesson \"c\": the text must have 1 to 2 lines"),
        "{all:?}"
    );
    assert!(
        has("lesson \"c\": a lesson must have 1 to 6 items"),
        "{all:?}"
    );
    // The long label, the empty one and the one of two lines.
    let items = all.iter().filter(|m| m.contains("an item must be")).count();
    assert_eq!(items, 3, "{all:?}");
}

#[test]
fn length_limits_are_inclusive() {
    let title = "t".repeat(MAX_TITLE_CHARS);
    let line = "x".repeat(MAX_LINE_CHARS);
    let label = "l".repeat(MAX_ITEM_CHARS);
    let items = (0..MAX_ITEMS)
        .map(|i| {
            format!(
                r#"(label: "{label}", done_when: Pressed("{}"))"#,
                CURSORS[i]
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let source = format!(
        r#"(lessons: [(id: "a", level: Full, trigger: FirstForecast, title: "{title}",
        text: "{line}\n{line}", items: [{items}])])"#
    );
    assert_eq!(errors(&source), Vec::<String>::new());
    assert_eq!(
        errors(&source.replace(&title, &format!("{title}t"))).len(),
        1
    );
    assert_eq!(
        errors(&source.replacen(&line, &format!("{line}x"), 1)).len(),
        1
    );
    assert_eq!(
        errors(&source.replacen(&label, &format!("{label}l"), 1)).len(),
        1
    );
}

/// Six different actions, for six items.
const CURSORS: [&str; 6] = [
    "CursorUp",
    "CursorDown",
    "CursorLeft",
    "CursorRight",
    "Info",
    "NextUnit",
];

#[test]
fn battles_and_faces_are_checked_when_known() {
    let source = r#"(lessons: [(id: "a", level: Full, trigger: FirstForecast, title: "T",
        text: "x", battles: ["test", "nowhere"], face: "smug",
        items: [(label: "x", done_when: Attacked)])])"#;
    // Unchecked without them.
    assert!(from_source("t.ron", source, Refs::default()).is_ok());
    let content = crate::load_embedded().unwrap();
    let guide = content.portraits["test_lord"].clone();
    let faces = BTreeMap::from([(GUIDE_PORTRAIT.to_owned(), guide)]);
    let refs = Refs {
        battles: Some(&content.battles),
        faces: Some(&faces),
    };
    let all: Vec<String> = from_source("t.ron", source, refs)
        .unwrap_err()
        .into_iter()
        .map(|e| e.message)
        .collect();
    assert_eq!(
        all,
        [
            "lesson \"a\": no battle \"nowhere\"",
            "lesson \"a\": the guide's portrait has no expression \"smug\"",
        ]
    );
}

#[test]
fn the_test_files_lessons_are_added_by_id() {
    let story = format!("(lessons: [{}])", lesson("a", UP));
    let part = format!("(lessons: [{}, {}])", lesson("b", UP), lesson("a", UP));
    let all: Vec<String> = from_sources("s.ron", &story, Some(("p.ron", &part)), Refs::default())
        .unwrap_err()
        .into_iter()
        .map(|e| e.to_string())
        .collect();
    assert_eq!(all.len(), 1, "{all:?}");
    assert!(all[0].starts_with("p.ron") && all[0].contains("lesson \"a\": the id is used twice"));
    let part = format!("(lessons: [{}])", lesson("b", UP));
    let table = from_sources("s.ron", &story, Some(("p.ron", &part)), Refs::default()).unwrap();
    let ids: Vec<&str> = table.lessons.iter().map(|l| l.id.as_str()).collect();
    assert_eq!(ids, ["a", "b"]);
    // A broken part is reported, and the story's lessons still checked.
    let all = from_sources(
        "s.ron",
        "(lessons: [",
        Some(("p.ron", "(")),
        Refs::default(),
    );
    assert_eq!(all.unwrap_err().len(), 2);
}

#[test]
fn a_translation_keeps_to_the_limits() {
    assert_eq!(text_problem("a\nb"), None);
    assert!(text_problem("").is_some() && text_problem("a\nb\nc").is_some());
    assert_eq!(item_problem("a"), None);
    assert!(item_problem("").is_some() && item_problem("a\nb").is_some());
    assert!(item_problem(&"é".repeat(MAX_ITEM_CHARS + 1)).is_some());
}
