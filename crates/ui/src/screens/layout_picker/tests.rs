use super::*;
use crate::screen::tests::ctx;
use crate::widgets::help::NOT_MAPPED;

fn input(actions: &[Action]) -> FrameInput {
    FrameInput::new(actions.to_vec(), 0.0, vec![])
}

fn first_launch_ctx() -> Ctx {
    Ctx::embedded().unwrap()
}

#[test]
fn down_and_confirm_pick_left_handed_and_save_it() {
    let mut c = first_launch_ctx();
    let mut p = LayoutPickerScreen::new();
    assert_eq!(p.name(), "layout_picker");
    assert_eq!(p.focused(), Layout::RightHanded);
    let t = p.update(&mut c, &input(&[Action::CursorDown]));
    assert_eq!(format!("{t:?}"), "None");
    assert_eq!(p.focused(), Layout::LeftHanded);
    assert_eq!(c.layout(), None);
    let t = p.update(&mut c, &input(&[Action::Confirm]));
    assert_eq!(format!("{t:?}"), "Pop");
    assert_eq!(c.layout(), Some(Layout::LeftHanded));
    assert_eq!(c.saved_layout(), Some(Layout::LeftHanded));
}

#[test]
fn confirm_picks_the_focused_layout() {
    let mut c = first_launch_ctx();
    let mut p = LayoutPickerScreen::new();
    let t = p.update(
        &mut c,
        &input(&[Action::CursorUp, Action::CursorUp, Action::Confirm]),
    );
    assert_eq!(format!("{t:?}"), "Pop");
    assert_eq!(c.saved_layout(), Some(Layout::RightHanded));
}

#[test]
fn cancel_and_other_actions_do_nothing() {
    let mut c = first_launch_ctx();
    let mut p = LayoutPickerScreen::default();
    let t = p.update(
        &mut c,
        &input(&[Action::Cancel, Action::Info, Action::CursorLeft]),
    );
    assert_eq!(format!("{t:?}"), "None");
    assert_eq!(c.layout(), None);
    assert_eq!(p.focused(), Layout::RightHanded);
}

/// Ticket 0226: any controller button closes the asked-for picker
/// without choosing; opened from Options, the pad steers it.
#[test]
fn a_pad_press_closes_only_the_asked_for_picker() {
    use crate::input::Button;
    let pad = input(&[Action::CursorDown, Action::Confirm])
        .with_buttons(vec![Button::DpadDown, Button::South], vec![]);
    let mut c = first_launch_ctx();
    let mut p = LayoutPickerScreen::new();
    assert_eq!(format!("{:?}", p.update(&mut c, &pad)), "Pop");
    assert_eq!(c.layout(), None);
    assert_eq!(p.focused(), Layout::RightHanded);
    // A release alone isn't a press.
    let up = input(&[]).with_buttons(vec![], vec![Button::South]);
    assert_eq!(format!("{:?}", p.update(&mut c, &up)), "None");
    let mut c = ctx();
    let mut p = LayoutPickerScreen::change(&c);
    assert_eq!(format!("{:?}", p.update(&mut c, &pad)), "Pop");
    assert_eq!(c.layout(), Some(Layout::LeftHanded));
}

#[test]
fn help_names_the_picker_keys() {
    assert_eq!(
        LayoutPickerScreen::help(&first_launch_ctx()),
        "w/Up s/Down choose · f/j/Enter/Space pick"
    );
    // Later (from Options) it names the chosen layout's keys.
    assert_eq!(LayoutPickerScreen::help(&ctx()), "Up Down choose · f pick");
}

#[test]
fn legend_comes_from_each_layout() {
    let def = &first_launch_ctx().content.keymap;
    let legend = |l| {
        LayoutPickerScreen::legend(&Keymap::for_layout(def, l))
            .into_iter()
            .map(|row| format!("{} {}", row.keys, row.what))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        legend(Layout::RightHanded),
        [
            "arrows move",
            "f select",
            "d/Escape back",
            "a prev unit",
            "s next unit",
            "e unit info",
            "w danger zone",
            "Space end turn",
            "Shift+Space auto-end",
            "r rewind",
        ]
    );
    assert_eq!(
        legend(Layout::LeftHanded),
        [
            "wasd move",
            "j select",
            "k/Escape back",
            "; prev unit",
            "l next unit",
            "i unit info",
            "o danger zone",
            "Space end turn",
            "Shift+Space auto-end",
            "u rewind",
        ]
    );
    let unbound = LayoutPickerScreen::legend(&Keymap::for_layout(
        &trpg_content::KeymapDef::default(),
        Layout::LeftHanded,
    ));
    assert_eq!(unbound.len(), 10);
    assert!(unbound.iter().all(|r| r.keys == NOT_MAPPED), "{unbound:?}");
}

#[test]
fn key_roles() {
    let def = &first_launch_ctx().content.keymap;
    let left = Keymap::for_layout(def, Layout::LeftHanded);
    assert_eq!(key_role(&left, Key::W), KeyRole::Movement);
    assert_eq!(key_role(&left, Key::J), KeyRole::Bound);
    assert_eq!(key_role(&left, Key::Up), KeyRole::Unbound);
}

#[test]
fn only_the_cursor_keys_are_movement_in_the_legend() {
    let km = Keymap::for_layout(&first_launch_ctx().content.keymap, Layout::RightHanded);
    let legend = LayoutPickerScreen::legend(&km);
    assert!(legend[0].movement);
    assert_eq!(legend.iter().filter(|r| r.movement).count(), 1);
}

#[test]
fn labels() {
    assert_eq!(label(Layout::RightHanded), "Right-handed");
    assert_eq!(label(Layout::LeftHanded), "Left-handed");
}

/// The view of the picker as it opens on first launch: both layouts in
/// order, the first one focused, the picker's own help.
#[test]
fn the_first_frame_offers_both_layouts_with_the_first_focused() {
    let c = first_launch_ctx();
    let view = LayoutPickerScreen::new().view(&c);
    assert_eq!(view.title, "Pick your layout");
    let names: Vec<_> = view
        .layouts
        .iter()
        .map(|l| (l.layout, l.name.as_str()))
        .collect();
    assert_eq!(
        names,
        [
            (Layout::RightHanded, "Right-handed"),
            (Layout::LeftHanded, "Left-handed")
        ]
    );
    assert_eq!(
        view.layouts.iter().map(|l| l.focused).collect::<Vec<_>>(),
        [true, false]
    );
    assert_eq!(view.focused().map(|l| l.layout), Some(Layout::RightHanded));
    assert_eq!(view.help, "w/Up s/Down choose · f/j/Enter/Space pick");
    // Each layout's legend is read from its own keys.
    let legend = |l: usize| view.layouts[l].legend[1].keys.clone();
    assert_eq!(legend(0), "f");
    assert_eq!(legend(1), "j");
}

#[test]
fn moving_the_cursor_moves_the_focus_in_the_view() {
    let mut c = first_launch_ctx();
    let mut p = LayoutPickerScreen::new();
    p.update(&mut c, &input(&[Action::CursorDown]));
    let view = p.view(&c);
    assert_eq!(view.focused().map(|l| l.layout), Some(Layout::LeftHanded));
    assert_eq!(
        view.layouts.iter().map(|l| l.focused).collect::<Vec<_>>(),
        [false, true]
    );
    // Up wraps back.
    p.update(&mut c, &input(&[Action::CursorUp]));
    assert_eq!(
        p.view(&c).focused().map(|l| l.layout),
        Some(Layout::RightHanded)
    );
}

/// Opened from Options: the layout in use is focused and the help names
/// Cancel too.
#[test]
fn opened_from_options_it_focuses_the_layout_in_use() {
    let c = ctx().with_layout(Layout::LeftHanded);
    let view = LayoutPickerScreen::change(&c).view(&c);
    assert_eq!(view.focused().map(|l| l.layout), Some(Layout::LeftHanded));
    assert_eq!(view.help, c.text_with("layout_picker.help_change", &[]));
    assert_ne!(view.help, LayoutPickerScreen::new().view(&c).help);
}

#[test]
fn the_keyboard_lists_every_key_with_its_role_in_each_layout() {
    let c = first_launch_ctx();
    let view = LayoutPickerScreen::new().view(&c);
    let (right, left) = (&view.layouts[0], &view.layouts[1]);
    assert_eq!(right.keys.len(), Key::ALL.len());
    // Right-handed: the arrows move, `f` selects, `w` does the danger zone.
    assert_eq!(right.role(Key::Up), KeyRole::Movement);
    assert_eq!(right.role(Key::F), KeyRole::Bound);
    assert_eq!(right.role(Key::W), KeyRole::Bound);
    assert_eq!(right.role(Key::J), KeyRole::Unbound);
    // Left-handed: `wasd` move, `j` selects, the arrows do nothing.
    assert_eq!(left.role(Key::W), KeyRole::Movement);
    assert_eq!(left.role(Key::J), KeyRole::Bound);
    assert_eq!(left.role(Key::Up), KeyRole::Unbound);
    // A key that isn't listed is unbound.
    let none = LayoutView {
        keys: vec![],
        ..left.clone()
    };
    assert_eq!(none.role(Key::W), KeyRole::Unbound);
}
