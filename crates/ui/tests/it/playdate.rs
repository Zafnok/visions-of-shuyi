//! The game as the Playdate's front end runs it (ticket 1033): its
//! 50 × 15 console, no keyboard, and only the D-pad, A (`South`) and B
//! (`East`) of a `PadKind::Playdate` pad. Screens keep their PC look until
//! their Playdate skins land (1037 on), so this checks where the buttons
//! lead, not how it looks; and that the one-bit renderer draws those
//! frames as 1032's did, whole or only what changed (ticket 1055).

use trpg_content::FontAtlasDef;
use trpg_content::font::ATLAS_1BIT_PATH;
use trpg_core::lead::DEFAULT_NAME;
use trpg_core::{Faction, Phase};
use trpg_ui::harness::Harness;
use trpg_ui::input::{Action, Device, PadKind};
use trpg_ui::onebit::{self, FRAME_BYTES, Frame};
use trpg_ui::save::SUSPEND_KEY;
use trpg_ui::screen::Keyboard;
use trpg_ui::screens::battle::mode::Mode;
use trpg_ui::widgets::help::key_name;
use trpg_ui::{
    ConsoleSizes, ConsoleSpec, Ctx, GlyphBuffer, KeyPrompt, Palette, ShuyiCtx, Storage, UiColor,
};

/// The context as `crates/playdate` sets it up (the tips and lessons
/// are turned on below), with the Playdate's own buttons (ticket 1034).
fn playdate(ctx: &mut Ctx) {
    ctx.console = ConsoleSpec::PLAYDATE;
    ctx.keyboard = Keyboard::Absent;
    ctx.device = Device::Pad(PadKind::Playdate);
    ctx.use_playdate_buttons();
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
    assert_eq!(h.game().ctx().game.lead.name, name);
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

/// The context as `crates/playdate` sets it up, with its palette of inks.
fn playdate_with_inks(ctx: &mut Ctx) {
    playdate(ctx);
    let palette = trpg_content::palette::load()
        .and_then(|base| trpg_content::ink::load_playdate(&base))
        .ok()
        .and_then(|inks| Palette::new::<UiColor>(&inks).ok());
    let Some(palette) = palette else {
        panic!("the Playdate palette does not load");
    };
    ctx.palette = palette;
}

/// Draws each frame it is shown as the Playdate's front end does: whole
/// the first time, then only what changed since the last; and checks
/// each against 1032's reference renderer.
struct OneBit {
    atlas: FontAtlasDef,
    shown: Option<GlyphBuffer>,
    frame: Box<Frame>,
    checked: usize,
}

impl OneBit {
    fn new(h: &Harness) -> Self {
        Self {
            atlas: h.game().ctx().game.content.font.clone(),
            shown: None,
            frame: Box::new([0; FRAME_BYTES]),
            checked: 0,
        }
    }

    fn check(&mut self, h: &Harness) {
        let bits = trpg_content::bundle::bytes(ATLAS_1BIT_PATH);
        let Some(font) = bits.and_then(|bits| onebit::Font::new(&self.atlas, bits).ok()) else {
            panic!("no one-bit font");
        };
        let buffer = h.game().buffer();
        match &self.shown {
            Some(old) => onebit::render_changed(old, buffer, &font, &(), &mut self.frame),
            None => onebit::render(buffer, &font, &(), &mut self.frame),
        }
        let mut whole = Box::new([0; FRAME_BYTES]);
        onebit::render(buffer, &font, &(), &mut whole);
        let mut reference = Box::new([0; FRAME_BYTES]);
        onebit::reference::render(buffer, &font, &(), &mut reference);
        let screen = h.top_screen();
        assert!(whole == reference, "{screen}: not as the reference drew it");
        assert!(self.frame == whole, "{screen}: what changed, drawn wrong");
        self.shown = Some(buffer.clone());
        self.checked += 1;
    }

    /// `check` after each of `frames` frames of `seconds`.
    fn frames(&mut self, h: &mut Harness, frames: usize, seconds: f32) {
        for _ in 0..frames {
            h.wait(seconds);
            self.check(h);
        }
    }
}

/// The title, the opening scene's lines typing out, and Chapter 1's first
/// battle with its cursor moving, on the Playdate's console and palette.
#[test]
fn the_one_bit_frames_are_the_references_whole_or_changed() {
    let mut h = Harness::first_launch_and(playdate_with_inks).with_shipped_story();
    h.use_pad(PadKind::Playdate).with_tips().with_lessons();
    let mut one_bit = OneBit::new(&h);
    one_bit.check(&h);
    // The title, then its menu.
    h.pad("South");
    one_bit.frames(&mut h, 3, 0.2);
    // New Game, Classic, the familiarity answer, then Start.
    h.pad("South South South");
    one_bit.check(&h);
    h.pad("DpadDown DpadDown South");
    one_bit.check(&h);
    // The opening scene, a line typing out a frame at a time.
    let mut saw_dialogue = false;
    for _ in 0..40 {
        h.wait(1.0 / 30.0);
        one_bit.check(&h);
        saw_dialogue |= h.top_screen() == "dialogue";
    }
    assert!(saw_dialogue, "no scene: {:?}", h.screens());
    skip_scenes(&mut h);
    one_bit.check(&h);
    // The battle's banner, its scene, then the map.
    h.pad("South");
    one_bit.check(&h);
    skip_scenes(&mut h);
    one_bit.frames(&mut h, 3, 0.4);
    assert_eq!(h.screens(), ["title", "battle"]);
    // The guide closed, the cursor around the lead.
    for step in [
        "South",
        "DpadUp",
        "DpadRight",
        "DpadDown",
        "DpadDown",
        "DpadLeft",
    ] {
        h.pad(step);
        one_bit.frames(&mut h, 2, 1.0 / 30.0);
    }
    assert!(one_bit.checked > 60);
}

/// Ticket 1034: on the Playdate, A + B together ends the turn (its
/// question shows, as on the PC) and holding B while pressing right or
/// left picks the next or previous ready unit, in Chapter 1's first
/// battle; B alone still cancels.
#[test]
fn a_and_b_end_the_turn_and_hold_b_with_left_or_right_picks_a_unit() {
    let mut h = Harness::first_launch_and(playdate).with_shipped_story();
    h.use_pad(PadKind::Playdate);
    // A to New Game, Classic and the familiarity answer, the name as it
    // is, Start; past the scenes and the banner.
    h.pad("South South South South DpadDown DpadDown South");
    skip_scenes(&mut h);
    h.pad("South");
    skip_scenes(&mut h);
    h.wait(1.0);
    assert_eq!(h.screens(), ["title", "battle"]);
    let battle = |h: &Harness| h.battle().map(|b| b.mode().clone());
    let idle = |h: &Harness| matches!(battle(h), Some(Mode::Idle { .. }));
    let on_a_ready_unit = |h: &Harness| {
        let unit = h.cursor_tile().and_then(|at| h.unit_at(at));
        unit.is_some_and(|u| u.faction == Faction::Player && !u.acted)
    };
    assert!(idle(&h) && on_a_ready_unit(&h));
    let start = h.cursor_tile();
    // Hold B, press right: the next ready unit, and B cancels nothing.
    h.pad("East+DpadRight");
    assert!(idle(&h) && on_a_ready_unit(&h));
    assert_ne!(h.cursor_tile(), start);
    // And left: back to the first.
    h.pad("East+DpadLeft");
    assert_eq!(h.cursor_tile(), start);
    assert!(idle(&h));
    // Help text names the chords through the keymap.
    let keys = h.game().ctx().help_keys();
    assert_eq!(key_name(keys, Action::EndTurn), "A+B");
    assert_eq!(key_name(keys, Action::NextUnit), "B+→");
    // A + B: the question, as on the PC.
    h.pad("South+East");
    assert!(matches!(battle(&h), Some(Mode::EndTurnPrompt { .. })));
    // B alone backs out of it; A + B again, then A, ends the turn.
    h.pad("East");
    assert!(idle(&h));
    assert_eq!(h.cursor_tile(), start, "B cancels, it doesn't move");
    h.pad("South+East South");
    let phase = h.battle().map(|b| b.state().phase());
    assert_ne!(phase, Some(Phase::Player));
}
