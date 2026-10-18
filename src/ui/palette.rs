//! The floating panel of beads: the colours of the pattern and the two in
//! hand.

use egui::{Color32, vec2};
use rust_i18n::t;

use obusina_panel::{Edge, FloatingPanel, Orientation, Placement, Room};

use crate::app::App;
use crate::ui::tools::gap;
use crate::ui::{button, icons, lines};

/// The name the place of the panel is kept by.
const NAME: &str = "palette";
/// The space between two swatches.
const SPACING: egui::Vec2 = vec2(4.0, 4.0);
/// Swatches in one line of a row, when the window is wide enough.
const ROW: usize = 8;
/// Swatches in one line of a column, when the window is tall enough.
const COLUMN: usize = 2;
/// The buttons after the swatches: the two colours in hand and the swap.
const IN_HAND: usize = 3;

/// Shows the panel of beads.
pub fn show(app: &mut App, context: &egui::Context) {
    let placement = app
        .settings
        .panel(NAME)
        .unwrap_or(Placement::docked(Edge::Right, 0.0));
    let shown = FloatingPanel::new(NAME, t!("panel.palette"))
        .placement(placement)
        .show(context, |ui, room| {
            let count = app.pattern.palette().len() + 1;
            let spacing = ui.spacing().item_spacing;
            swatches(app, ui, per_line(room, count, spacing));
            gap(ui, room.orientation);
            in_hand(app, ui);
        });
    if shown.changed {
        app.keep_panel(NAME, shown.placement);
    }
}

/// How many of `count` swatches stand in one line, so the block of swatches
/// and the colours in hand after it fit the length of `room`.
///
/// In a row a line runs along the panel: it holds [`ROW`], or fewer when the
/// window is narrow. In a column the lines are stacked along the panel: a
/// line holds [`COLUMN`], or more when the window is too short for the lines.
fn per_line(room: Room, count: usize, spacing: egui::Vec2) -> usize {
    let in_hand = lines::length(room.orientation, IN_HAND, spacing)
        + lines::gap_length(room.orientation, spacing);
    let length = room.length - in_hand;
    let step = button::SIZE + SPACING;
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
