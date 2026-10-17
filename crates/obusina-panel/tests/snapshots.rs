//! Pictures of a panel, compared with the ones in `img/`.
//! `UPDATE_SNAPSHOTS=1 cargo test -p obusina-panel --test snapshots` draws
//! them again.

use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotOptions};
use obusina_panel::{FloatingPanel, Orientation};

/// Where the pictures are kept: beside the README, which shows them.
const PICTURES: &str = "img";
/// A window that holds the panel either way.
const SIZE: egui::Vec2 = egui::vec2(240.0, 200.0);
/// The name of the grip.
const GRIP: &str = "Tools";

/// A harness showing a panel of three buttons, laid out as `orientation`.
fn harness(orientation: Orientation) -> Harness<'static> {
    Harness::builder()
        .with_size(SIZE)
        .with_theme(egui::Theme::Dark)
        .build_ui(move |ui| {
            let window = ui.ctx().content_rect();
            ui.painter()
                .rect_filled(window, 0.0, egui::Color32::from_gray(200));
            FloatingPanel::new("tools", GRIP)
                .place(egui::Align2::LEFT_TOP, egui::vec2(12.0, 12.0))
                .orientation(orientation)
                .show(ui.ctx(), |ui, _| {
                    for label in ["A", "B", "C"] {
                        let _ = ui.button(label);
                    }
                });
        })
}

/// Runs the harness until it settles and compares it with `img/<name>.png`.
fn picture(harness: &mut Harness<'_>, name: &str) {
    harness.run();
    harness.snapshot_options(name, &SnapshotOptions::new().output_path(PICTURES));
}

#[test]
fn horizontal() {
    picture(&mut harness(Orientation::Horizontal), "horizontal");
}

#[test]
fn vertical() {
    picture(&mut harness(Orientation::Vertical), "vertical");
}

#[test]
fn folded() {
    let mut harness = harness(Orientation::Horizontal);
    harness.run();
    harness.get_by_label(GRIP).click();
    picture(&mut harness, "folded");
}

/// The zones drawn while the grip is dragged to the left edge, before it is
/// let go.
#[test]
fn zones() {
    let mut harness = harness(Orientation::Horizontal);
    harness.run();
    let start = harness.get_by_label(GRIP).rect().center();
    harness.event(egui::Event::PointerMoved(start));
    harness.event(egui::Event::PointerButton {
        pos: start,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    harness.step();
    let target = egui::pos2(12.0, SIZE.y / 2.0);
    for step in 1..=5 {
        let point = start + (target - start) * (step as f32 / 5.0);
        harness.event(egui::Event::PointerMoved(point));
        harness.step();
    }
    harness.step();
    harness.snapshot_options("zones", &SnapshotOptions::new().output_path(PICTURES));
}
