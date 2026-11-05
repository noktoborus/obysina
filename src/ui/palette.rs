//! The floating panel of beads: the colours of the pattern, the one in hand
//! marked.

use egui::{Color32, vec2};
use rust_i18n::t;

use obusina_panel::{FloatingPanel, Orientation, Placement, Room};
use obusina_pattern::{ColourId, Rgb};

use crate::app::App;
use crate::ui::{button, icons};

/// The name the place of the panel is kept by.
const NAME: &str = "palette";
/// The space between two swatches.
const SPACING: egui::Vec2 = vec2(4.0, 4.0);
/// Swatches in one line of a row, when the window is wide enough.
const ROW: usize = 8;
/// Swatches in one line of a column, when the window is tall enough.
const COLUMN: usize = 2;

/// Shows the panel of beads.
pub fn show(app: &mut App, context: &egui::Context) {
    let placement = app
        .settings
        .panel(NAME)
        .unwrap_or(Placement::new(vec2(1.0, 0.0), Orientation::Vertical));
    let shown = FloatingPanel::new(NAME, t!("panel.palette"))
        .placement(placement)
        .show(context, |ui, room| {
            let count = shown(app).len() + 1;
            swatches(app, ui, per_line(room, count));
        });
    if shown.changed {
        app.keep_panel(NAME, shown.placement);
    }
}

/// How many of `count` swatches stand in one line, so the block of swatches
/// fits the length of `room`.
///
/// In a row a line runs along the panel: it holds [`ROW`], or fewer when the
/// window is narrow. In a column the lines are stacked along the panel: a
/// line holds [`COLUMN`], or more when the window is too short for the lines.
fn per_line(room: Room, count: usize) -> usize {
    let step = button::SIZE + SPACING;
    let length = room.length;
    match room.orientation {
        Orientation::Horizontal => {
            let fit = ((length + SPACING.x) / step.x).floor() as usize;
            ROW.min(fit).max(1)
        }
        Orientation::Vertical => {
            let lines = (((length + SPACING.y) / step.y).floor() as usize).max(1);
            COLUMN.max(count.div_ceil(lines))
        }
    }
}

/// One swatch for each colour, and one for an empty place, `per_line` in a
/// line.
///
/// The lines are one block, laid out like any other widget of the panel, so
/// the block stands in the middle across the panel with the rest. A grid of
/// egui would place its cells by a layout of its own.
fn swatches(app: &mut App, ui: &mut egui::Ui, per_line: usize) {
    let entries: Vec<Option<(usize, ColourId)>> =
        shown(app).into_iter().map(Some).chain([None]).collect();
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing = SPACING;
        for line in entries.chunks(per_line) {
            ui.horizontal(|ui| {
                for entry in line.iter().copied() {
                    let name = match entry {
                        Some((place, id)) => t!(
                            "palette.bead",
                            key = place + 1,
                            rgb = app
                                .pattern
                                .colour(id)
                                .and_then(|colour| colour.rgb)
                                .unwrap_or(Rgb([0; 3])),
                            count = app.pattern.count(id)
                        )
                        .to_string(),
                        None => t!("palette.empty").to_string(),
                    };
                    let colour = entry.map(|(_, id)| id);
                    if swatch(app, ui, entry, app.colour == colour, &name).clicked() {
                        app.colour = colour;
                    }
                }
            });
        }
    });
}

/// The colours the panel shows: the places of the first palette, each with
/// its id. A removed colour does not stand on the panel.
fn shown(app: &App) -> Vec<(usize, ColourId)> {
    app.pattern.palettes()[0]
        .places()
        .iter()
        .enumerate()
        .filter_map(|(place, colour)| {
            let colour = colour.as_ref().filter(|colour| colour.rgb.is_some())?;
            Some((place, colour.id))
        })
        .collect()
}

/// The swatch of a colour of the palette with its number, or of an empty
/// place.
fn swatch(
    app: &App,
    ui: &mut egui::Ui,
    entry: Option<(usize, ColourId)>,
    selected: bool,
    name: &str,
) -> egui::Response {
    let rgb = entry.and_then(|(_, id)| app.pattern.colour(id).and_then(|colour| colour.rgb));
    let (fill, label) = match (entry, rgb) {
        (Some((place, _)), Some(Rgb([red, green, blue]))) => {
            (Color32::from_rgb(red, green, blue), (place + 1).to_string())
        }
        _ => (ui.visuals().extreme_bg_color, icons::EMPTY.to_string()),
    };
    button::swatch(ui, fill, &label, selected, name)
}
