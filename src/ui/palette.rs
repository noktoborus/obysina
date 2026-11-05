//! The floating panel of beads: the colours of the pattern, the one in hand
//! marked.

use egui::{Color32, vec2};
use rust_i18n::t;

use obusina_panel::{FloatingPanel, Orientation, Placement, Room};
use obusina_pattern::{ColourId, Rgb};
use plate_menu::MenuItem;

use crate::ui::palette_window::delete_beads;

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
                        if app.colour == colour {
                            open_menu(app, colour);
                        } else {
                            app.colour = colour;
                        }
                    }
                }
            });
        }
    });
}

/// The colours the panel shows: the places of the palette shown, each with
/// its id. A removed colour does not stand on the panel.
fn shown(app: &App) -> Vec<(usize, ColourId)> {
    app.pattern.palettes()[app.palette]
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

/// The menu of what is in hand, opened by a second tap: hide or show and
/// delete the beads of a colour, and the settings of the palette. The empty
/// place has the settings only.
fn open_menu(app: &mut App, colour: Option<ColourId>) {
    let settings = MenuItem::new("settings", t!("palette.settings"));
    let (items, title) = match colour.and_then(|id| app.pattern.colour(id)) {
        Some(found) => {
            let hide = if found.hidden {
                MenuItem::new("hide", t!("palette.show"))
            } else {
                MenuItem::new("hide", t!("palette.hide"))
            };
            let beads = app.pattern.count(found.id);
            let delete = MenuItem::new("beads", delete_beads(beads)).enabled(beads > 0);
            let rgb = found.rgb.unwrap_or(Rgb([0; 3]));
            (
                vec![hide, delete, settings],
                t!("palette.in_hand", id = found.id, rgb = rgb),
            )
        }
        None => (vec![settings], t!("palette.empty")),
    };
    if let Err(error) = app.bead_menu.open(items) {
        log::error!("the menu of the colour in hand does not open: {error}");
        return;
    }
    app.bead_menu.notice(title);
    app.bead_menu_of = Some(colour);
}

/// Draws the menu of what is in hand while it is open, and does what is
/// chosen.
pub fn menu(app: &mut App, context: &egui::Context) {
    app.bead_menu.nothing_found(t!("menu.nothing_found"));
    let Some(chosen) = app.bead_menu.show(context) else {
        return;
    };
    let Some(colour) = app.bead_menu_of.take() else {
        return;
    };
    match (chosen.id.as_str(), colour) {
        ("settings", _) => app.palette_window.open = true,
        ("hide", Some(id)) => {
            let hidden = app.pattern.colour(id).is_some_and(|found| found.hidden);
            let changed = app.history.change_colours(&mut app.pattern, |pattern| {
                pattern.set_hidden(id, !hidden);
                Ok(())
            });
            if let Err(error) = changed {
                log::error!("the colour is not hidden: {error}");
            }
        }
        ("beads", Some(id)) => app.history.delete_beads(&mut app.pattern, id),
        _ => {}
    }
}
