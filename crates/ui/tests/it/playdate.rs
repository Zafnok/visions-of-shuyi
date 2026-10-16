//! The game as the Playdate's front end runs it (ticket 1033): its
//! 50 × 15 console, no keyboard, and only the D-pad, A (`South`) and B
//! (`East`) of a `PadKind::Playdate` pad. Screens keep their PC look until
//! their Playdate skins land (1037 on), so this checks where the buttons
//! lead, not how it looks.

use trpg_core::lead::DEFAULT_NAME;
use trpg_ui::harness::Harness;
use trpg_ui::input::{Device, PadKind};
use trpg_ui::save::SUSPEND_KEY;
use trpg_ui::screen::Keyboard;
use trpg_ui::{ConsoleSpec, Ctx, KeyPrompt, Storage};

/// The context as `crates/playdate` sets it up (the tips and lessons
/// are turned on below).
fn playdate(ctx: &mut Ctx) {
    ctx.console = ConsoleSpec::PLAYDATE;
    ctx.keyboard = Keyboard::Absent;
    ctx.device = Device::Pad(PadKind::Playdate);
    ctx.key_prompt = KeyPrompt::Waiting;
}

/// Skips every scene on screen: B, then A on "Skip scene?".
fn skip_scenes(h: &mut Harness) {
    for _ in 0..8 {
        if h.top_screen() != "dialogue" {
            return;
        }
        h.pad("East South");
    }
    panic!("the scenes never end: {:?}", h.screens());
}

/// The ticket's walk-through: the title, New Game, the lead's name on the
/// letter grid, and into Chapter 1's first battle, with no layout ever
/// asked for.
#[test]
fn the_d_pad_a_and_b_play_from_the_title_into_chapter_ones_first_battle() {
    let mut h = Harness::first_launch_and(playdate).with_shipped_story();
    h.use_pad(PadKind::Playdate).with_tips().with_lessons();
    assert_eq!(h.game().buffer().spec(), ConsoleSpec::PLAYDATE);
    // A ends the title's wait, then picks New Game, Classic and the
    // familiarity answer.
    h.pad("South South South South");
    assert_eq!(h.top_screen(), "lead_select");
    // The name: A on it opens the letter grid; B deletes a letter, A adds
    // the grid's first one (A), and A on Done (up, then left, from the
    // first letter) closes it.
    h.pad("DpadDown South East South DpadUp DpadLeft South");
    assert_eq!(h.top_screen(), "lead_select");
    // Then down to Start, and A.
    h.pad("DpadDown South");
    skip_scenes(&mut h);
    // The battle's banner (A), then its opening scene.
    h.pad("South");
    skip_scenes(&mut h);
    assert_eq!(h.screens(), ["title", "battle"]);
    assert_eq!(h.game().ctx().layout(), None);
    assert_eq!(h.game().ctx().device, Device::Pad(PadKind::Playdate));
    let mut name = DEFAULT_NAME.to_owned();
    name.pop();
    name.push('A');
    assert_eq!(h.game().ctx().lead.name, name);
    // A closes the first guide; A on the empty tile above the lead opens
    // the map menu, whose fifth entry is Suspend; A on it and A on
    // "Suspend the battle?" write the suspend save and go back to the
    // title.
    h.wait(1.0);
    h.pad("South DpadUp South DpadDown DpadDown DpadDown DpadDown South South");
    assert_eq!(h.screens(), ["title"]);
    let saved = h.game().ctx().storage.read(SUSPEND_KEY);
    assert!(saved.is_ok_and(|s| s.is_some()), "no suspend save");
}
