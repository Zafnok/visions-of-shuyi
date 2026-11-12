//! What the party knows when the story says it (0753): a fact a scene
//! relays needs an earlier source on the path the player took, and a line
//! that names who saw something needs that witness to have told it. These
//! tests play the real scenes, resolved for who is in the army on each
//! path, and read the player text.
//!
//! What they check the story against (the facts, the phrases that carry
//! them, the paths and where each fact is first said) tells the plot, so
//! it is in the private repository, `content/tests/knowledge.ron`
//! (ADR-0118 §3). Without it the test of the story skips.

use std::collections::{BTreeMap, BTreeSet};

use serde::Deserialize;

use super::*;
use crate::ron_loader::parse_ron;

/// The private file the test reads, in the content's test data.
const SPEC: &str = "knowledge.ron";

/// What the story is checked against.
#[derive(Debug, Deserialize)]
struct Spec {
    /// The speakers who know a fact without hearing it first.
    sources: Vec<String>,
    /// The one an attribution names: it needs them to have told it.
    witness: String,
    /// Everyone else the army may have; the scenes are played with them.
    others: Vec<String>,
    /// The facts, in order. The witness has told it once they say the
    /// first.
    facts: Vec<Fact>,
    /// Scenes that carry every fact but the attributions, on every path.
    carried_by: Vec<String>,
    paths: Vec<Path>,
    /// The reader's own checks: a text, and the facts it carries.
    #[serde(default)]
    reads: Vec<Read>,
}

/// A fact, as the player text carries it.
#[derive(Debug, Deserialize)]
struct Fact {
    id: String,
    /// Any of these, in the rendered text in lower case, carries it.
    phrases: Vec<String>,
    /// Whether it names who saw (it needs the witness's telling first).
    #[serde(default)]
    attribution: bool,
}

/// A path through the story: its scenes, in order, with who is there for
/// each besides the lead and the others.
#[derive(Debug, Deserialize)]
struct Path {
    name: String,
    steps: Vec<(String, Vec<String>)>,
    /// Where each fact is first said on the path.
    first_said: BTreeMap<String, String>,
    witness_told: bool,
    #[serde(default)]
    lines: Vec<Line>,
}

/// A line some scene has, or hasn't, for those there.
#[derive(Debug, Deserialize)]
struct Line {
    scene: String,
    there: Vec<String>,
    speaker: String,
    /// The line has this, in lower case.
    #[serde(default)]
    text: Option<String>,
    /// The line carries these facts.
    #[serde(default)]
    facts: Vec<String>,
    /// Whether such a line is there (the default) or not.
    #[serde(default = "yes")]
    found: bool,
}

/// A text and the facts it carries.
#[derive(Debug, Deserialize)]
struct Read {
    text: String,
    facts: Vec<String>,
}

fn yes() -> bool {
    true
}

impl Fact {
    /// Whether `text` (rendered, lower case) carries the fact.
    fn is_in(&self, text: &str) -> bool {
        self.phrases.iter().any(|p| text.contains(p.as_str()))
    }
}

impl Spec {
    /// The ids of the facts `text` carries, in order.
    fn facts_in(&self, text: &str) -> Vec<&str> {
        let carried = self.facts.iter().filter(|f| f.is_in(text));
        carried.map(|f| f.id.as_str()).collect()
    }

    /// Whether the fact `id` is an attribution.
    fn is_attribution(&self, id: &str) -> bool {
        self.facts.iter().any(|f| f.id == id && f.attribution)
    }
}

/// What a path's scenes established, read in play order.
#[derive(Debug, Default, PartialEq)]
struct Knowledge {
    /// Where each fact was first said, by anyone who could know it.
    first_said: BTreeMap<String, String>,
    /// Whether the witness has said the first fact themself.
    witness_told: bool,
    /// Every problem found: a relay of a fact nobody established, or an
    /// attribution the witness never made.
    problems: Vec<String>,
}

/// Who is there: the lead, the others and `there`.
fn present(spec: &Spec, there: &[String]) -> Present {
    let everyone = ["lead"]
        .into_iter()
        .chain(spec.others.iter().map(String::as_str))
        .chain(there.iter().map(String::as_str))
        .map(id);
    Present::only(everyone)
}

/// The lines of `scene_id` for those `there`: each speaker and its
/// rendered text, in lower case.
fn rendered(
    content: &crate::Content,
    spec: &Spec,
    scene_id: &str,
    there: &[String],
) -> Vec<(String, String)> {
    let scene = content
        .dialogue
        .get(scene_id)
        .unwrap_or_else(|| panic!("no scene {scene_id}"));
    scene
        .resolved(&present(spec, there))
        .lines()
        .iter()
        .map(|l| {
            (
                l.speaker.to_owned(),
                content.names.substitute(l.text).to_lowercase(),
            )
        })
        .collect()
}

/// Plays `steps` in order and reads what each scene tells the player.
fn play(content: &crate::Content, spec: &Spec, steps: &[(String, Vec<String>)]) -> Knowledge {
    let mut known = Knowledge::default();
    let first = spec.facts.first().map(|f| f.id.as_str());
    for (scene_id, there) in steps {
        for (speaker, text) in rendered(content, spec, scene_id, there) {
            let facts = spec.facts_in(&text);
            let source = spec.sources.contains(&speaker);
            for &fact in &facts {
                let known_before = known.first_said.contains_key(fact);
                if spec.is_attribution(fact) {
                    if !known.witness_told {
                        known.problems.push(format!(
                            "{scene_id}: {speaker} names who saw it ({fact}), who never \
                             told it on this path: {text:?}"
                        ));
                    }
                } else if !source && !known_before {
                    known.problems.push(format!(
                        "{scene_id}: {speaker} says {fact} with no earlier source on this \
                         path: {text:?}"
                    ));
                }
                known
                    .first_said
                    .entry(fact.to_owned())
                    .or_insert_with(|| scene_id.clone());
            }
            if speaker == spec.witness && first.is_some_and(|f| facts.contains(&f)) {
                known.witness_told = true;
            }
        }
    }
    known
}

/// Every way `path` breaks the story: problems in play, a fact first said
/// elsewhere, a scene of [`Spec::carried_by`] that drops a fact, a line
/// that is or isn't there.
fn check(content: &crate::Content, spec: &Spec, path: &Path) -> Vec<String> {
    let name = &path.name;
    let known = play(content, spec, &path.steps);
    let mut problems: Vec<String> = known
        .problems
        .iter()
        .map(|p| format!("{name}: {p}"))
        .collect();
    if known.first_said != path.first_said {
        problems.push(format!(
            "{name}: facts first said at {:?}, not {:?}",
            known.first_said, path.first_said
        ));
    }
    if known.witness_told != path.witness_told {
        problems.push(format!(
            "{name}: the witness told it: {}, not {}",
            known.witness_told, path.witness_told
        ));
    }
    for (scene_id, there) in &path.steps {
        if !spec.carried_by.contains(scene_id) {
            continue;
        }
        let text: Vec<String> = rendered(content, spec, scene_id, there)
            .into_iter()
            .map(|(_, t)| t)
            .collect();
        let text = text.join("\n");
        for fact in spec.facts.iter().filter(|f| !f.attribution) {
            if !fact.is_in(&text) {
                problems.push(format!("{name}: {scene_id} doesn't carry {}", fact.id));
            }
        }
    }
    for line in &path.lines {
        let lines = rendered(content, spec, &line.scene, &line.there);
        let found = lines.iter().any(|(speaker, text)| {
            let carried = spec.facts_in(text);
            *speaker == line.speaker
                && line.text.as_ref().is_none_or(|t| text.contains(t.as_str()))
                && line.facts.iter().all(|f| carried.contains(&f.as_str()))
        });
        if found != line.found {
            problems.push(format!(
                "{name}: {}: a line by {} like {line:?} is there: {found}",
                line.scene, line.speaker
            ));
        }
    }
    problems
}

/// The spec in `dir`.
fn spec(dir: &std::path::Path) -> Spec {
    let file = dir.join(SPEC);
    let source = std::fs::read_to_string(&file)
        .unwrap_or_else(|e| panic!("reading {}: {e}", file.display()));
    parse_ron(SPEC, &source).unwrap_or_else(|e| panic!("{SPEC}: {e:?}"))
}

/// Every path of the private spec keeps what the party knows, and the
/// fact reader reads the spec's own examples as it says.
#[test]
fn every_path_keeps_what_the_party_knows() {
    let Some(dir) = bundle::content_tests() else {
        return;
    };
    let spec = spec(&dir);
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    assert!(!spec.paths.is_empty());
    let problems: Vec<String> = spec
        .paths
        .iter()
        .flat_map(|path| check(&content, &spec, path))
        .collect();
    assert_eq!(problems, Vec::<String>::new());
    for read in &spec.reads {
        assert_eq!(spec.facts_in(&read.text), read.facts, "{:?}", read.text);
    }
}

/// A made-up spec over the test scene (`assets-test/dialogue/test.dlg`):
/// `facts`, each `(id, phrase, attribution)`, with the archer the only
/// source and `witness` the witness.
fn made_up(facts: &[(&str, &str, bool)], witness: &str) -> Spec {
    Spec {
        sources: vec!["test_archer".to_owned()],
        witness: witness.to_owned(),
        others: vec![],
        facts: facts
            .iter()
            .map(|&(id, phrase, attribution)| Fact {
                id: id.to_owned(),
                phrases: vec![phrase.to_owned()],
                attribution,
            })
            .collect(),
        carried_by: vec![],
        paths: vec![],
        reads: vec![],
    }
}

/// The test scene with the knight there, as a path's steps.
fn test_scene() -> Vec<(String, Vec<String>)> {
    vec![("test".to_owned(), vec!["test_knight".to_owned()])]
}

/// The reader finds each fact by its phrases, in order, and nothing in a
/// text without them.
#[test]
fn the_facts_are_read_from_the_words() {
    let spec = made_up(
        &[("rain", "the rain", false), ("saw", "the archer saw", true)],
        "test_archer",
    );
    assert_eq!(spec.facts_in("the rain had not stopped"), ["rain"]);
    assert_eq!(spec.facts_in("the archer saw the rain"), ["rain", "saw"]);
    assert_eq!(spec.facts_in("it was dry"), Vec::<&str>::new());
    assert!(spec.is_attribution("saw"));
    assert!(!spec.is_attribution("rain"));
}

/// A fact first said by someone who is no source is a problem; said again
/// once known, it isn't.
#[test]
fn a_relay_without_an_earlier_source_is_a_problem() {
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    // The narration says "the rain" twice, and it is no source.
    let spec = made_up(&[("rain", "the rain", false)], "test_archer");
    let known = play(&content, &spec, &test_scene());
    assert_eq!(
        known.first_said.get("rain").map(String::as_str),
        Some("test")
    );
    assert_eq!(known.problems.len(), 1, "{:?}", known.problems);
    assert!(known.problems[0].contains("no earlier source"));
    assert!(!known.witness_told);
}

/// The witness tells it by saying the first fact; an attribution before
/// that is a problem, after it is not.
#[test]
fn an_attribution_needs_the_witness_to_have_told_it() {
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    // The archer (a source) asks "about me", then the lord answers "about
    // you": the attribution.
    let facts = [("me", "about me", false), ("you", "about you", true)];
    let told = made_up(&facts, "test_archer");
    let known = play(&content, &told, &test_scene());
    assert!(known.witness_told);
    assert_eq!(known.problems, Vec::<String>::new());
    // With the lord the witness, the archer's line tells nothing.
    let untold = made_up(&facts, "test_lord");
    let known = play(&content, &untold, &test_scene());
    assert!(!known.witness_told);
    assert_eq!(known.problems.len(), 1, "{:?}", known.problems);
    assert!(known.problems[0].contains("never told it"));
}

/// A path's expectations: where each fact is first said, whether the
/// witness told it, a scene that must carry the facts, and a line that is
/// there or not.
#[test]
fn a_path_is_checked_against_what_it_expects() {
    let content = crate::load_embedded().unwrap_or_else(|e| panic!("{e}"));
    let mut spec = made_up(
        &[("me", "about me", false), ("you", "about you", true)],
        "test_archer",
    );
    spec.carried_by = vec!["test".to_owned()];
    let line = |text: &str, found| Line {
        scene: "test".to_owned(),
        there: vec![],
        speaker: "test_archer".to_owned(),
        text: Some(text.to_owned()),
        facts: vec!["me".to_owned()],
        found,
    };
    let mut path = Path {
        name: "made up".to_owned(),
        steps: test_scene(),
        first_said: BTreeMap::from([
            ("me".to_owned(), "test".to_owned()),
            ("you".to_owned(), "test".to_owned()),
        ]),
        witness_told: true,
        lines: vec![line("was that", true), line("came back", false)],
    };
    assert_eq!(check(&content, &spec, &path), Vec::<String>::new());
    path.first_said.remove("you");
    path.witness_told = false;
    path.lines = vec![line("was that", false)];
    spec.facts.push(Fact {
        id: "dry".to_owned(),
        phrases: vec!["the sun".to_owned()],
        attribution: false,
    });
    let problems: BTreeSet<String> = check(&content, &spec, &path).into_iter().collect();
    let kinds = [
        "first said",
        "the witness told it",
        "doesn't carry dry",
        "is there: true",
    ];
    assert_eq!(problems.len(), kinds.len(), "{problems:?}");
    for kind in kinds {
        assert!(
            problems.iter().any(|p| p.contains(kind)),
            "{kind}: {problems:?}"
        );
    }
}
