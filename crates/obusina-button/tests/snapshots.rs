//! Pictures of the button, compared with the ones in `img/`.
//! `UPDATE_SNAPSHOTS=1 cargo test -p obusina-button --test snapshots` draws
//! them again.

use egui_kittest::kittest::Queryable;
use egui_kittest::{Harness, SnapshotOptions};
use obusina_button::SquareButton;

/// Where the pictures are kept: beside the README, which shows them.
const PICTURES: &str = "img";
/// The size of every button of the pictures.
const SIZE: egui::Vec2 = egui::vec2(44.0, 36.0);

/// A row of buttons: plain, selected, disabled, an icon, and filled ones,
/// light, dark and selected; marked, light and dark; hatched, and hatched
/// and marked.
fn buttons(ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.add(SquareButton::new("A", SIZE));
        ui.add(SquareButton::new("A", SIZE).selected(true));
        ui.add_enabled(false, SquareButton::new("A", SIZE));
        ui.add(
            SquareButton::new("\u{270f}", SIZE)
                .text_size(18.0)
                .name("Pencil"),
        );
        ui.add(SquareButton::new("B", SIZE).fill(egui::Color32::from_rgb(0xf2, 0xef, 0xe6)));
        ui.add(SquareButton::new("C", SIZE).fill(egui::Color32::from_rgb(0x1b, 0x1b, 0x1b)));
        ui.add(
            SquareButton::new("D", SIZE)
                .fill(egui::Color32::from_rgb(0xc0, 0x30, 0x30))
                .selected(true),
        );
        ui.add(
            SquareButton::new("E", SIZE)
                .fill(egui::Color32::from_rgb(0xf2, 0xef, 0xe6))
                .marked(true),
        );
        ui.add(
            SquareButton::new("F", SIZE)
                .fill(egui::Color32::from_rgb(0x1b, 0x1b, 0x1b))
                .marked(true),
        );
        ui.add(
            SquareButton::new("G", SIZE)
                .fill(egui::Color32::from_rgb(0xc0, 0x30, 0x30))
                .hatched(true),
        );
        ui.add(SquareButton::new("H", SIZE).hatched(true).marked(true));
    });
}

#[test]
fn kinds() {
    let mut harness = Harness::builder()
        .with_size(egui::vec2(580.0, 60.0))
        .with_theme(egui::Theme::Dark)
        .build_ui(buttons);
    harness.run();
    harness.snapshot_options("kinds", &SnapshotOptions::new().output_path(PICTURES));
}

#[test]
fn a_button_is_found_by_its_name_and_clicked() {
    let mut clicks = 0;
    let mut harness = Harness::new_ui(|ui| {
        if ui
            .add(SquareButton::new("\u{270f}", SIZE).name("Pencil"))
            .clicked()
        {
            clicks += 1;
        }
    });
    harness.get_by_label("Pencil").click();
    harness.run();
    drop(harness);

    assert_eq!(clicks, 1);
}
