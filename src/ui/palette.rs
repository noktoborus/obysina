//! The floating panel of beads: the colours of the pattern and the two in
//! hand.

use egui::{Color32, RichText, Stroke, vec2};
use rust_i18n::t;

use obusina_panel::{FloatingPanel, Orientation};

use crate::app::App;
use crate::ui::access::Named;
use crate::ui::icons;
use crate::ui::tools::gap;

/// Where the panel stands the first time, from the top right of the window.
const PLACE: egui::Vec2 = vec2(-16.0, 16.0);
/// The size of a swatch: large enough for a finger.
const SWATCH: egui::Vec2 = vec2(44.0, 36.0);
/// Swatches in one line of a row.
const ROW: usize = 8;
/// Swatches in one line of a column.
const COLUMN: usize = 2;

/// Shows the panel of beads.
pub fn show(app: &mut App, context: &egui::Context) {
    FloatingPanel::new("palette", t!("panel.palette"))
        .place(egui::Align2::RIGHT_TOP, PLACE)
        .show(context, |ui, orientation| {
            let per_line = match orientation {
                Orientation::Horizontal => ROW,
                Orientation::Vertical => COLUMN,
            };
            swatches(app, ui, per_line);
            gap(ui, orientation);
            in_hand(app, ui);
        });
}

/// One swatch for each colour, and one for an empty place, `per_line` in a
/// line.
fn swatches(app: &mut App, ui: &mut egui::Ui, per_line: usize) {
    let counts = app.pattern.counts();
    let entries: Vec<Option<usize>> = (0..app.pattern.palette().len())
        .map(Some)
        .chain([None])
        .collect();
    egui::Grid::new("swatches")
        .spacing(vec2(4.0, 4.0))
        .show(ui, |ui| {
            for (place, colour) in entries.into_iter().enumerate() {
                let name = match colour {
                    Some(index) => t!(
                        "palette.bead",
                        key = app.pattern.palette()[index].key,
                        rgb = app.pattern.palette()[index].rgb,
                        count = counts[index]
                    )
                    .to_string(),
                    None => t!("palette.empty").to_string(),
                };
                let response = swatch(app, ui, colour, app.colours[0] == colour).named(&name);
                if response.clicked() {
                    app.colours[0] = colour;
                }
                if response.secondary_clicked() {
                    app.colours[1] = colour;
                }
                if place % per_line == per_line - 1 {
                    ui.end_row();
                }
            }
        });
}

/// The two colours in hand and the button that swaps them, which is the way
/// to the second colour by touch (`RULE ui.touch`).
fn in_hand(app: &mut App, ui: &mut egui::Ui) {
    let [first, second] = app.colours;
    swatch(app, ui, first, false).named(&t!("palette.first"));
    swatch(app, ui, second, false).named(&t!("palette.second"));
    if ui.button(icons::SWAP).tip(&t!("palette.swap")).clicked() {
        app.colours.swap(0, 1);
    }
}

/// A button of the colour `colour`, its key on it.
fn swatch(app: &App, ui: &mut egui::Ui, colour: Option<usize>, selected: bool) -> egui::Response {
    let (fill, text) = match colour {
        Some(index) => {
            let bead = &app.pattern.palette()[index];
            let [red, green, blue] = bead.rgb.0;
            (Color32::from_rgb(red, green, blue), bead.key.to_string())
        }
        None => (ui.visuals().extreme_bg_color, icons::EMPTY.to_string()),
    };
    let stroke = if selected {
        Stroke::new(3.0, ui.visuals().selection.stroke.color)
    } else {
        Stroke::new(1.0, ui.visuals().widgets.inactive.bg_stroke.color)
    };
    ui.add(
        egui::Button::new(RichText::new(text).color(contrast(fill)).strong())
            .fill(fill)
            .stroke(stroke)
            .min_size(SWATCH),
    )
}

/// Black or white, whichever reads on `fill`.
fn contrast(fill: Color32) -> Color32 {
    let light =
        0.299 * f32::from(fill.r()) + 0.587 * f32::from(fill.g()) + 0.114 * f32::from(fill.b());
    if light > 140.0 {
        Color32::BLACK
    } else {
        Color32::WHITE
    }
}
