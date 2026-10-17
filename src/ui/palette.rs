//! The floating panel of beads: the colours of the pattern and the two in
//! hand.

use egui::{Color32, vec2};
use rust_i18n::t;

use obusina_panel::{FloatingPanel, Orientation};

use crate::app::App;
use crate::ui::tools::gap;
use crate::ui::{button, icons};

/// Where the panel stands the first time, from the top right of the window.
const PLACE: egui::Vec2 = vec2(-16.0, 16.0);
/// The space between two swatches.
const SPACING: egui::Vec2 = vec2(4.0, 4.0);
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
///
/// The lines are one block, laid out like any other widget of the panel, so
/// the block stands in the middle across the panel with the rest. A grid of
/// egui would place its cells by a layout of its own.
fn swatches(app: &mut App, ui: &mut egui::Ui, per_line: usize) {
    let counts = app.pattern.counts();
    let entries: Vec<Option<usize>> = (0..app.pattern.palette().len())
        .map(Some)
        .chain([None])
        .collect();
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = SPACING;
        for line in entries.chunks(per_line) {
            ui.horizontal(|ui| {
                for colour in line.iter().copied() {
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
                    let response = swatch(app, ui, colour, app.colours[0] == colour, &name);
                    if response.clicked() {
                        app.colours[0] = colour;
                    }
                    if response.secondary_clicked() {
                        app.colours[1] = colour;
                    }
                }
            });
        }
    });
}

/// The two colours in hand and the button that swaps them, which is the way
/// to the second colour by touch (`RULE ui.touch`).
fn in_hand(app: &mut App, ui: &mut egui::Ui) {
    let [first, second] = app.colours;
    swatch(app, ui, first, false, &t!("palette.first"));
    swatch(app, ui, second, false, &t!("palette.second"));
    if button::icon(ui, icons::SWAP, &t!("palette.swap"), true).clicked() {
        app.colours.swap(0, 1);
    }
}

/// The swatch of a colour of the palette, or of an empty place.
fn swatch(
    app: &App,
    ui: &mut egui::Ui,
    colour: Option<usize>,
    selected: bool,
    name: &str,
) -> egui::Response {
    let (fill, label) = match colour {
        Some(index) => {
            let bead = &app.pattern.palette()[index];
            let [red, green, blue] = bead.rgb.0;
            (Color32::from_rgb(red, green, blue), bead.key.to_string())
        }
        None => (ui.visuals().extreme_bg_color, icons::EMPTY.to_string()),
    };
    button::swatch(ui, fill, &label, selected, name)
}
