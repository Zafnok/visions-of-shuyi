//! `@dies <character>`: a scene's death (ticket 0319,
//! `death-and-difficulty.md`, Hollis): when the scene plays, the character
//! leaves the army for good, in every mode
//! ([`Campaign::dies_in_scene`](trpg_core::Campaign::dies_in_scene)). It
//! goes at a scene's top level or in an `@if` block (so the death plays
//! only while the character is there), never in a reply's reaction: a
//! death never hangs on what the lead answers. The character must be a
//! named character, and never the lead.
use crate::prelude::*;

use trpg_core::{CharacterId, LEAD_ID};

use super::{Lines, ParsedScene, Scene, Stage, Step, is_id};
use crate::character::CharacterTable;
use crate::error::ContentError;

/// The characters `scene` kills, in order: its top-level `@dies` lines.
/// Run on a scene [resolved](Scene::resolved) for those present, so the
/// `@if` blocks that don't play are gone.
pub fn deaths(scene: &Scene) -> Vec<CharacterId> {
    scene
        .steps
        .iter()
        .filter_map(|step| match step {
            Step::Directive(Stage::Dies(character)) => Some(character.clone()),
            _ => None,
        })
        .collect()
}

/// One error for each `@dies` of `parsed` in a reply's reaction, of the
/// lead, or of someone who isn't a named character of `characters` (when
/// they loaded; a bad id has its own error).
pub(super) fn problems(
    parsed: &ParsedScene,
    characters: Option<&CharacterTable>,
) -> Vec<ContentError> {
    let mut errors = Vec::new();
    let mut err = |line: u32, message: String| {
        errors.push(ContentError::new(&parsed.file, message).at(line, None));
    };
    walk(
        &parsed.scene.steps,
        &parsed.lines,
        false,
        &mut |line, c, reply| {
            if reply {
                err(
                    line,
                    "a @dies goes at a scene's top level or in an @if block, not in a reply's \
                 reaction"
                        .to_owned(),
                );
            }
            if c.0 == LEAD_ID {
                err(line, "the lead never dies in a scene".to_owned());
            } else if is_id(&c.0) && characters.is_some_and(|t| !t.characters.contains_key(c)) {
                err(line, format!("@dies: no character \"{}\"", c.0));
            }
        },
    );
    errors
}

/// Calls `found` with each `@dies` of `steps` (written at `lines`): its
/// line, its character and whether it is in a reply's reaction (`reply`).
fn walk(
    steps: &[Step],
    lines: &Lines,
    reply: bool,
    found: &mut impl FnMut(u32, &CharacterId, bool),
) {
    let mut blocks = lines.blocks.iter();
    for (step, &line) in steps.iter().zip(&lines.steps) {
        let (parts, in_reply): (Vec<&[Step]>, bool) = match step {
            Step::Choice { options } => (options.iter().map(|o| &o.steps[..]).collect(), true),
            Step::If {
                then, otherwise, ..
            } => (vec![then, otherwise], reply),
            Step::Directive(Stage::Dies(c)) => {
                found(line, c, reply);
                continue;
            }
            _ => continue,
        };
        if let Some(block) = blocks.next() {
            for (part, at) in parts.into_iter().zip(block) {
                walk(part, &at.lines, in_reply, found);
            }
        }
    }
}
