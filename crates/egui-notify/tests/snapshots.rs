//! A picture of a toast with an action, compared with the one in `img/`.
//! `UPDATE_SNAPSHOTS=1 cargo test -p egui-notify --test snapshots` draws it
//! again.

use egui_kittest::{Harness, SnapshotOptions};
use egui_notify::{Anchor, Toasts};

/// Where the pictures are kept: beside the README, which shows them.
const PICTURES: &str = "img";

#[test]
fn action() {
    let mut toasts = Toasts::new().with_anchor(Anchor::BottomRight);
    toasts
        .basic("«2026-11-06 09:00» is in the trash")
        .action("Undo")
        .duration(None);
    let mut harness = Harness::builder()
        .with_size(egui::vec2(420.0, 90.0))
        .with_theme(egui::Theme::Dark)
        .build_ui_state(
            |ui, toasts: &mut Toasts| {
                let window = ui.ctx().content_rect();
                ui.ctx()
                    .layer_painter(egui::LayerId::background())
                    .rect_filled(window, 0.0, ui.visuals().panel_fill);
                toasts.show(ui.ctx());
            },
            toasts,
        );
    harness.run_steps(60);
    harness.snapshot_options("action", &SnapshotOptions::new().output_path(PICTURES));
}
