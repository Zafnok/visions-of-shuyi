//! Another window in front (ticket 1246, `docs/design/options.md`): the
//! game pauses on its last frame and goes silent, and comes back as it was
//! when its window has the focus again.

use shuyi_platform::SystemEvent;
use trpg_ui::harness::Harness;
use trpg_ui::input::RIGHT_HANDED;

/// The title after a moment, its music playing at the player's volumes.
fn title() -> Harness {
    let mut h = Harness::with_layout(RIGHT_HANDED);
    h.wait(0.5);
    h
}

fn silent(h: &Harness) -> bool {
    h.volumes() == (0.0, 0.0) && h.voice_volume() == 0.0
}

#[test]
fn without_focus_the_frame_holds_and_the_sound_is_off() {
    let mut h = title();
    let volumes = (h.volumes(), h.voice_volume());
    assert!(!silent(&h), "{volumes:?}");
    h.system(SystemEvent::FocusLost);
    assert!(h.app().is_paused());
    assert!(silent(&h), "{:?}", h.volumes());
    let held = h.snapshot();
    h.wait(2.0);
    assert_eq!(h.snapshot(), held);
    assert!(silent(&h), "{:?}", h.volumes());
    // Keys do nothing while paused.
    h.keys("f");
    assert_eq!(h.screens(), ["title"]);
}

#[test]
fn the_focus_back_brings_the_game_and_its_volumes_back() {
    let mut h = title();
    let volumes = (h.volumes(), h.voice_volume());
    h.system(SystemEvent::FocusLost);
    h.wait(1.0);
    h.system(SystemEvent::FocusGained);
    assert!(!h.app().is_paused());
    assert_eq!((h.volumes(), h.voice_volume()), volumes);
    h.keys("Down Down f");
    assert_eq!(h.screens(), ["title", "options"]);
}
