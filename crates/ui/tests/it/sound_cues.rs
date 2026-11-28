//! The engine's cue rules on the game's sounds (ticket 1349, engine
//! ADR-0139): a cue asked for several times in one frame or the next
//! sounds once, and two footsteps in a row never play the same file.
//! `Harness::sounds` lists every play the game asked for;
//! `Harness::dropped_sounds` the ones the rules dropped.

use shuyi_assets::keymap::Chord;
use shuyi_platform::audio::SoundRequest;
use shuyi_platform::audio::cues::DropWhy;
use trpg_core::Pos;
use trpg_ui::RawInputEvent;
use trpg_ui::audio::AudioRequest;
use trpg_ui::harness::{FRAME_DT, Harness};

/// The Quick Battle from the title, its banners closed: the cursor on the
/// lord at (3, 5) of a map 14 tiles wide.
fn quick_battle() -> Harness {
    let mut h = crate::support::quick_battle();
    assert_eq!(h.cursor_tile(), Some(Pos::new(3, 5)));
    h.clear_audio();
    h
}

/// The frames that asked for `cue` since the audio was last cleared, one
/// entry per play asked for.
fn asked(h: &Harness, cue: &str) -> Vec<u32> {
    let log = h.audio_log();
    let plays = log.iter().filter(|(_, r)| r.cue() == Some(cue));
    plays.map(|(frame, _)| *frame).collect()
}

/// The frames of `cue`'s plays the cue rules dropped, each for its gap.
fn dropped(h: &Harness, cue: &str) -> Vec<u32> {
    let drops = h.dropped_sounds();
    let of_cue = drops.iter().filter(|(_, d)| d.cue == cue);
    of_cue
        .map(|(frame, d)| {
            assert_eq!(d.why, DropWhy::Gap, "{cue} at frame {frame}");
            *frame
        })
        .collect()
}

/// A held arrow key repeats every 55 ms (`assets/data/keymap.ron`), so at
/// 60 frames a second its ticks are three or four frames apart: each tile
/// the cursor moves ticks, and none is dropped. A frame that moves it
/// several tiles (two arrows at once, or a late frame that is owed several
/// repeats) ticks once, and `dropped_sounds` lists the rest.
#[test]
fn a_held_cursor_key_ticks_once_a_step_and_a_stack_sounds_once() {
    let mut h = quick_battle();
    h.hold("Right", 0.6);
    let moved = h.cursor_tile().map(|p| p.x - 3);
    let ticks = asked(&h, "cursor_move");
    assert_eq!(
        Some(ticks.len()),
        moved.and_then(|m| usize::try_from(m).ok())
    );
    assert!(ticks.len() >= 4, "{ticks:?}");
    // The repeats' ticks: at least three frames (50 ms) apart.
    let repeats = ticks.get(1..).unwrap_or_default();
    for pair in repeats.windows(2) {
        assert!(pair[1] - pair[0] >= 3, "{ticks:?}");
    }
    assert_eq!(dropped(&h, "cursor_move"), Vec::<u32>::new());

    // Two arrows pressed in one frame: the cursor moves twice, ticks once.
    h.keys("Left Left Left");
    let at = h.cursor_tile();
    h.clear_audio();
    let (up, left) = (chord("Up"), chord("Left"));
    let both = [RawInputEvent::Down(up), RawInputEvent::Down(left)];
    h.frame(&both, FRAME_DT);
    let ups = [RawInputEvent::Up(up.key), RawInputEvent::Up(left.key)];
    h.frame(&ups, FRAME_DT);
    let moved_to = at.map(|p| Pos::new(p.x - 1, p.y - 1));
    assert_eq!(h.cursor_tile(), moved_to);
    let frames = asked(&h, "cursor_move");
    assert_eq!(frames.len(), 2, "{frames:?}");
    assert_eq!(frames[0], frames[1]);
    assert_eq!(dropped(&h, "cursor_move"), [frames[1]]);

    // A late frame owed several repeats of a held key ticks once too.
    h.clear_audio();
    let left = chord("Left");
    h.frame(&[RawInputEvent::Down(left)], FRAME_DT);
    h.frame(&[], 0.25);
    let from = h.cursor_tile();
    h.frame(&[], 0.3);
    let late = asked(&h, "cursor_move");
    let moved = from.zip(h.cursor_tile()).map(|(a, b)| a.x - b.x);
    h.frame(&[RawInputEvent::Up(left.key)], FRAME_DT);
    assert!(moved.is_some_and(|m| m >= 2), "{moved:?}");
    let last = late.last().copied();
    let in_late = late.iter().filter(|&&f| Some(f) == last).count();
    assert_eq!(dropped(&h, "cursor_move").len(), in_late - 1, "{late:?}");
}

/// `name` as a chord of the keyboard.
fn chord(name: &str) -> Chord {
    Chord::parse(name).unwrap_or_else(|e| panic!("{e}"))
}

/// The files the footsteps played, frame by frame for `frames`: an index
/// into `step_foot`'s `files`, from what the headless speaker was told.
fn footsteps(h: &mut Harness, frames: u32) -> Vec<usize> {
    let mut files = Vec::new();
    for _ in 0..frames {
        h.frame(&[], FRAME_DT);
        let asked = h.last_frame_audio().iter().any(|r| match r {
            AudioRequest::PlaySound { cue, .. } => cue == "step_foot",
            _ => false,
        });
        if !asked {
            continue;
        }
        let told = h.platform().speaker().requests();
        files.extend(told.iter().filter_map(|r| match r {
            SoundRequest::Sound { cue, variant, .. } if cue == "step_foot" => Some(*variant),
            _ => None,
        }));
    }
    files
}

/// `step_foot` has three files (`audio.md`): each step picks one of the
/// other two, never the one the step before played.
#[test]
fn two_footsteps_in_a_row_never_play_the_same_file() {
    let mut h = quick_battle();
    // The enemy phase: the brigands walk, a foot step a tile.
    h.keys("Space f");
    let files = footsteps(&mut h, 12 * 60);
    assert!(files.len() >= 6, "{files:?}");
    assert!(files.iter().all(|&f| f < 3), "{files:?}");
    for pair in files.windows(2) {
        assert_ne!(pair[0], pair[1], "{files:?}");
    }
    assert_eq!(dropped(&h, "step_foot"), Vec::<u32>::new());
}
