//! Pictures of a panel, compared with the ones in `img/`.
//! `UPDATE_SNAPSHOTS=1 cargo test -p obusina-panel --test snapshots` draws
//! them again.

use egui_kittest::{Harness, SnapshotOptions};
use obusina_panel::{FloatingPanel, Orientation, Placement};

/// Where the pictures are kept: beside the README, which shows them.
const PICTURES: &str = "img";
/// A window that holds the panel either way.
const SIZE: egui::Vec2 = egui::vec2(240.0, 200.0);
/// The name of the panel.
const NAME: &str = "Tools";

/// A harness showing a panel of three buttons, laid out as `orientation`.
fn harness(orientation: Orientation) -> Harness<'static> {
    Harness::builder()
        .with_size(SIZE)
        .with_theme(egui::Theme::Dark)
        .build_ui(move |ui| {
            let window = ui.ctx().content_rect();
            ui.painter()
                .rect_filled(window, 0.0, egui::Color32::from_gray(200));
            FloatingPanel::new("tools", NAME)
                .placement(Placement::new(egui::Vec2::ZERO))
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

/// A panel taken by the mouse and moved: lighter, with a deeper shadow,
/// under the pointer.
#[test]
fn lifted() {
    let mut harness = harness(Orientation::Horizontal);
    harness.run();
    let start = egui::pos2(20.0, 20.0);
    let button = |pos, pressed| egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    };
    harness.event(egui::Event::PointerMoved(start));
    harness.event(button(start, true));
    harness.step();
    for step in 1..=5 {
        let point = start + egui::vec2(10.0, 12.0) * step as f32;
        harness.event(egui::Event::PointerMoved(point));
        harness.step();
    }
    picture(&mut harness, "lifted");
}
