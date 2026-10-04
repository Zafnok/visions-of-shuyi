//! Who is there when a scene plays (ADR-0055). An `@if` block's steps play
//! only for a character who is [`Present`]; [`Scene::resolved`] gives the
//! scene as it plays for those present. [`check_presence`] keeps a script
//! from showing someone who may be gone: a character the army can lose
//! ([`Cast::may_be_absent`]) appears only inside an `@if` block for them,
//! or in a scene that can't play without them ([`Cast::certain`]).

use std::collections::{BTreeMap, BTreeSet};

use trpg_core::{BattleDef, CharacterId, LEAD_ID, Phase, SupportTable, TriggerWhen, Who};

use super::parse::{Lines, ParsedScene};
use super::{ChoiceOption, Scene, Step};
use crate::chapter::{ChapterDef, NewGameDef};
use crate::error::ContentError;

/// Who is there when a scene plays.
///
/// - Between battles (a chapter's scenes): the characters of the
///   campaign's roster. A unit that died in Classic has left it; one that
///   retreated in Casual is still in it.
/// - During a battle (a trigger's scene): the characters of the units on
///   the map at that moment, on any side.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Present {
    /// Everyone: every `@if` block plays its first part (the debug scene
    /// viewer; a scene played outside a campaign).
    Everyone,
    /// Only these characters.
    Only(BTreeSet<CharacterId>),
}

impl Present {
    /// Only `characters`.
    pub fn only(characters: impl IntoIterator<Item = CharacterId>) -> Self {
        Self::Only(characters.into_iter().collect())
    }

    /// Whether `character` is there.
    pub fn has(&self, character: &CharacterId) -> bool {
        match self {
            Present::Everyone => true,
            Present::Only(characters) => characters.contains(character),
        }
    }
}

impl Scene {
    /// The scene as it plays for those `present`: every `@if` block
    /// replaced by the steps of the part that holds, in reactions and in
    /// other blocks too. The lines keep their ids.
    #[must_use]
    pub fn resolved(&self, present: &Present) -> Scene {
        Scene {
            id: self.id.clone(),
            steps: resolved(&self.steps, present),
        }
    }

    /// Whether the scene has a speech or narration line to play, whichever
    /// reply is picked. Run on a [`Scene::resolved`] scene: one with
    /// nothing left to say for those present isn't played.
    pub fn has_text(&self) -> bool {
        self.steps.iter().any(Step::has_text)
    }
}

/// `steps` with every `@if` block replaced by the part that holds for
/// those `present`.
fn resolved(steps: &[Step], present: &Present) -> Vec<Step> {
    let mut out = Vec::with_capacity(steps.len());
    for step in steps {
        match step {
            Step::If {
                character,
                then,
                otherwise,
            } => {
                let part = if present.has(character) {
                    then
                } else {
                    otherwise
                };
                out.extend(resolved(part, present));
            }
            Step::Choice { options } => {
                let option = |o: &ChoiceOption| ChoiceOption {
                    steps: resolved(&o.steps, present),
                    ..o.clone()
                };
                out.push(Step::Choice {
                    options: options.iter().map(option).collect(),
                });
            }
            other => out.push(other.clone()),
        }
    }
    out
}

/// What [`check_presence`] knows about who can be missing from a scene.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Cast {
    /// The characters the army can lose, so that a scene may play without
    /// them: the starting roster but the lead, and everyone a battle can
    /// recruit.
    pub may_be_absent: BTreeSet<CharacterId>,
    /// By scene id, the characters who are there whenever the scene plays:
    ///
    /// - those its battle trigger is about (the unit that enters, fights,
    ///   is hurt, talks or falls: a falling unit hears its own last
    ///   words);
    /// - the pair of its support conversation;
    /// - before anyone can have fallen, the starting roster: in the first
    ///   chapter's intro scenes, and (those of them the battle places) in
    ///   the scenes its battle plays as turn 1's player phase starts.
    ///
    /// A scene played from several places keeps only those every one of
    /// them has; any other scene a chapter plays keeps nobody.
    pub certain: BTreeMap<String, BTreeSet<CharacterId>>,
}

impl Cast {
    /// The cast of a game that starts as `new_game` says and has these
    /// `battles`, `chapters` and `supports`.
    pub fn new(
        new_game: &NewGameDef,
        battles: &BTreeMap<String, BattleDef>,
        chapters: &BTreeMap<String, ChapterDef>,
        supports: &SupportTable,
    ) -> Self {
        let mut cast = Cast::default();
        let roster: BTreeSet<CharacterId> = new_game.roster.iter().cloned().collect();
        let army = roster.iter().filter(|c| c.0 != LEAD_ID);
        cast.may_be_absent.extend(army.cloned());
        let first = chapters.get(&new_game.first_chapter);
        for (id, battle) in battles {
            // Who the first battle's opening scenes can count on.
            let placed = battle.player_slots.iter().map(|s| &s.character);
            let opening: BTreeSet<CharacterId> = placed
                .filter(|c| first.is_some_and(|f| f.battle == *id) && roster.contains(c))
                .cloned()
                .collect();
            for trigger in &battle.triggers {
                let mut there = characters(&trigger.when);
                match &trigger.when {
                    TriggerWhen::UnitFell {
                        unit,
                        recruit: true,
                        ..
                    } => {
                        cast.may_be_absent.insert(unit.clone());
                    }
                    TriggerWhen::TurnStart {
                        turn: 1,
                        phase: Phase::Player,
                    } => there.extend(opening.iter().cloned()),
                    _ => {}
                }
                cast.played(&trigger.scene, there);
            }
        }
        for (id, chapter) in chapters {
            for scene in &chapter.intro_scenes {
                let is_first = *id == new_game.first_chapter;
                let there = if is_first {
                    roster.clone()
                } else {
                    BTreeSet::new()
                };
                cast.played(scene, there);
            }
            for scene in &chapter.victory_scenes {
                cast.played(scene, BTreeSet::new());
            }
        }
        for def in supports.pairs() {
            let c = &def.conversations;
            for scene in [&c.c, &c.b, &c.a] {
                cast.played(scene, def.pair.members().into_iter().cloned().collect());
            }
        }
        cast
    }

    /// Records that `scene` is played somewhere `there` are certain to be.
    pub(super) fn played(&mut self, scene: &str, there: BTreeSet<CharacterId>) {
        match self.certain.get_mut(scene) {
            Some(certain) => certain.retain(|c| there.contains(c)),
            None => {
                self.certain.insert(scene.to_owned(), there);
            }
        }
    }
}

/// The characters a trigger is about: on the map when its scene plays.
fn characters(when: &TriggerWhen) -> BTreeSet<CharacterId> {
    let about: Vec<&CharacterId> = match when {
        TriggerWhen::TurnStart { .. } => vec![],
        TriggerWhen::UnitEntersArea { who, .. } => match who {
            Who::Character(c) => vec![c],
            Who::Faction(_) => vec![],
        },
        TriggerWhen::CombatStart { unit, against } => {
            std::iter::once(unit).chain(against).collect()
        }
        TriggerWhen::UnitFell { unit, .. } | TriggerWhen::HalfHp { unit } => vec![unit],
        TriggerWhen::Talk { a, b } => vec![a, b],
    };
    about.into_iter().cloned().collect()
}

/// Checks that scene `parsed` never shows someone who may be gone. One
/// error for each line that:
///
/// - places, or gives a line to, a character who [`Cast::may_be_absent`]
///   outside an `@if` block for them, unless they are [`Cast::certain`]
///   for the scene;
/// - places, or gives a line to, a character in the `@else` of their own
///   `@if` block;
/// - opens an `@if` block for a character already known to be there or
///   gone at that line (the lead always is there), which could only ever
///   play one of its parts.
pub fn check_presence(parsed: &ParsedScene, cast: &Cast) -> Vec<ContentError> {
    let lead = CharacterId(LEAD_ID.to_owned());
    let certain = cast.certain.get(&parsed.scene.id);
    let mut walk = Walk {
        file: &parsed.file,
        cast,
        here: std::iter::once(&lead)
            .chain(certain.into_iter().flatten())
            .collect(),
        gone: Vec::new(),
        errors: Vec::new(),
    };
    walk.steps(&parsed.scene.steps, &parsed.lines);
    walk.errors
}

/// A walk through a scene's steps.
struct Walk<'a> {
    file: &'a str,
    cast: &'a Cast,
    /// Who is known to be there at the step reached.
    here: Vec<&'a CharacterId>,
    /// Who is known to be gone there.
    gone: Vec<&'a CharacterId>,
    errors: Vec<ContentError>,
}

impl<'a> Walk<'a> {
    fn err(&mut self, line: u32, message: String) {
        self.errors
            .push(ContentError::new(self.file, message).at(line, None));
    }

    fn steps(&mut self, steps: &'a [Step], lines: &Lines) {
        let mut blocks = lines.blocks.iter();
        for (step, &line) in steps.iter().zip(&lines.steps) {
            match step {
                Step::Place { character, .. } => self.appears(character, line),
                Step::Say { speaker, .. } => self.appears(speaker, line),
                Step::Choice { options } => {
                    let parts = blocks.next().into_iter().flatten();
                    for (option, part) in options.iter().zip(parts) {
                        self.steps(&option.steps, &part.lines);
                    }
                }
                Step::If {
                    character,
                    then,
                    otherwise,
                } => {
                    self.settled(character, line);
                    let Some([with, without]) = blocks.next().map(Vec::as_slice) else {
                        continue;
                    };
                    self.here.push(character);
                    self.steps(then, &with.lines);
                    self.here.pop();
                    self.gone.push(character);
                    self.steps(otherwise, &without.lines);
                    self.gone.pop();
                }
                Step::Caption { .. }
                | Step::Clear { .. }
                | Step::Narrate { .. }
                | Step::Music(_) => {}
            }
        }
    }

    /// Reports an `@if` block for `character` on `line` where it is
    /// already known whether they are there.
    fn settled(&mut self, character: &CharacterId, line: u32) {
        let id = &character.0;
        if self.gone.contains(&character) {
            self.err(
                line,
                format!(
                    "\"{id}\" is known to be gone here (this is inside the @else of an \
                     \"@if {id}\"); this block's first part would never play"
                ),
            );
        } else if self.here.contains(&character) {
            let why = if id == LEAD_ID {
                "the lead is in every scene's army"
            } else {
                "this is inside an \"@if\" for them, or the scene can't play without them"
            };
            self.err(
                line,
                format!(
                    "\"{id}\" is always here at this line ({why}); this block's @else would \
                     never play"
                ),
            );
        }
    }

    /// Reports `character` placed or speaking on `line` where they may be,
    /// or are, gone.
    fn appears(&mut self, character: &CharacterId, line: u32) {
        let id = &character.0;
        if self.gone.contains(&character) {
            self.err(
                line,
                format!("\"{id}\" is gone here: this is the @else of \"@if {id}\""),
            );
        } else if self.cast.may_be_absent.contains(character) && !self.here.contains(&character) {
            self.err(
                line,
                format!(
                    "\"{id}\" may have fallen or left the army by now; put their lines inside \
                     \"@if {id}\" ... \"@endif\" (assets/dialogue/README.md, \"Who is still \
                     there\")"
                ),
            );
        }
    }
}
