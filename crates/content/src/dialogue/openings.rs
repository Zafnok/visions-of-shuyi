//! Where an `@opening` may go (ticket 0734): at a scene's top level, on a
//! place from `assets/maps/`, and with something said after it before the
//! next one and before the scene ends. (Its file, and that it is on that
//! map: [`super::check_places`].)
use crate::prelude::*;

use super::{Lines, ParsedScene, PlaceLine, Stage, Step, setting};
use crate::error::ContentError;

/// One error for each `@opening` of `parsed` that is in a block, plays on
/// no map, or has nothing said after it.
pub(super) fn problems(parsed: &ParsedScene) -> Vec<ContentError> {
    let mut walk = Walk {
        file: &parsed.file,
        place: None,
        waiting: None,
        errors: Vec::new(),
    };
    walk.steps(&parsed.scene.steps, &parsed.lines, 0);
    if let Some(line) = walk.waiting {
        walk.err(
            line,
            "nothing is said after this @opening; it plays before the next speech or \
             narration line"
                .to_owned(),
        );
    }
    walk.errors
}

/// A walk through a scene's steps, in script order: a reply's reaction,
/// then the next reply; an `@if` block's steps, then its `@else`'s.
struct Walk<'a> {
    file: &'a str,
    /// The place in force at the top-level step reached.
    place: Option<&'a PlaceLine>,
    /// The line of the last `@opening`, until a speech or narration line
    /// follows it.
    waiting: Option<u32>,
    errors: Vec<ContentError>,
}

impl<'a> Walk<'a> {
    fn err(&mut self, line: u32, message: String) {
        self.errors
            .push(ContentError::new(self.file, message).at(line, None));
    }

    /// Walks `steps`, written at `lines`, `depth` blocks deep (0 at the
    /// scene's top level).
    fn steps(&mut self, steps: &'a [Step], lines: &Lines, depth: usize) {
        let mut blocks = lines.blocks.iter();
        for (step, &line) in steps.iter().zip(&lines.steps) {
            let parts: Vec<&'a [Step]> = match step {
                Step::Choice { options } => options.iter().map(|o| &o.steps[..]).collect(),
                Step::If {
                    then, otherwise, ..
                } => vec![then, otherwise],
                Step::Directive(Stage::Opening(_)) => {
                    self.opening(line, depth);
                    continue;
                }
                _ => {
                    if step.text().is_some() {
                        self.waiting = None;
                    }
                    if depth == 0 {
                        self.place = setting::last_setting(step).or(self.place);
                    }
                    continue;
                }
            };
            if let Some(block) = blocks.next() {
                for (part, at) in parts.into_iter().zip(block) {
                    self.steps(part, &at.lines, depth + 1);
                }
            }
            if depth == 0 {
                self.place = setting::last_setting(step).or(self.place);
            }
        }
    }

    /// Checks the `@opening` on `line`, `depth` blocks deep.
    fn opening(&mut self, line: u32, depth: usize) {
        if depth > 0 {
            self.err(
                line,
                "an @opening goes at a scene's top level, not in a reply's reaction or an @if \
                 block"
                    .to_owned(),
            );
            return;
        }
        if !matches!(self.place, Some(PlaceLine::Map { .. })) {
            self.err(
                line,
                "an @opening plays on a place from assets/maps/: put a \"@place <map> <x>,<y>\" \
                 line before it (not \"@place battle\" or \"@place none\")"
                    .to_owned(),
            );
        }
        if let Some(before) = self.waiting.replace(line) {
            self.err(
                line,
                format!(
                    "nothing is said between the @opening on line {before} and this one; an \
                     opening plays before the next speech or narration line"
                ),
            );
        }
    }
}
