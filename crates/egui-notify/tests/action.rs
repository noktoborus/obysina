//! The button of an action: a press on it answers its toast, dismisses it,
//! and reaches nothing under it.

use egui::{Event, Modifiers, PointerButton, Pos2};
use egui_kittest::Harness;
use egui_kittest::kittest::Queryable;
use egui_notify::{Anchor, ToastId, Toasts};

/// What the window holds: the toasts, the toast of the action, what `show`
/// gave back, and whether the button under the toasts was pressed.
struct State {
    toasts: Toasts,
    undo: ToastId,
    pressed: Vec<ToastId>,
    under: bool,
}

fn harness() -> Harness<'static, State> {
    let mut toasts = Toasts::new().with_anchor(Anchor::BottomRight);
    let undo = toasts
        .basic("In the trash")
        .action("Undo")
        .duration(None)
        .id();
    toasts.basic("Saved").duration(None);
    let state = State {
        toasts,
        undo,
        pressed: Vec::new(),
        under: false,
    };
    Harness::builder()
        .with_size(egui::vec2(420.0, 200.0))
        .build_ui_state(
            |ui, state: &mut State| {
                let all = ui.available_size();
                if ui.add_sized(all, egui::Button::new("Under")).clicked() {
                    state.under = true;
                }
                state.pressed.extend(state.toasts.show(ui.ctx()));
            },
            state,
        )
}

/// The first button pressed and let go at `at`.
fn press(harness: &mut Harness<'_, State>, at: Pos2) {
    harness.event(Event::PointerMoved(at));
    for pressed in [true, false] {
        harness.event(Event::PointerButton {
            pos: at,
            button: PointerButton::Primary,
            pressed,
            modifiers: Modifiers::NONE,
        });
        harness.step();
    }
}

#[test]
fn the_action_answers_its_toast_and_nothing_under_it() {
    let mut harness = harness();
    harness.run_steps(60);
    let undo = harness.get_by_label("Undo").rect().center();

    press(&mut harness, undo);
    harness.run_steps(60);

    let state = harness.state();
    assert_eq!(state.pressed, vec![state.undo]);
    assert!(!state.under, "the press reached the button under the toast");
    assert_eq!(state.toasts.len(), 1, "the toast of the action is gone");
}

/// Of two toasts, only the one with an action has a button.
#[test]
fn a_toast_with_no_action_has_no_button() {
    let mut harness = harness();
    harness.run_steps(60);

    let buttons = harness
        .query_all_by_role(egui::accesskit::Role::Button)
        .count();
    assert_eq!(buttons, 2, "the button under the toasts, and one action");
    assert_eq!(harness.query_all_by_label("Undo").count(), 1);
}
