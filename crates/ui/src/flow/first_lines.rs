//! A scene's first voice lines, loaded before it starts (ticket 1293).
//!
//! The dialogue screen asks for a box's clip the frame the box comes up,
//! and a clip not loaded 0.3 s after it was asked for is skipped: on the
//! web, where it is fetched, the first line of a scene was silent. So
//! whoever knows which scene comes next keeps its first lines loaded
//! ([`Ctx::keep_voices`]) a screen ahead: the title New Game's opening
//! scene, the Load Game list each save's too, and the flow, as it shows a
//! screen, the scene it shows and the scene it knows follows.
//! [`KeptVoices`] sends a set; [`opening_scene`] finds the scene a game
//! starts with.

use crate::prelude::*;

use trpg_content::{LineId, Present, new_campaign};
use trpg_core::{Campaign, GameMode, SaveFile, SavePoint};

use crate::screen::Ctx;
use crate::screens::DialogueScreen;

/// The lines one screen keeps loaded, as last sent.
#[derive(Debug, Clone, Default)]
pub struct KeptVoices {
    /// The voiced lines last handed to [`Ctx::keep_voices`].
    kept: Vec<LineId>,
    /// Whether a set was sent since the screen came on top.
    sent: bool,
}

impl KeptVoices {
    /// Whether any line can be kept: voices are on and clips were found.
    /// Without, finding the lines is skipped (the Playdate has none: the
    /// title's first frame is in its frame budget, `cargo shuyi bench`).
    pub fn wanted(ctx: &Ctx) -> bool {
        ctx.settings().voices && ctx.has_voices()
    }

    /// Whether a screen whose set doesn't change while it is on top
    /// should send it now: not yet since it came on top, and
    /// [`wanted`](Self::wanted).
    pub fn due(&self, ctx: &Ctx) -> bool {
        !self.sent && Self::wanted(ctx)
    }

    /// A screen came over: the set is sent again once back on top (that
    /// screen may have replaced it).
    pub fn covered(&mut self) {
        self.sent = false;
    }

    /// Keeps `lines` loaded: those with a clip, sent if they differ from
    /// the set sent last, or, any at all, on the first call on top. With
    /// voices off nothing is sent, and the set is due again.
    pub fn keep(&mut self, mut lines: Vec<LineId>, ctx: &mut Ctx) {
        if !ctx.settings().voices {
            self.sent = false;
            return;
        }
        lines.retain(|l| ctx.has_voice(l));
        let mut seen = Vec::with_capacity(lines.len());
        lines.retain(|l| {
            let new = !seen.contains(l);
            seen.push(l.clone());
            new
        });
        if (!self.sent && !lines.is_empty()) || lines != self.kept {
            ctx.keep_voices(&lines);
            self.kept = lines;
        }
        self.sent = true;
    }
}

/// How a game starts from the title.
#[derive(Debug, Clone, Copy)]
pub enum Start<'a> {
    /// New Game.
    NewGame,
    /// Loading a save.
    Save(&'a SaveFile),
}

/// The scene the game `start` plays first, if it starts with one: New
/// Game's first chapter's, a chapter save's next chapter's (after camp,
/// if camp opens first). A world-map save goes on on the map, and a
/// chapter that has none, or a save of nothing after it, plays none.
pub fn opening_scene(ctx: &Ctx, start: Start<'_>) -> Option<DialogueScreen> {
    match start {
        Start::NewGame => {
            let content = &ctx.game.content;
            let campaign = new_campaign(content, GameMode::Classic, ctx.game.lead.clone());
            chapter_opening(ctx, Some(&campaign), &campaign.chapter)
        }
        Start::Save(file) if file.point == SavePoint::ChapterCleared => {
            let campaign = &file.campaign;
            let cleared = ctx.game.content.chapters.get(&campaign.chapter)?;
            chapter_opening(ctx, Some(campaign), cleared.next.as_deref()?)
        }
        Start::Save(_) => None,
    }
}

/// The first lines of the scene the game `start` plays first
/// ([`opening_scene`]); none if it plays none.
pub fn opening_lines(ctx: &Ctx, start: Start<'_>) -> Vec<LineId> {
    opening_scene(ctx, start).map_or_else(Vec::new, |s| s.first_lines())
}

/// The first lines of the intro scene of every battle the army of
/// `campaign`, on a world map, may start with its next move (1296): at its
/// own place (Enter), or where one Travel ends, as the rules route it
/// (`WorldMap::route`). `FlowScreen::arrived` plays it the frame the map
/// pops, so the map keeps them while it is on top.
pub fn world_battle_lines(ctx: &Ctx, campaign: &Campaign) -> Vec<LineId> {
    let content = &ctx.game.content;
    let Some(world) = &campaign.world else {
        return Vec::new();
    };
    let Some(def) = content.worldmaps.get(&world.map) else {
        return Vec::new();
    };
    let map = &def.map;
    let mut ends: Vec<&str> = vec![world.at.as_str()];
    for node in &map.nodes {
        if node.id == world.at || !map.is_open(&node.id, campaign) {
            continue;
        }
        if let Some(end) = map
            .route(&world.at, &node.id, campaign)
            .and_then(|r| r.last().cloned())
            && let Some(node) = map.node(&end)
            && !ends.contains(&node.id.as_str())
        {
            ends.push(node.id.as_str());
        }
    }
    let mut lines = Vec::new();
    for end in ends {
        let mut there = campaign.clone();
        if let Some(w) = &mut there.world {
            end.clone_into(&mut w.at);
        }
        if let Some((_, chapter)) = super::world_fight(ctx, &there)
            && let Some(scene) = first_told(ctx, Some(&there), &chapter.intro_scenes)
        {
            lines.extend(scene.first_lines());
        }
    }
    lines
}

/// The scene chapter `id` starts with for `campaign`'s army, as
/// `FlowScreen::start_chapter` plays it: the first of its intro scenes
/// with anything to say, or of its opening scenes on a world map.
pub(super) fn chapter_opening(
    ctx: &Ctx,
    campaign: Option<&Campaign>,
    id: &str,
) -> Option<DialogueScreen> {
    let content = &ctx.game.content;
    let chapter = content.chapters.get(id)?;
    let on_map = chapter.world_map.as_ref();
    let scenes = if on_map.is_some_and(|m| content.worldmaps.contains_key(m)) {
        &chapter.opening_scenes
    } else {
        &chapter.intro_scenes
    };
    first_told(ctx, campaign, scenes)
}

/// The first of `scenes` with anything to say for `campaign`'s army, as
/// it would be told.
pub(super) fn first_told<'s>(
    ctx: &Ctx,
    campaign: Option<&Campaign>,
    scenes: impl IntoIterator<Item = &'s String>,
) -> Option<DialogueScreen> {
    scenes
        .into_iter()
        .filter_map(|id| ctx.game.content.dialogue.get(id))
        .map(|scene| {
            let lead = campaign.map_or_else(|| ctx.game.lead.clone(), |c| c.lead.clone());
            DialogueScreen::told(ctx, scene, lead, &present(campaign))
        })
        .find(DialogueScreen::has_text)
}

/// Who is there for `campaign`'s scenes: its roster's characters
/// (ADR-0055); everyone, with no campaign yet.
pub(super) fn present(campaign: Option<&Campaign>) -> Present {
    campaign.map_or(Present::Everyone, |c| {
        Present::only(c.roster.iter().filter_map(|u| u.character.clone()))
    })
}

#[cfg(test)]
mod tests;
