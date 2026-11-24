//! The first voice lines of the scenes the battle may still play over
//! itself (ticket 1296): those its triggers fired that wait their turn
//! (in a combat's playback or the queue), and those its triggers may yet
//! fire. They are kept loaded beside the guide's ([`GuideVoice`]): the
//! dialogue screen asks for a box's clip the frame it comes up, and on
//! the web a clip not loaded a moment later is skipped (`docs/voice.md`).
//!
//! A trigger may yet fire unless it is a `once` one that fired, a turn's
//! that has passed, one for another mode, or one about a character not on
//! the map and not among the reinforcements to come. Each scene is told
//! as [`BattleScreen::next_scene`] tells it: a fired one for those on the
//! map when it fired, one to come for those on the map now.
//!
//! [`GuideVoice`]: crate::tips::voice::GuideVoice

use crate::prelude::*;
use alloc::collections::BTreeSet;

use trpg_content::{LineId, Present};
use trpg_core::{CharacterId, Trigger, TriggerWhen};

use super::BattleScreen;
use super::Queued;
use super::mode::Mode;
use super::playback::SceneCue;
use crate::screen::Ctx;
use crate::screens::DialogueScreen;

impl BattleScreen {
    /// The first lines of every scene the battle may still play, those
    /// already fired first. Empty without voice clips (the Playdate).
    pub(super) fn coming_scene_lines(&self, ctx: &Ctx) -> Vec<LineId> {
        if !ctx.has_voices() {
            return Vec::new();
        }
        let here: BTreeSet<CharacterId> = self
            .state
            .units()
            .iter()
            .filter_map(|u| u.character.clone())
            .collect();
        let fired = self.waiting_cues().cloned();
        let to_come = self
            .state
            .triggers()
            .iter()
            .enumerate()
            .filter(|&(i, t)| self.may_fire(i, t, &here))
            .map(|(_, t)| SceneCue {
                id: t.scene.clone(),
                present: here.clone(),
            });
        let mut told: Vec<SceneCue> = Vec::new();
        let mut lines = Vec::new();
        for cue in fired.chain(to_come) {
            if told.contains(&cue) {
                continue;
            }
            if let Some(scene) = ctx.game.content.dialogue.get(&cue.id) {
                let present = Present::Only(cue.present.clone());
                let lead = ctx.game.lead.clone();
                lines
                    .extend(DialogueScreen::told_overlay(ctx, scene, lead, &present).first_lines());
            }
            told.push(cue);
        }
        lines
    }

    /// The scenes fired and not played yet: the combat's still to come,
    /// then the queue's.
    fn waiting_cues(&self) -> impl Iterator<Item = &SceneCue> {
        let playing = match &self.mode {
            Mode::Combat(pb) => pb.waiting_scenes(),
            _ => &[],
        };
        let queued = self.queue.iter().filter_map(|q| match q {
            Queued::Scene(cue) => Some(cue),
            _ => None,
        });
        playing.iter().chain(queued)
    }

    /// Whether trigger `index` may still fire, with `here` the characters
    /// on the map.
    fn may_fire(&self, index: usize, trigger: &Trigger, here: &BTreeSet<CharacterId>) -> bool {
        let state = &self.state;
        let may_be_here = |c: &CharacterId| {
            here.contains(c)
                || state
                    .reinforcements()
                    .iter()
                    .any(|r| r.unit.character.as_ref() == Some(c))
        };
        !state.has_fired(index)
            && match &trigger.when {
                TriggerWhen::TurnStart { turn, .. } => *turn >= state.turn(),
                TriggerWhen::UnitEntersArea { .. } => true,
                TriggerWhen::CombatStart { unit, .. } | TriggerWhen::HalfHp { unit } => {
                    may_be_here(unit)
                }
                TriggerWhen::UnitFell { unit, mode, .. } => {
                    mode.is_none_or(|m| m == state.mode()) && may_be_here(unit)
                }
                TriggerWhen::Talk { a, b } => may_be_here(a) && may_be_here(b),
            }
    }
}
