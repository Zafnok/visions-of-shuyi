//! The places' sound beds under the music (ticket 1332,
//! `docs/design/audio.md`, *Ambience*): one bed at a time, shared by every
//! screen ([`Shuyi::ambience`](crate::screen::Shuyi::ambience)), so a scene
//! over its own battle goes on hearing the battle's bed instead of
//! starting a second one.
//!
//! A screen with a place (the battle map, a scene's place, the world map
//! and its towns, camp) asks for its bed every frame with [`play`]: the
//! bed starts at silence and fades in over [`FADE_S`]; another bed fades
//! the one playing out and starts; none fades it out. `hushed` (a fight's
//! stage on screen) fades it to silence and back while its loop goes on,
//! so it comes back from where it is. A screen that leaves the stack calls
//! [`leave`]: the bed it started fades out with it. A screen with no place
//! (a menu, Options) asks for nothing, and the bed goes on under it.
//!
//! How loud it plays is its sound group's, [`AMBIENCE`], with its own
//! volume on Options; at 0 no bed starts. It dips under a voiced line by
//! the manifest's `ducks`, as the music does. The rules never hear it.

use crate::prelude::*;

use trpg_content::ambience::AMBIENCE;

use crate::audio::SoundId;
use crate::screen::Ctx;

/// Seconds a bed takes to fade in or out: its screen's music's fade
/// (`audio.md`, *Starting values*).
pub const FADE_S: f32 = 0.5;

/// [`FADE_S`] in milliseconds, for a stop.
const FADE_MS: u32 = 500;

/// The bed playing, if any.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct AmbiencePlayer {
    playing: Option<Bed>,
    /// Whether it plays no bed at all: the Playdate's, until the bench
    /// says beds run well there (`audio.md`, *Ambience*).
    off: bool,
}

/// A bed playing.
#[derive(Debug, Clone, PartialEq)]
struct Bed {
    /// Its sound cue.
    cue: String,
    /// Its handle.
    id: SoundId,
    /// Its volume now, 0–1, on its way to full or, hushed, to silence.
    level: f32,
    /// The screen that started it, by name: the one whose [`leave`] stops
    /// it.
    owner: &'static str,
}

impl AmbiencePlayer {
    /// A player that never plays a bed.
    pub fn off() -> Self {
        Self {
            playing: None,
            off: true,
        }
    }

    /// The sound cue of the bed playing, if any.
    pub fn cue(&self) -> Option<&str> {
        self.playing.as_ref().map(|b| b.cue.as_str())
    }
}

/// The bed of ambience `ambience` in light `light` in `ctx`'s content, if
/// it has one (a light without its own plays the noon bed).
pub fn bed(ctx: &Ctx, ambience: Option<&str>, light: &str) -> Option<String> {
    let table = &ctx.game.content.ambience;
    table.bed(ambience?, light).map(str::to_owned)
}

/// Plays bed `cue` for screen `owner` this frame, `dt` seconds after the
/// last: hushed to silence while `hushed`; `None` (or the Ambience volume
/// at 0) fades the bed playing out.
pub fn play(ctx: &mut Ctx, owner: &'static str, cue: Option<&str>, hushed: bool, dt: f32) {
    let on = !ctx.game.ambience.off && ctx.volume(AMBIENCE) > 0;
    let cue = cue.filter(|_| on);
    let (player, audio) = (&mut ctx.game.ambience, &mut ctx.audio);
    // A bed whose screen left the stack was stopped with it.
    player.playing = player.playing.take().filter(|b| audio.sound_is_live(b.id));
    if player.cue() != cue {
        if let Some(old) = player.playing.take() {
            audio.stop_sound(old.id, FADE_MS);
        }
        player.playing = cue.map(|cue| Bed {
            cue: cue.to_owned(),
            id: audio.start_sound_at(cue, 0.0),
            level: 0.0,
            owner,
        });
        return;
    }
    let Some(bed) = &mut player.playing else {
        return;
    };
    let step = if dt.is_finite() {
        dt.max(0.0) / FADE_S
    } else {
        0.0
    };
    // Down to silence while hushed, else up to full.
    let level = if hushed {
        (bed.level - step).max(0.0)
    } else {
        (bed.level + step).min(1.0)
    };
    if level.total_cmp(&bed.level).is_ne() {
        bed.level = level;
        audio.set_sound_volume(bed.id, level);
    }
}

/// Fades out the bed screen `owner` started, as it leaves the stack; a
/// bed another screen started goes on.
pub fn leave(ctx: &mut Ctx, owner: &'static str) {
    let player = &mut ctx.game.ambience;
    if player.playing.as_ref().is_some_and(|b| b.owner == owner)
        && let Some(bed) = player.playing.take()
    {
        ctx.audio.stop_sound(bed.id, FADE_MS);
    }
}

#[cfg(test)]
mod tests;
