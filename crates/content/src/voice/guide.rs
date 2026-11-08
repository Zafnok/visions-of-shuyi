//! The guide's lines (ticket 0476): what Vix says in a lesson's or a
//! tip's box, its `text`, voiced like a script's lines. They have no `.dlg`
//! scene, so one is built from the lessons and tips, [`GUIDE_SCENE`], with
//! ids made as a script line's are (ADR-0045 §3): rewording gives a new
//! id. A lesson's items and a tip's `how` are silent (0475): no line.
use crate::prelude::*;

use shuyi_assets::dialogue::{CharacterId, line_hash};

use crate::dialogue::{DialogueTable, LineId, Scene, Step};
use crate::lesson::LessonTable;
use crate::tip::TipTable;

/// The id of the guide's scene, and of its folder of clips.
pub const GUIDE_SCENE: &str = "vix";

/// Who says the guide's lines: her entry in the voice cast.
pub const GUIDE_SPEAKER: &str = "guide";

/// The words of a lesson's or tip's `text` as one line: its line breaks
/// are only where the box breaks it.
fn words(text: &str) -> String {
    text.lines().map(str::trim).collect::<Vec<_>>().join(" ")
}

/// The id of the line the guide says for a lesson's or tip's `text`. The
/// same words twice are one line, with one clip.
pub fn guide_line(text: &str) -> LineId {
    let hash = line_hash(GUIDE_SPEAKER, &words(text));
    LineId::new(format!("{GUIDE_SCENE}_{hash}"))
}

/// The guide's scene: a line for the `text` of every lesson, then of
/// every tip, in their files' order, each once.
pub fn guide_scene(lessons: &LessonTable, tips: &TipTable) -> Scene {
    let texts = lessons.lessons.iter().map(|l| l.text.as_str());
    let texts = texts.chain(tips.tips.iter().map(|t| t.text.as_str()));
    let mut steps: Vec<Step> = Vec::new();
    for text in texts {
        let line = guide_line(text);
        let said = |s: &Step| matches!(s, Step::Say { line: l, .. } if *l == line);
        if !steps.iter().any(said) {
            steps.push(Step::Say {
                speaker: CharacterId(GUIDE_SPEAKER.to_owned()),
                expression: None,
                text: words(text),
                direction: None,
                line,
            });
        }
    }
    Scene {
        id: GUIDE_SCENE.to_owned(),
        steps,
    }
}

/// `dialogue` with the guide's scene of `lessons` and `tips` in it: every
/// line a clip may belong to.
pub fn with_guide(
    mut dialogue: DialogueTable,
    lessons: &LessonTable,
    tips: &TipTable,
) -> DialogueTable {
    let scene = guide_scene(lessons, tips);
    dialogue.scenes.insert(scene.id.clone(), scene);
    dialogue
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Acceptance: the guide says every lesson's and tip's `text`, once
    /// each, and never an item or a `how` (0475: they are silent).
    #[test]
    fn the_guide_says_the_texts_and_never_the_items_or_how() {
        let content = crate::load_embedded().unwrap();
        let (lessons, tips) = (&content.lessons, &content.tips);
        let scene = guide_scene(lessons, tips);
        let lines = scene.lines();
        let ids: Vec<&LineId> = lines.iter().map(|l| l.id).collect();
        let texts = lessons.lessons.iter().map(|l| l.text.as_str());
        let texts: Vec<&str> = texts
            .chain(tips.tips.iter().map(|t| t.text.as_str()))
            .collect();
        assert_ne!(texts.len(), 0);
        for text in &texts {
            assert!(ids.contains(&&guide_line(text)), "{text}");
        }
        let mut unique = ids.clone();
        unique.sort();
        unique.dedup();
        assert_eq!(unique.len(), ids.len(), "each line once");
        let items = lessons.lessons.iter().flat_map(|l| &l.items);
        let silent: Vec<&str> = items
            .map(|i| i.label.as_str())
            .chain(tips.tips.iter().filter_map(|t| t.how.as_deref()))
            .collect();
        assert_ne!(silent.len(), 0);
        for line in &lines {
            assert_eq!(line.speaker, GUIDE_SPEAKER);
            assert!(texts.iter().any(|t| words(t) == line.text));
            for words in &silent {
                assert!(!line.text.contains(words), "{words} in {}", line.text);
            }
        }
        assert!(
            !content.dialogue.scenes.contains_key(GUIDE_SCENE),
            "no script scene takes the guide's id"
        );
    }

    /// A line's id is its words: rewording gives a new one, the box's
    /// line breaks don't.
    #[test]
    fn the_id_follows_the_words_not_the_line_breaks() {
        let id = guide_line("Pick up a unit.\nSend it on.");
        assert_eq!(id, guide_line("Pick up a unit. Send it on."));
        assert_ne!(id, guide_line("Pick up a unit.\nSend it off."));
        assert_eq!(id.scene_id(), GUIDE_SCENE);
    }
}
