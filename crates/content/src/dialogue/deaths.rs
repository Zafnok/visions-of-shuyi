//! `@dies <character>`: a scene's death (ticket 0319,
//! `death-and-difficulty.md`, Hollis): when the scene plays, the character
//! leaves the army for good, in every mode
//! ([`Campaign::dies_in_scene`](trpg_core::Campaign::dies_in_scene)).
//! `@joins <character>`: a scene's recruit (ticket 0848,
//! `battle-scenes-and-recruitment.md`, *In a scene, on a story beat*, Joss):
//! when the scene plays, the character joins the army
//! ([`Campaign::joins_in_scene`](trpg_core::Campaign::joins_in_scene)).
//!
//! Both go at a scene's top level or in an `@if` block (so the change
//! plays only while someone is there), never in a reply's reaction: the
//! army never hangs on what the lead answers. The character must be a
//! named character (a unit, not a speaker), and never the lead.
use crate::prelude::*;

use trpg_core::{CharacterId, LEAD_ID};

use super::{Lines, ParsedScene, Scene, Stage, Step, is_id};
use crate::character::CharacterTable;
use crate::error::ContentError;

/// The characters `scene` kills, in order: its top-level `@dies` lines.
/// Run on a scene [resolved](Scene::resolved) for those present, so the
/// `@if` blocks that don't play are gone.
pub fn deaths(scene: &Scene) -> Vec<CharacterId> {
    top_level(scene, |stage| match stage {
        Stage::Dies(character) => Some(character),
        _ => None,
    })
}

/// The characters `scene` recruits, in order: its top-level `@joins`
/// lines, on a scene [resolved](Scene::resolved) for those present.
pub fn joins(scene: &Scene) -> Vec<CharacterId> {
    top_level(scene, |stage| match stage {
        Stage::Joins(character) => Some(character),
        _ => None,
    })
}

/// The characters of `scene`'s top-level directives that `pick` takes.
fn top_level(scene: &Scene, pick: fn(&Stage) -> Option<&CharacterId>) -> Vec<CharacterId> {
    scene
        .steps
        .iter()
        .filter_map(|step| match step {
            Step::Directive(stage) => pick(stage).cloned(),
            _ => None,
        })
        .collect()
}

/// The characters every `@joins` of `parsed` names, in or out of `@if`
/// blocks: who the scene may recruit.
pub fn joined(parsed: &ParsedScene) -> Vec<CharacterId> {
    let mut found = Vec::new();
    walk(
        &parsed.scene.steps,
        &parsed.lines,
        false,
        &mut |_, (name, c), _| {
            if name == "joins" {
                found.push(c.clone());
            }
        },
    );
    found
}

/// One error for each `@dies` or `@joins` of `parsed` in a reply's
/// reaction, of the lead, or of someone who isn't a named character of
/// `characters` (when they loaded; a bad id has its own error).
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
        &mut |line, (name, c), reply| {
            if reply {
                err(
                    line,
                    format!(
                        "a @{name} goes at a scene's top level or in an @if block, not in a \
                         reply's reaction"
                    ),
                );
            }
            if c.0 == LEAD_ID {
                err(line, format!("the lead never {name} in a scene"));
            } else if is_id(&c.0) && characters.is_some_and(|t| !t.characters.contains_key(c)) {
                err(line, format!("@{name}: no character \"{}\"", c.0));
            }
        },
    );
    errors
}

/// Calls `found` with each `@dies` or `@joins` of `steps` (written at
/// `lines`): its line, its directive's name and character, and whether it
/// is in a reply's reaction (`reply`).
fn walk(
    steps: &[Step],
    lines: &Lines,
    reply: bool,
    found: &mut impl FnMut(u32, (&str, &CharacterId), bool),
) {
    let mut blocks = lines.blocks.iter();
    for (step, &line) in steps.iter().zip(&lines.steps) {
        let (parts, in_reply): (Vec<&[Step]>, bool) = match step {
            Step::Choice { options } => (options.iter().map(|o| &o.steps[..]).collect(), true),
            Step::If {
                then, otherwise, ..
            } => (vec![then, otherwise], reply),
            Step::Directive(Stage::Dies(c)) => {
                found(line, ("dies", c), reply);
                continue;
            }
            Step::Directive(Stage::Joins(c)) => {
                found(line, ("joins", c), reply);
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
