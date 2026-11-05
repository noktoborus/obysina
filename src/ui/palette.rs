//! The floating panel of beads: the colours of the palette shown, each with
//! the number of its place, and the empty place `0`; the one in hand marked
//! by a bar under it, a hidden one hatched.
//! A line runs along the panel and turns with it; a block stands as the keys
//! of a numpad (decision 10 of `docs/palette.md`). A shake of its grip turns
//! one into the other.
//!
//! ```text
//! line   < 1 2 3 4 5 6 7 8 9 0 >      block   7 8 9
//!                                             4 5 6
//!                                             1 2 3
//!                                             [ 0 ] ↔
//! ```
//!
//! With two palettes one button `↔` goes to the other, after `0`; with more,
//! `<` and `>`; with one, none.

use egui::{Color32, vec2};
use rust_i18n::t;

use obusina_panel::{FloatingPanel, Orientation, Placement, Room};
use obusina_pattern::{ColourId, Rgb};
use plate_menu::MenuItem;

use crate::app::App;
use crate::settings::PaletteView;
use crate::ui::palette_window::{block, delete_beads, step};
use crate::ui::{button, icons};

/// The name the place of the panel is kept by.
const NAME: &str = "palette";
/// The space between two swatches.
const SPACING: egui::Vec2 = vec2(4.0, 4.0);

/// One button of the panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Entry {
    /// A colour: its place and its id.
    Colour(usize, ColourId),
    /// The empty place, `0`.
    Empty,
    /// The previous palette.
    Back,
    /// The next palette.
    On,
    /// The other palette, of two.
    Other,
}

/// Shows the panel of beads.
pub fn show(app: &mut App, context: &egui::Context) {
    let placement = app
        .settings
        .panel(NAME)
        .unwrap_or(Placement::new(vec2(1.0, 0.0), Orientation::Vertical));
    let shown = FloatingPanel::new(NAME, t!("panel.palette"))
        .placement(placement)
        .show(context, |ui, room| {
            ui.spacing_mut().item_spacing = SPACING;
            match app.settings.palette_view() {
                PaletteView::Line => line(app, ui, room),
                PaletteView::Block => numpad(app, ui),
            }
        });
    if shown.changed {
        app.keep_panel(NAME, shown.placement);
    }
    if shown.shaken {
        let other = match app.settings.palette_view() {
            PaletteView::Line => PaletteView::Block,
            PaletteView::Block => PaletteView::Line,
        };
        app.keep_palette_view(other);
    }
}

/// The buttons of the line, in order.
fn entries(app: &App) -> Vec<Entry> {
    let palettes = app.pattern.palettes().len();
    let colours = shown(app)
        .into_iter()
        .map(|(place, id)| Entry::Colour(place, id));
    let mut entries = Vec::new();
    if palettes > 2 {
        entries.push(Entry::Back);
    }
    entries.extend(colours);
    entries.push(Entry::Empty);
    match palettes {
        1 => {}
        2 => entries.push(Entry::Other),
        _ => entries.push(Entry::On),
    }
    entries
}

/// How many of `count` buttons stand in one line, so they fit the length of
/// `room`. Along a row they are one line, cut where the window ends; along a
/// column they are one column, two or more when the window is too short.
fn per_line(room: Room, count: usize) -> usize {
    let step = button::SIZE + SPACING;
    match room.orientation {
        Orientation::Horizontal => {
            let fit = ((room.length + SPACING.x) / step.x).floor() as usize;
            count.min(fit).max(1)
        }
        Orientation::Vertical => {
            let lines = (((room.length + SPACING.y) / step.y).floor() as usize).max(1);
            count.div_ceil(lines).max(1)
        }
    }
}

/// The line: the buttons in one line along the panel. The lines are one
/// block, laid out like any other widget of the panel, so the block stands in
/// the middle across the panel with the rest.
fn line(app: &mut App, ui: &mut egui::Ui, room: Room) {
    let entries = entries(app);
    let per_line = per_line(room, entries.len());
    ui.vertical(|ui| {
        for chunk in entries.chunks(per_line) {
            ui.horizontal(|ui| {
                for entry in chunk {
                    button_of(app, ui, *entry, false);
                }
            });
        }
    });
}

/// The block: the colours as the keys of a numpad, rows with no colour
/// folded, a gap for an empty place; the wide `0` under them, the switch of
/// the palette beside it or under it.
fn numpad(app: &mut App, ui: &mut egui::Ui) {
    let colours = shown(app);
    let rows = block(obusina_pattern::MAX_COLOURS);
    let palettes = app.pattern.palettes().len();
    ui.vertical(|ui| {
        for row in rows {
            if !row
                .iter()
                .any(|place| colours.iter().any(|(at, _)| at == place))
            {
                continue;
            }
            ui.horizontal(|ui| {
                for place in row {
                    match colours.iter().find(|(at, _)| *at == place) {
                        Some((place, id)) => button_of(app, ui, Entry::Colour(*place, *id), false),
                        None => {
                            ui.allocate_exact_size(button::SIZE, egui::Sense::hover());
                        }
                    }
                }
            });
        }
        ui.horizontal(|ui| {
            button_of(app, ui, Entry::Empty, true);
            if palettes == 2 {
                button_of(app, ui, Entry::Other, false);
            }
        });
        if palettes > 2 {
            ui.horizontal(|ui| {
                button_of(app, ui, Entry::Back, false);
                button_of(app, ui, Entry::On, false);
            });
        }
    });
}

/// One button of the panel and what a tap on it does.
fn button_of(app: &mut App, ui: &mut egui::Ui, entry: Entry, wide: bool) {
    match entry {
        Entry::Colour(place, id) => {
            let colour = app.pattern.colour(id);
            let rgb = colour.and_then(|colour| colour.rgb).unwrap_or(Rgb([0; 3]));
            let name = t!(
                "palette.bead",
                key = place + 1,
                rgb = rgb,
                count = app.pattern.count(id)
            );
            let fill = colour
                .and_then(obusina_pattern::Colour::seen)
                .map(|Rgb([red, green, blue])| Color32::from_rgb(red, green, blue));
            let in_hand = app.colour == Some(id);
            if button::colour(ui, fill, &(place + 1).to_string(), in_hand, &name).clicked() {
                tap(app, Some(id));
            }
        }
        Entry::Empty => {
            let fill = ui.visuals().extreme_bg_color;
            let in_hand = app.colour.is_none();
            let name = t!("palette.empty");
            let response = if wide {
                button::wide(ui, fill, "0", in_hand, &name)
            } else {
                button::colour(ui, Some(fill), "0", in_hand, &name)
            };
            if response.clicked() {
                tap(app, None);
            }
        }
        Entry::Back => {
            if button::icon(ui, "<", &t!("palette.previous"), true).clicked() {
                step(app, -1);
            }
        }
        Entry::On => {
            if button::icon(ui, ">", &t!("palette.next"), true).clicked() {
                step(app, 1);
            }
        }
        Entry::Other => {
            if button::icon(ui, icons::SWITCH, &t!("palette.switch"), true).clicked() {
                step(app, 1);
            }
        }
    }
}

/// A tap on a colour or the empty place: it goes in hand; what is in hand
/// opens its menu.
fn tap(app: &mut App, colour: Option<ColourId>) {
    if app.colour == colour {
        open_menu(app, colour);
    } else {
        app.colour = colour;
    }
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
